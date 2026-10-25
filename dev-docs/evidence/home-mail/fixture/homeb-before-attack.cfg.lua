daemonize = false
pidfile = "/home/mailb/prosody.pid"
certificates = "/home/mailb"
data_path = "/home/mailb/data"
plugin_paths = { "/usr/local/faction-mail-modules" }
log = { { levels = { min = "info" }; to = "file"; filename = "/home/mailb/prosody.log" } }
interfaces = { "127.0.0.1" }
c2s_ports = { }
c2s_direct_tls_interfaces = { "127.0.0.1" }
c2s_direct_tls_ports = { 32223 }
s2s_interfaces = { "127.0.0.1" }
s2s_ports = { 5270 }
s2s_direct_tls_ports = { }
legacy_ssl_ports = { }
http_ports = { }
https_ports = { }
c2s_require_encryption = true
s2s_require_encryption = true
s2s_secure_auth = true
allow_registration = false
authentication = "internal_hashed"
ssl = { key = "/home/mailb/tls.key"; certificate = "/home/mailb/tls.crt"; cafile = "/home/mailb/ca.crt" }
modules_enabled = { "tls"; "saslauth"; "disco"; "roster"; "pep"; "mam"; "offline"; "limits"; "s2sout_override"; "faction_probe_peers" }
modules_disabled = { "dialback"; "bosh"; "websocket"; "http"; "register"; "csi" }
s2sout_override = { ["homea.test"] = "tcp://127.0.0.1:5269" }
faction_probe_peer = "homea.test"
archive_expires_after = "1d"
default_archive_policy = true
max_archive_query_results = 20
max_messages = 32
c2s_stanza_size_limit = 16384
s2s_stanza_size_limit = 32768
VirtualHost "homeb.test"
