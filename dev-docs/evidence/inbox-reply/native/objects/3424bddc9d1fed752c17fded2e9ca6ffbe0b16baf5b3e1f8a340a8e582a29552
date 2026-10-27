#!/usr/bin/env python3
"""Observe native grace/source penalties and same-source recovery; private VM only."""
import argparse
import json
import socket
import subprocess
import time
from pathlib import Path
from headroom_memory import sample_guard
from memory_profile import summarize


def run(a):
    assert a.disposable_fixture
    common=['/usr/bin/ssh','-F','/dev/null','-o','BatchMode=yes','-o','IdentitiesOnly=yes',
            '-o','StrictHostKeyChecking=yes','-o','LogLevel=ERROR','-o','ConnectTimeout=3',
            '-o','UserKnownHostsFile='+str(a.known_hosts)]
    admin=common+['-i',str(a.admin_identity),'-p',str(a.admin_port),'root@'+a.host]
    member=common+['-i',str(a.identity),'-p',str(a.port),'-N','factionterm@'+a.host]
    def remote(cmd):
        x=subprocess.run(admin+['test -f /etc/faction-disposable-test && '+cmd],capture_output=True,timeout=20)
        assert x.returncode==0,'private source fairness operation failed';return x.stdout
    inspect=lambda:json.loads(remote('/usr/local/libexec/faction-key-admission --inspect'))
    initial=inspect();assert initial['active']==0
    ids=remote('id -u factiond; id -u factionterm; cat /var/run/faction-sshd.pid').decode().splitlines()
    assert len(ids)==3 and all(x.isdecimal() for x in ids)
    metric='/usr/local/libexec/faction-netbsd-metrics '+' '.join(ids)
    inode=remote("stat -f '%d %i' /var/run/faction-auth/admission.state")
    effective=remote('/usr/sbin/sshd -T -f /etc/ssh/faction/key_admission_sshd_config').decode().splitlines()
    policy=next(x.split(' ',1)[1] for x in effective if x.startswith('persourcepenalties '))
    assert policy and policy!='no'
    held=[];samples=[];attempts=[];offered=drops=0;began=time.monotonic()
    try:
        for _ in range(64):
            s=socket.create_connection((a.host,a.port),3);s.settimeout(3);offered+=1
            try:banner=s.recv(256)
            except ConnectionResetError:banner=b''
            if banner.startswith(b'SSH-2.0-'):held.append(s)
            else:s.close();drops+=1
            if len(held)==8:break
        assert len(held)==8
        for _ in range(16):
            s=socket.create_connection((a.host,a.port),3);s.settimeout(3);offered+=1
            try:banner=s.recv(256)
            except ConnectionResetError:banner=b''
            s.close();assert not banner.startswith(b'SSH-2.0-');drops+=1
        for _ in range(36):
            sample=summarize(json.loads(remote(metric)));samples.append(sample)
            assert sample_guard(sample)['passed'] and sample['roles']['ssh']['count']<=17
            time.sleep(1)
        assert samples[-1]['roles']['ssh']['count']==1
        for _ in range(30):
            old=inspect();p=subprocess.Popen(member,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
            try:
                time.sleep(1.5);new=inspect();accepted=p.poll() is None and new['active']==1
                attempts.append({'at_seconds':time.monotonic()-began,'accepted':accepted,
                                 'new_reservations':new['launches']-old['launches']})
                a.output.with_suffix('.partial.json').write_text(json.dumps({'samples':samples,'same_source_attempts':attempts,'offers':offered,'excess_drops':drops})+'\n')
                if accepted:break
                assert p.poll() is not None and new['launches']==old['launches']
            finally:
                if p.poll() is None:p.kill()
                p.wait()
            sample=summarize(json.loads(remote(metric)));samples.append(sample)
            assert sample_guard(sample)['passed'];time.sleep(4)
        assert attempts and not attempts[0]['accepted'] and attempts[-1]['accepted']
        assert any(not x['accepted'] for x in attempts)
        time.sleep(.5);assert inspect()['active']==0
        assert remote('cat /var/run/faction-sshd.pid').decode().strip()==ids[2]
        assert remote("stat -f '%d %i' /var/run/faction-auth/admission.state")==inode
        after_policy=remote('/usr/sbin/sshd -T -f /etc/ssh/faction/key_admission_sshd_config').decode().splitlines()
        assert next(x.split(' ',1)[1] for x in after_policy if x.startswith('persourcepenalties '))==policy
        a.output.write_text(json.dumps({'schema':1,'silent_accepted':8,'offers':offered,'excess_drops':drops,
            'samples':samples,'same_source_attempts':attempts,'seconds':time.monotonic()-began,
            'listener_reset':False,'registry_reset':False,'source_penalties_disabled':False,
            'same_listener':True,'same_registry_inode':True,'effective_source_penalties':policy,
            'after':inspect(),'scope':'single synthetic shared source; native source-penalty denial and timed recovery, not fair admission guarantee'},indent=2)+'\n')
        print('Source fairness: initial legitimate same-source attempt denied; recovered without reset after %.1f seconds.' % attempts[-1]['at_seconds'])
    finally:
        for s in held:s.close()


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--disposable-fixture',action='store_true');p.add_argument('--host',required=True)
    for n in ['port','admin-port']:p.add_argument('--'+n,type=int,required=True)
    for n in ['identity','admin-identity','known-hosts','output']:p.add_argument('--'+n,type=Path,required=True)
    run(p.parse_args())
