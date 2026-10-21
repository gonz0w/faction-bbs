#!/usr/bin/env python3
"""Sustained silent-SSH pressure under the explicit 250-slot/250-ms private VM profile."""
import argparse,json,socket,subprocess,time
from pathlib import Path
from memory_profile import summarize
from headroom_memory import sample_guard

def run(a):
    assert a.disposable_fixture
    common=['/usr/bin/ssh','-F','/dev/null','-o','BatchMode=yes','-o','IdentitiesOnly=yes',
        '-o','StrictHostKeyChecking=yes','-o','UserKnownHostsFile='+str(a.known_hosts)]
    admin=common+['-p',str(a.admin_port),'-i',str(a.admin_identity),'root@'+a.host]
    def remote(command):
        x=subprocess.run(admin+['set -e; PATH=/usr/sbin:/sbin:/usr/bin:/bin:/usr/pkg/bin; export PATH; '+command],capture_output=True,timeout=20)
        assert x.returncode==0,'private pressure fixture operation failed';return x.stdout
    remote('test -f /etc/faction-disposable-test; test -S /var/run/faction/terminal.sock')
    ids=remote('id -u factiond; id -u factionterm; cat /var/run/faction-sshd.pid').decode().splitlines()
    assert len(ids)==3 and all(s.isdecimal() for s in ids)
    metric='/usr/local/libexec/faction-netbsd-metrics '+' '.join(ids)
    initial=json.loads(remote('cat /var/run/faction-admission.status'))
    assert initial['active']==0 and initial['capacity']==250 and initial['refill_ms']==250 and initial['burst']==8
    sockets=[];samples=[];snapshots=[];began=time.monotonic();due=began;next_sample=began;attempts=0;banners=0;early_closes=0
    try:
        while time.monotonic()-began<35:
            now=time.monotonic()
            if now>=next_sample:
                sample=summarize(json.loads(remote(metric)));samples.append(sample)
                snapshots.append(json.loads(remote('cat /var/run/faction-admission.status')))
                assert sample_guard(sample)['passed'],'pressure headroom guard failed'
                next_sample=time.monotonic()+1
            if now<due:time.sleep(min(.02,due-now));continue
            s=socket.create_connection((a.host,a.port),3);s.settimeout(3);sockets.append(s)
            try:data=s.recv(256)
            except ConnectionResetError:data=b''
            if data.startswith(b'SSH-2.0-'):banners+=1
            else:assert data==b'';early_closes+=1
            attempts+=1
            # Eight initial offers, then four per second; no catch-up flood after
            # sampling stalls. The separate default-cap test supplies saturation.
            due=time.monotonic()+(0 if attempts<8 else .26)
        end=json.loads(remote('cat /var/run/faction-admission.status'))
        assert banners>=100 and end['peak']<=128 and end['launches']>=100 and end['fork_failures']==0
        a.output.write_text(json.dumps({'schema':1,'scope':'35-second paced silent preauthentication; no arbitrary flood or soak claim',
            'seconds':time.monotonic()-began,'attempts':attempts,'banners':banners,'early_closes':early_closes,
            'initial':initial,'final':end,'samples':samples,'counter_samples':snapshots,
            'minimum_headroom_bytes':min(sample_guard(s)['reclaimable_headroom_bytes'] for s in samples)},indent=2)+'\n')
    finally:
        for s in sockets:s.close()

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--disposable-fixture',action='store_true');p.add_argument('--host',required=True)
    for n in ['admin-identity','known-hosts','output']:p.add_argument('--'+n,type=Path,required=True)
    for n in ['port','admin-port']:p.add_argument('--'+n,type=int,required=True)
    run(p.parse_args())
