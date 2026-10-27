//! Lab-only compatibility driver for the qualified #208 TLS peer.
//! Synthetic transport bytes are not the product OMEMO codec or encrypted-mail proof.
use base64::{Engine, engine::general_purpose::STANDARD};
use faction_xmpp::{Element, Error, Home, Session, connect};
use std::{io::Read, path::Path, time::Duration};
const LOCAL_JID: &str = "alice@home.test/faction208";
const PEER_JID: &str = "bob@away.test/device1";
const CLIENT_NS: &str = "jabber:client";
const CIPHER_NS: &str = "urn:xmpp:omemo:2";

fn status(error: Error) -> &'static str {
    match error {
        Error::Config => "config",
        Error::KeylogRefused => "keylog_refused",
        Error::Connect => "connect",
        Error::Tls => "tls",
        Error::Pin => "pin",
        Error::Auth => "auth",
        Error::Mechanism => "mechanism",
        Error::Bind => "bind",
        Error::Protocol => "protocol",
        Error::XmlLimit => "xml_limit",
        Error::XmlMalformed => "xml_malformed",
        Error::OutputLimit => "output_limit",
        Error::Lost => "lost",
        Error::Timeout => "timeout",
    }
}
fn arguments() -> Result<Option<Duration>, Error> {
    let args: Vec<_> = std::env::args_os()
        .skip(1)
        .take(4)
        .map(|arg| arg.into_string().map_err(|_| Error::Config))
        .collect::<Result<_, _>>()?;
    match args.as_slice() {
        [command] if command == "probe" => Ok(None),
        [command, flag, value] if command == "probe" && flag == "--cancel-after-ms" => {
            let value: u64 = value.parse().map_err(|_| Error::Config)?;
            if value > 10000 {
                return Err(Error::Config);
            }
            Ok(Some(Duration::from_millis(value)))
        }
        _ => Err(Error::Config),
    }
}
fn configuration(input: &str) -> Result<(Home, &str, &str), Error> {
    if input.len() > 8192 || !input.ends_with('\n') {
        return Err(Error::Config);
    }
    let fields: Vec<_> = input.lines().collect();
    if fields.len() != 5 {
        return Err(Error::Config);
    }
    let port: u16 = fields[0].parse().map_err(|_| Error::Config)?;
    if port == 0
        || !Path::new(fields[1]).is_absolute()
        || fields[1].len() > 1024
        || fields[1].chars().any(char::is_control)
        || fields[2].len() != 64
        || !fields[2].bytes().all(|b| b.is_ascii_hexdigit())
        || fields[3].is_empty()
        || fields[3].len() > 128
        || fields[3].chars().any(char::is_control)
    {
        return Err(Error::Config);
    }
    let mut pin = [0; 32];
    for (i, byte) in pin.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&fields[2][i * 2..i * 2 + 2], 16).map_err(|_| Error::Config)?;
    }
    let payload = STANDARD.decode(fields[4]).map_err(|_| Error::Config)?;
    if payload.is_empty() || payload.len() > 1024 || STANDARD.encode(&payload) != fields[4] {
        return Err(Error::Config);
    }
    // Only trusted generated regular lab files. Product private-file validation
    // belongs to the native client and is not qualified by this example.
    let metadata = std::fs::symlink_metadata(fields[1]).map_err(|_| Error::Config)?;
    if !metadata.is_file() || metadata.len() > 65536 {
        return Err(Error::Config);
    }
    let mut ca = Vec::new();
    std::fs::File::open(fields[1])
        .map_err(|_| Error::Config)?
        .take(65537)
        .read_to_end(&mut ca)
        .map_err(|_| Error::Config)?;
    if ca.len() > 65536 {
        return Err(Error::Config);
    }
    let home = Home::new(
        ([127, 0, 0, 1], port).into(),
        "home.test",
        LOCAL_JID,
        &ca,
        pin,
    )?;
    Ok((home, fields[3], fields[4]))
}
async fn exchange(session: &mut Session, payload: &str) -> Result<(), Error> {
    let encrypted = Element::builder("encrypted", CIPHER_NS)
        .append(
            Element::builder("payload", CIPHER_NS)
                .append(payload)
                .build(),
        )
        .build();
    let outgoing = Element::builder("message", CLIENT_NS)
        .attr("from".try_into().unwrap(), LOCAL_JID)
        .attr("to".try_into().unwrap(), PEER_JID)
        .attr("type".try_into().unwrap(), "chat")
        .attr("id".try_into().unwrap(), "synthetic-208")
        .append(encrypted.clone())
        .build();
    session.send(&outgoing).await?;
    let inbound = session.next().await?;
    if !inbound.is("message", CLIENT_NS)
        || inbound.attr("from") != Some(PEER_JID)
        || inbound.attr("to") != Some(LOCAL_JID)
        || inbound.attr("type") != Some("chat")
        || inbound.attr("id") != Some("synthetic-208")
        || inbound.children().count() != 1
        || inbound.get_child("encrypted", CIPHER_NS) != Some(&encrypted)
        || !inbound.text().trim().is_empty()
    {
        return Err(Error::Protocol);
    }
    Ok(())
}
fn run() -> Result<(&'static str, usize), Error> {
    let cancel = arguments()?;
    if ["SSLKEYLOGFILE", "TLSKEYLOGFILE"]
        .iter()
        .any(|key| std::env::var_os(key).is_some())
    {
        return Err(Error::KeylogRefused);
    }
    let mut input = String::new();
    std::io::stdin()
        .take(8193)
        .read_to_string(&mut input)
        .map_err(|_| Error::Config)?;
    let (home, password, payload) = configuration(&input)?;
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|_| Error::Config)?;
    let outcome = runtime.block_on(async {
        let work = async {
            let mut session = connect(&home, password).await?;
            exchange(&mut session, payload).await
        };
        let cancellation = async {
            match cancel {
                Some(after) => tokio::time::sleep(after).await,
                None => std::future::pending().await,
            }
        };
        tokio::select! { biased;
            _ = cancellation => "cancelled",
            result = work => match result { Ok(()) => "ok", Err(error) => status(error) },
        }
    });
    Ok((outcome, 1))
}
fn main() {
    let (status, attempts) = run().unwrap_or_else(|error| (status(error), 0));
    println!("{{\"status\":\"{status}\",\"attempts\":{attempts}}}");
    if status != "ok" {
        std::process::exit(1);
    }
}
