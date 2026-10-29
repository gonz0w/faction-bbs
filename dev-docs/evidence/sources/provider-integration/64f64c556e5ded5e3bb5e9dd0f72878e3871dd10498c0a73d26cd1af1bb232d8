//! Optional member-device Direct TLS XMPP transport, adopted from qualification #208.
//!
//! One explicit numeric endpoint, CA/name/leaf pin, PLAIN authentication and exact
//! resource binding. No DNS, reconnect task, key access, durable queue or receipts.
//! Call from a current-thread Tokio runtime. Dropping a connection future, session,
//! or pending send/read closes the transport. Every call shares the connection's
//! original cooperative ten-second deadline; this is not OS preemption.

pub mod blob_iq;
pub mod guard;

use futures_util::{SinkExt, StreamExt};
use guard::{Guard, GuardedIo, Rejection, SharedGuard};
use native_tls::{Certificate, Protocol, TlsConnector};
use sasl::common::Credentials;
use sha2::{Digest, Sha256};
use std::{borrow::Cow, cell::RefCell, io, net::SocketAddr, rc::Rc, time::Duration};
use tokio::{
    io::{AsyncRead, AsyncWrite},
    net::TcpStream,
    time::Instant,
};
use tokio_xmpp::{
    jid::FullJid,
    xmlstream::{StreamHeader, Timeouts, XmlStream, initiate_stream},
};

pub use tokio_xmpp::minidom::Element;

/// Fixed original connection-plus-session cooperative lifetime.
pub const LIFETIME: Duration = Duration::from_secs(10);
/// Maximum encoded outgoing stanza, checked before writing any of its bytes.
pub const MAX_OUTPUT_BYTES: usize = 16384;
const CLIENT_NS: &str = "jabber:client";
const BIND_NS: &str = "urn:ietf:params:xml:ns:xmpp-bind";
const BIND_ID: &str = "bind-209";

/// Safe local classifications. Never includes a remote diagnostic or secret.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Config,
    KeylogRefused,
    Connect,
    Tls,
    Pin,
    Auth,
    Mechanism,
    Bind,
    Protocol,
    XmlLimit,
    XmlMalformed,
    OutputLimit,
    Lost,
    Timeout,
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for Error {}

/// Checked trusted-local home configuration. Contains no account credential.
///
/// The caller validates and preloads its private regular config/CA files before
/// construction. This type performs no filesystem access. The endpoint is numeric;
/// `domain` supplies TLS verification/SNI and the XMPP stream destination only.
pub struct Home {
    endpoint: SocketAddr,
    domain: String,
    full_jid: String,
    username: String,
    resource: String,
    connector: TlsConnector,
    pin: [u8; 32],
}
impl Home {
    pub fn new(
        endpoint: SocketAddr,
        domain: &str,
        full_jid: &str,
        ca_pem: &[u8],
        leaf_pin: [u8; 32],
    ) -> Result<Self, Error> {
        if endpoint.port() == 0
            || endpoint.ip().is_unspecified()
            || endpoint.ip().is_multicast()
            || !valid_domain(domain)
            || full_jid.len() > 3071
            || full_jid.chars().any(char::is_control)
            || ca_pem.len() > 65536
        {
            return Err(Error::Config);
        }
        let jid = FullJid::new(full_jid).map_err(|_| Error::Config)?;
        let username = jid.node().ok_or(Error::Config)?.as_str().to_owned();
        if jid.to_string() != full_jid || jid.domain().as_str() != domain {
            return Err(Error::Config);
        }
        let resource = jid.resource().as_str().to_owned();
        let ca = Certificate::from_pem(ca_pem).map_err(|_| Error::Config)?;
        let connector = TlsConnector::builder()
            .disable_built_in_roots(true)
            .add_root_certificate(ca)
            .min_protocol_version(Some(Protocol::Tlsv12))
            .build()
            .map_err(|_| Error::Tls)?;
        Ok(Self {
            endpoint,
            domain: domain.into(),
            full_jid: full_jid.into(),
            username,
            resource,
            connector,
            pin: leaf_pin,
        })
    }
    pub fn endpoint(&self) -> SocketAddr {
        self.endpoint
    }
    pub fn domain(&self) -> &str {
        &self.domain
    }
    pub fn full_jid(&self) -> &str {
        &self.full_jid
    }
}
fn valid_domain(domain: &str) -> bool {
    !domain.is_empty()
        && domain.len() <= 253
        && domain.split('.').all(|label| {
            !label.is_empty()
                && label.len() <= 63
                && !label.starts_with('-')
                && !label.ends_with('-')
                && label
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        })
}

