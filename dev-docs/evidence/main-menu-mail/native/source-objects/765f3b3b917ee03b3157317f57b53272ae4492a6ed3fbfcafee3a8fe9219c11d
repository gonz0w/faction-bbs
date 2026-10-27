#!/usr/bin/env python3
"""Synthetic local/worker calendar/login controls. No clock changes or listeners."""
import argparse
import datetime
import json
import os
import secrets
import subprocess
import tempfile
from pathlib import Path
from smoke_local_console import Console
from smoke_member_worker import Worker

FOOTER = 'Ctrl-C exits; Esc: Enter to recover.'
HEADER = "version=1\nenabled=true\ntimezone='UTC'\n"
def rule(pack, start, end):
    return f"[[season]]\npack='{pack}'\nstart='{start}'\nend='{end}'\npriority=10\n"

def smoke(a):
    password = secrets.token_hex(20)
    children, checks = [], []
    with tempfile.TemporaryDirectory(prefix='faction-seasonal-') as tmp:
        root=Path(tmp);board=root/'board';base=root/'base';base.mkdir()
        (base/'theme.toml').write_text("version=1\nid='base'\nboard='BASE BOARD'\nwelcome='BASE welcome'\n")
        calendar=root/'calendar';calendar.mkdir()
        for name in ['first','second','manual']:
            pack=calendar/name;pack.mkdir()
            (pack/'theme.toml').write_text(f"version=1\nid='{name}'\nwelcome='{name} welcome'\nidentity_ascii='login.txt'\nidentity_ansi='login.ans'\ncreator='Faction test'\nsource='Original synthetic fixture'\npermission='BSD-3-Clause'\n")
            (pack/'login.txt').write_text(f'{name} ASCII LOGIN\n')
            (pack/'login.ans').write_bytes(f'\x1b[35m{name} ANSI LOGIN\x1b[0m'.encode())
        schedule=calendar/'schedule.toml'
        normal=HEADER+rule('first','01-01','07-01')+rule('second','07-01','01-01')
        schedule.write_text(normal)
        selected='first' if datetime.datetime.now(datetime.timezone.utc).month<7 else 'second'
        options=['--theme',str(base),'--theme-schedule',str(calendar)]
        def start(command, handle='sysop', extra=(), columns=80, rows=24):
            c=Console(a.binary,board,command,extra,columns=columns,rows=rows);children.append(c)
            c.expect('Handle: ');c.send(handle);return c
        def setup(command, handle):
            c=start(command,handle);c.password(password,'New password (12+ characters, hidden): ');c.password(password,'Confirm password (hidden): ')
            c.expect('Lobby created.' if command=='init' else 'Registered pending sysop approval.');c.finish()
        def login(mode='wide', extra=()):
            c=start('login',extra=[*options,'--plain' if mode=='plain' else '--ansi',*extra],columns=40 if mode=='compact' else 80,rows=18 if mode=='compact' else 24)
            c.password(password);return c
        def menu(c, mode):
            if mode in ['wide','compact']:c.expect('Press Enter to continue: ');c.send('')
            c.expect('Command (then Enter): ' if mode=='plain' else FOOTER)
        def quit(c, mode):
            c.send('q') if mode=='plain' else c.raw(b'q');c.expect('Logged out.');c.finish()
        try:
            setup('init','sysop');setup('register','pending')
            c=start('login','pending',['--ansi',*options]);c.password(password)
            c.expect('sysop approval required. No community access.');c.finish()
            assert b'LOGIN' not in c.transcript and b'BASE BOARD' not in c.transcript and b'\x1b[2J' not in c.transcript
            checks.append('pending_denied_before_calendar_identity')
            for mode in ['wide','compact','plain']:
                c=login(mode);c.expect(selected+(' ANSI LOGIN' if mode=='wide' else ' ASCII LOGIN'));menu(c,mode)
                if mode=='plain':assert b'\x1b[' not in c.transcript
                # File edits cannot change the selected snapshot during a draft.
                if mode=='wide':
                    c.raw(b'p');c.expect('Room ID: ');c.send('1');c.expect('Reply to post ID (blank for new message): ');c.send('');c.expect('Subject (blank for generated/reply default): ');c.send('Seasonal draft')
                    c.expect('Message (one line, up to 1024 bytes): ')
                    (calendar/selected/'login.txt').write_text('CHANGED AFTER LOGIN')
                    c.send('The chosen season stays coherent.');c.expect('Posted #1.');c.expect('Press Enter to return to menu: ');c.send('');c.expect(FOOTER)
                    assert b'CHANGED AFTER LOGIN' not in c.transcript
                    (calendar/selected/'login.txt').write_text(selected+' ASCII LOGIN\n')
                quit(c,mode);checks.append('calendar_'+mode+'_identity_and_session')
            schedule.write_text('broken calendar')
            c=login(extra=['--season',str(calendar/'manual')]);c.expect('manual ANSI LOGIN');menu(c,'wide');assert b'rejected' not in c.transcript;quit(c,'wide');checks.append('manual_bypasses_broken_calendar')
            schedule.write_text(HEADER.replace('true','false')+rule('missing','01-01','07-01'))
            c=login();menu(c,'wide');assert b'BASE BOARD' in c.transcript and b'LOGIN' not in c.transcript;quit(c,'wide');checks.append('disabled_calendar_uses_base_without_assets')
            schedule.write_text('broken calendar');c=login();menu(c,'wide');assert b'Season/schedule rejected:' in c.transcript and b'BASE BOARD' in c.transcript;quit(c,'wide');checks.append('invalid_local_calendar_preserves_base')
            schedule.write_text(normal)
            c=Worker(a.member,board,options);children.append(c);c.expect('[L]ogin [R]egister [Q]uit: ');c.send('l');c.expect('Handle: ');c.send('sysop');c.password(password);c.expect(selected+' ANSI LOGIN');menu(c,'wide');quit(c,'wide');checks.append('inherited_worker_scheduled_identity')
            # Each timezone gets a new authoring process. The BBS never changes TZ,
            # the host clock or a member-selected date. Explicit POSIX rules need
            # no network or named-zone package and exercise real libc DST handling.
            posix='EST5EDT,M3.2.0/2,M11.1.0/2'
            cases=[('UTC','2027-01-01T00:30:00+00:00','first'),('system','2027-01-01T00:30:00+00:00','second')]
            for zone,iso,expected in cases:
                schedule.write_text(HEADER.replace('UTC',zone)+rule('first','01-01','12-31')+rule('second','12-31','01-01'))
                stamp=str(int(datetime.datetime.fromisoformat(iso).timestamp()))
                result=subprocess.run([str(a.preview),'--theme',str(base),'--theme-schedule',str(calendar),'--timestamp',stamp,'--plain'],env=dict(os.environ,TZ=posix),capture_output=True,check=True,timeout=10)
                assert ('theme='+expected+',').encode() in result.stderr
            schedule.write_text(HEADER.replace('UTC','system')+rule('first','07-01','07-02')+rule('second','07-02','07-01'))
            stamp=str(int(datetime.datetime(2026,7,1,4,30,tzinfo=datetime.timezone.utc).timestamp()))
            result=subprocess.run([str(a.preview),'--theme',str(base),'--theme-schedule',str(calendar),'--timestamp',stamp,'--plain'],env=dict(os.environ,TZ=posix),capture_output=True,check=True,timeout=10)
            assert b'theme=first,' in result.stderr
            checks.append('actual_os_utc_local_year_boundary_and_dst')
            assert all(password.encode() not in c.transcript for c in children)
            if a.output:a.output.write_text(json.dumps({'result':'pass','count':len(checks),'checks':checks,'host_clock_changed':False,'capacity_claim':False},indent=2)+'\n')
            print(f'Seasonal software PTYs passed {len(checks)} controls: approval, ANSI/compact/plain, draft coherence, manual/disabled/error recovery, inherited worker and actual OS timezone/DST.')
        finally:
            for c in children:c.close()

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('binary',type=Path);p.add_argument('member',type=Path);p.add_argument('preview',type=Path);p.add_argument('--output',type=Path)
    a=p.parse_args();a.binary=a.binary.resolve(strict=True);a.member=a.member.resolve(strict=True);a.preview=a.preview.resolve(strict=True);smoke(a)
