#!/usr/bin/env python3
"""Actual two-board launcher/approval/Files oracle; owned marked NetBSD fixture only."""
import argparse,json,os,secrets,shlex,subprocess,tempfile,time,tomllib
from pathlib import Path
from check_restricted_ssh import SSHConsole
from smoke_board_launcher import LauncherConsole,catalog,PICK,RETURN,CONFIRM

MENU='Ctrl-C exits; Esc: Enter to recover.'
DISCONNECT='LOCAL: disconnect board (drafts lost), return to Boards? [y/N]: '

def run(a):
    assert a.disposable_fixture
    admin=json.loads(a.admin_argv.read_text());second_admin=json.loads(a.second_admin_argv.read_text());members=[json.loads(a.first_member_argv.read_text()),json.loads(a.second_member_argv.read_text())]
    passwords=[a.first_password_file.read_text().strip(),a.second_password_file.read_text().strip()]
    visitor='visit-'+secrets.token_hex(3);member_secret=secrets.token_hex(20)
    children=[];checks=[];observations={};last=0
    def remote(cmd,second=False):
        r=subprocess.run((second_admin if second else admin)+['set -e; PATH=/usr/sbin:/sbin:/usr/bin:/bin:/usr/pkg/bin; export PATH; test -f /etc/faction-disposable-test; '+cmd],capture_output=True,timeout=20)
        if r.returncode:(a.output.parent/'board-visits-remote-private.log').write_bytes(r.stdout+r.stderr)
        assert r.returncode==0,'owned visit fixture query failed';return r.stdout.decode()
    remote('test -S /var/run/faction/terminal.sock');remote('test -S /var/run/faction/terminal.sock; test -f /var/db/faction-visit-b/faction.sqlite3',True)
    def delay():
        nonlocal last
        gap=5.1-(time.monotonic()-last)
        if gap>0:time.sleep(gap)
        last=time.monotonic()
    def enter(board,choice,handle,password):
        delay();c=SSHConsole(members[board]);children.append(c);c.expect('[L]ogin [R]egister [Q]uit: ');c.send(choice);c.expect('Handle: ');c.send(handle);c.password(password)
        if choice=='r':c.password(password,'Confirm password (hidden): ')
        return c
    def approved(c,name):
        c.expect('Press Enter to continue: ');c.send('');c.expect(MENU);assert name.encode() in c.transcript
    def logout(c):c.raw(b'q');c.expect('Logged out.');c.finish()
    def approve(board):
        c=enter(board,'l','sysop',passwords[board]);approved(c,'Visit Board '+('A' if board==0 else 'B'))
        c.raw(b's');c.expect('[C]reate room: ');c.send('a');c.expect('Member handle: ');c.send(visitor);c.expect('Status updated; previous sessions expired.');c.expect('Press Enter to return to menu: ');c.send('');c.expect(MENU)
        return c
    def choose(c,key):
        c.raw(key);c.expect(CONFIRM);delay();c.send('y');c.expect('[L]ogin [R]egister [Q]uit: ')
    def login(c,name):
        c.send('l');c.expect('Handle: ');c.send(visitor);c.password(member_secret);approved(c,name)
    def back(c):c.expect(RETURN);c.send('');c.expect(PICK)
    def native_files(c):
        c.raw(b'f');c.expect('Faction // Files on your device');c.expect('Back to the BBS');c.raw(b'a');c.expect('After area ID [0]: ');c.send('');c.expect('Visit transfer area');c.expect('Enter to return to Files: ');c.send('');c.expect('Back to the BBS');c.raw(b'b');c.expect(MENU)
    try:
        c=enter(0,'r',visitor,member_secret);c.expect('Registration pending; sysop approval required.');c.finish()
        c=approve(0);c.raw(b'f');c.expect('files> ');c.send('create '+visitor+' members 1048576 Visit transfer area');c.expect('Created file area');c.expect('files> ');c.send('back');c.expect(MENU);logout(c)
        checks.append('first_board_has_its_own_approved_member_and_file_area')
        with tempfile.TemporaryDirectory(prefix='faction-board-visits-') as tmp:
            root=Path(tmp);root.chmod(0o700)
            def private(p,s):p.write_text(s);p.chmod(0o600)
            first=a.first_profile.resolve();second=a.second_profile.resolve();profile=tomllib.loads(first.read_text())
            key=root/'wrong';subprocess.run(['/usr/bin/ssh-keygen','-q','-t','ed25519','-N','','-f',str(key)],check=True,capture_output=True)
            wrong=root/'wrong-known';private(wrong,f'[{profile["host"]}]:{profile["port"]} '+key.with_suffix('.pub').read_text())
            unknown=root/'empty-known';private(unknown,'')
            paths=[first,second]
            for name,known in [('wrong',wrong),('unknown',unknown)]:
                d={**profile,'known_hosts':str(known)};p=root/(name+'.toml');private(p,'\n'.join(k+'='+json.dumps(v) for k,v in d.items())+'\n');paths.append(p)
            boards=root/'boards.toml';private(boards,catalog(paths))
            c=LauncherConsole([str(a.native_binary.resolve()),'--boards',str(boards)]);children.append(c);c.expect(PICK)
            c.raw(b'a');c.expect(CONFIRM);c.send('n');back(c);assert b'[L]ogin' not in c.transcript;checks.append('local_confirmation_cancel_keeps_launcher_off_network')
            choose(c,b'a');login(c,'Visit Board A');native_files(c);checks.append('host_native_files_negotiation_area_listing_and_bbs_return')
            rows=subprocess.check_output(['ps','-axo','pid=,ppid=,comm='],text=True).splitlines()
            children_count=sum(len(s.split())>=3 and s.split()[1]==str(c.pid) and s.split()[-1].endswith('ssh') for s in rows);assert children_count==1
            observations['host_active_ssh_children']=children_count
            c.raw(b'\x1db');c.expect(DISCONNECT);c.send('n');c.expect('Staying on this board.');c.raw(b'\x1db');c.expect(DISCONNECT);c.send('y');back(c);checks.append('local_disconnect_requires_confirmation_and_returns_to_launcher')
            choose(c,b'b');c.send('r');c.expect('Handle: ');c.send(visitor);c.password(member_secret);c.password(member_secret,'Confirm password (hidden): ');c.expect('Registration pending; sysop approval required.');back(c);checks.append('second_board_registration_independent_of_first_approval')
            choose(c,b'b');c.send('l');c.expect('Handle: ');c.send(visitor);c.password(member_secret);c.expect('No community access.');back(c);checks.append('second_board_pending_member_denied')
            op=approve(1);logout(op);choose(c,b'b');login(c,'Visit Board B');c.raw(b'q');c.expect('Logged out.');back(c);checks.append('second_board_local_approval_then_normal_disconnect_to_launcher')
            for key,notice in [(b'c','HOST IDENTIFICATION HAS CHANGED'),(b'd','Host key verification failed.')]:
                mark=len(c.transcript);c.raw(key);c.expect(CONFIRM);delay();c.send('y');c.expect(notice);c.expect('Connection stopped: ');back(c);assert b'[L]ogin' not in c.transcript[mark:]
            checks.append('changed_and_unknown_host_keys_fail_before_bbs_and_return_locally')
            c.raw(b'q');c.finish()
        # Actual NetBSD native client, not a host binary or a simulated stream.
        delay();guest=json.loads(a.guest_launcher_argv.read_text());c=LauncherConsole(guest);children.append(c);c.expect(PICK);choose(c,b'a');login(c,'Visit Board A');native_files(c)
        code="import subprocess; rows=[s.split() for s in subprocess.check_output(['ps','-ww','-U','factionbuild','-o','pid,ppid,comm'],text=True).splitlines()[1:]]; p=[s[0] for s in rows if len(s)>=3 and s[2].rsplit('/',1)[-1]=='faction-client']; assert len(p)==1; n=sum(len(s)>=3 and s[1]==p[0] and s[2].rsplit('/',1)[-1]=='ssh' for s in rows); assert n==1; print(n)"
        observations['guest_active_ssh_children']=int(remote('python3.13 -c '+shlex.quote(code)).strip())
        c.raw(b'\x1db');c.expect(DISCONNECT);c.send('y');back(c);choose(c,b'b');login(c,'Visit Board B');c.raw(b'q');c.expect('Logged out.');back(c);c.raw(b'q');c.finish();checks.append('actual_netbsd_native_client_visits_both_boards_with_fresh_login')
        lwps=(remote("ps -o nlwp= -p $(cat /var/run/faction/terminal.pid)")+remote("ps -o nlwp= -p $(cat /var/run/faction/terminal.pid)",True)).split();assert list(map(int,lwps))==[4,4];observations['board_owner_lwps']=list(map(int,lwps));checks.append('one_local_ssh_child_per_visit_and_unchanged_owner_threads')
        assert all(member_secret.encode() not in c.transcript for c in children)
        result=dict(result='pass',count=len(checks),checks=checks,observations=observations,normal_sessions_per_board=8,admission_refill_ms=5000,remote_profile_import=False,federation_claim=False,capacity_claim=False)
        a.output.write_text(json.dumps(result,indent=2,sort_keys=True)+'\n');a.output.chmod(0o600);print(f'Actual two-board native visits passed {len(checks)} controls.')
    finally:
        for c in children:c.close()

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--disposable-fixture',action='store_true',required=True)
    for n in ['admin-argv','second-admin-argv','first-member-argv','second-member-argv','first-password-file','second-password-file','first-profile','second-profile','native-binary','guest-launcher-argv','output']:p.add_argument('--'+n,type=Path,required=True)
    run(p.parse_args())
