//! One request/response to a separately trusted endpoint; decoded metadata stays unverified.
use super::*;
use faction_delegation_codec as codec;
/// Trusted runtime configuration, never selected by a request or claim.
#[derive(Clone)]
pub struct HttpEndpoint {
    address: SocketAddr,
    host: String,
    server_name: String,
    leaf_pin: [u8; 32],
    credentials: ClientCredentials,
    #[cfg(test)]
    read_retries: Option<Arc<std::sync::atomic::AtomicUsize>>,
}
impl HttpEndpoint {
    pub fn new(
        address: SocketAddr,
        host: String,
        server_name: String,
        leaf_pin: [u8; 32],
        credentials: ClientCredentials,
    ) -> Result<Self, Stage> {
        if leaf_pin == [0; 32] || address.port() == 0 {
            return Err(Stage::Configuration);
        }
        for name in [&host, &server_name] {
            codec::encode_introspection_http(
                name,
                &codec::UnverifiedIntrospectionRequest {
                    client_id: "validation".into(),
                    token: codec::UnverifiedToken::from_bytes([1; 32]),
                },
            )
            .map_err(|_| Stage::Configuration)?;
        }
        Ok(Self {
            address,
            host,
            server_name,
            leaf_pin,
            credentials,
            #[cfg(test)]
            read_retries: None,
        })
    }
    fn exchange(
        &self,
        request: &[u8],
        deadline: Instant,
        cancel: Arc<AtomicBool>,
        probe: Arc<dyn Fn() -> bool + Send + Sync>,
    ) -> Result<Vec<u8>, Stage> {
        check(deadline, &cancel)?;
        if !probe() {
            return Err(Stage::Denied);
        }
        if request.len() > codec::MAX_HEADER + codec::MAX_JSON {
            return Err(Stage::Frame);
        }
        let handshake = deadline.min(Instant::now() + HANDSHAKE_LIMIT);
        let mut b =
            SslConnector::builder(SslMethod::tls_client()).map_err(|_| Stage::Configuration)?;
        b.set_min_proto_version(Some(SslVersion::TLS1_3))
            .map_err(|_| Stage::Configuration)?;
        b.set_max_proto_version(Some(SslVersion::TLS1_3))
            .map_err(|_| Stage::Configuration)?;
        let mut store =
            openssl::x509::store::X509StoreBuilder::new().map_err(|_| Stage::Configuration)?;
        store
            .add_cert(self.credentials.ca.clone())
            .map_err(|_| Stage::Configuration)?;
        b.set_cert_store(store.build());
        b.set_verify(SslVerifyMode::PEER);
        b.set_session_cache_mode(SslSessionCacheMode::OFF);
        b.set_options(SslOptions::NO_TICKET);
        b.set_max_early_data(0).map_err(|_| Stage::Configuration)?;
        b.set_certificate(&self.credentials.certificate)
            .map_err(|_| Stage::Configuration)?;
        b.set_private_key(&self.credentials.key)
            .map_err(|_| Stage::Configuration)?;
        b.check_private_key().map_err(|_| Stage::Configuration)?;
        check(handshake, &cancel)?;
        if !probe() {
            return Err(Stage::Denied);
        }
        // A single bounded connect attempt; no retry or DNS/proxy discovery.
        let socket = TcpStream::connect_timeout(&self.address, connect_budget(handshake))
            .map_err(|_| classify(handshake, &cancel, Stage::Admission))?;
        let mut io = DeadlineIo::new(socket, handshake, cancel.clone());
        io.policy_probe = Some(probe.clone());
        #[cfg(test)]
        {
            io.read_retries = self.read_retries.clone();
        }
        let mut configuration = b.build().configure().map_err(|_| Stage::Configuration)?;
        configuration.set_verify_hostname(true);
        let mut stream = configuration
            .connect(&self.server_name, io)
            .map_err(|_| classify(handshake, &cancel, Stage::Handshake))?;
        check(handshake, &cancel)?;
        if !probe()
            || stream.ssl().verify_result() != X509VerifyResult::OK
            || stream.ssl().session_reused()
        {
            return Err(Stage::Handshake);
        }
        let leaf = stream.ssl().peer_certificate().ok_or(Stage::Handshake)?;
        if sha256(&leaf.to_der().map_err(|_| Stage::Handshake)?) != self.leaf_pin {
            return Err(Stage::Pin);
        }
        let client = self.credentials.certificate.clone();
        stream.get_mut().policy_probe = Some(Arc::new(move || {
            probe() && leaf_current(&client, wall_now()) && leaf_current(&leaf, wall_now())
        }));
        stream.get_mut().deadline = deadline;
        stream
            .write_all(request)
            .and_then(|_| stream.flush())
            .map_err(|_| classify(deadline, &cancel, Stage::Frame))?;
        let bytes =
            read_response(&mut stream).map_err(|_| classify(deadline, &cancel, Stage::Frame))?;
        check(deadline, &cancel)?;
        if !stream.get_mut().policy_probe.as_ref().is_some_and(|p| p()) {
            return Err(Stage::Denied);
        }
        Ok(bytes)
    }
}
fn read_response(stream: &mut SslStream<DeadlineIo>) -> Result<Vec<u8>, Stage> {
    let mut head = [0; codec::MAX_HEADER];
    let mut used = 0;
    let end = loop {
        if used == head.len() {
            return Err(Stage::Frame);
        }
        let n = stream.read(&mut head[used..]).map_err(|_| Stage::Frame)?;
        if n == 0 {
            return Err(Stage::Frame);
        }
        used += n;
        if let Some(i) = head[..used].windows(4).position(|b| b == b"\r\n\r\n") {
            break i + 4;
        }
    };
    let length = codec::decode_http_response_head(&head[..end]).map_err(|_| Stage::Frame)?;
    let total = end.checked_add(length).ok_or(Stage::Frame)?;
    if length > codec::MAX_JSON || used > total {
        return Err(Stage::Frame);
    }
    let mut bytes = Vec::with_capacity(total);
    bytes.extend_from_slice(&head[..used]);
    let mut scratch = [0; codec::MAX_JSON + 1];
    while bytes.len() < total {
        let bound = (total - bytes.len() + 1).min(scratch.len());
        let n = stream
            .read(&mut scratch[..bound])
            .map_err(|_| Stage::Frame)?;
        if n == 0 || n > total - bytes.len() {
            return Err(Stage::Frame);
        }
        bytes.extend_from_slice(&scratch[..n]);
    }
    if stream.ssl().pending() != 0 {
        return Err(Stage::Frame);
    }
    Ok(bytes)
}
impl crate::Control {
    pub fn query_publication(
        &self,
        endpoint: &HttpEndpoint,
        query: &codec::UnverifiedPublicationQuery,
        deadline: Instant,
        cancel: Arc<AtomicBool>,
    ) -> Result<codec::UnverifiedPublicationObservation, Error> {
        let bytes = codec::encode_publication_query_http(&endpoint.host, query)
            .map_err(|_| Error::Transport(Stage::Frame))?;
        let probe = self.outbound_http_probe()?;
        let bytes = endpoint
            .exchange(&bytes, deadline, cancel, probe)
            .map_err(Error::Transport)?;
        let result = codec::decode_publication_response_http(&bytes)
            .map_err(|_| Error::Transport(Stage::Frame))?;
        if let codec::UnverifiedPublicationObservation::Current {
            query: observed,
            home,
            ..
        } = &result
            && (observed.as_ref() != query || home != &endpoint.host)
        {
            return Err(Error::Transport(Stage::Denied));
        }
        Ok(result)
    }
    pub fn query_introspection(
        &self,
        endpoint: &HttpEndpoint,
        query: &codec::UnverifiedIntrospectionRequest,
        deadline: Instant,
        cancel: Arc<AtomicBool>,
    ) -> Result<codec::UnverifiedIntrospection, Error> {
        let bytes = codec::encode_introspection_http(&endpoint.host, query)
            .map_err(|_| Error::Transport(Stage::Frame))?;
        let probe = self.outbound_http_probe()?;
        let bytes = endpoint
            .exchange(&bytes, deadline, cancel, probe)
            .map_err(Error::Transport)?;
        codec::decode_introspection_response_http(&bytes)
            .map_err(|_| Error::Transport(Stage::Frame))
    }
}
fn connect_budget(handshake: Instant) -> Duration {
    handshake.saturating_duration_since(Instant::now())
}
#[cfg(test)]
mod tests {
    #[test]
    fn actual_connect_budget_preserves_remaining_admission_and_inherited_deadline() {
        let budget =
            super::connect_budget(std::time::Instant::now() + std::time::Duration::from_secs(2));
        assert!(budget > std::time::Duration::from_secs(1));
        assert!(budget <= std::time::Duration::from_secs(2));
        assert_eq!(
            super::connect_budget(std::time::Instant::now() - std::time::Duration::from_millis(1)),
            std::time::Duration::ZERO
        );
    }
    #[test]
    fn connect_uses_remaining_admission_budget_not_bio_poll_slice() {
        let production = include_str!("http_client.rs")
            .split("#[cfg(test)]\nmod tests")
            .next()
            .unwrap();
        assert!(
            !production.contains(".min(CANCEL_SLICE)"),
            "single TCP connection attempt cannot use the BIO polling slice as its lifetime"
        );
    }
}

#[cfg(test)]
#[path = "http_client_tests.rs"]
mod lifecycle_tests;
