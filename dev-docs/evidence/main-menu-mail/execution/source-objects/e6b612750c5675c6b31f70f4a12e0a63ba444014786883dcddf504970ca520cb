#!/usr/bin/env python3
"""Synthetic SSH admission abuse/lifecycle probes; marked disposable NetBSD VM only."""
import argparse,json,socket,subprocess,time
from pathlib import Path

def run(a):
    assert a.disposable_fixture
    common=['/usr/bin/ssh','-F','/dev/null','-o','BatchMode=yes','-o','IdentitiesOnly=yes',
        '-o','LogLevel=ERROR','-o','ConnectTimeout=5','-o','StrictHostKeyChecking=yes',
        '-o','UserKnownHostsFile='+str(a.known_hosts)]
    admin=common+['-p',str(a.admin_port),'-i',str(a.admin_identity),'root@'+a.host]
    member=common+['-p',str(a.port),'-i',str(a.identity),'-N','factionterm@'+a.host]
    sockets=[];clients=[];results=[];observations=[]
    def remote(command,ok=True):
        p=subprocess.run(admin+['set -e; PATH=/usr/sbin:/sbin:/usr/bin:/bin:/usr/pkg/bin; export PATH; '+command],capture_output=True,timeout=30)
        assert not ok or p.returncode==0,'private admission operation failed'
        return p
    def status():
        for _ in range(10):
            try:return json.loads(remote('cat /var/run/faction-admission.status').stdout)
            except json.JSONDecodeError:time.sleep(.02)
        raise AssertionError('status readback unavailable')
    def wait(predicate,seconds=6):
        end=time.monotonic()+seconds
        while time.monotonic()<end:
            s=status()
            if predicate(s):return s
            time.sleep(.15)
        raise AssertionError('admission counter deadline')
    def passed(name,s=None):
        results.append({'oracle':name,'result':'pass'})
        if s is not None:observations.append({'oracle':name,'counters':s})
        print('Passed: '+name,flush=True)
    def start():remote('/usr/local/libexec/faction-admission --listen 0.0.0.0');wait(lambda s:s['active']==s['launches']==0)
    def stop():remote('/etc/rc.d/faction_admission onestop');assert remote('cat /var/run/faction-sshd.pid').stdout==b'0\n'
    def clear():
        for s in sockets:s.close()
        sockets.clear()
        for p in clients:
            if p.poll() is None:p.kill()
            p.wait()
        clients.clear()
    def raw(banner=True):
        s=socket.create_connection((a.host,a.port),3);s.settimeout(3);sockets.append(s)
        if banner:assert s.recv(256).startswith(b'SSH-2.0-')
        else:
            try:assert s.recv(256)==b''
            except ConnectionResetError:pass
        return s
    def idle(count):
        for _ in range(count):clients.append(subprocess.Popen(member,stdin=subprocess.DEVNULL,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL))
        result=wait(lambda s:s['active']==count)
        for _ in range(40):
            rows=remote('ps -U factionterm -o user= -o comm=').stdout.decode().splitlines()
            n=sum(line.split()[:1]==['factionterm'] and 'sshd-session' in line for line in rows)
            if n==count:return result
            time.sleep(.1)
        raise AssertionError('SSH public-key authentication did not complete')
    remote('test -f /etc/faction-disposable-test')
    binary=remote('sha256 -q /usr/local/libexec/faction-admission').stdout.decode().strip()
    try:
        # Called only with the separate listener stopped and no existing clients.
        for flags in ['--capacity 9','--refill-ms 100','--capacity 251 --experimental-capacity','--burst 9','--listen localhost','--unknown value']:
            assert remote('/usr/local/libexec/faction-admission '+flags,ok=False).returncode!=0
        passed('strict options preserve eight transports and five-second defaults')
        assert remote("su -m factionterm -c '/usr/local/libexec/faction-admission --foreground'",ok=False).returncode!=0
        passed('terminal identity cannot become an admission listener')
        start()
        assert remote('/usr/local/libexec/faction-admission --listen 0.0.0.0',ok=False).returncode!=0
        passed('second launcher cannot take the persistent lock or listener')
        for _ in range(8):raw()
        for _ in range(32):raw(False)
        s=wait(lambda s:s['full']>=32)
        assert s['peak']==s['active']==s['launches']==8 and s['capacity']==8 and s['refill_ms']==5000
        passed('slow preauthentication plus excess flood never exceeds eight SSH launches',s)
        s=wait(lambda s:s['active']==0,35)
        assert s['launches']==8
        passed('base SSH LoginGraceTime releases silent preauthentication slots',s)
        clear();stop();start()
        idle(8)
        assert all(p.poll() is None for p in clients)
        for _ in range(32):raw(False)
        s=wait(lambda s:s['full']>=32);assert s['peak']==s['active']==s['launches']==8
        passed('authenticated no-channel transports retain their aggregate slots',s)
        s=wait(lambda s:s['active']==0,35)
        assert s['launches']==8
        passed('base SSH unused-connection timeout reaps authenticated no-channel clients',s)
        clear();stop();start()
        for _ in range(8):raw()
        clear();wait(lambda s:s['active']==0)
        for _ in range(16):raw(False)
        s=wait(lambda s:s['rate']>=16);assert s['launches']==8 and s['active']==0
        passed('disconnected churn consumes the fixed global launch budget',s)
        clear();time.sleep(5.2);raw()
        s=wait(lambda s:s['launches']==9);assert s['active']==1
        passed('monotonic refill admits again without an address ban or ten-minute blackout',s)
        clear();wait(lambda s:s['active']==0);stop();start()
        unknown=member.copy();unknown[unknown.index(str(a.identity))]=str(a.unknown_identity)
        bad=subprocess.run(unknown,capture_output=True,timeout=8);assert bad.returncode!=0
        wait(lambda s:s['active']==0)
        passed('unknown invitation key is denied before local BBS access')
        clear();stop();start();idle(8)
        before=status();began=time.monotonic();stop();elapsed=time.monotonic()-began
        assert elapsed<10 and all(p.wait(timeout=4)!=0 for p in clients)
        assert status()['active']==0
        passed('loaded stop terminates and reaps all tracked SSH monitors',before)
        clear();start();idle(2)
        pid=remote('cat /var/run/faction-sshd.pid').stdout.decode().strip();assert pid.isdecimal()
        children=remote('pgrep -P '+pid).stdout.decode().splitlines();assert len(children)==2 and all(p.isdecimal() for p in children)
        remote('kill -KILL '+pid);time.sleep(.2)
        assert remote('/usr/local/libexec/faction-admission --listen 0.0.0.0',ok=False).returncode!=0
        passed('dirty master crash refuses restart even after its PID dies')
        clear()
        for child in children:
            for _ in range(40):
                if remote('kill -0 '+child,ok=False).returncode:break
                time.sleep(.1)
            else:raise AssertionError('old SSH child survived client close')
        # Explicit fixture operator recovery: prove both old monitor identities
        # gone, then acquire the same inode lock and reset its dirty record.
        command="/usr/pkg/bin/python3.13 - <<'PY'\nimport fcntl,os\np='/var/run/faction-sshd.pid'\nf=os.open(p,os.O_RDWR|os.O_NOFOLLOW);fcntl.flock(f,fcntl.LOCK_EX|fcntl.LOCK_NB)\nassert os.read(f,32)=="+repr((pid+'\n').encode())+"\nos.lseek(f,0,0);os.write(f,b'0\\n');os.ftruncate(f,2);os.close(f)\nPY"
        remote(command);start();raw();clear();wait(lambda s:s['active']==0);stop()
        passed('verified quiescent operator recovery reuses the locked inode')
        remote('chmod 644 /etc/ssh/faction/admission_sshd_config')
        try:assert remote('/usr/local/libexec/faction-admission',ok=False).returncode!=0
        finally:remote('chmod 600 /etc/ssh/faction/admission_sshd_config')
        passed('nonprivate SSH configuration fails before opening a listener')
        a.output.write_text(json.dumps({'schema':1,'binary_sha256':binary,'results':results,
            'observations':observations,'loaded_stop_seconds':elapsed,'scope':'synthetic bounded IPv4 admission; no production or soak claim'},indent=2)+'\n')
    finally:
        clear()
        remote('if /etc/rc.d/faction_admission onestatus >/dev/null 2>&1; then /etc/rc.d/faction_admission onestop; fi',ok=False)

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--disposable-fixture',action='store_true')
    p.add_argument('--host',required=True)
    for name in ['identity','admin-identity','unknown-identity','known-hosts','output']:p.add_argument('--'+name,type=Path,required=True)
    for name in ['port','admin-port']:p.add_argument('--'+name,type=int,required=True)
    run(p.parse_args())
