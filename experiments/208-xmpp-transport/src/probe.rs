//! Research-only loopback Direct TLS / SASL PLAIN profile; never opens a vault.
use crate::guard::{Guard, GuardedIo, SharedGuard};
use base64::{engine::general_purpose::STANDARD, Engine};
use futures_util::{SinkExt, StreamExt};
use native_tls::{Certificate, Protocol, TlsConnector};
use sasl::common::Credentials;
use sha2::{Digest, Sha256};
use std::{
    borrow::Cow, cell::RefCell, future::Future, io::Read, path::Path, rc::Rc, time::Duration,
};
use tokio::net::TcpStream;
use tokio_xmpp::{
    minidom::Element,
    xmlstream::{initiate_stream, StreamHeader, Timeouts},
};

pub const LIFETIME: Duration = Duration::from_secs(10);
pub const LOCAL_JID: &str = "alice@home.test/faction208";
pub const PEER_JID: &str = "bob@away.test/device1";
pub const CIPHER_NS: &str = "urn:xmpp:omemo:2";
const CLIENT_NS: &str = "jabber:client";
const BIND_NS: &str = "urn:ietf:params:xml:ns:xmpp-bind";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    Ok,
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
    Lost,
    Timeout,
    Cancelled,
}
impl Status {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Config => "config",
            Self::KeylogRefused => "keylog_refused",
            Self::Connect => "connect",
            Self::Tls => "tls",
            Self::Pin => "pin",
            Self::Auth => "auth",
            Self::Mechanism => "mechanism",
            Self::Bind => "bind",
            Self::Protocol => "protocol",
            Self::XmlLimit => "xml_limit",
            Self::XmlMalformed => "xml_malformed",
            Self::Lost => "lost",
            Self::Timeout => "timeout",
            Self::Cancelled => "cancelled",
        }
    }
}
pub struct Config {
    port: u16,
    ca_path: String,
    pin: [u8; 32],
    password: String,
    payload: String,
}
impl Config {
    pub fn parse(input: &str) -> Result<Self, Status> {
        if input.len() > 8192 || !input.ends_with('\n') {
            return Err(Status::Config);
        }
        let fields: Vec<_> = input.lines().collect();
        if fields.len() != 5 {
            return Err(Status::Config);
        }
        let port: u16 = fields[0].parse().map_err(|_| Status::Config)?;
        if port == 0
            || fields[1].len() > 1024
            || !Path::new(fields[1]).is_absolute()
            || fields[1].chars().any(char::is_control)
        {
            return Err(Status::Config);
        }
        let pin = parse_pin(fields[2])?;
        if fields[3].is_empty() || fields[3].len() > 128 || fields[3].chars().any(char::is_control)
        {
            return Err(Status::Config);
        }
        let decoded = STANDARD.decode(fields[4]).map_err(|_| Status::Config)?;
        if decoded.is_empty() || decoded.len() > 1024 || STANDARD.encode(&decoded) != fields[4] {
            return Err(Status::Config);
        }
        Ok(Self {
            port,
            ca_path: fields[1].into(),
            pin,
            password: fields[3].into(),
            payload: fields[4].into(),
        })
    }
}
fn parse_pin(text: &str) -> Result<[u8; 32], Status> {
    if text.len() != 64 || !text.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(Status::Config);
    }
    let mut result = [0; 32];
    for (i, out) in result.iter_mut().enumerate() {
        *out = u8::from_str_radix(&text[i * 2..i * 2 + 2], 16).map_err(|_| Status::Config)?;
    }
    Ok(result)
}
/// Select drops the complete operation (including its socket) on cancel/deadline.
/// No task, timer worker, reconnect loop, DNS or background client is spawned.
pub async fn bounded<F: Future<Output = Status>>(
    work: F,
    lifetime: Duration,
    cancel: Option<Duration>,
) -> Status {
    tokio::pin!(work);
    let cancellation = async {
        match cancel {
            Some(after) => tokio::time::sleep(after).await,
            None => std::future::pending().await,
        }
    };
    tokio::select! { biased;
        _ = cancellation => Status::Cancelled,
        _ = tokio::time::sleep(lifetime) => Status::Timeout,
        result = &mut work => result,
    }
}
pub struct Report {
    pub status: Status,
    pub attempts: usize,
    pub received_bytes: usize,
    pub events: usize,
}
pub async fn run(config: Config, cancel: Option<Duration>) -> Report {
    let state = Rc::new(RefCell::new(Guard::new()));
    let mut attempts = 0;
    let status = bounded(
        exchange(config, state.clone(), &mut attempts),
        LIFETIME,
        cancel,
    )
    .await;
    let guard = state.borrow();
    let status = match guard.rejection() {
        Some(crate::guard::Rejection::Limit) => Status::XmlLimit,
        Some(crate::guard::Rejection::Malformed) => Status::XmlMalformed,
        None => status,
    };
    Report {
        status,
        attempts,
        received_bytes: guard.bytes(),
        events: guard.events(),
    }
}
fn header() -> StreamHeader<'static> {
    StreamHeader {
        to: Some(Cow::Borrowed("home.test")),
        from: None,
        id: None,
    }
}
async fn exchange(config: Config, state: SharedGuard, attempts: &mut usize) -> Status {
    match exchange_result(config, state, attempts).await {
        Ok(()) => Status::Ok,
        Err(status) => status,
    }
}
async fn exchange_result(
    config: Config,
    state: SharedGuard,
    attempts: &mut usize,
) -> Result<(), Status> {
    // The fixture CA is explicitly supplied; unrelated system roots grant no trust.
    let mut ca = Vec::new();
    std::fs::File::open(&config.ca_path)
        .map_err(|_| Status::Config)?
        .take(65537)
        .read_to_end(&mut ca)
        .map_err(|_| Status::Config)?;
    if ca.len() > 65536 {
        return Err(Status::Config);
    }
    let ca = Certificate::from_pem(&ca).map_err(|_| Status::Config)?;
    let connector = TlsConnector::builder()
        .disable_built_in_roots(true)
        .add_root_certificate(ca)
        .min_protocol_version(Some(Protocol::Tlsv12))
        .build()
        .map_err(|_| Status::Tls)?;
    *attempts = 1;
    let tcp = TcpStream::connect((std::net::Ipv4Addr::LOCALHOST, config.port))
        .await
        .map_err(|_| Status::Connect)?;
    let tls = tokio_native_tls::TlsConnector::from(connector)
        .connect("home.test", tcp)
        .await
        .map_err(|_| Status::Tls)?;
    let certificate = tls
        .get_ref()
        .peer_certificate()
        .map_err(|_| Status::Tls)?
        .ok_or(Status::Tls)?;
    let der = certificate.to_der().map_err(|_| Status::Tls)?;
    if Sha256::digest(der).as_slice() != config.pin {
        return Err(Status::Pin);
    }
    // The guard is inserted after verified TLS and before every XML parser/DOM.
    let (mut io, _) = GuardedIo::new(tls);
    io.replace_state(state.clone());
    let pending = initiate_stream(
        io,
        CLIENT_NS,
        header(),
        Timeouts {
            read_timeout: LIFETIME,
            response_timeout: LIFETIME,
        },
    )
    .await
    .map_err(|_| Status::Lost)?;
    let (features, stream) = pending
        .recv_features()
        .await
        .map_err(|_| Status::Protocol)?;
    if !features.sasl_mechanisms.contains("PLAIN") {
        return Err(Status::Mechanism);
    }
    // Filtering is deliberate: the library's stronger mechanisms are not qualified here.
    let credentials = Credentials::default()
        .with_username("alice")
        .with_password(config.password);
    let restarting = tokio_xmpp::client_login(stream, ["PLAIN".to_string()].into(), credentials)
        .await
        .map_err(|_| Status::Auth)?;
    state
        .borrow_mut()
        .restart()
        .map_err(|_| Status::XmlMalformed)?;
    let pending = restarting
        .send_header(header())
        .await
        .map_err(|_| Status::Lost)?;
    let (features, mut stream) = pending
        .recv_features::<Element>()
        .await
        .map_err(|_| Status::Protocol)?;
    if features.bind.is_none() {
        return Err(Status::Bind);
    }
    let bind = Element::builder("iq", CLIENT_NS)
        .attr(rxml::xml_ncname!("type").to_owned(), "set")
        .attr(rxml::xml_ncname!("id").to_owned(), "bind-208")
        .append(
            Element::builder("bind", BIND_NS)
                .append(
                    Element::builder("resource", BIND_NS)
                        .append("faction208")
                        .build(),
                )
                .build(),
        )
        .build();
    stream.send(&bind).await.map_err(|_| Status::Lost)?;
    let response = stream
        .next()
        .await
        .ok_or(Status::Lost)?
        .map_err(classify_read_error)?;
    let bound = response
        .get_child("bind", BIND_NS)
        .and_then(|bind| bind.get_child("jid", BIND_NS));
    if !response.is("iq", CLIENT_NS)
        || response.attr("type") != Some("result")
        || response.attr("id") != Some("bind-208")
        || bound.map(Element::text).as_deref() != Some(LOCAL_JID)
    {
        return Err(Status::Bind);
    }
    let encrypted = Element::builder("encrypted", CIPHER_NS)
        .append(
            Element::builder("payload", CIPHER_NS)
                .append(config.payload.clone())
                .build(),
        )
        .build();
    let outgoing = Element::builder("message", CLIENT_NS)
        .attr(rxml::xml_ncname!("from").to_owned(), LOCAL_JID)
        .attr(rxml::xml_ncname!("to").to_owned(), PEER_JID)
        .attr(rxml::xml_ncname!("type").to_owned(), "chat")
        .attr(rxml::xml_ncname!("id").to_owned(), "synthetic-208")
        .append(encrypted.clone())
        .build();
    stream.send(&outgoing).await.map_err(|_| Status::Lost)?;
    let inbound = stream
        .next()
        .await
        .ok_or(Status::Lost)?
        .map_err(classify_read_error)?;
    if !inbound.is("message", CLIENT_NS)
        || inbound.attr("from") != Some(PEER_JID)
        || inbound.attr("to") != Some(LOCAL_JID)
        || inbound.attr("type") != Some("chat")
        || inbound.attr("id") != Some("synthetic-208")
        || inbound.children().count() != 1
        || inbound.get_child("encrypted", CIPHER_NS) != Some(&encrypted)
        || !inbound.text().trim().is_empty()
    {
        return Err(Status::Protocol);
    }
    Ok(())
}

/// EOF/reset/closed-stream status is separate from malformed protocol status.
/// No text matching or peer-controlled diagnostic reaches the report.
pub fn classify_read_error(error: tokio_xmpp::xmlstream::ReadError) -> Status {
    use std::io::ErrorKind;
    use tokio_xmpp::xmlstream::ReadError;
    match error {
        ReadError::StreamFooterReceived => Status::Lost,
        ReadError::SoftTimeout => Status::Timeout,
        ReadError::HardError(error) => match error.kind() {
            ErrorKind::UnexpectedEof
            | ErrorKind::ConnectionReset
            | ErrorKind::ConnectionAborted
            | ErrorKind::BrokenPipe
            | ErrorKind::NotConnected => Status::Lost,
            ErrorKind::TimedOut => Status::Timeout,
            _ if matches!(
                error
                    .get_ref()
                    .and_then(|inner| inner.downcast_ref::<rxml::Error>()),
                Some(rxml::Error::InvalidEof(_))
            ) =>
            {
                Status::Lost
            }
            _ => Status::Protocol,
        },
        ReadError::ParseError(_) => Status::Protocol,
    }
}
