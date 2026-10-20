#!/usr/bin/env python3
"""Exercise the real local console through disposable PTYs; Python stdlib only.

Pass the path to a built faction executable. Generates credentials/state in a
private temporary directory, never prints passwords/transcripts, opens no listener.
This is a CLI/auth regression check, not a remote deployment or capacity test.
"""
import argparse
import errno
import os
import pty
import secrets
import select
import signal
import tempfile
import termios
import time
from pathlib import Path


class Console:
    def __init__(self, binary, state, command):
        self.pid, self.fd = pty.fork()
        if self.pid == 0:
            os.execv(str(binary), [str(binary), '--state-dir', str(state), command])
        self.buffer = b''
        self.transcript = b''
        self.done = False

    def expect(self, text):
        wanted = text.encode()
        deadline = time.monotonic() + 15
        while wanted not in self.buffer:
            if time.monotonic() > deadline:
                raise AssertionError(f'console timeout waiting for {text!r}')
            if not select.select([self.fd], [], [], 0.2)[0]:
                continue
            try:
                data = os.read(self.fd, 8192)
            except OSError as exc:
                if exc.errno != errno.EIO:
                    raise
                data = b''
            if not data:
                raise AssertionError(f'console exited before {text!r}')
            self.buffer += data
            self.transcript += data
        _, self.buffer = self.buffer.split(wanted, 1)

    def send(self, value):
        os.write(self.fd, value.encode() + b'\n')

    def password(self, value, prompt='Password (hidden): '):
        self.expect(prompt)
        assert not termios.tcgetattr(self.fd)[3] & termios.ECHO, 'password echo enabled'
        self.send(value)

    def finish(self, code=0):
        deadline = time.monotonic() + 15
        while time.monotonic() < deadline:
            pid, status = os.waitpid(self.pid, os.WNOHANG)
            if pid:
                self.done = True
                assert os.waitstatus_to_exitcode(status) == code, 'unexpected console exit'
                assert termios.tcgetattr(self.fd)[3] & termios.ECHO, 'terminal echo not restored'
                return
            select.select([], [], [], 0.02)
        raise AssertionError('console failed to exit')

    def close(self):
        if not self.done:
            try:
                os.kill(self.pid, signal.SIGKILL)
                os.waitpid(self.pid, 0)
            except ProcessLookupError:
                pass
        os.close(self.fd)


def smoke(binary):
    children = []
    root_secret, member_secret, recovered = [secrets.token_hex(20) for _ in range(3)]
    with tempfile.TemporaryDirectory(prefix='faction-console-') as root:
        state = Path(root) / 'board'

        def start(command, name):
            c = Console(binary, state, command)
            children.append(c)
            c.expect('Handle: ')
            c.send(name)
            return c

        def setup(command, name, password, expected):
            c = start(command, name)
            c.password(password, 'New password (12+ characters, hidden): ')
            c.password(password, 'Confirm password (hidden): ')
            c.expect(expected)
            c.finish()

        def login(name, password):
            c = start('login', name)
            c.password(password)
            return c

        def admin(action, name):
            c = login('sysop', root_secret)
            c.expect('Command (then Enter): ')
            c.send('s')
            c.expect('[C]reate room: ')
            c.send(action)
            c.expect('Member handle: ')
            c.send(name)
            c.expect('Status updated; previous sessions expired.')
            c.expect('Command (then Enter): ')
            c.send('q')
            c.expect('Logged out.')
            c.finish()

        try:
            setup('init', 'sysop', root_secret, 'Sysop initialized. Lobby created.')
            setup('register', 'member', member_secret, 'Registered pending sysop approval.')
            c = login('member', member_secret)
            c.expect('Account Pending; sysop approval required. No community access.')
            c.finish()
            assert b'Command (then Enter)' not in c.transcript
            admin('a', 'member')
            c = login('member', member_secret)
            c.expect('Command (then Enter): ')
            c.send('p')
            c.expect('Room ID: ')
            c.send('1')
            c.expect('Reply to post ID (blank for new message): ')
            c.send('')
            c.expect('Message (one line, up to 1024 bytes): ')
            c.send('First room post from a real terminal.')
            c.expect('Posted #1.')
            c.expect('Command (then Enter): ')
            c.send('n')
            c.expect('Room ID: ')
            c.send('1')
            c.expect('#1 by member')
            c.expect('First room post from a real terminal.')
            c.expect('More? [y/N]: ')
            c.send('n')
            c.expect('Command (then Enter): ')
            # Revoke in an independent process while this member remains connected.
            admin('v', 'member')
            c.send('r')
            c.expect('session expired; log in again')
            c.finish(code=1)
            admin('a', 'member')
            c = login('member', member_secret)
            c.expect('Command (then Enter): ')
            c.send('r')
            c.expect('Lobby (0 unread)')
            c.expect('Command (then Enter): ')
            c.send('q')
            c.expect('Logged out.')
            c.finish()
            c = start('recover', 'sysop')
            c.expect('Type RECOVER: ')
            c.send('RECOVER')
            c.password(recovered, 'New password (12+ characters, hidden): ')
            c.password(recovered, 'Confirm password (hidden): ')
            c.expect('Sysop password reset; prior sessions expired.')
            c.finish()
            c = login('sysop', root_secret)
            c.expect('invalid credentials')
            c.finish(code=1)
            c = login('sysop', recovered)
            c.expect('Command (then Enter): ')
            c.send('q')
            c.expect('Logged out.')
            c.finish()
            c = start('login', 'member')
            c.password('x' * 129)
            c.expect('password exceeds 128 bytes')
            c.finish(code=1)
            for c in children:
                assert all(s.encode() not in c.transcript for s in [root_secret, member_secret, recovered]), 'credential leaked'
            print('PTY smoke passed: hidden/bounded passwords, bootstrap, pending denial, approval, durable post/unread, active revocation, recovery and echo restoration.')
        finally:
            for c in children:
                c.close()


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('binary', type=Path)
    smoke(parser.parse_args().binary.resolve(strict=True))
