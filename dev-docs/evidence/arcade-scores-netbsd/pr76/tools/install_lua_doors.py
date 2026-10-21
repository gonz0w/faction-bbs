#!/usr/bin/env python3
"""Explicit first installation of optional Lua doors after the normal NetBSD setup.

Leaves the door service disabled. Stop the board for installation, then start doors
before the shared backend. Does not install a privileged launcher or host policy.
"""
import argparse
import grp
import os
from pathlib import Path
import pwd
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]


def run(*args):
    subprocess.run(args, check=True)


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--binary', type=Path, default=ROOT/'target/release/faction-doors')
    a = p.parse_args()
    if not sys.platform.startswith('netbsd') or os.getuid() != 0 or os.geteuid() != 0:
        p.exit(1, 'Install as root on the target NetBSD host.\n')
    try:
        owner = pwd.getpwnam('factiond')
        config = Path('/etc/faction/service.toml')
        if not (Path('/var/db/faction/faction.sqlite3').is_file() and config.is_file() and a.binary.is_file()):
            raise ValueError('initialize the normal board and build the Lua evaluator first')
        if Path('/var/run/faction/terminal.sock').exists():
            raise ValueError('stop member SSH and the backend before installing doors')
        if 'doors_uid' in config.read_text():
            raise ValueError('doors already configured; use the manual update guide')
        try:
            pwd.getpwnam('factiongames')
        except KeyError:
            pass
        else:
            raise ValueError('factiongames already exists; inspect its purpose before manual setup')
        try:
            grp.getgrnam('factiongames')
        except KeyError:
            pass
        else:
            raise ValueError('factiongames group already exists')
        for name in ['/usr/local/share/faction/doors', '/usr/local/libexec/faction-doors',
                     '/etc/rc.d/faction_doors', '/etc/rc.conf.d/faction_doors',
                     '/var/log/faction-doors.log', '/var/run/faction-doors']:
            if Path(name).exists() or Path(name).is_symlink():
                raise ValueError('refusing to replace existing door installation')
        os.umask(0o077)
        run('/usr/sbin/groupadd', 'factiongames')
        run('/usr/sbin/useradd', '-g', 'factiongames', '-d', '/nonexistent', '-s', '/bin/sh', 'factiongames')
        games = pwd.getpwnam('factiongames')
        if games.pw_uid in (0, owner.pw_uid, pwd.getpwnam('factionterm').pw_uid):
            raise ValueError('game, owner and terminal identities must differ')
        for directory in ['/usr/local/share', '/usr/local/share/faction', '/usr/local/share/faction/doors']:
            run('/usr/bin/install', '-d', '-o', 'root', '-g', 'wheel', '-m', '755', directory)
        for script in sorted((ROOT/'doors/examples').glob('*.lua')):
            run('/usr/bin/install', '-o', 'root', '-g', 'wheel', '-m', '644', str(script), '/usr/local/share/faction/doors/'+script.name)
        run('/usr/bin/install', '-o', 'root', '-g', 'wheel', '-m', '755', str(a.binary), '/usr/local/libexec/faction-doors')
        run('/usr/bin/install', '-o', 'root', '-g', 'wheel', '-m', '555', str(ROOT/'deploy/netbsd/faction_doors'), '/etc/rc.d/faction_doors')
        run('/usr/bin/install', '-d', '-o', 'factiongames', '-g', 'factionipc', '-m', '2710', '/var/run/faction-doors')
        for name, value, mode in [('/etc/rc.conf.d/faction_doors', 'faction_doors=NO\n', 0o644),
                                  ('/var/log/faction-doors.log', '', 0o600)]:
            fd = os.open(name, os.O_WRONLY|os.O_CREAT|os.O_EXCL, mode)
            with os.fdopen(fd, 'w') as stream:
                stream.write(value)
            os.chmod(name, mode)
        # Preserve a protected copy before making the one optional config change.
        backup = config.with_name('service.before-doors.toml')
        fd = os.open(backup, os.O_WRONLY|os.O_CREAT|os.O_EXCL, 0o600)
        with os.fdopen(fd, 'w') as stream:
            stream.write(config.read_text())
        with config.open('a') as stream:
            stream.write(f'\n# Optional separate-UID Lua games. Remove to disable.\ndoors_uid = {games.pw_uid}\n')
        print('Lua doors installed, disabled. No listener was started.')
        print('Start /etc/rc.d/faction_doors onestart, then the normal shared backend and member SSH.')
        print('Edit /usr/local/share/faction/doors/doors.lua for your arcade. See docs/lua-doors.md.')
    except (OSError, ValueError, KeyError, subprocess.CalledProcessError) as exc:
        p.exit(1, f'Installation stopped: {exc}. Inspect partial setup before retrying.\n')


if __name__ == '__main__':
    main()
