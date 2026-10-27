#!/usr/bin/env python3
"""Live publisher/SSH/operator controls on a marked disposable NetBSD fixture."""
import argparse,json,shlex,subprocess,time,tomllib
from pathlib import Path
from smoke_operator import Operator
from check_restricted_ssh import SSHConsole


def run(a):
    admin=json.loads(a.admin_argv.read_text()); member=json.loads(a.member_argv.read_text())
    password=a.password_file.read_text().strip(); checks=[]; children=[]; suspended=None
    state='/var/db/faction-profile-fixture'
    def remote(cmd,data=None):
        r=subprocess.run(admin+['set -e; PATH=/usr/sbin:/sbin:/usr/bin:/bin:/usr/pkg/bin; export PATH; test -f /etc/faction-disposable-test; '+cmd],input=data,capture_output=True,timeout=25)
        assert r.returncode==0,'owned health fixture operation failed; no public diagnostic content'
        return r.stdout
    def snap():
        raw=remote('cat '+state+'/operator.status').decode(); assert len(raw)<=2048
        s=dict(line.split('=',1) for line in raw.splitlines()); assert s['pid'].isdecimal()
        return s
    def current():
        deadline=time.monotonic()+12
        while time.monotonic()<deadline:
            s=snap()
            if s.get('version')=='2' and s.get('running')=='1': return s
            time.sleep(.2)
        raise AssertionError('new publisher snapshot unavailable')
    command='/usr/local/libexec/faction-sysop --state-dir '+state
    def once(): return remote('su - factiond -c '+shlex.quote(command+' --plain --once'))
    original=remote('cat /etc/faction/service.toml'); cfg=tomllib.loads(original.decode())
    assert cfg['state_dir']==state and 'operator_status' not in cfg
    try:
        remote('/etc/rc.d/faction_broker onestop; install -o root -g wheel -m 755 '+shlex.quote(str(a.sysop_binary))+' /usr/local/libexec/faction-sysop')
        remote('cat > /etc/faction/service.toml',original+b'operator_status=true\n')
        remote('/etc/rc.d/faction_broker onestart'); first=current()
        assert b'Owner: LEASE HELD / current instance' in once()
        checks.append('opt_in_actual_publisher_emits_v2_and_holds_matching_kernel_lease')
        denied=subprocess.run(admin+["test -f /etc/faction-disposable-test; su -m factionterm -c '"+command+" --once'"],capture_output=True,timeout=10)
        assert denied.returncode!=0 and b'OS operator permission required' in denied.stderr
        checks.append('restricted_member_uid_cannot_read_operator_state')
        c=SSHConsole(member);children.append(c);c.expect('[L]ogin [R]egister [Q]uit: ');c.send('l');c.expect('Handle: ');c.send('sysop');c.password(password)
        c.expect('Press Enter to continue: ');c.send('');c.expect('Ctrl-C exits; Esc: Enter to recover.')
        deadline=time.monotonic()+12
        while snap()['connected']!='1':
            assert time.monotonic()<deadline;time.sleep(.2)
        assert b'Connected: 1 / 8' in once()
        c.raw(b'q');c.expect('Logged out.');c.finish()
        checks.append('actual_member_ssh_retains_separate_local_approval_and_aggregate_counts')
        launch="echo $$ > "+state+"/operator.fixture.pid; exec "+command+" --ansi --refresh 2"
        op=Operator(admin[:-1]+['-tt',admin[-1],'su - factiond -c '+shlex.quote(launch)]);children.append(op)
        op.expect('Owner: LEASE HELD / current instance');op.resize(40,18);op.expect('40x18');op.expect('Q:quit P:pause R:redraw B/H/D/I:views')
        checks.append('native_owner_pty_has_classic_compact_health_and_resize')
        remote('/etc/rc.d/faction_broker onestop');op.expect('STOPPED');op.expect('NO LEASE / publisher inactive')
        checks.append('graceful_stop_releases_lease_and_keeps_final_stopped_sample')
        remote('/etc/rc.d/faction_broker onestart');second=current();assert second['instance']!=first['instance']
        op.expect('LEASE HELD / current instance')
        suspended=second['pid'];remote('kill -STOP '+suspended)
        # A held lock does not assert responsiveness. Exercise a reused-PID old instance.
        old=dict(second);old['instance']=first['instance']
        remote('cat > '+state+'/operator.status',''.join(k+'='+v+'\n' for k,v in old.items()).encode())
        op.expect('DIFFERENT / last-known sample')
        remote('cat > '+state+'/operator.status',''.join(k+'='+v+'\n' for k,v in second.items()).encode())
        op.expect('LEASE HELD / current instance');remote('kill -CONT '+suspended);suspended=None
        checks.append('restart_changes_instance_and_same_pid_old_sample_is_rejected')
        live=current();remote('kill -KILL '+live['pid']);op.expect('NO LEASE / publisher inactive')
        assert snap()['running']=='1' and b'NO LEASE / publisher inactive' in once()
        checks.append('abrupt_exit_is_detected_independently_of_still_running_marked_sample')
        remote('/etc/rc.d/faction_broker onestart');current();op.expect('LEASE HELD / current instance')
        op.raw(b'q');op.finish();checks.append('fresh_restart_recovers_and_operator_returns_to_os_shell')
        assert all(password.encode() not in x.transcript for x in children)
        result=dict(result='pass',count=len(checks),checks=checks,normal_sessions=8,admission_refill_ms=5000,capacity_claim=False,hardening_claim=False)
        a.output.write_text(json.dumps(result,indent=2)+'\n');print('Actual shared operator health passed',len(checks),'controls.')
    finally:
        if suspended: remote('kill -CONT '+suspended+' || true')
        for child in children: child.close()
        remote('/etc/rc.d/faction_broker onestop || true');remote('cat > /etc/faction/service.toml',original)


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--disposable-fixture',action='store_true',required=True)
    for n in ['admin-argv','member-argv','password-file','sysop-binary','output']:p.add_argument('--'+n,type=Path,required=True)
    run(p.parse_args())
