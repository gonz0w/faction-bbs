#!/usr/bin/env python3
"""Owned NetBSD shared-service/operator proof; marked disposable fixtures only."""
import argparse,json,subprocess,time
from pathlib import Path
from check_restricted_ssh import SSHConsole
from smoke_operator import Operator


def run(a):
    assert a.disposable_fixture
    admin=json.loads(a.admin_argv.read_text());member=json.loads(a.member_argv.read_text())
    password=a.password_file.read_text().strip();children=[];checks=[];observations={}
    state='/var/db/faction-profile-fixture';status=state+'/operator.status'
    def remote(command,ok=True):
        r=subprocess.run(admin+['set -e; PATH=/usr/sbin:/sbin:/usr/bin:/bin:/usr/pkg/bin; export PATH; test -f /etc/faction-disposable-test; '+command],capture_output=True,timeout=20)
        assert not ok or r.returncode==0,'owned operator fixture action failed (private diagnostics only)'
        return r
    def snapshot():
        raw=remote('cat '+status).stdout.decode();assert len(raw)<=2048
        return dict(line.split('=',1) for line in raw.splitlines())
    def wait_for(key,value):
        deadline=time.monotonic()+12
        while time.monotonic()<deadline:
            s=snapshot()
            if s.get(key)==str(value):return s
            time.sleep(.2)
        raise AssertionError('bounded owner snapshot did not update '+key)
    try:
        remote('test ! -e '+status)
        checks.append('normal_default_has_no_operator_snapshot')
        remote('/etc/rc.d/faction_broker onestop; install -o root -g wheel -m 755 /home/factionbuild/target103/release/faction-sysop /usr/local/libexec/faction-sysop; printf "\\noperator_status=true\\n" >> /etc/faction/service.toml; /etc/rc.d/faction_broker onestart')
        time.sleep(.3)
        first=wait_for('running',1);assert first['capacity']=='8' and first['connected']=='0'
        checks.append('opt_in_rc_startup_and_strict_live_snapshot')
        denied=remote("su -m factionterm -c '/usr/local/libexec/faction-sysop --state-dir "+state+" --once'",ok=False)
        assert denied.returncode!=0 and b'OS operator permission required' in denied.stderr
        checks.append('actual_member_uid_denied_operator_access')
        host=remote("su - factiond -c '/usr/local/libexec/faction-sysop --state-dir "+state+" --once --view host --plain'").stdout
        assert b'CPU 2 busy: warming up' in host and b'RAM used: unavailable' not in host and b'Disk free: unavailable' not in host
        observations['host_summary']=host.decode().splitlines()
        checks.append('native_cpu_ram_load_and_disk_summary')
        c=SSHConsole(member);children.append(c);c.expect('[L]ogin [R]egister [Q]uit: ');c.send('l');c.expect('Handle: ');c.send('sysop');c.password(password)
        c.expect('Press Enter to continue: ');c.send('');c.expect('Ctrl-C exits; Esc: Enter to recover.')
        s=wait_for('connected',1);assert s['counts'].split(',')[:6]==['1','0','1','0','1','0']
        assert password not in str(s) and 'sysop' not in str(s)
        checks.append('actual_member_ssh_and_aggregate_visibility')
        command="su - factiond -c 'echo $$ > "+state+"/operator.fixture.pid; exec /usr/local/libexec/faction-sysop --state-dir "+state+" --ansi --refresh 2'"
        op=Operator(admin[:-1]+['-tt',admin[-1],command]);children.append(op)
        op.expect('LOCAL SYSOP');op.expect('Connected: 1 / 8');op.raw(b'h');op.expect('RAM used:');op.expect('Net RX/TX:');time.sleep(2.2);op.raw(b'r');op.expect('KiB/s');op.expect('CPU hist:');op.raw(b'i');op.expect('OS owner/root access; not a BBS role.')
        op.buffer=b'';op.resize(40,18);op.expect('40x18');op.expect('Q:quit P:pause R:redraw B/H/I:views');op.raw(b'b');op.expect('Connected: 1 / 8')
        checks.append('native_owner_pty_panels_and_actual_ssh_resize')
        samples=[]
        for _ in range(12):
            # Whole VM free pages and separate dashboard/owner RSS; no collector helper.
            raw=remote("ps -o rss,vsz,nlwp -p $(cat /var/db/faction-profile-fixture/operator.fixture.pid) | awk 'NR==2 {print}'; ps -o rss,vsz,nlwp -p $(cat /var/run/faction/terminal.pid) | awk 'NR==2 {print}'; vmstat -s | awk '/pages free$/{print}'; /sbin/sysctl -n hw.pagesize").stdout.decode().splitlines()
            assert len(raw)==4, 'unexpected native resource sample'
            dash,owner=[list(map(int,line.split())) for line in raw[:2]]
            free=int(raw[2].split()[0]);page=int(raw[3].split()[0])
            assert dash[2]==1 and owner[2]==4
            samples.append(dict(dashboard_rss_kib=dash[0],dashboard_vsz_kib=dash[1],owner_rss_kib=owner[0],owner_vsz_kib=owner[1],dashboard_lwps=dash[2],owner_lwps=owner[2],vm_free_bytes=free*page))
            time.sleep(.5)
        observations['resource_samples']=samples
        checks.append('separate_dashboard_and_whole_vm_resource_samples')
        op.raw(b'q');op.finish();c.raw(b'q');c.expect('Logged out.');c.finish();wait_for('connected',0)
        checks.append('operator_exit_and_member_logout')
        remote('/etc/rc.d/faction_broker onestop');s=wait_for('running',0);assert s['connected']=='0'
        stopped=remote("su - factiond -c '/usr/local/libexec/faction-sysop --state-dir "+state+" --once --plain'").stdout;assert b'STOPPED' in stopped
        checks.append('graceful_stop_publishes_final_snapshot')
        remote('/etc/rc.d/faction_broker onestart');wait_for('running',1)
        checks.append('service_restart_replaces_stopped_sample')
        assert all(password.encode() not in child.transcript for child in children)
        r=dict(result='pass',count=len(checks),checks=checks,observations=observations,normal_sessions=8,capacity_claim=False,physical_sbc_claim=False)
        a.output.write_text(json.dumps(r,indent=2,sort_keys=True)+'\n');a.output.chmod(0o600)
        print(f'Native shared operator integration passed {len(checks)} controls.')
    finally:
        for child in children:child.close()
        remote('/etc/rc.d/faction_broker onestop || true')


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--disposable-fixture',action='store_true',required=True)
    for name in ['admin-argv','member-argv','password-file','output']:p.add_argument('--'+name,type=Path,required=True)
    run(p.parse_args())
