#!/usr/bin/env python3
"""Non-posting journeys on an explicitly prepared seeded preview, using its real SSH config."""
import argparse
import fcntl
import os
from pathlib import Path
import pty
import struct
import termios
from smoke_local_console import Console

MENU = 'Ctrl-C exits; Esc: Enter to recover.'


class PreviewConsole(Console):
    def __init__(self, config, columns=100, rows=32):
        self.pid, self.fd = pty.fork()
        if self.pid == 0:
            fcntl.ioctl(0, termios.TIOCSWINSZ, struct.pack('HHHH', rows, columns, 0, 0))
            os.environ['TERM'] = 'xterm-ghostty'
            os.closerange(3, os.sysconf('SC_OPEN_MAX'))
            os.execv('/usr/bin/ssh', ['/usr/bin/ssh', '-F', str(config), 'faction'])
        self.buffer, self.transcript, self.done = b'', b'', False


def smoke(args):
    # Password stays in an owner-private file; never argv or output.
    info = args.password_file.lstat()
    if info.st_uid != os.getuid() or info.st_mode & 0o077 or not args.password_file.is_file():
        raise ValueError('private owned password file required')
    password = args.password_file.read_text().strip()
    c = PreviewConsole(args.ssh_config, columns=100 if not args.compact else 40,
                       rows=32 if not args.compact else 18)
    try:
        c.expect('[L]ogin [R]egister [Q]uit: ');c.send('l')
        c.expect('Handle: ');c.send(args.handle);c.password(password)
        c.expect('NIGHTWIRE / 01' if not args.compact else 'F A C T I O N')
        c.expect('Press Enter to continue: ');c.send('');c.expect(MENU)
        c.raw(b'r');c.expect('Introductions & signals');c.expect('N/P:page U:next unread Q:main')
        c.raw(b'b');c.expect('ROOM #2');c.expect('N/Enter:new H:history P:post')
        c.raw(b'q');c.expect(MENU)
        c.raw(b'w');c.expect('The graffiti wall');c.expect('Wall> ')
        c.send('q');c.expect(MENU)
        c.raw(b'm');c.expect('People & projects');c.expect('Member pages> ')
        c.send('q');c.expect(MENU)
        c.raw(b'c');c.expect('chat[#lobby]> ');c.send('/quit');c.expect(MENU)
        c.raw(b'd');c.expect('After Hours Arcade');c.expect('Ctrl-C returns to the BBS.')
        c.raw(b'n');c.expect('Neon Salvage //');c.expect('Ctrl-C returns to the BBS.')
        c.raw(b'b');c.expect(MENU)
        c.raw(b'q');c.expect('Logged out.');c.finish()
        assert password.encode() not in c.transcript
    finally:
        c.close()
    print('Prepared preview SSH passed: Ghostty TERM, Nightwire identity, named rooms, wall, '
          'member pages, local chat, Lua arcade and logout' + (' at 40x18.' if args.compact else '.'))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--ssh-config', type=Path, required=True)
    parser.add_argument('--password-file', type=Path, required=True)
    parser.add_argument('--handle', default='cam')
    parser.add_argument('--compact', action='store_true')
    smoke(parser.parse_args())
