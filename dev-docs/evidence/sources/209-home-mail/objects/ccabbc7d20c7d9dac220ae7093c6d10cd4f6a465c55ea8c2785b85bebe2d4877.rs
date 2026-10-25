//! Explicit bounded member-to-home cycles. No background or remote unlock entry.
use crate::{
    mail_checkpoint::CheckpointVault,
    mail_data::{address, invalid, unhex},
};
use rustix::fs::{self, Mode as FsMode, OFlags};
use std::{
    fs::File,
    io::{self, Read},
    net::SocketAddr,
    os::unix::fs::MetadataExt,
    path::{Component, Path},
};
/// Trusted local configuration. Home passwords and device secrets have no field.
pub struct Settings {
    pub account: String,
    endpoint: SocketAddr,
    domain: String,
    ca: Vec<u8>,
    pin: [u8; 32],
}
fn private_file(path: &Path, cap: u64) -> io::Result<Vec<u8>> {
    if !path.is_absolute()
        || path
            .components()
            .any(|p| matches!(p, Component::ParentDir | Component::CurDir))
    {
        return Err(invalid());
    }
    let mut dir = File::from(fs::open(
        "/",
        OFlags::RDONLY | OFlags::DIRECTORY | OFlags::CLOEXEC,
        FsMode::empty(),
    )?);
    let components: Vec<_> = path
        .components()
        .filter_map(|p| {
            if let Component::Normal(n) = p {
                Some(n)
            } else {
                None
            }
        })
        .collect();
    let (name, parents) = components.split_last().ok_or_else(invalid)?;
    for parent in parents {
        dir = File::from(fs::openat(
            &dir,
            *parent,
            OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
            FsMode::empty(),
        )?);
    }
    let m = dir.metadata()?;
    if m.uid() != rustix::process::geteuid().as_raw() || m.mode() & 0o7777 != 0o700 {
        return Err(invalid());
    }
    let file = File::from(fs::openat(
        &dir,
        *name,
        OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC,
        FsMode::empty(),
    )?);
    let m = file.metadata()?;
    if !m.is_file()
        || m.nlink() != 1
        || m.uid() != rustix::process::geteuid().as_raw()
        || m.mode() & 0o7777 != 0o600
        || m.len() > cap
    {
        return Err(invalid());
    }
    let mut bytes = Vec::new();
    file.take(cap + 1).read_to_end(&mut bytes)?;
    if bytes.is_empty() || bytes.len() as u64 > cap {
        return Err(invalid());
    }
    Ok(bytes)
}
impl Settings {
    pub fn load(path: &Path) -> io::Result<Self> {
        let bytes = private_file(path, 8192)?;
        let text = std::str::from_utf8(&bytes).map_err(|_| invalid())?;
        let mut table = text.parse::<toml::Table>().map_err(|_| invalid())?;
        if table.remove("version").and_then(|x| x.as_integer()) != Some(1) {
            return Err(invalid());
        }
        let mut take = |name: &str| {
            table
                .remove(name)
                .and_then(|x| x.as_str().map(str::to_owned))
                .ok_or_else(invalid)
        };
        let account = take("account")?;
        let endpoint = take("endpoint")?
            .parse::<SocketAddr>()
            .map_err(|_| invalid())?;
        let domain = take("domain")?;
        let ca = take("ca")?;
        let pin = unhex(&take("pin")?)?;
        if !table.is_empty()
            || !address(&account)
            || account.split_once('@').map(|x| x.1) != Some(domain.as_str())
            || endpoint.port() == 0
            || endpoint.ip().is_unspecified()
            || endpoint.ip().is_multicast()
        {
            return Err(invalid());
        }
        let ca = private_file(Path::new(&ca), 65536)?;
        Ok(Self {
            account,
            endpoint,
            domain,
            ca,
            pin,
        })
    }
    pub fn transport(&self, device: u32) -> Result<faction_xmpp::Home, faction_xmpp::Error> {
        faction_xmpp::Home::new(
            self.endpoint,
            &self.domain,
            &format!("{}/faction{}", self.account, device),
            &self.ca,
            self.pin,
        )
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Mode {
    Publish,
    Prepare,
    Sync,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    Configuration,
    Account,
    State,
    Protocol,
    PeerUnavailable,
    Transport(faction_xmpp::Error),
}
#[derive(Default, Debug)]
pub struct Report {
    pub sent: usize,
    pub accepted: usize,
    pub duplicates: usize,
}
fn backend<T>(r: io::Result<T>) -> Result<T, Error> {
    r.map_err(|_| Error::State)
}
fn protocol<T>(r: io::Result<T>) -> Result<T, Error> {
    r.map_err(|_| Error::Protocol)
}
fn local_response(
    xml: &faction_xmpp::Element,
    id: &str,
    settings: &Settings,
) -> Result<faction_xmpp::Element, Error> {
    if !xml.is("iq", "jabber:client") || xml.attr("id") != Some(id) {
        return Err(Error::Protocol);
    }
    let own = |s: &str| s.split('/').next() == Some(settings.account.as_str());
    if xml.attr("to").is_some_and(|s| !own(s))
        || xml
            .attr("from")
            .is_some_and(|s| !own(s) && s != settings.domain)
    {
        return Err(Error::Protocol);
    }
    let mut result = xml.clone();
    result.set_attr(
        "".into(),
        "from".try_into().map_err(|_| Error::Protocol)?,
        settings.account.as_str(),
    );
    if result.attr("to").is_none() {
        result.set_attr(
            "".into(),
            "to".try_into().map_err(|_| Error::Protocol)?,
            settings.account.as_str(),
        );
    }
    Ok(result)
}
fn accept(
    vault: &mut CheckpointVault,
    xml: &faction_xmpp::Element,
    now: u64,
    report: &mut Report,
) -> Result<(), Error> {
    if xml.is("presence", "jabber:client") {
        return Ok(());
    }
    if !xml.is("message", "jabber:client") {
        return Ok(());
    }
    if xml
        .get_child("event", "http://jabber.org/protocol/pubsub#event")
        .is_some()
    {
        return Ok(());
    }
    let state = backend(vault.inspection())?;
    let peer = state.peer.ok_or(Error::State)?;
    let binding = crate::mail_wire::Binding {
        peer: peer.address,
        peer_device: peer.device,
        peer_identity: *peer.identity,
        own: backend(vault.legacy())?.home.as_str(),
        own_device: state.local_device,
    };
    let wire = protocol(crate::mail_wire::decode_message(xml, binding))?;
    let receipt = backend(vault.receive_network(&wire, now))?;
    if receipt.duplicate {
        report.duplicates += 1;
    } else {
        report.accepted += 1;
    }
    Ok(())
}
// Kept internal so deterministic tests can exercise real checkpoint mutations
// interleaved with matching IQ replies without weakening the transport boundary.
trait Channel {
    async fn send(&mut self, xml: &faction_xmpp::Element) -> Result<(), faction_xmpp::Error>;
    async fn next(&mut self) -> Result<faction_xmpp::Element, faction_xmpp::Error>;
}
impl Channel for faction_xmpp::Session {
    async fn send(&mut self, xml: &faction_xmpp::Element) -> Result<(), faction_xmpp::Error> {
        faction_xmpp::Session::send(self, xml).await
    }
    async fn next(&mut self) -> Result<faction_xmpp::Element, faction_xmpp::Error> {
        faction_xmpp::Session::next(self).await
    }
}
struct Ready {
    device: u32,
    peer: Option<(String, u32, [u8; 32])>,
    bundle: faction_omemo::Bundle,
    prekeys: Vec<(u32, [u8; 32])>,
}
async fn request<C: Channel>(
    session: &mut C,
    vault: &mut CheckpointVault,
    xml: faction_xmpp::Element,
    id: &str,
    now: u64,
    report: &mut Report,
) -> Result<faction_xmpp::Element, Error> {
    session.send(&xml).await.map_err(Error::Transport)?;
    for _ in 0..32 {
        let response = session.next().await.map_err(Error::Transport)?;
        if response.is("iq", "jabber:client") && response.attr("id") == Some(id) {
            return Ok(response);
        }
        accept(vault, &response, now, report)?;
    }
    Err(Error::Protocol)
}
fn preflight(
    vault: &CheckpointVault,
    settings: &Settings,
    mode: Mode,
    now: u64,
) -> Result<Ready, Error> {
    if backend(vault.legacy())?.home != settings.account {
        return Err(Error::Account);
    }
    let state = backend(vault.inspection())?;
    if state.history_only
        || now < state.created
        || now > state.expires
        || state.peer.as_ref().is_some_and(|p| p.revoked)
    {
        return Err(Error::State);
    }
    let device = state.local_device;
    let started = state.started;
    let peer = state
        .peer
        .map(|p| (p.address.to_owned(), p.device, *p.identity));
    if mode != Mode::Publish && peer.is_none() || mode == Mode::Prepare && started {
        return Err(Error::State);
    }
    let (bundle, prekeys) = backend(vault.publication(now))?;
    Ok(Ready {
        device,
        peer,
        bundle,
        prekeys,
    })
}
/// One deliberate connection/cycle. A write never retires pending ciphertext.
/// All key generation and accepted plaintext remain in the member checkpoint.
pub async fn cycle(
    vault: &mut CheckpointVault,
    settings: &Settings,
    password: &str,
    mode: Mode,
    now: u64,
) -> Result<Report, Error> {
    let ready = preflight(vault, settings, mode, now)?;
    let home = settings.transport(ready.device).map_err(Error::Transport)?;
    let mut session = faction_xmpp::connect(&home, password)
        .await
        .map_err(Error::Transport)?;
    exchange(vault, settings, &mut session, mode, now, ready).await
}
async fn exchange<C: Channel>(
    vault: &mut CheckpointVault,
    settings: &Settings,
    session: &mut C,
    mode: Mode,
    now: u64,
    ready: Ready,
) -> Result<Report, Error> {
    let Ready {
        device,
        peer,
        bundle,
        prekeys,
    } = ready;
    let clock = std::time::Instant::now();
    let mut report = Report::default();
    let own_query = protocol(crate::mail_wire::devices_query(
        "own-devices",
        &settings.account,
        &settings.account,
    ))?;
    let own = request(session, vault, own_query, "own-devices", now, &mut report).await?;
    let own = local_response(&own, "own-devices", settings)?;
    let current = if own.attr("type") == Some("error") {
        let error = own
            .get_child("error", "jabber:client")
            .ok_or(Error::Protocol)?;
        if error
            .get_child("item-not-found", "urn:ietf:params:xml:ns:xmpp-stanzas")
            .is_none()
        {
            return Err(Error::Protocol);
        }
        Vec::new()
    } else {
        protocol(crate::mail_wire::devices_result(
            &own,
            "own-devices",
            &settings.account,
            &settings.account,
        ))?
    };
    if current.iter().any(|&id| id != device) {
        return Err(Error::State);
    }
    for (id, publication) in [
        (
            "pub-bundle",
            protocol(crate::mail_wire::bundle_publish(
                "pub-bundle",
                &settings.account,
                device,
                &bundle,
                &prekeys,
            ))?,
        ),
        (
            "pub-devices",
            protocol(crate::mail_wire::devices_publish(
                "pub-devices",
                &settings.account,
                &[device],
            ))?,
        ),
    ] {
        let response = request(
            session,
            vault,
            publication,
            id,
            now + clock.elapsed().as_secs(),
            &mut report,
        )
        .await?;
        if local_response(&response, id, settings)?.attr("type") != Some("result") {
            return Err(Error::Protocol);
        }
    }
    session
        .send(&faction_xmpp::Element::builder("presence", "jabber:client").build())
        .await
        .map_err(Error::Transport)?;
    if mode == Mode::Publish {
        return Ok(report);
    }
    let (address, peer_device, identity) = peer.ok_or(Error::State)?;
    let query = protocol(crate::mail_wire::devices_query(
        "peer-devices",
        &settings.account,
        &address,
    ))?;
    let response = request(
        session,
        vault,
        query,
        "peer-devices",
        now + clock.elapsed().as_secs(),
        &mut report,
    )
    .await?;
    let devices =
        crate::mail_wire::devices_result(&response, "peer-devices", &settings.account, &address)
            .map_err(|_| Error::PeerUnavailable)?;
    if !devices.contains(&peer_device) {
        return Err(Error::PeerUnavailable);
    }
    let query = protocol(crate::mail_wire::bundle_query(
        "peer-bundle",
        &settings.account,
        &address,
        peer_device,
    ))?;
    let response = request(
        session,
        vault,
        query,
        "peer-bundle",
        now + clock.elapsed().as_secs(),
        &mut report,
    )
    .await?;
    let peer_bundle = protocol(crate::mail_wire::bundle_result(
        &response,
        "peer-bundle",
        &settings.account,
        &address,
        peer_device,
        &identity,
    ))?;
    if mode == Mode::Prepare {
        backend(vault.start(&peer_bundle, now + clock.elapsed().as_secs()))?;
        return Ok(report);
    }
    for delivery in backend(vault.pending_deliveries(now + clock.elapsed().as_secs()))? {
        let xml = protocol(crate::mail_wire::message(
            &delivery.wire,
            delivery.operation,
        ))?;
        session.send(&xml).await.map_err(Error::Transport)?;
        report.sent += 1;
    }
    for _ in 0..32 {
        match tokio::time::timeout(std::time::Duration::from_secs(2), session.next()).await {
            Err(_) => return Ok(report),
            Ok(Err(error)) => return Err(Error::Transport(error)),
            Ok(Ok(xml)) => accept(vault, &xml, now + clock.elapsed().as_secs(), &mut report)?,
        }
    }
    Err(Error::Protocol)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        fs,
        os::unix::fs::{PermissionsExt, symlink},
    };
    struct Owned(std::path::PathBuf);
    impl Owned {
        fn new() -> Self {
            let mut id = [0; 16];
            getrandom::fill(&mut id).unwrap();
            let path = std::env::temp_dir()
                .canonicalize()
                .unwrap()
                .join(format!("faction209-config-{}", crate::mail_data::hex(&id)));
            fs::create_dir(&path).unwrap();
            fs::set_permissions(&path, fs::Permissions::from_mode(0o700)).unwrap();
            Self(path)
        }
        fn files(&self) -> std::path::PathBuf {
            let ca = self.0.join("ca.pem");
            fs::write(&ca, b"generated public fixture CA placeholder").unwrap();
            fs::set_permissions(&ca, fs::Permissions::from_mode(0o600)).unwrap();
            let path = self.0.join("home.toml");
            fs::write(&path,format!("version=1\naccount='alice@home.test'\nendpoint='127.0.0.1:5223'\ndomain='home.test'\nca='{}'\npin='{}'\n",ca.display(),"12".repeat(32))).unwrap();
            fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
            path
        }
    }
    impl Drop for Owned {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    #[test]
    fn foreign_account_and_expired_state_cannot_open_a_home_connection() {
        let owned = Owned::new();
        let path = owned.files();
        let mut settings = Settings::load(&path).unwrap();
        let store = owned.0.join("store");
        let t = 1_792_886_400;
        let (mut vault, _) = CheckpointVault::create(
            crate::mail_vault::Store::at(&store, true).unwrap(),
            crate::mail_data::Data::new("alice@home.test".into()).unwrap(),
            b"synthetic private passphrase",
            101,
            t,
        )
        .unwrap();
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        settings.account = "mallory@home.test".into();
        assert!(matches!(
            runtime.block_on(cycle(
                &mut vault,
                &settings,
                "synthetic-home-password",
                Mode::Publish,
                t
            )),
            Err(Error::Account)
        ));
        settings.account = "alice@home.test".into();
        assert!(matches!(
            runtime.block_on(cycle(
                &mut vault,
                &settings,
                "synthetic-home-password",
                Mode::Publish,
                t + crate::mail_checkpoint::SESSION_AGE + 1
            )),
            Err(Error::State)
        ));
    }
    #[test]
    fn private_regular_local_configuration_is_preloaded() {
        let owned = Owned::new();
        assert!(Settings::load(&owned.files()).is_ok());
    }
    #[test]
    fn symlinks_hardlinks_fifo_and_public_files_are_refused() {
        let owned = Owned::new();
        let path = owned.files();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).unwrap();
        assert!(Settings::load(&path).is_err());
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
        let link = owned.0.join("link");
        symlink(&path, &link).unwrap();
        assert!(Settings::load(&link).is_err());
        let hard = owned.0.join("hard");
        fs::hard_link(&path, &hard).unwrap();
        assert!(Settings::load(&path).is_err());
        fs::remove_file(hard).unwrap();
        fs::remove_file(owned.0.join("ca.pem")).unwrap();
        assert!(
            std::process::Command::new("mkfifo")
                .arg(owned.0.join("ca.pem"))
                .status()
                .unwrap()
                .success()
        );
        assert!(Settings::load(&path).is_err());
    }
    #[test]
    fn unknown_fields_credentials_and_non_numeric_endpoints_are_rejected() {
        let owned = Owned::new();
        let path = owned.files();
        let good = fs::read_to_string(&path).unwrap();
        for text in [
            format!("{good}password='forbidden'\n"),
            good.replace("127.0.0.1:5223", "home.test:5223"),
            good.replace("alice@home.test", "alice@elsewhere.test"),
        ] {
            fs::write(&path, text).unwrap();
            assert!(Settings::load(&path).is_err());
        }
    }
}
