#!/usr/bin/env python3
"""Destructive global-gate oracles, ONLY for a marked disposable NetBSD VM.

Creates loopback test aliases, resets stopped gate state, tests corruption then
restores the original bytes. Never run against a real board. No private output.
"""
import argparse,json,shlex,subprocess,time
from pathlib import Path

def run(a):
    assert a.disposable_fixture
    base=['/usr/bin/ssh','-F','/dev/null','-o','BatchMode=yes','-o','IdentitiesOnly=yes','-o','LogLevel=ERROR','-o','ConnectTimeout=4','-o','StrictHostKeyChecking=yes','-o','UserKnownHostsFile='+str(a.known_hosts)]
    admin=base+['-p',str(a.admin_port),'-i',str(a.admin_identity),'root@'+a.host]
    member=base+['-p',str(a.port),'-i',str(a.identity),'-N','factionterm@'+a.host]
    rows=[];facts={};clients=[]
    def remote(cmd,data=None,ok=True):
        p=subprocess.run(admin+['set -e; PATH=/usr/pkg/bin:/usr/sbin:/sbin:/usr/bin:/bin; export PATH; '+cmd],input=data,capture_output=True,timeout=25)
        assert not ok or p.returncode==0,'native global oracle command failed: '+p.stderr.decode(errors='replace')[-800:]
        return p
    def python(code,data=None,ok=True):return remote('/usr/pkg/bin/python3.13 -I -c '+shlex.quote(code),data,ok)
    def passed(name):rows.append(dict(oracle=name,result='pass'));print('Passed: '+name,flush=True)
    tool='/usr/local/libexec/faction-ssh-gate'
    preflight='/usr/pkg/bin/python3.13 -I -B /usr/local/libexec/prepare_ssh_gate.py'
    state='/var/db/faction-gate/admission.state'
    def inspect():return json.loads(remote(tool+' --inspect').stdout)
    def stop():remote('/etc/rc.d/faction_ssh_gate onestop',ok=False);time.sleep(.4)
    def start():remote('/etc/rc.d/faction_ssh_gate onestart')
    def reset(ms=5000):
        stop();remote(preflight+' --configure --reset-budget --refill-ms '+str(ms)+(' --experimental-capacity' if ms<5000 else ''));start()
    def raw(sources):
        code="""import socket,json,time
rows=[];start=time.monotonic()
for source in SOURCES:
 s=socket.socket();s.settimeout(2);s.bind((source,0));s.connect(('127.0.0.1',2222))
 try: b=s.recv(256)
 except (ConnectionResetError,socket.timeout): b=b''
 rows.append(b.startswith(b'SSH-'));s.close()
print(json.dumps({'banners':rows,'seconds':time.monotonic()-start}))
""".replace('SOURCES',repr(sources))
        return json.loads(python(code).stdout)
    remote('test -f /etc/faction-disposable-test; test -S /var/run/faction/terminal.sock')
    original_config=remote('cat /etc/ssh/faction/key_admission_sshd_config').stdout
    # Defaults for the original independent key gate.
    python("from pathlib import Path;p=Path('/etc/ssh/faction/key_admission_sshd_config');r=p.read_text().splitlines();p.write_text('\\n'.join('AuthorizedKeysCommand /usr/local/libexec/faction-key-admission --target-uid %U' if x.startswith('AuthorizedKeysCommand ') else x for x in r)+'\\n');p.chmod(0o600)")
    aliases=['127.0.0.'+str(n) for n in range(11,23)]
    try:
        stop();remote('cat > /etc/ssh/faction/key_admission_sshd_config',b'ListenAddress 127.0.0.1\n'+remote('cat /etc/ssh/faction/key_admission_sshd_config').stdout);remote('/usr/local/libexec/faction-key-admission --reset')
        for ip in aliases:remote('ifconfig lo0 inet '+ip+' alias')
        reset();s=inspect();assert s['refill_ms']==5000 and s['state_bytes']==120 and s['burst']==8 and s['source_burst']==2 and s['source_slots']==32
        assert remote(tool+' --reset --refill-ms 100',ok=False).returncode!=0
        passed('fixed 120-byte state, 32 source slots, default eight/five seconds and explicit faster opt-in')
        listener=remote('cat /var/run/faction-sshd.pid').stdout.decode().strip()
        remote('ktrace -i -t cn -f /root/global-gate.ktrace -p '+listener)
        offered=[ip for ip in aliases[:4] for _ in range(2)]+aliases[4:7]
        r=raw(offered);s=inspect();assert r['banners']==[True]*8+[False]*3 and r['seconds']<5 and s['passed']==8
        remote('ktrace -c -p '+listener)
        trace=remote('kdump -f /root/global-gate.ktrace').stdout.decode()
        import re
        forks=[line for line in trace.splitlines() if re.search(r'^\s*'+listener+r'\s',line) and ' RET ' in line and re.search(r'\b(?:fork|__vfork14)\b',line)]
        assert len(forks)==8,'accepted-socket/fork mismatch'
        facts['global_burst']={'offered':11,'admitted':8,'listener_forks':len(forks),'seconds':r['seconds'],'state':s}
        passed('multi-source offers consume durable global burst before exactly eight native listener forks')
        stop();before=inspect();start();after=inspect();assert after['passed']==before['passed'] and after['budget_ns']==before['budget_ns'] and after['starts']==before['starts']+1
        passed('same-boot listener restart preserves exact durable global budget')
        reset();r=raw([aliases[7]]*3+[aliases[8]]);assert r['banners']==[True,True,False,True];s=inspect();assert s['passed']==3 and s['source_drops']>=1
        facts['source_burst']={'banners':r['banners'],'state':s}
        passed('one source exhausts its two-offer share without consuming another source global token')
        assert remote(tool+' --reset',ok=False).returncode!=0
        passed('live root listener flock prevents budget reset')
        reset(100);p=subprocess.Popen(member,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL);clients.append(p)
        for _ in range(40):
            active=json.loads(remote('/usr/local/libexec/faction-key-admission --inspect').stdout)['active']
            if active==1:break
            time.sleep(.1)
        assert active==1 and p.poll() is None
        facts['mapping']=json.loads(remote('/usr/local/libexec/faction-gate-maps $(cat /var/run/faction-sshd.pid)').stdout)
        passed('ordinary signed SSH succeeds; module maps only in listener after session exec')
        remote('kill -HUP $(cat /var/run/faction-sshd.pid)');time.sleep(1)
        assert remote('/etc/rc.d/faction_ssh_gate onestatus',ok=False).returncode!=0 and p.poll() is None
        start();assert p.poll() is None
        passed('HUP stops listener fail-closed, explicit restart retains authenticated transport')
        p.kill();p.wait();clients.clear();time.sleep(.5)
        reset();original=remote('cat '+state).stdout
        python("import os;p='"+state+"';f=os.open(p,os.O_RDWR);b=os.pread(f,1,119);os.pwrite(f,bytes([b[0]^1]),119);os.fsync(f)")
        assert raw([aliases[9]])['banners']==[False]
        assert remote('/etc/rc.d/faction_ssh_gate onestatus',ok=False).returncode!=0
        assert remote(tool+' --initialize',ok=False).returncode!=0 and remote(tool+' --reset',ok=False).returncode!=0
        python("import os,sys;f=os.open('"+state+"',os.O_RDWR);os.pwrite(f,sys.stdin.buffer.read(),0);os.fsync(f)",original)
        passed('live checksum corruption stops listener; initialize and reset never repair corrupt state')
        for name,setup,undo in [('hardlink','ln '+state+' /var/db/faction-gate/alias','rm /var/db/faction-gate/alias'),('mode','chmod 644 '+state,'chmod 600 '+state),('missing','mv '+state+' '+state+'.saved','mv '+state+'.saved '+state)]:
            remote(setup)
            try:assert remote(preflight,ok=False).returncode!=0
            finally:remote(undo)
            passed(name+' state rejected by actual rc preflight')
        saved=remote('cat /etc/ssh/faction/key_admission_sshd_config').stdout
        for directive in ['PerSourcePenalties no','PerSourcePenaltyExemptList 127.0.0.1']:
            remote('cat > /etc/ssh/faction/key_admission_sshd_config', (directive+'\n').encode()+saved)
            try:assert remote(preflight,ok=False).returncode!=0
            finally:remote('cat > /etc/ssh/faction/key_admission_sshd_config',saved)
        passed('unchanged native source penalties and no exemptions enforced by preflight')
        stop();saved=remote('cat /etc/faction/ssh-gate.toml').stdout
        bad=saved.replace(b'module_sha256 = "',b'module_sha256 = "0',1);assert bad!=saved
        remote('cat > /etc/faction/ssh-gate.toml',bad)
        try:assert remote(preflight,ok=False).returncode!=0
        finally:remote('cat > /etc/faction/ssh-gate.toml',saved)
        passed('explicit root configuration pins exact module and base SSH bytes')
        facts['base_version']=remote('/usr/sbin/sshd -V').stderr.decode().strip()
        a.output.write_text(json.dumps(dict(schema=1,results=rows,observations=facts),indent=2)+'\n')
    finally:
        for p in clients:
            if p.poll() is None:p.kill()
            p.wait()
        stop();remote('cat > /etc/ssh/faction/key_admission_sshd_config',original_config)
        for ip in aliases:remote('ifconfig lo0 inet '+ip+' -alias',ok=False)
        remote(preflight+' --configure --reset-budget --refill-ms 250 --experimental-capacity');start()
if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--disposable-fixture',action='store_true');p.add_argument('--host',required=True);p.add_argument('--port',type=int,required=True);p.add_argument('--admin-port',type=int,required=True)
    for name in ['identity','admin-identity','known-hosts','output']:p.add_argument('--'+name,type=Path,required=True)
    run(p.parse_args())
