#!/usr/bin/env python3
"""Installed shared-service admission/configuration/rollback oracles, disposable VM only."""
import argparse,json,os,signal,subprocess,time
from pathlib import Path
from check_integrated_oracles import Terminal

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
    remote('test -f /etc/faction-disposable-test; test -f /var/db/faction-profile/.profile-fixture; /etc/rc.d/faction_broker onestop')
    uid=remote('id -u factionterm').stdout.decode().strip();assert uid.isdecimal()
    base=f'version=1\nterminal_uid={uid}\nstate_dir="/var/db/faction-profile"\n'
    def config(text=base):remote('rm -f /etc/faction/service.toml; cat > /etc/faction/service.toml; chmod 644 /etc/faction/service.toml',text.encode())
    def selection(value):remote('printf "faction_broker=YES\\nfaction_broker_backend='+value+'\\n" > /etc/rc.conf.d/faction_broker')
    def check(ok=True):return remote("su -m factiond -c '/usr/local/libexec/faction-service --check-config /etc/faction/service.toml'",ok=ok)
    def start():remote('/etc/rc.d/faction_broker onestart; /etc/rc.d/faction_broker onestatus')
    def stop():remote('/etc/rc.d/faction_broker onestop');assert remote('/etc/rc.d/faction_broker onestatus',ok=False).returncode!=0
    def drop(c):
        if not c.done:
            try:os.kill(c.pid,signal.SIGKILL)
            except ProcessLookupError:pass
            os.waitpid(c.pid,0);c.done=True
        os.close(c.fd)
    selection('shared');config();check();start()
    try:
        inode=remote('stat -f %i /var/run/faction/terminal.pid').stdout
        assert remote('/etc/rc.d/faction_broker onestart',ok=False).returncode!=0
        passed('duplicate start refuses live service and persistent pidfile lock')
        for _ in range(8):
            c=Terminal(member);children.append(c);c.expect('[L]ogin [R]egister [Q]uit: ')
        time.sleep(5.1) # Refill one launch token: the ninth denial tests capacity.
        c=Terminal(member);children.append(c);c.finish(1)
        assert b'[L]ogin' not in c.transcript
        assert remote('pgrep -x /usr/local/libexec/faction-service | wc -l').stdout.strip()==b'1'
        assert remote('pgrep -x /usr/local/libexec/faction-member',ok=False).returncode!=0
        passed('unconfigured service admits eight live terminals and rejects ninth after token refill')
        began=time.monotonic();stop();stop_seconds=time.monotonic()-began;assert stop_seconds<10
        time.sleep(.3)
        assert remote('pgrep -x /usr/local/libexec/faction-tty',ok=False).returncode!=0
        assert remote('stat -f %i /var/run/faction/terminal.pid').stdout==inode
        passed('TERM closes terminals within stop budget without unlinking pidfile')
        for c in children:drop(c)
        children.clear()
        start();selection('workers');remote('/etc/rc.d/faction_broker onestatus');stop()
        start();assert remote('pgrep -x /usr/local/libexec/faction-broker').returncode==0
        selection('shared');remote('/etc/rc.d/faction_broker onestatus');stop()
        passed('status and stop find actual backend after selection changes; worker rollback starts')
        for suffix in ['max_sessions=250\n','launch_ms=100\n','max_sessions=251\nexperimental_capacity=true\n','ansi="yes"\n','extra=true\n','version=2\n']:
            config(base+suffix);assert check(False).returncode!=0
            assert remote('/etc/rc.d/faction_broker onestart',ok=False).returncode!=0
            assert remote('/etc/rc.d/faction_broker onestatus',ok=False).returncode!=0
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
        assert remote('/etc/rc.d/faction_broker onestatus',ok=False).returncode!=0
        start();remote('/etc/rc.d/faction_broker onerestart; /etc/rc.d/faction_broker onestatus');stop()
        passed('HUP stops without reload and restart recovers with persistent lock')
        config();check();assert remote('test ! -S /var/run/faction/terminal.sock').returncode==0
        passed('check-config validates without opening listener or owner operations')
        args.output.write_text(json.dumps({'schema':1,'results':results,'default_eight_stop_seconds':stop_seconds},indent=2)+'\n')
        print(f'Installed shared-service lifecycle: {len(results)} checks passed.')
    finally:
        remote('/etc/rc.d/faction_broker onestop',ok=False)
        for c in children:drop(c)
        selection('shared');config()

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--disposable-fixture',action='store_true');p.add_argument('--host',required=True)
    for flag in ['port','admin-port']:p.add_argument('--'+flag,type=int,required=True)
    for flag in ['identity','admin-identity','known-hosts','output']:p.add_argument('--'+flag,type=Path,required=True)
    run(p.parse_args())
