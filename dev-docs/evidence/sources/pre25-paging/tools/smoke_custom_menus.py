#!/usr/bin/env python3
"""Real local/inherited-worker grid and SAUCE login controls; no listener or clock."""
import argparse
import json
import secrets
import tempfile
from pathlib import Path
from smoke_local_console import Console
from smoke_member_worker import Worker

FOOTER='Ctrl-C exits; Esc: Enter to recover.'
SYSOP='Sysop: [L]ist [A]pprove [V]revoke [C]reate room: '

def smoke(a):
    password=secrets.token_hex(20);children=[];checks=[]
    options=['--theme',str(a.theme)]
    with tempfile.TemporaryDirectory(prefix='faction-grid-') as tmp:
        state=Path(tmp)/'board'
        try:
            c=Console(a.binary,state,'init');children.append(c);c.expect('Handle: ');c.send('sysop');c.password(password,'New password (12+ characters, hidden): ');c.password(password,'Confirm password (hidden): ');c.expect('Lobby created.');c.finish()
            def login(mode):
                c=Console(a.binary,state,'login',[*options,'--plain' if mode=='plain' else '--ansi'],columns=40 if mode=='compact' else 80,rows=18 if mode=='compact' else 24);children.append(c);c.expect('Handle: ');c.send('sysop');c.password(password)
                if mode!='plain':c.expect('Press Enter to continue: ');c.send('');c.expect(FOOTER)
                else:c.expect('Command (then Enter): ')
                return c
            c=login('wide');assert '╓'.encode() in c.transcript and b'SAUCE' not in c.transcript and b'\x1b[5m' not in c.transcript
            c.raw(b'\x1b[C');c.expect('Membership and local controls');c.raw(b'\r');c.expect(SYSOP);c.send('l');c.expect('Press Enter to return to menu: ');c.send('');c.expect(FOOTER);c.raw(b'q');c.expect('Logged out.');c.finish();checks.append('local_grid_spatial_right_enter_and_ice_login')
            c=login('compact');assert b'PRIVATE FREQUENCIES' in c.transcript and '╓'.encode() not in c.transcript
            c.raw(b'\x1b[C');c.expect('New messages');c.raw(b'\r');c.expect('Room ID: ');c.send('1');c.expect('Press Enter to return to menu: ');c.send('');c.expect(FOOTER);c.raw(b'q');c.expect('Logged out.');c.finish();checks.append('compact_ordered_arrows_and_ascii_identity')
            c=login('plain');assert b'\x1b[' not in c.transcript;c.send('r');c.expect('Lobby');c.expect('Command (then Enter): ');c.send('q');c.expect('Logged out.');c.finish();checks.append('plain_hotkeys_and_ascii_identity')
            c=Worker(a.member,state,options);children.append(c);c.expect('[L]ogin [R]egister [Q]uit: ');c.send('l');c.expect('Handle: ');c.send('sysop');c.password(password);c.expect('Press Enter to continue: ');c.send('');c.expect(FOOTER);c.raw(b'\x1b[C');c.expect('Membership and local controls');c.raw(b'\r');c.expect(SYSOP);c.send('l');c.expect('Press Enter to return to menu: ');c.send('');c.expect(FOOTER);c.raw(b'q');c.expect('Logged out.');c.finish();checks.append('inherited_worker_grid_and_safe_import')
            assert all(password.encode() not in c.transcript for c in children)
            if a.output:a.output.write_text(json.dumps({'result':'pass','count':len(checks),'checks':checks,'capacity_claim':False},indent=2)+'\n')
            print(f'Custom menus passed {len(checks)} local/worker PTY controls.')
        finally:
            for c in children:c.close()

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('binary',type=Path);p.add_argument('member',type=Path);p.add_argument('--theme',type=Path,default=Path('themes/grid'));p.add_argument('--output',type=Path)
    a=p.parse_args();a.binary=a.binary.resolve(strict=True);a.member=a.member.resolve(strict=True);a.theme=a.theme.resolve(strict=True);smoke(a)
