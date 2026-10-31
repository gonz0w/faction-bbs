//! Explicit complete composed policy and fixed private relay configuration.
//! No endpoint, role or credential comes from a device request or locator.
use super::{Result, input, mapped};
#[path = "composed_loop.rs"]
mod composed_loop;
use faction_files::private_blobs::Store;
use faction_provider::Error as ProviderError;
use faction_provider::{
    Runtime,
    authority::Registry,
    delivery::{CarrierEndpoint, Delivery, Event, RelayPeer, RelayRoutes},
    services::{
        IssuerRoute, PublicationAssociation, PublicationWireProfile, ResourceAssociation,
        ServicePolicy, ServiceRegistration, ServiceUse,
    },
    tls::{ClientCredentials, HttpEndpoint, Server},
};
use input::{boolean, digest, number, string};
use openssl::{pkey::PKey, x509::X509};
use std::{
    ffi::OsString,
    io::{self, Write},
    net::{SocketAddr, TcpListener},
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::Duration,
};

fn tables(t: &mut toml::Table, field: &str) -> Result<Vec<toml::Table>> {
    let values = t
        .remove(field)
        .and_then(|v| v.as_array().cloned())
        .ok_or(())?;
    if values.is_empty() || values.len() > 32 {
        return Err(());
    }
    values
        .into_iter()
        .map(|v| v.as_table().cloned().ok_or(()))
        .collect()
}
fn strings(t: &mut toml::Table, field: &str) -> Result<Vec<String>> {
    let values = t
        .remove(field)
        .and_then(|v| v.as_array().cloned())
        .ok_or(())?;
    if values.is_empty() || values.len() > 32 {
        return Err(());
    }
    values
        .into_iter()
        .map(|v| v.as_str().map(str::to_owned).ok_or(()))
        .collect()
}
fn exact(t: toml::Table) -> Result<()> {
    if t.is_empty() { Ok(()) } else { Err(()) }
}
fn address(t: &mut toml::Table) -> Result<SocketAddr> {
    let address = mapped(string(t, "address")?.parse::<SocketAddr>())?;
    if !address.ip().is_loopback() || address.port() == 0 {
        return Err(());
    }
    Ok(address)
}
fn private_path(t: &mut toml::Table, field: &str, root: &Path) -> Result<PathBuf> {
    let path = PathBuf::from(string(t, field)?);
    if !input::absolute(&path) || !path.starts_with(root) || path == root {
        return Err(());
    }
    Ok(path)
}
struct Endpoint {
    address: SocketAddr,
    host: String,
    server_name: String,
    leaf_pin: [u8; 32],
    ca: PathBuf,
    certificate: PathBuf,
    key: PathBuf,
}
impl Endpoint {
    fn parse(mut t: toml::Table, root: &Path) -> Result<Self> {
        let result = Self {
            address: address(&mut t)?,
            host: string(&mut t, "http_host")?,
            server_name: string(&mut t, "server_name")?,
            leaf_pin: digest(&string(&mut t, "leaf_pin")?)?,
            ca: private_path(&mut t, "ca", root)?,
            certificate: private_path(&mut t, "certificate", root)?,
            key: private_path(&mut t, "key", root)?,
        };
        exact(t)?;
        Ok(result)
    }
    fn credentials(&self, root: &Path) -> Result<ClientCredentials> {
        let ca = pem(&self.ca, root)?;
        let certificate = pem(&self.certificate, root)?;
        let key = input::private_file(&self.key, root, 65_536)?;
        mapped(ClientCredentials::from_pem(
            &mapped(ca.to_pem())?,
            &mapped(certificate.to_pem())?,
            &key,
        ))
    }
    fn http(&self, root: &Path) -> Result<HttpEndpoint> {
        mapped(HttpEndpoint::new(
            self.address,
            self.host.clone(),
            self.server_name.clone(),
            self.leaf_pin,
            self.credentials(root)?,
        ))
    }
    fn carrier(&self, root: &Path) -> Result<CarrierEndpoint> {
        // Carrier has no HTTP host field. Require the explicit names to agree.
        if self.host != self.server_name {
            return Err(());
        }
        mapped(CarrierEndpoint::new(
            self.address,
            self.server_name.clone(),
            self.leaf_pin,
            self.credentials(root)?,
        ))
    }
}
fn endpoint(t: &mut toml::Table, field: &str, root: &Path) -> Result<Endpoint> {
    Endpoint::parse(
        t.remove(field)
            .and_then(|v| v.as_table().cloned())
            .ok_or(())?,
        root,
    )
}
fn pem(path: &Path, root: &Path) -> Result<X509> {
    let certs = mapped(X509::stack_from_pem(&input::private_file(
        path, root, 65_536,
    )?))?;
    if certs.len() != 1 {
        return Err(());
    }
    Ok(certs[0].clone())
}
struct Peer {
    home: String,
    publication: Endpoint,
    introspection: Endpoint,
    resource: Endpoint,
}
struct Config {
    base: input::Config,
    client_cas: Vec<PathBuf>,
    publication_clients: Vec<String>,
    local: Endpoint,
    peers: Vec<Peer>,
}
impl Config {
    fn load(path: &Path) -> Result<Self> {
        let root = path.parent().ok_or(())?;
        let mut t = input::table(path, root, 65_536)?;
        if number(&mut t, "version")? != 2 {
            return Err(());
        }
        let client_cas = strings(&mut t, "client_cas")?
            .into_iter()
            .map(PathBuf::from)
            .collect::<Vec<_>>();
        for path in &client_cas {
            if !input::absolute(path) || !path.starts_with(root) || path == root {
                return Err(());
            }
        }
        let publication_clients = strings(&mut t, "publication_clients")?;
        let local = endpoint(&mut t, "local_relay", root)?;
        let peers = tables(&mut t, "peers")?
            .into_iter()
            .map(|mut p| {
                let result = Peer {
                    home: string(&mut p, "home")?,
                    publication: endpoint(&mut p, "publication", root)?,
                    introspection: endpoint(&mut p, "introspection", root)?,
                    resource: endpoint(&mut p, "resource", root)?,
                };
                exact(p)?;
                Ok(result)
            })
            .collect::<Result<Vec<_>>>()?;
        t.insert("version".into(), toml::Value::Integer(1));
        let base = input::Config::from_table(root.to_path_buf(), t)?;
        mapped(PublicationWireProfile::new(
            base.home.clone(),
            publication_clients.clone(),
        ))?;
        if peers
            .iter()
            .enumerate()
            .any(|(i, p)| p.home == base.home || peers[..i].iter().any(|old| old.home == p.home))
        {
            return Err(());
        }
        Ok(Self {
            base,
            client_cas,
            publication_clients,
            local,
            peers,
        })
    }
}
/// Complete version 2 public proposal. Legacy version 1 parsing is independent.
fn proposal(path: &Path, config: &Config) -> Result<(Registry, ServicePolicy)> {
    let bytes = input::private_file(path, &config.base.root, 65_536)?;
    let mut t = mapped(mapped(std::str::from_utf8(&bytes))?.parse::<toml::Table>())?;
    if number(&mut t, "version")? != 2 {
        return Err(());
    }
    let home = string(&mut t, "local_home")?;
    if home != config.base.home {
        return Err(());
    }
    let issuer = string(&mut t, "issuer")?;
    let services = tables(&mut t, "services")?
        .into_iter()
        .map(|mut s| {
            let purpose = match string(&mut s, "purpose")?.as_str() {
                "observation" => ServiceUse::Observation,
                "relay" => ServiceUse::Relay,
                "introspection" => ServiceUse::Introspection,
                "resource-client" => ServiceUse::ResourceClient,
                "outbound-introspection" => ServiceUse::OutboundIntrospection,
                "issuer-endpoint" => ServiceUse::IssuerEndpoint,
                "outbound-observation" => ServiceUse::OutboundObservation,
                _ => return Err(()),
            };
            let entry = ServiceRegistration {
                home: string(&mut s, "home")?,
                client_id: string(&mut s, "client_id")?,
                dns_san: string(&mut s, "dns_san")?,
                certificate_sha256: digest(&string(&mut s, "certificate_sha256")?)?,
                enabled: boolean(&mut s, "enabled")?,
                valid_from: number(&mut s, "valid_from")?,
                valid_until: number(&mut s, "valid_until")?,
            };
            exact(s)?;
            Ok((entry, purpose))
        })
        .collect::<Result<Vec<_>>>()?;
    let routes = tables(&mut t, "issuer_routes")?
        .into_iter()
        .map(|mut r| {
            let result = IssuerRoute {
                sender_home: string(&mut r, "sender_home")?,
                audience: string(&mut r, "audience")?,
                relay_client_id: string(&mut r, "relay_client_id")?,
                introspection_client_id: string(&mut r, "introspection_client_id")?,
            };
            exact(r)?;
            Ok(result)
        })
        .collect::<Result<Vec<_>>>()?;
    let resources = tables(&mut t, "resources")?
        .into_iter()
        .map(|mut r| {
            let result = ResourceAssociation {
                recipient_home: string(&mut r, "recipient_home")?,
                issuer: string(&mut r, "issuer")?,
                audience: string(&mut r, "audience")?,
                relay_client_id: string(&mut r, "relay_client_id")?,
                outbound_client_id: string(&mut r, "outbound_client_id")?,
                endpoint_id: string(&mut r, "endpoint_id")?,
                address: address(&mut r)?,
                http_host: string(&mut r, "http_host")?,
                ca_sha256: digest(&string(&mut r, "ca_sha256")?)?,
                remote_generation: number(&mut r, "remote_generation")?,
                remote_reset: number(&mut r, "remote_reset")?,
            };
            exact(r)?;
            Ok(result)
        })
        .collect::<Result<Vec<_>>>()?;
    let publications = tables(&mut t, "publications")?
        .into_iter()
        .map(|mut r| {
            let result = PublicationAssociation {
                recipient_home: string(&mut r, "recipient_home")?,
                outbound_client_id: string(&mut r, "outbound_client_id")?,
                endpoint_id: string(&mut r, "endpoint_id")?,
                address: address(&mut r)?,
                http_host: string(&mut r, "http_host")?,
                ca_sha256: digest(&string(&mut r, "ca_sha256")?)?,
                remote_generation: number(&mut r, "remote_generation")?,
                remote_reset: number(&mut r, "remote_reset")?,
            };
            exact(r)?;
            Ok(result)
        })
        .collect::<Result<Vec<_>>>()?;
    t.insert("version".into(), toml::Value::Integer(1));
    let registry = input::proposal_bytes(mapped(toml::to_string(&t))?.as_bytes())?;
    // Local device actors remain local; invitations belong in the complete registry.
    if registry
        .bindings()
        .iter()
        .any(|b| b.principal.account.split_once('@').map(|(_, h)| h) != Some(home.as_str()))
    {
        return Err(());
    }
    let policy = mapped(ServicePolicy::new_composed_publication(
        home,
        issuer,
        services,
        routes,
        resources,
        publications,
    ))?;
    Ok((registry, policy))
}