/// One authenticated session. Holds no password and starts no background work.
pub struct Session {
    cycle: Cycle<tokio_native_tls::TlsStream<TcpStream>>,
}
impl Session {
    /// A successful write is not proof of peer delivery or acceptance.
    pub async fn send(&mut self, stanza: &Element) -> Result<(), Error> {
        self.cycle.send(stanza).await
    }
    /// Returns bounded XML only; higher layers authenticate message content.
    pub async fn next(&mut self) -> Result<Element, Error> {
        self.cycle.next().await
    }
}

/// Connect once using preloaded local trust and a memory-only caller credential.
///
/// Refuses TLS key logging before attempting a connection. The qualified SASL
/// library makes temporary ordinary memory copies; this does not promise complete
/// password-buffer erasure. The caller should retain its input in zeroizing memory.
pub async fn connect(home: &Home, password: &str) -> Result<Session, Error> {
    if ["SSLKEYLOGFILE", "TLSKEYLOGFILE"]
        .iter()
        .any(|key| std::env::var_os(key).is_some())
    {
        return Err(Error::KeylogRefused);
    }
    if password.is_empty() || password.len() > 128 || password.chars().any(char::is_control) {
        return Err(Error::Config);
    }
    let deadline = Instant::now() + LIFETIME;
    let state = Rc::new(RefCell::new(Guard::new()));
    let result =
        tokio::time::timeout_at(deadline, establish(home, password, state.clone(), deadline))
            .await
            .map_err(|_| Error::Timeout)?;
    match result {
        Ok(session) => Ok(session),
        Err(error) => Err(guard_error(&state, error)),
    }
}
fn header(domain: &str) -> StreamHeader<'_> {
    StreamHeader {
        to: Some(Cow::Borrowed(domain)),
        from: None,
        id: None,
    }
}
fn timeouts() -> Timeouts {
    Timeouts {
        read_timeout: LIFETIME,
        response_timeout: LIFETIME,
    }
}
async fn establish(
    home: &Home,
    password: &str,
    state: SharedGuard,
    deadline: Instant,
) -> Result<Session, Error> {
    let tcp = TcpStream::connect(home.endpoint)
        .await
        .map_err(|_| Error::Connect)?;
    let tls = tokio_native_tls::TlsConnector::from(home.connector.clone())
        .connect(&home.domain, tcp)
        .await
        .map_err(|_| Error::Tls)?;
    let certificate = tls
        .get_ref()
        .peer_certificate()
        .map_err(|_| Error::Tls)?
        .ok_or(Error::Tls)?;
    let der = certificate.to_der().map_err(|_| Error::Tls)?;
    if Sha256::digest(der).as_slice() != home.pin {
        return Err(Error::Pin);
    }
    // Chain, name and DER leaf pin have all passed before XML or SASL credentials.
    let (mut io, _) = GuardedIo::new(tls);
    io.replace_state(state.clone());
    let pending = initiate_stream(io, CLIENT_NS, header(&home.domain), timeouts())
        .await
        .map_err(|_| Error::Lost)?;
    let (features, stream) = pending.recv_features().await.map_err(|_| Error::Protocol)?;
    if !features.sasl_mechanisms.contains("PLAIN") {
        return Err(Error::Mechanism);
    }
    let credentials = Credentials::default()
        .with_username(&home.username)
        .with_password(password);
    let restarting = tokio_xmpp::client_login(stream, ["PLAIN".to_owned()].into(), credentials)
        .await
        .map_err(|_| Error::Auth)?;
    state
        .borrow_mut()
        .restart()
        .map_err(|_| Error::XmlMalformed)?;
    let pending = restarting
        .send_header(header(&home.domain))
        .await
        .map_err(|_| Error::Lost)?;
    let (features, mut stream) = pending
        .recv_features::<Element>()
        .await
        .map_err(|_| Error::Protocol)?;
    if features.bind.is_none() {
        return Err(Error::Bind);
    }
    let bind = Element::builder("iq", CLIENT_NS)
        .attr(rxml::xml_ncname!("type").to_owned(), "set")
        .attr(rxml::xml_ncname!("id").to_owned(), BIND_ID)
        .append(
            Element::builder("bind", BIND_NS)
                .append(
                    Element::builder("resource", BIND_NS)
                        .append(home.resource.clone())
                        .build(),
                )
                .build(),
        )
        .build();
    validate_output(&bind)?;
    stream.send(&bind).await.map_err(|_| Error::Lost)?;
    let response = stream
        .next()
        .await
        .ok_or(Error::Lost)?
        .map_err(classify_read_error)?;
    validate_binding(&response, &home.full_jid)?;
    Ok(Session {
        cycle: Cycle {
            stream: Some(stream),
            state,
            deadline,
        },
    })
}
fn validate_binding(response: &Element, wanted: &str) -> Result<(), Error> {
    let bind = response.get_child("bind", BIND_NS).ok_or(Error::Bind)?;
    let jid = bind.get_child("jid", BIND_NS).ok_or(Error::Bind)?;
    if !response.is("iq", CLIENT_NS)
        || response.attr("type") != Some("result")
        || response.attr("id") != Some(BIND_ID)
        || response.children().count() != 1
        || !response.text().trim().is_empty()
        || bind.children().count() != 1
        || !bind.text().trim().is_empty()
        || jid.children().next().is_some()
        || jid.text() != wanted
    {
        return Err(Error::Bind);
    }
    Ok(())
}

