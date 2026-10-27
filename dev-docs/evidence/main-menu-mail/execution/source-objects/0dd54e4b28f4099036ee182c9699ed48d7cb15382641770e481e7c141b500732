#!/usr/bin/env python3
"""Original Nightwire artwork login controls on an owned disposable NetBSD member-SSH fixture.

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
THEMES='/var/db/faction-nightwire-fixture-themes'

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
    def finish(c,plain=False):
        c.expect('Press Enter to continue: ');c.send('');c.expect('Command (then Enter): ' if plain else FOOTER)
        if plain:c.send('q');c.expect('Logged out.');c.finish()
        else:quit(c)
    def pages(c):
        c.expect('NIGHTWIRE / 01');c.expect('Art page 1/3. Enter: next; S+Enter: skip: ');c.send('');c.expect('NIGHTWIRE / 02');c.expect('Art page 2/3. Enter: next; S+Enter: skip: ');c.send('');c.expect('NIGHTWIRE / 03')
    try:
        remote('test -f /etc/faction-disposable-test')
        original=tomllib.loads(remote('cat /etc/faction/service.toml').stdout.decode());assert original['state_dir']=='/var/db/faction-profile-fixture'
        remote('/etc/rc.d/faction_broker onestop; test ! -e '+THEMES+'; cp -R '+shlex.quote(str(a.themes_dir))+' '+THEMES+'; chown -R factiond:factionipc '+THEMES+'; chmod -R go-rwx '+THEMES)
        config='version=1\nstate_dir="/var/db/faction-profile-fixture"\nsocket="/var/run/faction/terminal.sock"\nterminal_uid='+str(original['terminal_uid'])+'\nansi=true\ntheme="'+THEMES+'/nightwire"\n'
        start(config)
        c=connect();pages(c);finish(c);checks.append('restricted_ssh_original_three_page_art')
        c=connect('native');c.expect('Art page 1/3. Enter: next; S+Enter: skip: ');c.send('s');c.expect('F A C T I O N');finish(c);assert b'NIGHTWIRE / 02' not in c.transcript;checks.append('native_original_art_skip_to_compact_card')
        remote('/etc/rc.d/faction_broker onestop');start(config.replace('/nightwire"','/nightwire-short"'))
        c=connect();c.expect('NIGHTWIRE / 01');finish(c);assert b'Art page ' not in c.transcript and b'NIGHTWIRE / 02' not in c.transcript;checks.append('short_cover_identity_without_pager')
        for season in ['autumn','winter']:
            remote('/etc/rc.d/faction_broker onestop');start(config+'season="'+THEMES+'/nightwire-seasons/'+season+'"\n')
            c=connect();pages(c);c.expect(season.upper()+' / independent frequencies.');finish(c);checks.append(season+'_original_art_and_welcome')
        remote('/etc/rc.d/faction_broker onestop');start(config)
        c=connect(columns=40);c.expect('F A C T I O N');finish(c);assert b'Art page ' not in c.transcript;checks.append('compact_ssh_original_ascii_card')
        remote('/etc/rc.d/faction_broker onestop');start(config.replace('ansi=true','ansi=false'))
        c=connect();c.expect('F A C T I O N');finish(c,plain=True);assert b'\x1b[' not in c.transcript and b'Art page ' not in c.transcript;checks.append('plain_ssh_original_ascii_card')
        assert all(password.encode() not in c.transcript and b'SAUCE' not in c.transcript for c in children)
        if a.output:a.output.write_text(json.dumps({'result':'pass','count':len(checks),'checks':checks,'normal_sessions':8,'admission_refill_ms':5000,'capacity_claim':False},indent=2)+'\n')
        print(f'Original Nightwire artwork passed {len(checks)} actual restricted SSH/native login routes.')
    finally:
        for c in children:c.close()
        remote('/etc/rc.d/faction_broker onestop || true')

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--disposable-fixture',action='store_true',required=True)
    for name in ['admin-argv','member-argv','native-binary','native-profile','password-file','themes-dir']:p.add_argument('--'+name,type=Path,required=True)
    p.add_argument('--output',type=Path);smoke(p.parse_args())
