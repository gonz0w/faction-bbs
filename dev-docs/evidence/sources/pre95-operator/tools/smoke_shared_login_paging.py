#!/usr/bin/env python3
"""Long-form login-art paging controls on an owned disposable NetBSD member-SSH fixture.

Requires the existing synthetic /var/db/faction-profile-fixture board, marker,
private admin/member argv, credentials and native profile. Changes only its
stopped service configuration and a new fixture theme directory; never use on a
real board. No guest clock, host policy or normal admission limit changes.
"""
import argparse
import json
import shlex
import secrets
import subprocess
import time
import tomllib
from pathlib import Path
from check_restricted_ssh import SSHConsole
from smoke_native_client_ssh import NativeConsole

FOOTER='Ctrl-C exits; Esc: Enter to recover.'
THEMES='/var/db/faction-paging-fixture-themes'

def smoke(a):
    assert a.disposable_fixture
    admin=json.loads(a.admin_argv.read_text());member=json.loads(a.member_argv.read_text());password=a.password_file.read_text().strip()
    children,checks=[],[];calls=0;pending='paging-'+secrets.token_hex(3)
    def remote(command,data=None,ok=True):
        x=subprocess.run(admin+['set -e; PATH=/usr/sbin:/sbin:/usr/bin:/bin; export PATH; '+command],input=data,capture_output=True,timeout=45)
        assert not ok or x.returncode==0,'owned seasonal fixture action failed'
        return x
    def write(path,data,owner='factiond:factionipc'):
        remote('cat > '+shlex.quote(path)+'; chown '+owner+' '+shlex.quote(path)+'; chmod 600 '+shlex.quote(path),data.encode())
    def start(config):
        write('/etc/faction/service.toml',config,'root:factionipc');remote('chmod 640 /etc/faction/service.toml; /etc/rc.d/faction_broker onestart')
    def connect(route='ssh',columns=80,handle='sysop'):
        nonlocal calls
        if calls:time.sleep(5.1)
        calls+=1
        c=SSHConsole(member,columns=columns,rows=24 if columns==80 else 18) if route=='ssh' else NativeConsole([str(a.native_binary.resolve()),'--profile',str(a.native_profile.resolve())])
        children.append(c);c.expect('[L]ogin [R]egister [Q]uit: ');c.send('l');c.expect('Handle: ');c.send(handle);c.password(password);return c
    def approved(c,marker):
        c.expect(marker);c.expect('Press Enter to continue: ');c.send('');c.expect(FOOTER)
    def quit(c):c.raw(b'q');c.expect('Logged out.');c.finish()
    def page(n):return f'Art page {n}/3. Enter: next; S+Enter: skip: '
    def finish(c):c.expect('Press Enter to continue: ');c.send('');c.expect(FOOTER)
    def admin_action(c,action):
        c.raw(b's');c.expect('[C]reate room: ');c.send(action);c.expect('Member handle: ');c.send(pending);c.expect('Status updated; previous sessions expired.');c.expect('Press Enter to return to menu: ');c.send('');c.expect(FOOTER)
    try:
        remote('test -f /etc/faction-disposable-test')
        original=tomllib.loads(remote('cat /etc/faction/service.toml').stdout.decode())
        assert original['state_dir']=='/var/db/faction-profile-fixture'
        remote('/etc/rc.d/faction_broker onestop; test ! -e '+THEMES+'; cp -R '+shlex.quote(str(a.themes_dir))+' '+THEMES+'; chown -R factiond:factionipc '+THEMES+'; chmod -R go-rwx '+THEMES)
        config='version=1\nstate_dir="/var/db/faction-profile-fixture"\nsocket="/var/run/faction/terminal.sock"\nterminal_uid='+str(original['terminal_uid'])+'\nansi=true\ntheme="'+THEMES+'/longform"\n'
        start(config)
        c=connect();c.expect(page(1));assert b'THROUGH THE RELAY' not in c.transcript and FOOTER.encode() not in c.transcript
        # Corrupt the live file. This and later logins retain the preloaded catalog.
        write(THEMES+'/longform/login.ans','FIRST\x1b[17;1HLATER\x1b]52;c;bad\x07')
        c.raw(b'\n\n');c.expect(page(2));c.send('q');c.expect(page(2));assert b'Logged out.' not in c.transcript
        c.send('');c.expect('03 / YOUR LOCAL FREQUENCY');finish(c);quit(c)
        checks.append('restricted_ssh_ordered_pages_typeahead_and_preloaded_coherence')
        c=connect();c.expect(page(1));c.send('s');finish(c);assert b'THROUGH THE RELAY' not in c.transcript;quit(c);checks.append('skip_to_ascii_welcome_then_menu')
        c=connect(columns=40);finish(c);assert b'Art page ' not in c.transcript and b'PRIVATE FREQUENCIES' in c.transcript;quit(c);checks.append('compact_ssh_ascii_fallback_without_paging')
        c=connect('native');c.expect(page(1));c.send('');c.expect(page(2));c.send('');c.expect('03 / YOUR LOCAL FREQUENCY');finish(c);quit(c);checks.append('native_client_three_page_login')
        c=connect();c.expect(page(1));c.resize(40,18);finish(c);assert b'THROUGH THE RELAY' not in c.transcript;quit(c);checks.append('ssh_resize_cancels_art_at_drained_boundary')
        time.sleep(5.1);calls+=1;c=SSHConsole(member);children.append(c);c.expect('[L]ogin [R]egister [Q]uit: ');c.send('r');c.expect('Handle: ');c.send(pending);c.password(password);c.password(password,'Confirm password (hidden): ');c.expect('Registration pending; sysop approval required.');c.finish()
        c=connect(handle=pending);c.expect('Sysop approval required. No community access.');c.finish();assert b'INCOMING SIGNALS' not in c.transcript and b'\x1b[2J' not in c.transcript;checks.append('pending_ssh_never_gets_long_art')
        owner=connect();owner.expect(page(1));owner.send('s');finish(owner);admin_action(owner,'a')
        c=connect(handle=pending);c.expect(page(1));admin_action(owner,'v');c.send('');c.expect('session expired; log in again');c.finish(1)
        assert b'THROUGH THE RELAY' not in c.transcript and FOOTER.encode() not in c.transcript
        quit(owner);checks.append('current_revocation_denies_next_art_page')
        remote('/etc/rc.d/faction_broker onestop')
        result=remote("su - factiond -c '/usr/local/libexec/faction-service --check-config /etc/faction/service.toml'",ok=False);assert result.returncode!=0
        remote('test ! -S /var/run/faction/terminal.sock');checks.append('bad_later_page_fails_preflight_before_bind')
        assert all(password.encode() not in c.transcript for c in children)
        if a.output:a.output.write_text(json.dumps({'result':'pass','count':len(checks),'checks':checks,'normal_sessions':8,'admission_refill_ms':5000,'capacity_claim':False},indent=2)+'\n')
        print(f'Shared login paging passed {len(checks)} actual SSH/native/resize/revocation controls.')
    finally:
        for c in children:c.close()
        remote('/etc/rc.d/faction_broker onestop || true')

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--disposable-fixture',action='store_true',required=True)
    for name in ['admin-argv','member-argv','native-binary','native-profile','password-file','themes-dir']:p.add_argument('--'+name,type=Path,required=True)
    p.add_argument('--output',type=Path);smoke(p.parse_args())
