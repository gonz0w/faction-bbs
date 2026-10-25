#!/usr/bin/env python3
"""Exercise the shipped theme over an already-owned restricted SSH fixture.

Requires synthetic credentials, approved local account and base theme selected by
the operator. Never provisions a public service or prints credentials/transcripts.
Native route uses the existing local profile and retains its control filters.
"""
import argparse
import json
import time
from pathlib import Path
from check_restricted_ssh import SSHConsole
from smoke_native_client_ssh import NativeConsole

FOOTER = 'Ctrl-C exits; Esc: Enter to recover.'

def smoke(a):
    assert a.disposable_fixture
    password = a.password_file.read_text().strip()
    member = json.loads(a.member_argv.read_text())
    children, checks = [], []
    try:
        for route, columns in [('ssh', 80), ('ssh', 40), ('native', 80)]:
            if checks: time.sleep(5.1)  # Respect normal admission, never tune it.
            c = (SSHConsole(member, columns=columns, rows=24 if columns == 80 else 18)
                 if route == 'ssh' else NativeConsole([str(a.native_binary.resolve()), '--profile', str(a.native_profile.resolve())]))
            children.append(c)
            c.expect('[L]ogin [R]egister [Q]uit: '); c.send('l')
            c.expect('Handle: '); c.send(a.handle); c.password(password)
            c.expect('Press Enter to continue: '); c.send(''); c.expect(FOOTER)
            assert b'LOCAL / MAIN' in c.transcript
            assert (b'Find a room; follow the talk' in c.transcript) == (columns == 80)
            start = len(c.transcript)
            c.raw(b'\x1b'); c.raw(b'[B'); c.expect(f'\x1b[6;{3 if columns == 80 else 2}H')
            assert b'Room ID: ' not in c.transcript[start:]
            c.raw(b'\x1b[A'); c.expect(f'\x1b[5;{3 if columns == 80 else 2}H'); c.raw(b'\r')
            c.expect('#1 (0 unread)'); c.expect('N/P:page U:next unread Q:main'); c.raw(b'q'); c.expect(FOOTER)
            c.raw(b'R'); c.expect('#1 (0 unread)'); c.expect('N/P:page U:next unread Q:main'); c.raw(b'q'); c.expect(FOOTER)
            c.raw(b'q'); c.expect('Logged out.'); c.finish()
            assert password.encode() not in c.transcript
            checks.append({'route': route, 'columns': columns, 'rows': 24 if columns == 80 else 18,
                           'arrows_and_enter': True, 'direct_hotkey': True, 'logout': True, 'result': 'pass'})
        if a.output: a.output.write_text(json.dumps({'result': 'pass', 'routes': checks, 'capacity_claim': False}, indent=2)+'\n')
        print('Restricted member menus passed: 80/40-column SSH and 80-column native client, lightbar/Enter/hotkeys and classic ASCII.')
    finally:
        for c in children: c.close()

if __name__ == '__main__':
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--disposable-fixture', action='store_true', required=True)
    p.add_argument('--member-argv', type=Path, required=True)
    p.add_argument('--native-binary', type=Path, required=True)
    p.add_argument('--native-profile', type=Path, required=True)
    p.add_argument('--password-file', type=Path, required=True)
    p.add_argument('--handle', default='sysop')
    p.add_argument('--output', type=Path)
    smoke(p.parse_args())
