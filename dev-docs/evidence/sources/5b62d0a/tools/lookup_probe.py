#!/usr/bin/env python3
"""Bounded synthetic SSH authentication work probe; private marked VM only.

Paramiko 4.0.0 is an optional research client, never a server dependency. Host
keys are pinned. No private keys, protocol packets or endpoints enter results.
"""
import argparse,base64,json,socket,subprocess,threading,time
from pathlib import Path

class Connection:
    def __init__(self,a):
        import paramiko
        from paramiko.auth_handler import AuthHandler
        from paramiko.message import Message
        assert paramiko.__version__=='4.0.0'
        self.Message=Message;self.alg,self.encoded=a.public_key.read_text().split()[:2]
        self.blob=base64.b64decode(self.encoded,validate=True)
        outer=self
        class Handler(AuthHandler):
            def __init__(self,t):super().__init__(t);self.event=threading.Event();self.reply=None
            @property
            def _client_handler_table(self):return {6:self.service,60:self.accepted,51:self.denied,52:self.success}
            def service(self,m):assert m.get_text()=='ssh-userauth';self.reply='service';self.event.set()
            def accepted(self,m):assert m.get_text()==outer.alg and m.get_string()==outer.blob;self.reply='key_ok';self.event.set()
            def denied(self,m):self.reply='denied';self.event.set()
            def success(self,m):self.reply='authenticated';self.event.set()
        self.t=paramiko.Transport(socket.create_connection((a.host,a.port),3))
        try:
            self.t.start_client(timeout=4)
            known=paramiko.HostKeys(str(a.known_hosts));label='['+a.host+']:'+str(a.port)
            assert known.check(label,self.t.get_remote_server_key()),'host key changed'
            self.h=Handler(self.t);self.t.auth_handler=self.h
            m=Message();m.add_byte(bytes([5]));m.add_string('ssh-userauth');self.t._send_message(m)
            assert self.h.event.wait(4) and self.h.reply=='service'
        except BaseException:self.close();raise
    def request(self,a,mode):
        import paramiko
        M=self.Message;m=M();m.add_byte(bytes([50]));m.add_string('factionterm');m.add_string('ssh-connection');m.add_string('publickey')
        signed=mode=='bad_signature';m.add_boolean(signed);m.add_string(self.alg)
        blob=self.blob
        if mode=='unknown':blob=paramiko.Ed25519Key.from_private_key_file(str(a.unknown_identity)).asbytes()
        m.add_string(blob)
        if signed:
            data=M();data.add_string(self.t.session_id);data.add_bytes(m.asbytes())
            wrong=paramiko.Ed25519Key.from_private_key_file(str(a.unknown_identity))
            m.add_string(wrong.sign_ssh_data(data.asbytes()))
        self.h.event.clear();self.h.reply=None;began=time.monotonic()
        try:
            self.t._send_message(m)
            while time.monotonic()-began<3 and self.t.is_active() and not self.h.event.wait(.02):pass
            response=self.h.reply or ('disconnected' if not self.t.is_active() else 'timeout')
        except (EOFError,OSError):response='disconnected'
        assert response!='authenticated' and not self.t.is_authenticated(),'adversarial request authenticated'
        return {'response':response,'seconds':time.monotonic()-began}
    def close(self):self.t.close()

def admin(a,command):
    argv=['/usr/bin/ssh','-F','/dev/null','-o','BatchMode=yes','-o','IdentitiesOnly=yes','-o','LogLevel=ERROR',
        '-o','StrictHostKeyChecking=yes','-o','UserKnownHostsFile='+str(a.known_hosts),'-i',str(a.admin_identity),'-p',str(a.admin_port),'root@'+a.host]
    p=subprocess.run(argv+['test -f /etc/faction-disposable-test && '+command],capture_output=True,timeout=20)
    assert p.returncode==0,'lookup fixture operation failed';return p.stdout

def cohort(a,mode='known',count=8):
    assert a.disposable_fixture and mode in ['known','unknown','bad_signature','full'] and 1<=count<=8
    before=json.loads(admin(a,'/usr/local/libexec/faction-key-admission --inspect'))
    connections=[];drops=0;began=time.monotonic();rows=[[] for _ in range(count)]
    try:
        # Fixed offers to fill MaxStartups' probabilistic eight-slot cohort. Only
        # banner/KEX retries happen here; no key query until the cohort is full.
        for _ in range(48):
            try:c=Connection(a);connections.append(c)
            except (EOFError,OSError,Exception) as exc:
                # Paramiko SSHException has no useful public packet data here.
                if isinstance(exc,AssertionError):raise
                drops+=1
            if len(connections)==count:break
            assert time.monotonic()-began<15,'cohort setup deadline'
        assert len(connections)==count,'eight preauth connections not established'
        for index in range(8):
            for i,c in enumerate(connections):
                if rows[i] and rows[i][-1]['response'] in ['disconnected','timeout']:continue
                rows[i].append(c.request(a,'known' if mode=='full' else mode))
            if all(r and r[-1]['response']=='disconnected' for r in rows):break
        expected=4 if mode=='known' else 0
        for r in rows:
            assert sum(x['response']=='key_ok' for x in r)==expected
            assert r[-1]['response']=='disconnected' and len(r)<=7,'SSH failure bound exceeded'
        during=json.loads(admin(a,'/usr/local/libexec/faction-key-admission --inspect'))
    finally:
        for c in connections:c.close()
    time.sleep(.3);after=json.loads(admin(a,'/usr/local/libexec/faction-key-admission --inspect'))
    charged=(after['launches']+after['reuses'])-(before['launches']+before['reuses'])
    if mode=='known':assert charged==4*count
    assert after['active']==before['active']
    return {'mode':mode,'cohort':count,'setup_drops':drops,'rows':rows,'seconds':time.monotonic()-began,'before':before,'during':during,'after':after,'charged_lookups':charged,'authenticated':False}

def parser():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--disposable-fixture',action='store_true');p.add_argument('--host',required=True)
    for n in ['port','admin-port']:p.add_argument('--'+n,type=int,required=True)
    for n in ['public-key','unknown-identity','admin-identity','known-hosts','output']:p.add_argument('--'+n,type=Path,required=True)
    p.add_argument('--mode',choices=['known','unknown','bad_signature','full'],default='known');p.add_argument('--count',type=int,default=8)
    return p
if __name__=='__main__':
    a=parser().parse_args();d=cohort(a,a.mode,a.count);a.output.write_text(json.dumps(d,indent=2)+'\n');print('Bounded cohort:',d['mode'],d['cohort'],'charged lookups',d['charged_lookups'])
