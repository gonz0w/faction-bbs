#!/usr/bin/env python3
"""Prepare a first small-board install on NetBSD; start services separately.

Run without --install to inspect the plan. This is deliberately not an upgrader.
Only Python's standard library and NetBSD base account/install/SSH tools are used.
"""
from __future__ import annotations

import argparse
import grp
import ipaddress
import os
from pathlib import Path
import pwd
import stat
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]
BINARIES = ('faction', 'faction-member', 'faction-broker', 'faction-service', 'faction-tty', 'faction-daemon')
SERVICES = ('faction_broker', 'faction_sshd')
FRESH_PATHS = (
    '/home/factiond', '/home/factionterm', '/var/db/faction', '/var/run/faction',
    '/etc/faction', '/etc/ssh/faction', '/var/log/faction-broker.log',
    *(f'/usr/local/libexec/{name}' for name in BINARIES),
    *(f'/etc/rc.d/{name}' for name in SERVICES),
    *(f'/etc/rc.conf.d/{name}' for name in SERVICES),
)


def listen_address(value: str) -> str:
    try:
        address = ipaddress.ip_address(value)
    except ValueError as exc:
        raise argparse.ArgumentTypeError('choose a literal local IP address') from exc
    if address.is_unspecified or address.is_multicast:
        raise argparse.ArgumentTypeError('choose one interface address, not a wildcard or multicast')
    return str(address)


def read_public_keys(path: Path) -> str:
    """Accept plain OpenSSH public keys, then apply our fixed restricted options."""
    import base64
    import binascii
    import struct
    keys = []
    with path.open('rb') as stream:
        data = stream.read(65537)
    if len(data) > 65536:
        raise ValueError('public key input exceeds 64 KiB')
    for line in data.decode('ascii').splitlines():
        if not line.strip() or line.lstrip().startswith('#'):
            continue
        fields = line.split()
        if len(fields) < 2 or fields[0] not in (
            'ssh-ed25519', 'ecdsa-sha2-nistp256', 'ecdsa-sha2-nistp384',
            'ecdsa-sha2-nistp521', 'ssh-rsa', 'sk-ssh-ed25519@openssh.com',
            'sk-ecdsa-sha2-nistp256@openssh.com',
        ):
            raise ValueError('use plain .pub lines without authorized_keys options or private keys')
        try:
            blob = base64.b64decode(fields[1], validate=True)
            size = struct.unpack('>I', blob[:4])[0]
            if blob[4:4 + size].decode('ascii') != fields[0]:
                raise ValueError('public key type does not match its encoding')
        except (binascii.Error, struct.error, UnicodeError) as exc:
            raise ValueError('invalid public key encoding') from exc
        # No member-provided command/environment/options are copied.
        keys.append(f'restrict,pty {fields[0]} {fields[1]}\n')
    if not keys:
        raise ValueError('supply at least one invited public key')
    return ''.join(dict.fromkeys(keys))


def render(owner_uid: int, terminal_uid: int, address: str, port: int) -> tuple[str, str]:
    ssh = (ROOT / 'deploy/netbsd/sshd_config.in').read_text()
    ssh = ssh.replace('@FACTION_OWNER_UID@', str(owner_uid))
    ssh = ssh.replace('ListenAddress 127.0.0.1', f'ListenAddress {address}')
    ssh = ssh.replace('Port 2222\n', f'Port {port}\n', 1)
    service = (ROOT / 'deploy/netbsd/service.toml.in').read_text()
    service = service.replace('@FACTION_TERMINAL_UID@', str(terminal_uid))
    service += '\ntheme = "/var/db/faction/themes/base"\n'
    return ssh, service


def run(*args: str) -> None:
    subprocess.run(args, check=True)


def preflight(binary_dir: Path, keys: str) -> None:
    if not sys.platform.startswith('netbsd'):
        raise ValueError('installation/preflight must run on the target NetBSD host')
    for name in BINARIES:
        path = binary_dir / name
        if not path.is_file() or not os.access(path, os.X_OK):
            raise ValueError(f'build the native executable first: {path}')
        with path.open('rb') as stream:
            magic = stream.read(4)
        if magic != b'\x7fELF':
            raise ValueError(f'expected native NetBSD ELF executable: {path}')
    for name in ('factiond', 'factionterm'):
        try:
            pwd.getpwnam(name)
        except KeyError:
            pass
        else:
            raise ValueError(f'{name} already exists; use the manual setup/upgrade guide')
    try:
        grp.getgrnam('factionipc')
    except KeyError:
        pass
    else:
        raise ValueError('factionipc already exists; use the manual setup/upgrade guide')
    for value in FRESH_PATHS:
        path = Path(value)
        if path.exists() or path.is_symlink():
            raise ValueError(f'refusing to replace existing path: {path}')
        # Configuration and executable parents must be administrator controlled.
        for parent in path.parents:
            if parent.exists():
                info = parent.lstat()
                if not stat.S_ISDIR(info.st_mode) or info.st_uid != 0 or info.st_mode & 0o022:
                    raise ValueError(f'unsafe installation parent: {parent}')
    if not Path('/etc/ssh/ssh_host_ed25519_key').is_file():
        raise ValueError('create the target host SSH keys with ssh-keygen -A first')
    if 'PRIVATE KEY' in keys:
        raise ValueError('only public keys belong in the invitation file')


