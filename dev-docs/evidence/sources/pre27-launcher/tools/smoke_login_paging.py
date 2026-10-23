#!/usr/bin/env python3
"""Bounded long-art local/worker PTY controls; no listener, clock or real board."""
import argparse
import json
import secrets
import tempfile
from pathlib import Path
from smoke_local_console import Console
from smoke_member_worker import Worker

FOOTER='Ctrl-C exits; Esc: Enter to recover.'
PAGE=lambda n:f'Art page {n}/3. Enter: next; S+Enter: skip: '

def smoke(a):
    password=secrets.token_hex(20);children=[];checks=[]
    with tempfile.TemporaryDirectory(prefix='faction-paging-') as tmp:
        state=Path(tmp)/'board';options=['--ansi','--theme',str(a.theme)]
        def start(command,handle='sysop',extra=(),columns=80,rows=24):
            c=Console(a.binary,state,command,extra,columns=columns,rows=rows);children.append(c);c.expect('Handle: ');c.send(handle);return c
        def login(handle='sysop',mode='wide'):
            c=start('login',handle,extra=[*options,*(['--plain'] if mode=='plain' else [])],columns=40 if mode=='compact' else 80,rows=18 if mode=='compact' else 24);c.password(password);return c
        def setup(command,handle):
            c=start(command,handle);c.password(password,'New password (12+ characters, hidden): ');c.password(password,'Confirm password (hidden): ');c.expect('Lobby created.' if command=='init' else 'Registered pending sysop approval.');c.finish()
        def admin(action):
            c=start('login');c.password(password);c.expect('Command (then Enter): ');c.send('s');c.expect('[C]reate room: ');c.send(action);c.expect('Member handle: ');c.send('member');c.expect('Status updated; previous sessions expired.');c.expect('Command (then Enter): ');c.send('q');c.expect('Logged out.');c.finish()
        def finish(c,mode='wide'):
            c.expect('Press Enter to continue: ');c.send('');c.expect(FOOTER);c.raw(b'q');c.expect('Logged out.');c.finish()
        try:
            setup('init','sysop');setup('register','member')
            c=login('member');c.expect('sysop approval required. No community access.');c.finish();assert b'INCOMING SIGNALS' not in c.transcript and b'\x1b[2J' not in c.transcript;checks.append('pending_denied_before_long_identity')
            c=login();c.expect(PAGE(1));assert b'THROUGH THE RELAY' not in c.transcript and FOOTER.encode() not in c.transcript
            c.send('q');c.expect(PAGE(1));assert b'Logged out.' not in c.transcript
            c.send('');c.expect(PAGE(2));assert b'THROUGH THE RELAY' in c.transcript and b'03 / YOUR LOCAL FREQUENCY' not in c.transcript
            c.send('');c.expect('03 / YOUR LOCAL FREQUENCY');finish(c);checks.append('three_pages_in_order_invalid_key_does_not_open_menu')
            c=login();c.expect(PAGE(1));c.send('s');finish(c);assert b'THROUGH THE RELAY' not in c.transcript;checks.append('skip_remaining_art_preserves_welcome_and_menu')
            c=login(mode='compact');finish(c);assert b'Art page ' not in c.transcript and b'PRIVATE FREQUENCIES' in c.transcript;checks.append('compact_ascii_fallback_without_paging')
            c=login(mode='plain');c.expect('Command (then Enter): ');assert b'\x1b[' not in c.transcript and b'Art page ' not in c.transcript;c.send('q');c.expect('Logged out.');c.finish();checks.append('plain_ascii_fallback_without_paging')
            c=login();c.expect(PAGE(1));c.resize(40,18);c.send('');finish(c);assert b'THROUGH THE RELAY' not in c.transcript;checks.append('resize_boundary_cancels_long_art_for_ascii')
            c=Worker(a.member,state,['--theme',str(a.theme)]);children.append(c);c.expect('[L]ogin [R]egister [Q]uit: ');c.send('l');c.expect('Handle: ');c.send('sysop');c.password(password);c.expect(PAGE(1));c.send('');c.expect(PAGE(2));c.send('');c.expect('03 / YOUR LOCAL FREQUENCY');finish(c);checks.append('inherited_worker_three_page_flow')
            admin('a');c=login('member');c.expect(PAGE(1));admin('v');c.send('');c.finish(1);assert b'THROUGH THE RELAY' not in c.transcript and FOOTER.encode() not in c.transcript;checks.append('approval_rechecked_before_next_page')
            assert all(password.encode() not in c.transcript for c in children)
            if a.output:a.output.write_text(json.dumps({'result':'pass','count':len(checks),'checks':checks,'capacity_claim':False},indent=2)+'\n')
            print(f'Long login paging passed {len(checks)} local/worker PTY controls.')
        finally:
            for c in children:c.close()

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('binary',type=Path);p.add_argument('member',type=Path);p.add_argument('--theme',type=Path,default=Path('themes/longform'));p.add_argument('--output',type=Path)
    a=p.parse_args();a.binary=a.binary.resolve(strict=True);a.member=a.member.resolve(strict=True);a.theme=a.theme.resolve(strict=True);smoke(a)
