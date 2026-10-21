#!/usr/bin/env python3
"""Native base-SSH key gate lifecycle/abuse/state controls, marked private VM only."""
import argparse,json,shlex,subprocess,time
from pathlib import Path

def run(a):
    assert a.disposable_fixture
    common=['/usr/bin/ssh','-F','/dev/null','-o','BatchMode=yes','-o','IdentitiesOnly=yes','-o','LogLevel=ERROR',
        '-o','ConnectTimeout=5','-o','StrictHostKeyChecking=yes','-o','UserKnownHostsFile='+str(a.known_hosts)]
    admin=common+['-p',str(a.admin_port),'-i',str(a.admin_identity),'root@'+a.host]
    member=common+['-p',str(a.port),'-i',str(a.identity),'-N','factionterm@'+a.host]
    clients=[];results=[];observations=[]
    def remote(command,ok=True,data=None):
        p=subprocess.run(admin+['set -e; PATH=/usr/pkg/bin:/usr/sbin:/sbin:/usr/bin:/bin; export PATH; '+command],input=data,capture_output=True,timeout=20)
        assert not ok or p.returncode==0,'private key admission fixture operation failed';return p
    def state():return json.loads(remote('/usr/local/libexec/faction-key-admission --inspect').stdout)
    def wait(n):
        for _ in range(60):
            s=state()
            if s['active']==n:return s
            time.sleep(.1)
        raise AssertionError('key admission cleanup deadline')
    def passed(name,s=None):
        results.append({'oracle':name,'result':'pass'})
        if s is not None:observations.append({'oracle':name,'state':s})
        print('Passed: '+name,flush=True)
    def clear():
        for p in clients:
            if p.poll() is None:p.kill()
            p.wait()
        clients.clear();wait(0)
    def start():
        remote('if /etc/rc.d/faction_key_admission onestatus >/dev/null 2>&1; then /etc/rc.d/faction_key_admission onestop; fi')
        command="from pathlib import Path; p=Path('/etc/ssh/faction/key_admission_sshd_config'); r=p.read_text().splitlines(); p.write_text('\\n'.join('AuthorizedKeysCommand /usr/local/libexec/faction-key-admission --target-uid %U' if x.startswith('AuthorizedKeysCommand ') else x for x in r)+'\\n'); p.chmod(0o600)"
        remote('/usr/pkg/bin/python3.13 -c '+shlex.quote(command))
        remote('/usr/local/libexec/faction-key-admission --reset; /etc/rc.d/faction_key_admission onestart')
    def open_one():
        p=subprocess.Popen(member,stdin=subprocess.DEVNULL,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL);clients.append(p)
        wait(len(clients));time.sleep(.12);assert p.poll() is None;return p
    def denied(identity=None):
        argv=list(member)
        if identity:argv[argv.index(str(a.identity))]=str(identity)
        p=subprocess.run(argv,capture_output=True,timeout=8);assert p.returncode!=0
    remote('test -f /etc/faction-disposable-test; test -S /var/run/faction/terminal.sock')
    binary=remote('sha256 -q /usr/local/libexec/faction-key-admission').stdout.decode().strip()
    try:
        clear();start()
        for flags in ['--capacity 9','--refill-ms 100','--capacity 251 --experimental-capacity']:
            assert remote('/usr/local/libexec/faction-key-admission --reset '+flags,ok=False).returncode!=0
        passed('eight-slot five-second defaults and explicit experiment bounds')
        for user in ['factionterm','factionauth']:
            p=remote("su -m "+user+" -c '/usr/local/libexec/faction-key-admission --target-uid 1001; result=$?; exit $result'",ok=False)
            assert p.returncode!=0 and not p.stdout
        passed('terminal and direct helper callers cannot print invitation keys')
        for _ in range(8):open_one()
        denied();s=state();assert s['active']==s['peak']==s['launches']==8 and s['full']>=1 and s['reuses']>=8
        passed('authenticated no-channel cap and repeated key lookups share one process-birth slot',s)
        assert remote('/usr/local/libexec/faction-key-admission --reset',ok=False).returncode!=0
        assert state()['active']==8
        passed('root reset refuses to erase live monitor reservations')
        inode=remote('stat -f %i /var/run/faction-auth/admission.state').stdout
        remote('mv /var/run/faction-auth/admission.state /var/run/faction-auth/admission.saved')
        denied()
        remote('test ! -e /var/run/faction-auth/admission.state')
        assert remote('/usr/pkg/bin/python3.13 -I -B /usr/local/libexec/check_key_admission_config.py --initialize',ok=False).returncode!=0
        remote('/etc/rc.d/faction_key_admission onestop')
        assert remote('/usr/pkg/bin/python3.13 -I -B /usr/local/libexec/check_key_admission_config.py --initialize',ok=False).returncode!=0
        remote('mv /var/run/faction-auth/admission.saved /var/run/faction-auth/admission.state; /etc/rc.d/faction_key_admission onestart')
        assert remote('stat -f %i /var/run/faction-auth/admission.state').stdout==inode
        assert state()['active']==8
        passed('missing state cannot initialize over a live listener or surviving authenticated cohort')
        remote('/etc/rc.d/faction_key_admission onestop; /etc/rc.d/faction_key_admission onestart')
        assert all(p.poll() is None for p in clients);denied();assert state()['active']==8
        passed('listener restart cannot forget authenticated monitors reparented to init',state())
        clear();start()
        for _ in range(8):open_one();clear()
        denied();s=state();assert s['launches']==8 and s['rate']>=1
        passed('short-lived authenticated churn consumes the global monotonic budget',s)
        time.sleep(5.2);open_one();assert state()['launches']==9
        passed('five-second refill admits again without resetting the state inode')
        clear();start();denied(a.unknown_identity);assert state()['active']==0
        passed('unknown key remains denied without an AuthorizedKeysFile fallback')
        # Corrupt bytes never become a fresh empty registry. Restore the exact
        # prior bytes under its lock in this synthetic fixture only.
        original=remote('cat /var/run/faction-auth/admission.state').stdout
        code="import fcntl,os; p='/var/run/faction-auth/admission.state'; f=os.open(p,os.O_RDWR); fcntl.flock(f,fcntl.LOCK_EX); os.lseek(f,-1,os.SEEK_END); b=os.read(f,1); os.lseek(f,-1,os.SEEK_END); os.write(f,bytes([b[0]^1])); os.fsync(f)"
        remote('/usr/pkg/bin/python3.13 -c '+shlex.quote(code));denied()
        assert remote('/usr/local/libexec/faction-key-admission --inspect',ok=False).returncode!=0
        restore="import fcntl,os,sys; f=os.open('/var/run/faction-auth/admission.state',os.O_RDWR); fcntl.flock(f,fcntl.LOCK_EX); b=sys.stdin.buffer.read(); assert os.fstat(f).st_size==len(b); os.pwrite(f,b,0); os.fsync(f)"
        remote('/usr/pkg/bin/python3.13 -c '+shlex.quote(restore),data=original)
        passed('checksum corruption fails closed and cannot silently clear authority')
        clear();start()
        remote('ln /var/run/faction-auth/admission.state /var/run/faction-auth/alias')
        try:assert remote('/usr/local/libexec/faction-key-admission --inspect',ok=False).returncode!=0
        finally:remote('rm /var/run/faction-auth/alias')
        remote('chmod 644 /var/run/faction-auth/admission.state')
        try:assert remote('/usr/local/libexec/faction-key-admission --inspect',ok=False).returncode!=0
        finally:remote('chmod 600 /var/run/faction-auth/admission.state')
        passed('hardlinked and nonprivate registries fail metadata checks')
        assert remote("su -m factionterm -c 'test -r /var/run/faction-auth/admission.state'",ok=False).returncode!=0
        passed('dedicated helper registry is unreadable to the terminal UID')
        open_one();s=wait(1);time.sleep(31);wait(0);assert clients[0].wait(timeout=3)!=0
        passed('base SSH unused-connection timeout releases the authenticated reservation',s)
        clear();start()
        # Syntax-valid bypasses must be refused by the actual rc.d preflight.
        original=remote('cat /etc/ssh/faction/key_admission_sshd_config').stdout
        for old,new in [(b'AuthorizedKeysFile none',b'AuthorizedKeysFile /etc/ssh/faction/authorized_keys'),
                        (b'DisableForwarding yes',b'DisableForwarding no'),
                        (b'AuthorizedKeysCommandUser factionauth',b'AuthorizedKeysCommandUser root')]:
            bad=original.replace(old,new);assert bad!=original
            remote('cat > /etc/ssh/faction/key_admission_sshd_config; chmod 600 /etc/ssh/faction/key_admission_sshd_config',data=bad)
            assert remote('/usr/pkg/bin/python3.13 -I -B /usr/local/libexec/check_key_admission_config.py',ok=False).returncode!=0
        remote('cat > /etc/ssh/faction/key_admission_sshd_config; chmod 600 /etc/ssh/faction/key_admission_sshd_config',data=original)
        passed('native preflight refuses key-file bypass forwarding and privileged helper identities')
        a.output.write_text(json.dumps({'schema':1,'binary_sha256':binary,'results':results,'observations':observations,
            'scope':'native authenticated key admission; preauthentication remains bounded by base SSH'},indent=2)+'\n')
    finally:clear()

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--disposable-fixture',action='store_true');p.add_argument('--host',required=True)
    for n in ['identity','admin-identity','unknown-identity','known-hosts','output']:p.add_argument('--'+n,type=Path,required=True)
    for n in ['port','admin-port']:p.add_argument('--'+n,type=int,required=True)
    run(p.parse_args())