// Ownership moves into each operation. Dropping a pending operation loses the
// partial stream and leaves the session unusable, instead of reusing partial XML.
struct Cycle<Io> {
    stream: Option<XmlStream<GuardedIo<Io>, Element>>,
    state: SharedGuard,
    deadline: Instant,
}
impl<Io: AsyncRead + AsyncWrite + Unpin> Cycle<Io> {
    async fn send(&mut self, stanza: &Element) -> Result<(), Error> {
        let mut stream = self.stream.take().ok_or(Error::Lost)?;
        if Instant::now() >= self.deadline {
            return Err(Error::Timeout);
        }
        validate_output(stanza)?;
        tokio::time::timeout_at(self.deadline, stream.send(stanza))
            .await
            .map_err(|_| Error::Timeout)?
            .map_err(|_| guard_error(&self.state, Error::Lost))?;
        self.stream = Some(stream);
        Ok(())
    }
    async fn next(&mut self) -> Result<Element, Error> {
        let mut stream = self.stream.take().ok_or(Error::Lost)?;
        if Instant::now() >= self.deadline {
            return Err(Error::Timeout);
        }
        let element = tokio::time::timeout_at(self.deadline, stream.next())
            .await
            .map_err(|_| Error::Timeout)?
            .ok_or(Error::Lost)?
            .map_err(|error| guard_error(&self.state, classify_read_error(error)))?;
        self.stream = Some(stream);
        Ok(element)
    }
}
fn guard_error(state: &SharedGuard, fallback: Error) -> Error {
    match state.borrow().rejection() {
        Some(Rejection::Limit) => Error::XmlLimit,
        Some(Rejection::Malformed) => Error::XmlMalformed,
        None => fallback,
    }
}
/// Check the qualified XSO encoding before any network write. Both standalone
/// and stream-context encodings must fit; this deliberately counts standalone
/// namespace overhead conservatively. Caller-created DOM depth and raw bytes are
/// bounded before encoding, so an oversized text/attribute cannot allocate an
/// unbounded escaped temporary inside the serializer.
pub fn validate_output(stanza: &Element) -> Result<(), Error> {
    use rxml::writer::TrackNamespace;
    use xso::AsXml;
    let mut pending = vec![(stanza, 1)];
    let mut nodes = 0usize;
    let mut raw_bytes = 0usize;
    while let Some((element, depth)) = pending.pop() {
        nodes += 1;
        if depth >= guard::MAX_DEPTH || nodes >= guard::MAX_EVENTS {
            return Err(Error::OutputLimit);
        }
        let mut count = |amount: usize| -> Result<(), Error> {
            raw_bytes = raw_bytes.saturating_add(amount);
            if raw_bytes > MAX_OUTPUT_BYTES {
                Err(Error::OutputLimit)
            } else {
                Ok(())
            }
        };
        count(element.name().len())?;
        count(element.ns().len())?;
        for ((namespace, name), value) in element.attrs() {
            count(namespace.as_str().len())?;
            count(name.len())?;
            count(value.len())?;
        }
        for text in element.texts() {
            count(text.len())?;
        }
        for child in element.children() {
            if pending.len() >= guard::MAX_EVENTS {
                return Err(Error::OutputLimit);
            }
            pending.push((child, depth + 1));
        }
    }
    let mut standalone = rxml::writer::Encoder::new();
    let mut stream = rxml::writer::Encoder::new();
    let stream_ns = "http://etherx.jabber.org/streams";
    stream
        .ns_tracker_mut()
        .declare_fixed(Some(rxml::xml_ncname!("stream")), stream_ns.into());
    stream
        .ns_tracker_mut()
        .declare_fixed(None, CLIENT_NS.into());
    let mut output = rxml::bytes::BytesMut::new();
    stream
        .encode(
            rxml::Item::ElementHeadStart(stream_ns.into(), rxml::xml_ncname!("stream")),
            &mut output,
        )
        .map_err(|_| Error::OutputLimit)?;
    stream
        .encode(rxml::Item::ElementHeadEnd, &mut output)
        .map_err(|_| Error::OutputLimit)?;
    output.clear();
    let mut standalone_bytes = 0usize;
    let mut stream_bytes = 0usize;
    for item in stanza.as_xml_iter().map_err(|_| Error::OutputLimit)? {
        let item = item.map_err(|_| Error::OutputLimit)?;
        standalone
            .encode(item.as_rxml_item(), &mut output)
            .map_err(|_| Error::OutputLimit)?;
        standalone_bytes += output.len();
        output.clear();
        stream
            .encode(item.as_rxml_item(), &mut output)
            .map_err(|_| Error::OutputLimit)?;
        stream_bytes += output.len();
        output.clear();
        if standalone_bytes > MAX_OUTPUT_BYTES || stream_bytes > MAX_OUTPUT_BYTES {
            return Err(Error::OutputLimit);
        }
    }
    Ok(())
}

/// Classify typed read failures without formatting peer-controlled diagnostics.
pub fn classify_read_error(error: tokio_xmpp::xmlstream::ReadError) -> Error {
    use io::ErrorKind;
    use tokio_xmpp::xmlstream::ReadError;
    match error {
        ReadError::StreamFooterReceived => Error::Lost,
        ReadError::SoftTimeout => Error::Timeout,
        ReadError::HardError(error) => match error.kind() {
            ErrorKind::UnexpectedEof
            | ErrorKind::ConnectionReset
            | ErrorKind::ConnectionAborted
            | ErrorKind::BrokenPipe
            | ErrorKind::NotConnected => Error::Lost,
            ErrorKind::TimedOut => Error::Timeout,
            _ if matches!(
                error
                    .get_ref()
                    .and_then(|inner| inner.downcast_ref::<rxml::Error>()),
                Some(rxml::Error::InvalidEof(_))
            ) =>
            {
                Error::Lost
            }
            _ => Error::Protocol,
        },
        ReadError::ParseError(_) => Error::Protocol,
    }
}

#[cfg(test)]
mod session_tests;