pub(super) fn run(args: &[OsString]) -> Result<()> {
    let action = args.first().and_then(|a| a.to_str()).ok_or(())?;
    if args.len()
        != if matches!(action, "serve-composed" | "inspect-composed") {
            2
        } else {
            3
        }
    {
        return Err(());
    }
    let config = Config::load(Path::new(&args[1]))?;
    match action {
        "init-composed" | "apply-composed" => {
            let (registry, policy) = proposal(Path::new(&args[2]), &config)?;
            if action == "init-composed" {
                let _runtime = mapped(Runtime::initialize_composed_durable(
                    &config.base.policy,
                    registry,
                    policy,
                ))?;
                mapped(writeln!(io::stdout(), "initialized"))?;
            } else {
                let runtime = mapped(Runtime::reopen_durable(&config.base.policy))?;
                mapped(
                    runtime
                        .control()
                        .replace_delegation_policy(registry, policy),
                )?;
                mapped(writeln!(io::stdout(), "applied"))?;
            }
            mapped(io::stdout().flush())
        }
        "inspect-composed" => {
            let runtime = mapped(Runtime::reopen_durable(&config.base.policy))?;
            let s = mapped(runtime.policy_summary())?;
            mapped(writeln!(
                io::stdout(),
                "policy generation={} reset={} available={} bindings={} history={}",
                s.generation,
                s.reset_epoch,
                u8::from(s.available),
                s.binding_count,
                s.history_count
            ))?;
            mapped(io::stdout().flush())
        }
        "reset-composed" => {
            let epoch = args[2].to_str().ok_or(())?.parse::<u64>().map_err(|_| ())?;
            let runtime = mapped(Runtime::reopen_durable(&config.base.policy))?;
            mapped(runtime.control().reset(epoch))?;
            mapped(writeln!(io::stdout(), "reset"))?;
            mapped(io::stdout().flush())
        }
        "serve-composed" => serve(config),
        _ => Err(()),
    }
}
fn serve(config: Config) -> Result<()> {
    let c = &config.base;
    let runtime = mapped(Runtime::reopen_durable(&c.policy))?;
    mapped(runtime.configure_composed_relay(&c.home))?;
    let profile = mapped(PublicationWireProfile::new(
        c.home.clone(),
        config.publication_clients,
    ))?;
    let (_, certificate, key) = super::server_material(c)?;
    let certificate = mapped(X509::from_pem(&certificate))?;
    let key = mapped(PKey::private_key_from_pem_callback(&key, |_| {
        Err(openssl::error::ErrorStack::get())
    }))?;
    let cas = config
        .client_cas
        .iter()
        .map(|p| pem(p, &c.root))
        .collect::<Result<Vec<_>>>()?;
    let server = mapped(Server::new_composed(&certificate, &key, &cas))?;
    let store = Mutex::new(mapped(Store::open(&c.blobs))?);
    let local = config.local.carrier(&c.root)?;
    let peers = config
        .peers
        .iter()
        .map(|p| {
            Ok(RelayPeer {
                home: p.home.clone(),
                publication: p.publication.http(&c.root)?,
                introspection: p.introspection.http(&c.root)?,
                resource: p.resource.carrier(&c.root)?,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    let routes = mapped(RelayRoutes::new(&runtime, local, peers))?;
    mapped(faction_unix::install_shutdown())?;
    let listener = mapped(TcpListener::bind(c.endpoint))?;
    // The local relay address is fixed approval data; ephemeral discovery never
    // rewrites policy or route configuration.
    if mapped(listener.local_addr())? != config.local.address {
        return Err(());
    }
    let port = mapped(listener.local_addr())?.port();
    let cancel = Arc::new(AtomicBool::new(false));
    thread::scope(|scope| {
        let observer = scope.spawn(|| {
            while !cancel.load(Ordering::SeqCst) {
                if faction_unix::shutdown_requested() {
                    cancel.store(true, Ordering::SeqCst);
                    runtime.stop();
                    break;
                }
                thread::sleep(Duration::from_millis(10));
            }
        });
        let result = composed_loop::run(
            scope,
            composed_loop::Inputs {
                server: &server,
                listener,
                runtime: &runtime,
                cancel: &cancel,
                routes: &routes,
                store: &store,
                home: &c.home,
                profile: &profile,
                port,
            },
            #[cfg(test)]
            None,
        )?;
        mapped(observer.join())?;
        result
    })?;
    mapped(writeln!(io::stdout(), "stopped"))?;
    mapped(io::stdout().flush())
}
