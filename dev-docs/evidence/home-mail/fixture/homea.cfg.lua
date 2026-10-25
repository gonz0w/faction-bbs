daemonize = false
pidfile = "/home/maila/prosody.pid"
certificates = "/home/maila"
data_path = "/home/maila/data"
plugin_paths = { "/usr/local/faction-mail-modules" }
log = { { levels = { min = "info" }; to = "file"; filename = "/home/maila/prosody.log" } }
interfaces = { "127.0.0.1" }
c2s_ports = { }
c2s_direct_tls_interfaces = { "127.0.0.1" }
c2s_direct_tls_ports = { 32222 }
s2s_interfaces = { "127.0.0.1" }
s2s_ports = { 5269 }
s2s_direct_tls_ports = { }
legacy_ssl_ports = { }
http_ports = { }
https_ports = { }
c2s_require_encryption = true
s2s_require_encryption = true
s2s_secure_auth = true
allow_registration = false
authentication = "internal_hashed"
ssl = { key = "/home/maila/tls.key"; certificate = "/home/maila/tls.crt"; cafile = "/home/maila/ca.crt" }
modules_enabled = { "tls"; "saslauth"; "disco"; "roster"; "pep"; "mam"; "offline"; "limits"; "s2sout_override"; "faction_probe_peers" }
modules_disabled = { "dialback"; "bosh"; "websocket"; "http"; "register"; "csi" }
s2sout_override = { ["homeb.test"] = "tcp://127.0.0.1:5270" }
faction_probe_peer = "homeb.test"
archive_expires_after = "1d"
default_archive_policy = true
max_archive_query_results = 20
max_messages = 32
c2s_stanza_size_limit = 16384
s2s_stanza_size_limit = 32768
VirtualHost "homea.test"
