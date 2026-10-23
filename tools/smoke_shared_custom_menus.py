#!/usr/bin/env python3
"""Custom-grid and SAUCE/iCE controls on an owned disposable NetBSD member-SSH fixture.

Requires the existing synthetic /var/db/faction-profile-fixture board, marker,
private admin/member argv, credentials and native profile. Changes only its
stopped service configuration and a new fixture theme directory; never use on a
real board. No guest clock, host policy or normal admission limit changes.
"""
import argparse
import json
import shlex
import subprocess
import time
import tomllib
from pathlib import Path
from check_restricted_ssh import SSHConsole
from smoke_native_client_ssh import NativeConsole

FOOTER='Ctrl-C exits; Esc: Enter to recover.'
THEMES='/var/db/faction-grid-fixture-themes'

def smoke(a):
    assert a.disposable_fixture
    admin=json.loads(a.admin_argv.read_text());member=json.loads(a.member_argv.read_text());password=a.password_file.read_text().strip()
    children,checks=[],[];calls=0
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
    try:
        remote('test -f /etc/faction-disposable-test')
        original=tomllib.loads(remote('cat /etc/faction/service.toml').stdout.decode())
        assert original['state_dir']=='/var/db/faction-profile-fixture'
        remote('/etc/rc.d/faction_broker onestop; test ! -e '+THEMES+'; cp -R '+shlex.quote(str(a.themes_dir))+' '+THEMES+'; chown -R factiond:factionipc '+THEMES+'; chmod -R go-rwx '+THEMES)
        config='version=1\nstate_dir="/var/db/faction-profile-fixture"\nsocket="/var/run/faction/terminal.sock"\nterminal_uid='+str(original['terminal_uid'])+'\nansi=true\ntheme="'+THEMES+'/grid"\n'
        start(config)
        c=connect();approved(c,'PRIVATE FREQUENCIES')
        assert '╓'.encode() in c.transcript and b'SAUCE' not in c.transcript and b'\x1b[5m' not in c.transcript
        # Different reads must restore an entry index, not replay spatial Down.
        c.raw(b'\x1b[C');c.expect('Membership and local controls')
        c.raw(b'\x1b[B');c.expect('Text, color and graphics')
        c.raw(b'\r');c.expect('Terminal: [C]lassic');c.expect('[D]one: ')
        c.send('d');c.expect(FOOTER);quit(c)
        checks.append('restricted_ssh_grid_spatial_focus_across_reads_and_enter')
        c=connect(columns=40);approved(c,'[ rooms | files | chat | doors ]')
        assert '╓'.encode() not in c.transcript
        c.raw(b'r');c.expect('Lobby');c.expect('N/P:page U:next unread Q:main');c.raw(b'q');c.expect(FOOTER);quit(c)
        checks.append('compact_ssh_ordered_menu_and_ascii_login')
        c=connect('native');approved(c,'PRIVATE FREQUENCIES')
        c.raw(b'\x1b[C');c.expect('Membership and local controls');c.raw(b'\r');c.expect('Sysop: [L]ist');c.expect('[C]reate room: ')
        c.send('l');c.expect('Press Enter to return to menu: ');c.send('');c.expect(FOOTER);quit(c)
        assert b'SAUCE' not in c.transcript and b'\x1b[5m' not in c.transcript
        checks.append('native_client_grid_and_canonical_cp437_ice')
        remote('/etc/rc.d/faction_broker onestop')
        # Duplicate coordinates fail before any terminal listener is bound.
        path=THEMES+'/grid/theme.toml';source=remote('cat '+path).stdout.decode()
        write(path,source.replace('row=2','row=1',1))
        result=remote("su - factiond -c '/usr/local/libexec/faction-service --check-config /etc/faction/service.toml'",ok=False)
        assert result.returncode!=0
        remote('test ! -S /var/run/faction/terminal.sock')
        checks.append('invalid_grid_fails_preflight_before_bind')
        assert all(password.encode() not in c.transcript for c in children)
        if a.output:a.output.write_text(json.dumps({'result':'pass','count':len(checks),'checks':checks,'normal_sessions':8,'admission_refill_ms':5000,'capacity_claim':False},indent=2)+'\n')
        print(f'Shared custom menus passed {len(checks)} actual restricted SSH/native/preflight controls.')
    finally:
        for c in children:c.close()
        remote('/etc/rc.d/faction_broker onestop || true')

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--disposable-fixture',action='store_true',required=True)
    for name in ['admin-argv','member-argv','native-binary','native-profile','password-file','themes-dir']:p.add_argument('--'+name,type=Path,required=True)
    p.add_argument('--output',type=Path);smoke(p.parse_args())
