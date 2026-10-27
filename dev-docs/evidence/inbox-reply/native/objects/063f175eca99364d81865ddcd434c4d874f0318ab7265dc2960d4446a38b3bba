#!/usr/bin/env python3
"""Silent preauthentication concurrency/deadline controls; private NetBSD fixture only."""
import argparse,json,socket,subprocess,time
from pathlib import Path
from memory_profile import summarize
from headroom_memory import sample_guard

def run(a):
    assert a.disposable_fixture
    common=['/usr/bin/ssh','-F','/dev/null','-o','BatchMode=yes','-o','IdentitiesOnly=yes','-o','StrictHostKeyChecking=yes',
        '-o','UserKnownHostsFile='+str(a.known_hosts)]
    admin=common+['-p',str(a.admin_port),'-i',str(a.admin_identity),'root@'+a.host]
    def remote(command):
        x=subprocess.run(admin+['set -e; PATH=/usr/sbin:/sbin:/usr/bin:/bin:/usr/pkg/bin; export PATH; '+command],capture_output=True,timeout=20)
        assert x.returncode==0,'private preauthentication fixture failed';return x.stdout
    remote('test -f /etc/faction-disposable-test; test -S /var/run/faction/terminal.sock')
    assert json.loads(remote('/usr/local/libexec/faction-key-admission --inspect'))['active']==0
    ids=remote('id -u factiond; id -u factionterm; cat /var/run/faction-sshd.pid').decode().splitlines()
    assert len(ids)==3 and all(x.isdecimal() for x in ids)
    metric='/usr/local/libexec/faction-netbsd-metrics '+' '.join(ids)
    held=[];offered=0;drops=0;samples=[]
    def offer():
        nonlocal offered,drops
        s=socket.create_connection((a.host,a.port),3);s.settimeout(3);offered+=1
        try:data=s.recv(256)
        except ConnectionResetError:data=b''
        if data.startswith(b'SSH-2.0-'):held.append(s);return True
        s.close();drops+=1;return False
    try:
        for _ in range(64):
            offer()
            if len(held)==8:break
        assert len(held)==8
        for _ in range(16):assert not offer()
        began=time.monotonic()
        while time.monotonic()-began<36:
            s=summarize(json.loads(remote(metric)));samples.append(s);assert sample_guard(s)['passed']
            assert s['roles']['ssh']['count']<=17 and s['roles'].get('frontend',{}).get('count',0)==0
            time.sleep(1)
        after=summarize(json.loads(remote(metric)))
        assert after['roles']['ssh']['count']==1
        assert json.loads(remote('/usr/local/libexec/faction-key-admission --inspect'))['active']==0
        # Controlled fixture reset of base source-penalty history after the grace
        # violations. It does not clear the independent key-admission registry.
        remote('/etc/rc.d/faction_key_admission onestop; /etc/rc.d/faction_key_admission onestart')
        a.output.write_text(json.dumps({'schema':1,'accepted_silent':8,'excess_drops':drops,'offers':offered,
            'samples':samples,'after_deadlines':after,'seconds':time.monotonic()-began,
            'scope':'base MaxStartups eight and login grace at idle; source penalties reset only for next fixture tests'},indent=2)+'\n')
    finally:
        for s in held:s.close()

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--disposable-fixture',action='store_true');p.add_argument('--host',required=True)
    for n in ['admin-identity','known-hosts','output']:p.add_argument('--'+n,type=Path,required=True)
    for n in ['port','admin-port']:p.add_argument('--'+n,type=int,required=True)
    run(p.parse_args())
