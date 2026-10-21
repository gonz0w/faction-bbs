#!/usr/bin/env python3
"""Fixed-ceiling unsigned-key probe; private disposable NetBSD fixture only.

Research client only: Paramiko 4.0.0 is deliberately NOT a server dependency.
Uses its private AuthHandler API; pin/read back the client environment separately.
No signing, BBS login, message content or key material is saved in the result.
"""
import argparse
import base64
import json
import socket
import subprocess
import threading
import time
from pathlib import Path


def run(a):
    import paramiko
    from paramiko.auth_handler import AuthHandler
    from paramiko.message import Message
    assert a.disposable_fixture and paramiko.__version__ == '4.0.0'
    admin = ['/usr/bin/ssh','-F','/dev/null','-o','BatchMode=yes','-o','IdentitiesOnly=yes',
             '-o','StrictHostKeyChecking=yes','-o','UserKnownHostsFile='+str(a.known_hosts),
             '-i',str(a.admin_identity),'-p',str(a.admin_port),'root@'+a.host]
    def remote(cmd):
        p = subprocess.run(admin+['test -f /etc/faction-disposable-test && '+cmd],capture_output=True,timeout=20)
        assert p.returncode == 0, 'unsigned probe fixture operation failed'
        return json.loads(p.stdout)
    before = remote('/usr/local/libexec/faction-key-admission --inspect')
    assert before['active'] == 0, 'probe requires quiescent member transports'
    alg, encoded = a.public_key.read_text().split()[:2]
    blob = base64.b64decode(encoded,validate=True)
    known = paramiko.HostKeys(str(a.known_hosts))
    host_label = '['+a.host+']:'+str(a.port)
    assert host_label in known
    class QueryHandler(AuthHandler):
        def __init__(self, transport):
            super().__init__(transport); self.event=threading.Event(); self.reply=None
        @property
        def _client_handler_table(self):
            return {6:self.service,60:self.accepted,51:self.denied,52:self.unexpected}
        def service(self,m):
            assert m.get_text() == 'ssh-userauth'; self.reply='service'; self.event.set()
        def accepted(self,m):
            assert m.get_text() == alg and m.get_string() == blob
            self.reply='key_ok'; self.event.set()
        def denied(self,m): self.reply='denied'; self.event.set()
        def unexpected(self,m): self.reply='unexpected_authenticated'; self.event.set()
    t = paramiko.Transport(socket.create_connection((a.host,a.port),5))
    rows=[]; began=time.monotonic()
    try:
        t.start_client(timeout=5)
        assert known.check(host_label,t.get_remote_server_key()), 'host key changed'
        h=QueryHandler(t); t.auth_handler=h
        m=Message(); m.add_byte(bytes([5])); m.add_string('ssh-userauth'); t._send_message(m)
        assert h.event.wait(5) and h.reply == 'service'
        for _ in range(64):
            if time.monotonic()-began >= 20: break
            h.event.clear(); h.reply=None
            m=Message(); m.add_byte(bytes([50])); m.add_string('factionterm')
            m.add_string('ssh-connection'); m.add_string('publickey'); m.add_boolean(False)
            m.add_string(alg); m.add_string(blob); start=time.monotonic(); t._send_message(m)
            if not h.event.wait(3): break
            rows.append({'response':h.reply,'seconds':time.monotonic()-start})
            assert not t.is_authenticated(), 'unsigned query authenticated unexpectedly'
            if h.reply != 'key_ok': break
        during=remote('/usr/local/libexec/faction-key-admission --inspect')
        assert during['active'] == 1 and during['launches'] == before['launches']+1
        assert rows and all(r['response']=='key_ok' for r in rows)
    finally: t.close()
    time.sleep(.5)
    after=remote('/usr/local/libexec/faction-key-admission --inspect')
    assert after['active'] == 0
    result={'schema':1,'maximum_queries':64,'maximum_seconds':20,'queries':rows,
            'seconds':time.monotonic()-began,'before':before,'during':during,'after':after,
            'authenticated':False,'signed_requests':0,'client_version':paramiko.__version__,
            'scope':'bounded single private-fixture connection; no BBS access or throughput claim'}
    a.output.write_text(json.dumps(result,indent=2)+'\n')
    print('Unsigned key experiment: %d responses; one reservation; no authentication.' % len(rows))


if __name__ == '__main__':
    p=argparse.ArgumentParser(description=__doc__); p.add_argument('--disposable-fixture',action='store_true')
    p.add_argument('--host',required=True)
    for n in ['port','admin-port']: p.add_argument('--'+n,type=int,required=True)
    for n in ['public-key','admin-identity','known-hosts','output']:p.add_argument('--'+n,type=Path,required=True)
    run(p.parse_args())