def install(binary_dir: Path, keys: str, address: str, port: int) -> None:
    if os.getuid() != 0 or os.geteuid() != 0:
        raise ValueError('--install requires root on the target NetBSD host')
    os.umask(0o077)
    run('/usr/sbin/groupadd', 'factionipc')
    for name in ('factiond', 'factionterm'):
        run('/usr/sbin/useradd', '-g', 'factionipc', '-d', f'/home/{name}', '-s', '/bin/sh', name)
    owner_uid = pwd.getpwnam('factiond').pw_uid
    terminal_uid = pwd.getpwnam('factionterm').pw_uid
    def directory(path: str, owner: str, group: str, mode: str) -> None:
        run('/usr/bin/install', '-d', '-o', owner, '-g', group, '-m', mode, path)
    def copy(source: Path, destination: str, mode: str = '755', owner: str = 'root', group: str = 'wheel') -> None:
        run('/usr/bin/install', '-o', owner, '-g', group, '-m', mode, str(source), destination)
    def write(path: str, data: str, mode: int) -> None:
        # First install only: never truncate an existing configuration/key file.
        fd = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL, mode)
        with os.fdopen(fd, 'w') as stream:
            stream.write(data)
        os.chmod(path, mode)
    directory('/home/factiond', 'factiond', 'factionipc', '700')
    directory('/var/db/faction', 'factiond', 'factionipc', '700')
    directory('/var/run/faction', 'factiond', 'factionipc', '710')
    for path in ('/home/factionterm', '/usr/local/libexec', '/etc/faction', '/etc/ssh/faction', '/etc/rc.conf.d'):
        directory(path, 'root', 'wheel', '755')
    for name in BINARIES:
        copy(binary_dir / name, f'/usr/local/libexec/{name}')
    for name in SERVICES:
        copy(ROOT / f'deploy/netbsd/{name}', f'/etc/rc.d/{name}', '555')
        write(f'/etc/rc.conf.d/{name}', f'{name}=NO\n', 0o644)
    for path in ('/var/db/faction/themes', '/var/db/faction/themes/base'):
        directory(path, 'factiond', 'factionipc', '700')
    copy(ROOT / 'themes/base/theme.toml', '/var/db/faction/themes/base/theme.toml', '600', 'factiond', 'factionipc')
    ssh, service = render(owner_uid, terminal_uid, address, port)
    write('/etc/ssh/faction/authorized_keys', keys, 0o644)
    write('/etc/ssh/faction/sshd_config', ssh, 0o600)
    write('/etc/faction/service.toml', service, 0o644)
    write('/var/log/faction-broker.log', '', 0o600)
    run('/usr/sbin/sshd', '-t', '-f', '/etc/ssh/faction/sshd_config')
    run('/usr/bin/su', '-m', 'factiond', '-c',
        '/usr/local/libexec/faction-service --check-config /etc/faction/service.toml')
    print('Installed. Both services are disabled; no listener was started.')
    print('Next, in a real terminal, create the first local sysop:')
    print("  su - factiond -c '/usr/local/libexec/faction --state-dir /var/db/faction init'")
    print('Customize /var/db/faction/themes/base/theme.toml, then follow docs/quickstart-netbsd.md.')


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary-dir', type=Path, default=ROOT / 'target/release')
    parser.add_argument('--authorized-keys', type=Path, required=True, help='plain invited OpenSSH .pub lines; never a private key')
    parser.add_argument('--listen-address', type=listen_address, default='127.0.0.1')
    parser.add_argument('--port', type=int, default=2222)
    parser.add_argument('--install', action='store_true', help='perform the first install as root; otherwise read-only preflight')
    args = parser.parse_args()
    try:
        if not 1024 <= args.port <= 65535:
            raise ValueError('use a dedicated port between 1024 and 65535')
        keys = read_public_keys(args.authorized_keys)
        binary_dir = args.binary_dir.resolve()
        preflight(binary_dir, keys)
        print(f'First install: shared backend, eight sessions, ANSI theme, SSH {args.listen_address} port {args.port}.')
        print('Accounts: factiond (private owner), factionterm (restricted member terminal), factionipc group.')
        print('No changes to the administrative SSH daemon, firewall, allocator, NTP or sysctls.')
        if args.install:
            install(binary_dir, keys, args.listen_address, args.port)
        else:
            print('Preflight passed; no changes made. Repeat with --install to install.')
    except (OSError, ValueError, subprocess.CalledProcessError) as exc:
        parser.exit(1, f'Installation stopped: {exc}\nNo services were automatically started. Inspect partial files before retrying.\n')


if __name__ == '__main__':
    main()
