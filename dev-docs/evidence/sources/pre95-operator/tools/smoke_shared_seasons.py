#!/usr/bin/env python3
"""Seasonal login controls on an owned disposable NetBSD member-SSH fixture.

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
THEMES='/var/db/faction-season-fixture-themes'

def smoke(a):
    assert a.disposable_fixture
    admin=json.loads(a.admin_argv.read_text());member=json.loads(a.member_argv.read_text());password=a.password_file.read_text().strip()
    children,checks=[],[];calls=0;pending='pending-season-'+secrets.token_hex(3)
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
        month=int(remote('date -u +%m').stdout.decode());selected='winter' if month<7 else 'autumn'
        schedule="version=1\nenabled=true\ntimezone='UTC'\n[[season]]\npack='winter'\nstart='01-01'\nend='07-01'\npriority=10\n[[season]]\npack='autumn'\nstart='07-01'\nend='01-01'\npriority=10\n"
        write(THEMES+'/seasons/schedule.toml',schedule)
        config='version=1\nstate_dir="/var/db/faction-profile-fixture"\nsocket="/var/run/faction/terminal.sock"\nterminal_uid='+str(original['terminal_uid'])+'\nansi=true\ntheme="'+THEMES+'/base"\ntheme_schedule="'+THEMES+'/seasons"\n'
        start(config)
        c=connect();approved(c,selected.upper()+' FREQUENCIES');checks.append('restricted_ssh_calendar_selects_original_ansi_after_login')
        c.raw(b'p');c.expect('Room ID: ');c.send('1');c.expect('Reply to post ID (blank for new message): ');c.send('');c.expect('Subject (blank for generated/reply default): ');c.send('Season stays pinned');c.expect('Message (one line, up to 1024 bytes): ')
        # Neither current nor later sessions reread files from the running catalog.
        write(THEMES+'/seasons/schedule.toml','broken after startup')
        write(THEMES+'/seasons/'+selected+'/login.ans','\x1b]52;c;untrusted\x07')
        c.send('The loaded theme survives while composing.');c.expect('Posted #');c.expect('Press Enter to return to menu: ');c.send('');c.expect(FOOTER)
        assert b'CHANGED' not in c.transcript and b'\x1b]52' not in c.transcript
        quit(c)
        compact=connect(columns=40);approved(compact,'FACTION //');assert b'FREQUENCIES' not in compact.transcript;quit(compact)
        native=connect('native');approved(native,selected.upper()+' FREQUENCIES');assert b'\x1b]52' not in native.transcript;quit(native)
        checks.append('preloaded_theme_stays_coherent_through_draft_file_edits_and_later_logins')
        checks.append('compact_ascii_and_native_client_classic_controls')
        # Real pending-account login must never reveal an identity screen.
        time.sleep(5.1);calls+=1;c=SSHConsole(member);children.append(c);c.expect('[L]ogin [R]egister [Q]uit: ');c.send('r');c.expect('Handle: ');c.send(pending);c.password(password);c.password(password,'Confirm password (hidden): ');c.expect('Registration pending; sysop approval required.');c.finish()
        c=connect(handle=pending);c.expect('Sysop approval required. No community access.');c.finish();assert b'FREQUENCIES' not in c.transcript and b'FACTION //' not in c.transcript and b'\x1b[2J' not in c.transcript;checks.append('restricted_pending_member_gets_no_season_identity')
        remote('/etc/rc.d/faction_broker onestop')
        # Enabled calendars validate every pack before any listener binds.
        write(THEMES+'/seasons/schedule.toml',schedule)
        result=remote("su - factiond -c '/usr/local/libexec/faction-service --check-config /etc/faction/service.toml'",ok=False);assert result.returncode!=0
        remote('test ! -S /var/run/faction/terminal.sock');checks.append('invalid_enabled_art_fails_preflight_before_binding')
        write(THEMES+'/seasons/schedule.toml',schedule.replace('enabled=true','enabled=false'));start(config)
        c=connect();approved(c,'Faction BBS');assert b'FREQUENCIES' not in c.transcript;quit(c);remote('/etc/rc.d/faction_broker onestop');checks.append('disabled_calendar_skips_broken_assets_and_uses_base')
        # Recover manually even while the calendar file itself is malformed.
        write(THEMES+'/seasons/schedule.toml','broken calendar')
        remote('cp '+shlex.quote(str(a.themes_dir/'seasons/winter/login.ans'))+' '+THEMES+'/seasons/winter/login.ans; chown factiond:factionipc '+THEMES+'/seasons/winter/login.ans; chmod 600 '+THEMES+'/seasons/winter/login.ans')
        start(config+'season="'+THEMES+'/seasons/winter"\n')
        c=connect();approved(c,'WINTER FREQUENCIES');quit(c);checks.append('manual_override_recovers_broken_calendar')
        remote('/etc/rc.d/faction_broker onestop')
        for name in ['autumn','winter']:
            source=shlex.quote(str(a.themes_dir/'seasons'/name/'login.ans'))
            target=THEMES+'/seasons/'+name+'/login.ans'
            remote('cp '+source+' '+target+'; chown factiond:factionipc '+target+'; chmod 600 '+target)
        write(THEMES+'/seasons/schedule.toml',schedule.replace("timezone='UTC'","timezone='system'"))
        start(config)
        c=connect();approved(c,selected.upper()+' FREQUENCIES');quit(c)
        checks.append('system_timezone_shared_startup_and_member_login')
        assert all(password.encode() not in c.transcript for c in children)
        if a.output:a.output.write_text(json.dumps({'result':'pass','count':len(checks),'checks':checks,'selected_theme':selected,'host_clock_changed':False,'normal_sessions':8,'admission_refill_ms':5000,'capacity_claim':False},indent=2)+'\n')
        print(f'Shared seasonal fixture passed {len(checks)} restricted SSH/native/approval, immutable catalog, preflight and recovery controls.')
    finally:
        for c in children:c.close()
        remote('/etc/rc.d/faction_broker onestop || true')

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--disposable-fixture',action='store_true',required=True)
    for name in ['admin-argv','member-argv','native-binary','native-profile','password-file','themes-dir']:p.add_argument('--'+name,type=Path,required=True)
    p.add_argument('--output',type=Path);smoke(p.parse_args())
