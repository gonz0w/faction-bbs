//! Lab-only read-only peer-device IQ metadata; never prints text or key material.
use faction_client::mail_network::Settings;
use faction_xmpp::{Element, connect};
use std::{io::Read, path::Path};
use zeroize::Zeroizing;
const PEER: &str = "bob@homeb.test";
fn shape(xml: &Element, depth: usize) {
    if depth > 12 { return; }
    let names: Vec<_> = xml.attrs().iter().map(|((ns, name), _)| format!("{}:{}", ns.as_str(), name.as_str())).collect();
    println!("element depth={depth} name={:?} ns={:?} attribute_names={names:?}", xml.name(), xml.ns());
    if let Some(node) = xml.attr("node") {
        let known = if node == "urn:xmpp:omemo:2:devices" { node } else { "[other]" };
        println!("node={known:?}");
    }
    if xml.name() == "item" {
        let id = xml.attr("id").filter(|id| *id == "current" || id.parse::<u32>().is_ok());
        println!("item_id={id:?} publisher_present={} publisher_matches_peer={}", xml.attr("publisher").is_some(), xml.attr("publisher").is_some_and(|value| value.split('/').next() == Some(PEER)));
    }
    for child in xml.children() { shape(child, depth + 1); }
}
fn run() -> Result<(), ()> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 1 { return Err(()); }
    let settings = Settings::load(Path::new(&args[0])).map_err(|_| ())?;
    let mut password = Zeroizing::new(String::new());
    std::io::stdin().take(130).read_to_string(&mut password).map_err(|_| ())?;
    if !password.ends_with('\n') || password.lines().count() != 1 { return Err(()); }
    let password = password.strip_suffix('\n').ok_or(())?;
    let home = settings.transport(2101).map_err(|_| ())?;
    let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().map_err(|_| ())?;
    runtime.block_on(async {
        let mut session = connect(&home, password).await.map_err(|_| ())?;
        let iq = faction_client::mail_wire::devices_query("diagnose-devices-209", &settings.account, PEER).map_err(|_| ())?;
        session.send(&iq).await.map_err(|_| ())?;
        for _ in 0..32 {
            let xml = session.next().await.map_err(|_| ())?;
            if xml.is("iq", "jabber:client") && xml.attr("id") == Some("diagnose-devices-209") {
                println!("iq id={:?} type={:?} from={:?} to={:?}", xml.attr("id"), xml.attr("type"), xml.attr("from"), xml.attr("to"));
                shape(&xml, 1);
                return Ok(());
            }
        }
        Err(())
    })
}
fn main() { if run().is_err() { eprintln!("diagnostic unavailable"); std::process::exit(1); } }
