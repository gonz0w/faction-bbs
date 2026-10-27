#!/usr/bin/env python3
"""Installed shared-service admission/configuration/rollback oracles, disposable VM only."""
import argparse,json,os,signal,subprocess,time
from pathlib import Path
from check_isolation_oracles import Terminal

def run(args):
    assert args.disposable_fixture
    common=['/usr/bin/ssh','-F','/dev/null','-o','BatchMode=yes','-o','IdentitiesOnly=yes','-o','LogLevel=ERROR','-o','ConnectTimeout=5','-o','StrictHostKeyChecking=yes','-o','UserKnownHostsFile='+str(args.known_hosts)]
    admin=common+['-p',str(args.admin_port),'-i',str(args.admin_identity),'root@'+args.host]
    member=common+['-p',str(args.port),'-i',str(args.identity),'-tt','factionterm@'+args.host]
    results=[];children=[]
    def remote(command,data=None,ok=True):
        p=subprocess.run(admin+['set -e; PATH=/usr/sbin:/sbin:/usr/bin:/bin:/usr/pkg/bin; export PATH; '+command],input=data,capture_output=True,timeout=30)
        assert not ok or p.returncode==0,'private service fixture action failed'
        return p
    def passed(name):results.append({'oracle':name,'result':'pass'})
    remote('test -f /etc/faction-disposable-test; test -f /var/db/faction-profile/.profile-fixture; /etc/rc.d/faction_isolated onestop')
    uid=remote('id -u factionterm').stdout.decode().strip();assert uid.isdecimal()
    base=f'version=1\nterminal_uid={uid}\nstate_dir="/var/db/faction-profile"\n'
    def config(text=base):remote('rm -f /etc/faction/service.toml; cat > /etc/faction/service.toml; chmod 644 /etc/faction/service.toml',text.encode())
    def selection(value):assert value=='shared'
    def check(ok=True):
        x=remote('/usr/pkg/bin/python3.13 /usr/local/libexec/faction-isolated/prepare_isolation.py --configure',ok=False)
        if x.returncode==0:
            x=remote('/usr/bin/env -i PATH=/usr/bin:/bin /usr/local/libexec/faction-isolated/faction-service --check-config /etc/faction/service.toml </dev/null >>/var/log/faction-isolated.log 2>&1',ok=False)
        assert not ok or x.returncode==0,'isolated configuration rejected'
        return x
    def start():check();remote('/etc/rc.d/faction_isolated onestart; /etc/rc.d/faction_isolated onestatus')
    def stop():remote('/etc/rc.d/faction_isolated onestop');assert remote('/etc/rc.d/faction_isolated onestatus',ok=False).returncode!=0
    def drop(c):
        if not c.done:
            try:os.kill(c.pid,signal.SIGKILL)
            except ProcessLookupError:pass
            os.waitpid(c.pid,0);c.done=True
        os.close(c.fd)
    selection('shared');config();check();start()
    try:
        inode=remote('stat -f %i /var/run/faction/terminal.pid').stdout
        assert remote('/etc/rc.d/faction_isolated onestart',ok=False).returncode!=0
        passed('duplicate start refuses live service and persistent pidfile lock')
        for _ in range(8):
            c=Terminal(member);children.append(c);c.expect('[L]ogin [R]egister [Q]uit: ')
        time.sleep(5.1) # Refill one launch token: the ninth denial tests capacity.
        c=Terminal(member);children.append(c);c.finish(1)
        assert b'[L]ogin' not in c.transcript
        assert remote('pgrep -x /usr/local/libexec/faction-isolated/faction-service | wc -l').stdout.strip()==b'1'
        assert remote('pgrep -x /usr/local/libexec/faction-member',ok=False).returncode!=0
        passed('unconfigured service admits eight live terminals and rejects ninth after token refill')
        began=time.monotonic();stop();stop_seconds=time.monotonic()-began;assert stop_seconds<10
        time.sleep(.3)
        assert remote('pgrep -x /usr/local/libexec/faction-isolated/faction-tty',ok=False).returncode!=0
        assert remote('stat -f %i /var/run/faction/terminal.pid').stdout==inode
        passed('TERM closes terminals within stop budget without unlinking pidfile')
        for c in children:drop(c)
        children.clear()
        for suffix in ['max_sessions=250\n','launch_ms=100\n','max_sessions=251\nexperimental_capacity=true\n','ansi="yes"\n','extra=true\n','version=2\n']:
            config(base+suffix);assert check(False).returncode!=0
            assert remote('/etc/rc.d/faction_isolated onestart',ok=False).returncode!=0
            assert remote('/etc/rc.d/faction_isolated onestatus',ok=False).returncode!=0
        passed('invalid unknown wrong-type and implicit capacity configuration cannot start')
        config();remote('chmod 666 /etc/faction/service.toml');assert check(False).returncode!=0
        remote('chmod 644 /etc/faction/service.toml; ln /etc/faction/service.toml /etc/faction/service.link');assert check(False).returncode!=0
        remote('rm /etc/faction/service.link; mv /etc/faction/service.toml /etc/faction/service.real; ln -s service.real /etc/faction/service.toml');assert check(False).returncode!=0
        remote('rm /etc/faction/service.toml /etc/faction/service.real; mkfifo /etc/faction/service.toml; chmod 644 /etc/faction/service.toml');assert check(False).returncode!=0
        passed('unsafe writable hardlinked symlink and FIFO configurations fail without waiting')
        config(base+'max_sessions=250\nlaunch_ms=100\nexperimental_capacity=true\n');check()
        passed('explicit experimental 250 configuration validates')
        config(base+'theme="/does/not/exist"\n');assert check(False).returncode!=0
        passed('invalid theme fails before socket bind')
        config(base+'session_seconds=1\n');start()
        c=Terminal(member);children.append(c);c.expect('[L]ogin [R]egister [Q]uit: ');c.finish(1);stop()
        passed('configured session lifetime expires real terminal')
        config();start();remote('kill -HUP $(cat /var/run/faction/terminal.pid)');time.sleep(.5)
        assert remote('/etc/rc.d/faction_isolated onestatus',ok=False).returncode!=0
        start();remote('/etc/rc.d/faction_isolated onerestart; /etc/rc.d/faction_isolated onestatus');stop()
        passed('HUP stops without reload and restart recovers with persistent lock')
        config();check();assert remote('test ! -S /var/run/faction/terminal.sock').returncode==0
        passed('check-config validates without opening listener or owner operations')
        # The root initializer independently rejects malformed policy before main.
        ids=remote('cat /etc/faction/isolation.ids').stdout
        def initializer():return remote('/usr/bin/env -i PATH=/usr/bin:/bin /usr/local/libexec/faction-isolated/faction-service --check-config /etc/faction/service.toml </dev/null >>/var/log/faction-isolated.log 2>&1',ok=False)
        for bad in [b'0\n1001\n1000\n/var/db/faction-profile\n', b'42949672960\n1001\n1000\n/var/db/faction-profile\n', b'1000\n1000\n1000\n/var/db/faction-profile\n', b'1000\n1001\n1000\n/var/db/../faction-profile\n', b'1000\x00\n1001\n1000\n/var/db/faction-profile\n']:
            remote('cat > /etc/faction/isolation.ids; chmod 600 /etc/faction/isolation.ids',bad)
            assert initializer().returncode!=0
        remote('cat > /etc/faction/isolation.ids; chmod 600 /etc/faction/isolation.ids',ids)
        remote('chmod 644 /etc/faction/isolation.ids');assert initializer().returncode!=0
        remote('chmod 600 /etc/faction/isolation.ids; ln /etc/faction/isolation.ids /etc/faction/isolation.link');assert initializer().returncode!=0
        remote('rm /etc/faction/isolation.link')
        passed('initializer rejects zero duplicate overflow traversal NUL public and hardlinked identity policy')
        assert remote('/sbin/sysctl -n vfs.generic.usermount').stdout.strip()==b'0'
        # This release's running securelevel refuses enabling user mounts.
        assert remote('/sbin/sysctl -w vfs.generic.usermount=1',ok=False).returncode!=0
        assert remote('/sbin/sysctl -n vfs.generic.usermount').stdout.strip()==b'0'
        passed('native kernel refuses enabling user mounts at running securelevel')
        remote('/sbin/npfctl stop')
        try:assert remote('/etc/rc.d/faction_isolated onestart',ok=False).returncode!=0
        finally:remote('/sbin/npfctl start')
        passed('inactive firewall refuses start without automatically enabling it')
        remote('printf "group default { pass all }\\n" > /var/tmp/faction-wrong-npf; /sbin/npfctl reload /var/tmp/faction-wrong-npf')
        try:assert remote('/etc/rc.d/faction_isolated onestart',ok=False).returncode!=0
        finally:remote('/sbin/npfctl reload /etc/faction/npf-isolation.conf')
        passed('different active firewall refuses start without replacing operator policy')
        assert remote('/usr/pkg/bin/python3.13 -O /usr/local/libexec/faction-isolated/prepare_isolation.py --verify',ok=False).returncode!=0
        remote('PYTHONOPTIMIZE=1 PYTHONPATH=/does/not/exist /etc/rc.d/faction_isolated onestart; /etc/rc.d/faction_isolated onestop')
        passed('optimized preflight rejects bypass and rc start scrubs inherited Python environment')
        remote('test -x /usr/local/libexec/faction-service; test -x /usr/local/libexec/faction-tty')
        passed('normal service and frontend rollback binaries remain installed independently')
        args.output.write_text(json.dumps({'schema':1,'results':results,'default_eight_stop_seconds':stop_seconds},indent=2)+'\n')
        print(f'Installed isolated-service lifecycle: {len(results)} checks passed.')
    finally:
        remote('/etc/rc.d/faction_isolated onestop',ok=False)
        for c in children:drop(c)
        selection('shared');config();check()

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--disposable-fixture',action='store_true');p.add_argument('--host',required=True)
    for flag in ['port','admin-port']:p.add_argument('--'+flag,type=int,required=True)
    for flag in ['identity','admin-identity','known-hosts','output']:p.add_argument('--'+flag,type=Path,required=True)
    run(p.parse_args())
