#!/usr/bin/env python3
"""Member worker on inherited PTY descriptors, deliberately no controlling /dev/tty."""
import argparse
import fcntl
import os
import pty
import secrets
import struct
import subprocess
import tempfile
import termios
from pathlib import Path
from smoke_local_console import Console


class Worker(Console):
    def __init__(self, binary, state):
        master, slave = pty.openpty()
        fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack('HHHH', 24, 80, 0, 0))
        self.pid = os.fork()
        if self.pid == 0:
            os.setsid()
            for number in [0, 1, 2]:
                os.dup2(slave, number)
            os.closerange(3, os.sysconf('SC_OPEN_MAX'))
            # Opening no terminal after setsid means no controlling terminal.
            os.execve(str(binary), [str(binary), '--state-dir', str(state)], {'TERM': 'xterm'})
        os.close(slave)
        self.fd, self.buffer, self.transcript, self.done = master, b'', b'', False


def smoke(binary, member):
    password = secrets.token_hex(24)
    children = []
    with tempfile.TemporaryDirectory(prefix='faction-member-') as root:
        state = Path(root) / 'board'
        def owner(command, handle):
            c = Console(binary, state, command); children.append(c)
            c.expect('Handle: '); c.send(handle); return c
        def worker(choice):
            c = Worker(member, state); children.append(c)
            c.expect('[L]ogin [R]egister [Q]uit: '); c.send(choice); return c
        try:
            c = owner('init', 'sysop')
            c.password(password, 'New password (12+ characters, hidden): ')
            c.password(password, 'Confirm password (hidden): ')
            c.expect('Lobby created.'); c.finish()
            c = worker('r'); c.expect('Handle: '); c.send('alice')
            c.password(password, 'New password (12+ characters, hidden): ')
            c.password(password, 'Confirm password (hidden): ')
            c.expect('Registered pending sysop approval.'); c.finish()
            c = worker('l'); c.expect('Handle: '); c.send('alice'); c.password(password)
            c.expect('Account Pending; sysop approval required. No community access.'); c.finish()
            assert b'Press Enter to continue' not in c.transcript
            c = owner('login', 'sysop'); c.password(password)
            c.expect('Command (then Enter): '); c.send('s'); c.expect('[C]reate room: ')
            c.send('a'); c.expect('Member handle: '); c.send('alice')
            c.expect('Status updated; previous sessions expired.')
            c.expect('Command (then Enter): '); c.send('q'); c.expect('Logged out.'); c.finish()
            c = worker('l'); c.expect('Handle: '); c.send('alice'); c.password(password)
            c.expect('Press Enter to continue: '); c.send('')
            c.expect('Ctrl-C exits; Esc: Enter to recover.'); c.raw(b'c')
            c.expect('chat[#lobby]> '); c.send('inherited descriptor chat')
            c.expect('inherited descriptor chat'); c.send('/quit')
            c.expect('Press Enter to return to menu: '); c.send('')
            c.expect('Ctrl-C exits; Esc: Enter to recover.'); c.raw(b'q'); c.expect('Logged out.'); c.finish()
            c = worker('recover'); c.expect('input too long'); c.finish(1)
            for command in ['init', 'recover', 'export', 'restore', 'verify-export']:
                p = subprocess.run([str(member), '--state-dir', str(state), command], capture_output=True, timeout=5)
                assert p.returncode != 0, 'owner command admitted by member worker'
            for c in children:
                assert password.encode() not in c.transcript, 'password echoed'
        finally:
            for c in children:
                c.close()
    print('Member worker passed: inherited PTY without controlling terminal, pending/approval, ANSI/chat, hidden passwords and owner-command denial.')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('binary', type=Path)
    parser.add_argument('member', type=Path)
    args = parser.parse_args()
    smoke(args.binary.resolve(), args.member.resolve())
