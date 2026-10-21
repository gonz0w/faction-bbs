#!/usr/bin/env python3
"""Prepare two synthetic XMPP homes in a marked disposable NetBSD VM only.

Requires an already built, inspected Prosody source tree and the separately
inspected official S2S override module. Not a product installer or host upgrader.
"""
import argparse,shlex,json,secrets,subprocess,os,hashlib
from pathlib import Path
p1=argparse.ArgumentParser(description=__doc__)
p1.add_argument('--admin-argv',type=Path,required=True)
p1.add_argument('--output-dir',type=Path,required=True)
p1.add_argument('--s2s-override',type=Path,required=True)
p1.add_argument('--alice-port',type=int,required=True)
p1.add_argument('--bob-port',type=int,required=True)
p1.add_argument('--disposable-synthetic-fixture',action='store_true',required=True)
a=p1.parse_args();p=a.output_dir
assert p.is_dir() and p.stat().st_uid==os.geteuid() and p.stat().st_mode & 0o077==0
assert 0<a.alice_port<65536 and 0<a.bob_port<65536 and a.alice_port!=a.bob_port
assert hashlib.sha256(a.s2s_override.read_bytes()).hexdigest() == '88fed46b5095a8f589e429d68aae238a91deea011025e6aa8702ed9fd079d281'
admin=json.loads(a.admin_argv.read_text())
class Remote:
 @staticmethod
 def remote(cmd,data=None,timeout=45):
  x=subprocess.run(admin+['set -e; PATH=/usr/sbin:/sbin:/usr/bin:/bin:/usr/pkg/bin; export PATH; '+cmd],input=data,capture_output=True,timeout=timeout)
  if x.returncode:
   (p/'prepare-private.log').write_bytes(x.stdout+x.stderr)
   raise RuntimeError('synthetic guest setup failed; inspect private log')
  return x
m=Remote()
m.remote('test -f /etc/faction-disposable-test; test -f /usr/local/faction-mail-probe/prosody; test ! -d /home/maila; test ! -d /home/mailb; mkdir -p /usr/local/faction-mail-modules /root/mail-ca')
m.remote('cat > /usr/local/faction-mail-modules/mod_s2sout_override.lua',data=a.s2s_override.read_bytes())
# Closed peer policy before routing/DNS and authenticated incoming sessions.
policy='''local st = require "util.stanza";
local jid = require "util.jid";
local peer = module:get_option_string("faction_probe_peer");
module:hook("route/remote", function(event)
 local host = event.to_host or jid.host(event.stanza.attr.to);
 if host ~= peer then
  event.origin.send(st.error_reply(event.stanza,"cancel","service-unavailable"));
  return true;
 end
end, 10000);
module:hook("s2s-in-established", function(event)
 if event.session.from_host ~= peer then event.session:close("policy-violation"); end
end, 10000);
'''
m.remote('cat > /usr/local/faction-mail-modules/mod_faction_probe_peers.lua',data=policy.encode())
m.remote("cd /root/mail-ca; openssl req -new -x509 -newkey ed25519 -nodes -keyout ca.key -out ca.crt -days 2 -config /dev/null -subj /CN=Faction-synthetic-mail-CA -addext basicConstraints=critical,CA:TRUE -addext keyUsage=critical,keyCertSign,cRLSign",timeout=30)
profile={'ca':str(p/'mail-ca.crt'),'marker':'faction-private-'+secrets.token_hex(24)}
c={'member_port':a.alice_port,'files_port':a.bob_port}
for user,domain,other,port,s2s,remote_s2s,name,hostport in [('maila','homea.test','homeb.test',2222,5269,5270,'alice',c['member_port']),('mailb','homeb.test','homea.test',2223,5270,5269,'bob',c['files_port'])]:
 m.remote(f'groupadd {user}; useradd -g {user} -d /home/{user} -s /bin/sh {user}; install -d -o {user} -g {user} -m 700 /home/{user} /home/{user}/data')
 m.remote(f'cd /root/mail-ca; openssl req -new -newkey ed25519 -nodes -keyout {user}.key -out {user}.csr -config /dev/null -subj /CN={domain} -addext subjectAltName=DNS:{domain}; openssl x509 -req -in {user}.csr -CA ca.crt -CAkey ca.key -CAcreateserial -days 2 -out {user}.crt -copy_extensions copy')
 m.remote(f'install -o {user} -g {user} -m 600 /root/mail-ca/{user}.key /home/{user}/tls.key; install -o {user} -g {user} -m 600 /root/mail-ca/{user}.crt /home/{user}/tls.crt; install -o {user} -g {user} -m 600 /root/mail-ca/ca.crt /home/{user}/ca.crt')
 conf=f'''daemonize = false
pidfile = "/home/{user}/prosody.pid"
certificates = "/home/{user}"
data_path = "/home/{user}/data"
plugin_paths = {{ "/usr/local/faction-mail-modules" }}
log = {{ {{ levels = {{ min = "info" }}; to = "file"; filename = "/home/{user}/prosody.log" }} }}
interfaces = {{ "127.0.0.1" }}
c2s_interfaces = {{ "10.0.2.15" }}
c2s_ports = {{ {port} }}
s2s_ports = {{ {s2s} }}
http_ports = {{ }}
https_ports = {{ }}
c2s_require_encryption = true
s2s_require_encryption = true
s2s_secure_auth = true
allow_registration = false
authentication = "internal_hashed"
ssl = {{ key = "/home/{user}/tls.key"; certificate = "/home/{user}/tls.crt"; cafile = "/home/{user}/ca.crt" }}
modules_enabled = {{ "tls"; "saslauth"; "disco"; "roster"; "pep"; "mam"; "offline"; "limits"; "s2sout_override"; "faction_probe_peers" }}
modules_disabled = {{ "dialback"; "bosh"; "websocket"; "http"; "register"; "csi" }}
s2sout_override = {{ ["{other}"] = "tcp://127.0.0.1:{remote_s2s}" }}
faction_probe_peer = "{other}"
archive_expires_after = "1d"
default_archive_policy = true
max_archive_query_results = 20
max_messages = 32
c2s_stanza_size_limit = 16384
s2s_stanza_size_limit = 32768
VirtualHost "{domain}"
'''
 m.remote(f'cat > /home/{user}/prosody.cfg.lua; chown {user}:{user} /home/{user}/prosody.cfg.lua; chmod 600 /home/{user}/prosody.cfg.lua',data=conf.encode())
 password=secrets.token_hex(24);profile['alice' if name=='alice' else 'bob']={'jid':name+'@'+domain,'password':password,'host':'127.0.0.1','port':hostport}
 # Credentials are passed over the private admin channel to this synthetic fixture only.
 code=f'cd /usr/local/faction-mail-probe; /usr/pkg/bin/lua5.4 ./prosodyctl --config /home/{user}/prosody.cfg.lua register {name} {domain} {password}'
 m.remote('su -m '+user+' -c '+shlex.quote(code))
 code=f'cd /usr/local/faction-mail-probe; nohup /usr/pkg/bin/lua5.4 ./prosody --config /home/{user}/prosody.cfg.lua > /home/{user}/startup.log 2>&1 < /dev/null &'
 m.remote('su -m '+user+' -c '+shlex.quote(code))
(p/'mail-ca.crt').write_bytes(m.remote('cat /root/mail-ca/ca.crt').stdout)
(p/'mail-fixture.json').write_text(json.dumps(profile));(p/'mail-fixture.json').chmod(0o600)
print('Two private independently owned XMPP homes started with verified fixture TLS, no HTTP/public DNS.')
