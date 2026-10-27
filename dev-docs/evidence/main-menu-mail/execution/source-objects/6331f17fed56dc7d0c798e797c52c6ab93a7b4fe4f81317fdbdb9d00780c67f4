#!/usr/bin/env python3
"""First install of optional NetBSD file areas. Leaves the separate listener disabled."""
import argparse
import grp
import os
from pathlib import Path
import pwd
import stat
import subprocess
import sys
from install_netbsd import listen_address

ROOT = Path(__file__).resolve().parents[1]


def run(*args):
    subprocess.run(args, check=True)


def write(path, text, mode=0o644):
    fd = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL, mode)
    with os.fdopen(fd, 'w') as out:
        out.write(text)
    os.chmod(path, mode)


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--binary', type=Path, default=ROOT/'target/release/faction-files')
    p.add_argument('--listen-address', type=listen_address, default='127.0.0.1')
    p.add_argument('--port', type=int, default=2223)
    p.add_argument('--install', action='store_true')
    a = p.parse_args()
    try:
        if not sys.platform.startswith('netbsd') or os.getuid() != 0 or os.geteuid() != 0:
            raise ValueError('run preflight/install as root on the target NetBSD host')
        if not 1024 <= a.port <= 65535:
            raise ValueError('choose a dedicated port from 1024 through 65535')
        config = Path('/etc/faction/service.toml')
        owner = pwd.getpwnam('factiond')
        terminal = pwd.getpwnam('factionterm')
        for path in (config, a.binary.resolve()):
            m = path.lstat()
            if not stat.S_ISREG(m.st_mode) or (path == config and m.st_nlink != 1):
                raise ValueError('expected regular configuration/native binary')
        cm = config.stat()
        if cm.st_uid != 0 or cm.st_mode & 0o022:
            raise ValueError('service configuration must be administrator controlled')
        if a.binary.read_bytes()[:4] != b'\x7fELF':
            raise ValueError('build the native NetBSD faction-files binary first')
        if not Path('/var/db/faction/faction.sqlite3').is_file():
            raise ValueError('initialize the normal board first')
        if Path('/var/run/faction/terminal.sock').exists():
            raise ValueError('stop member admission and the board before setup')
        a.binary = a.binary.resolve()
        original = config.read_text()
        if 'files_uid' in original or 'files_auth_uid' in original:
            raise ValueError('files already configured; use the update guide')
        paths = ('/var/db/faction-files', '/usr/local/libexec/faction-files',
                 '/etc/faction/files.toml', '/etc/faction/service.before-files.toml',
                 '/etc/ssh/faction/files_sshd_config', '/etc/rc.d/faction_files_sshd',
                 '/etc/rc.conf.d/faction_files_sshd', '/var/log/faction-files.log')
        for value in paths:
            path = Path(value)
            if path.exists() or path.is_symlink():
                raise ValueError(f'refusing to replace {path}')
            for parent in path.parents:
                if parent.exists():
                    m = parent.lstat()
                    if not stat.S_ISDIR(m.st_mode) or m.st_uid != 0 or m.st_mode & 0o022:
                        raise ValueError(f'unsafe parent: {parent}')
        for name in ('factionfiles', 'factionfileauth'):
            try:
                pwd.getpwnam(name)
            except KeyError:
                pass
            else:
                raise ValueError(f'{name} already exists')
            try:
                grp.getgrnam(name)
            except KeyError:
                pass
            else:
                raise ValueError(f'{name} group already exists')
        print(f'Optional files: two streams, separate SSH {a.listen_address}:{a.port}.')
        print('No services, firewall rules or network tuning are enabled automatically.')
        if not a.install:
            print('Preflight passed. Repeat with --install to prepare the disabled service.')
            return
        os.umask(0o077)
        os.chdir('/')
        for name in ('factionfiles', 'factionfileauth'):
            run('/usr/sbin/groupadd', name)
            run('/usr/sbin/useradd', '-g', name, '-G', 'factionipc', '-d', '/nonexistent', '-s', '/bin/sh', name)
        worker = pwd.getpwnam('factionfiles')
        auth = pwd.getpwnam('factionfileauth')
        if len({0, owner.pw_uid, terminal.pw_uid, worker.pw_uid, auth.pw_uid}) != 5:
            raise ValueError('owner, terminal, file and key-lookup UIDs must differ')
        run('/usr/bin/install', '-d', '-o', 'factionfiles', '-g', 'factionfiles', '-m', '700', '/var/db/faction-files')
        run('/usr/bin/install', '-o', 'root', '-g', 'wheel', '-m', '755', str(a.binary.resolve()), '/usr/local/libexec/faction-files')
        run('/usr/bin/install', '-o', 'root', '-g', 'wheel', '-m', '555', str(ROOT/'deploy/netbsd/faction_files_sshd'), '/etc/rc.d/faction_files_sshd')
        ssh = (ROOT/'deploy/netbsd/files_sshd_config.in').read_text().replace('@LISTEN_ADDRESS@', a.listen_address).replace('@PORT@', str(a.port))
        write('/etc/ssh/faction/files_sshd_config', ssh, 0o600)
        write('/etc/faction/files.toml', f'owner_uid = {owner.pw_uid}\nsocket = "/var/run/faction/file-policy.sock"\nstore = "/var/db/faction-files"\n')
        write('/etc/rc.conf.d/faction_files_sshd', 'faction_files_sshd=NO\n')
        write('/var/log/faction-files.log', '', 0o600)
        write('/etc/faction/service.before-files.toml', original, 0o600)
        # Root keys must precede all TOML array tables, including classic_games.
        with config.open('r+') as out:
            out.write(f'files_uid = {worker.pw_uid}\nfiles_auth_uid = {auth.pw_uid}\n'+original)
            out.truncate()
        run('/usr/sbin/sshd', '-t', '-f', '/etc/ssh/faction/files_sshd_config')
        run('/usr/bin/su', '-m', 'factiond', '-c', '/usr/local/libexec/faction-service --check-config /etc/faction/service.toml')
        print('Installed, disabled. Start the backend, then faction_files_sshd onestart. See docs/file-areas.md.')
    except (OSError, ValueError, KeyError, subprocess.CalledProcessError) as exc:
        p.exit(1, f'Installation stopped: {exc}. No services were started; inspect partial setup before retrying.\n')


if __name__ == '__main__':
    main()
