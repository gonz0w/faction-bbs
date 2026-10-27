#!/usr/bin/env python3
"""Control an explicitly prepared, private NetBSD ARM64 preview; never install a host."""
import argparse
import contextlib
import fcntl
import json
import os
from pathlib import Path
import signal
import socket
import stat
import subprocess
import sys
import time


def private_directory(path):
    info = path.lstat()
    if not stat.S_ISDIR(info.st_mode) or info.st_uid != os.getuid() or info.st_mode & 0o077:
        raise ValueError('state directory must be an owned, non-symlink mode-0700 directory')
    return path.resolve()


def private_json(path):
    info = path.lstat()
    if (not stat.S_ISREG(info.st_mode) or info.st_uid != os.getuid()
            or info.st_mode & 0o077 or info.st_nlink != 1 or info.st_size > 16384):
        raise ValueError('private state file has unsafe ownership, mode, type, links or size')
    return json.loads(path.read_text())


def save(path, value):
    # State directory is owner-private. Refuse a pre-existing symlink/hard link.
    if path.exists() or path.is_symlink():
        private_json(path)
    temporary = path.with_name(path.name + '.new')
    with temporary.open('x') as output:
        os.chmod(temporary, 0o600)
        json.dump(value, output, indent=2)
        output.write('\n')
    temporary.replace(path)


def process(pid):
    if not isinstance(pid, int) or isinstance(pid, bool) or pid <= 1:
        return None
    fields = []
    for column in ('uid=', 'lstart=', 'command='):
        result = subprocess.run(['ps', '-ww', '-p', str(pid), '-o', column],
                                capture_output=True, text=True, check=False)
        if result.returncode or not result.stdout.strip():
            return None
        fields.append(result.stdout.strip())
    if fields[0] != str(os.getuid()):
        raise ValueError('recorded PID belongs to another user')
    return dict(pid=pid, birth=fields[1], command=fields[2])


def same_process(record):
    actual = process(record.get('pid'))
    if actual is not None and actual != record:
        raise ValueError('recorded PID identity changed; refusing to control it')
    return actual is not None


def read_config(state):
    config = private_json(state / 'playground.json')
    return validate_config(state, config)


def validate_config(state, config):
    if set(config) != {'version', 'kernel', 'admin_port', 'member_port'} or type(config['version']) is not int or config['version'] != 1:
        raise ValueError('unsupported playground configuration')
    for name in ('admin_port', 'member_port'):
        port = config[name]
        if type(port) is not int or not 1024 <= port <= 65535:
            raise ValueError('invalid loopback port')
    if config['admin_port'] == config['member_port']:
        raise ValueError('preview ports must be different')
    kernel = Path(config['kernel'])
    if not kernel.is_absolute() or not kernel.is_file():
        raise ValueError('prepared kernel is unavailable')
    if ',' in str(state) or ',' in str(kernel) or len(str(state / 'serial.sock').encode()) > 100:
        raise ValueError('preview paths do not fit the QEMU disk/serial interfaces')
    return config


def qemu_command(state, config):
    return ['qemu-system-aarch64', '-M', 'virt', '-accel', 'hvf', '-cpu', 'host',
            '-smp', '2', '-m', '1024', '-kernel', config['kernel'],
            '-append', 'root=NAME=netbsd-root -c',
            '-drive', f'file={state}/disk.qcow2,if=none,format=qcow2,id=hd0',
            '-device', 'virtio-blk-device,drive=hd0', '-netdev',
            f'user,id=net0,hostfwd=tcp:127.0.0.1:{config["admin_port"]}-:22,'
            f'hostfwd=tcp:127.0.0.1:{config["member_port"]}-:2222',
            '-device', 'virtio-net-device,netdev=net0', '-device', 'virtio-rng-device',
            '-display', 'none', '-monitor', 'none',
            '-serial', f'unix:{state}/serial.sock,server=on,wait=on']


def ssh_command(state, kind, config):
    args = private_json(state / f'{kind}-argv.json')
    return validate_ssh(args, kind, config)


def validate_ssh(args, kind, config):
    port = str(config['admin_port' if kind == 'admin' else 'member_port'])
    target = ('root' if kind == 'admin' else 'factionterm') + '@127.0.0.1'
    if (not isinstance(args, list) or not all(isinstance(v, str) for v in args)
            or not args or args[0] != '/usr/bin/ssh' or args[-1] != target
            or '-p' not in args or args[args.index('-p') + 1] != port
            or 'StrictHostKeyChecking=yes' not in args
            or 'BatchMode=yes' not in args):
        raise ValueError('expected the prepared pinned loopback SSH command')
    return args


def register(state, kernel):
    """Adopt an already booted, explicitly prepared clone, never an arbitrary PID."""
    if (state / 'playground.json').exists():
        raise ValueError('already registered; existing preview data is preserved')
    admin = private_json(state / 'ssh-admin.json')
    config = dict(version=1, kernel=str(kernel.resolve()),
                  admin_port=int(admin[admin.index('-p') + 1]),
                  member_port=int((state / 'member-port').read_text()))
    validate_config(state, config)
    validate_ssh(admin, 'admin', config)
    ssh_command(state, 'member', config)
    qemu = process(int((state / 'qemu.pid').read_text()))
    controller = process(int((state / 'controller.pid').read_text()))
    if (qemu is None or controller is None
            or not qemu['command'].endswith(' ' + ' '.join(qemu_command(state, config)[1:]))
            or str(state / 'serial-controller.py') not in controller['command']):
        raise ValueError('prepared clone process identity does not match')
    # Publish only after every adoption check has passed.
    save(state / 'admin-argv.json', admin)
    save(state / 'playground.json', config)
    save(state / 'processes.json', dict(qemu=qemu, controller=controller))


def records(state):
    data = private_json(state / 'processes.json')
    if set(data) != {'qemu', 'controller'}:
        raise ValueError('invalid process records')
    return data


def start(state, config):
    old = records(state)
    live = [same_process(old[name]) for name in ('qemu', 'controller')]
    if any(live):
        if all(live):
            print('Preview already running; data preserved.')
            return
        raise ValueError('partial running preview; inspect it before restarting')
    # No second opener for the disk, including an unrecorded surviving VM.
    opened = subprocess.run(['lsof', '-t', str(state / 'disk.qcow2')], capture_output=True)
    if opened.returncode not in (0, 1) or opened.stdout.strip():
        raise ValueError('preview disk is open or its ownership could not be checked')
    with contextlib.ExitStack() as stack:
        for name in ('admin_port', 'member_port'):
            check = stack.enter_context(socket.socket())
            check.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
            check.bind(('127.0.0.1', config[name]))
    # These are stale only after the process/disk checks above.
    for name in ('serial.sock',):
        (state / name).unlink(missing_ok=True)
    with (state / 'qemu.log').open('ab') as log:
        vm = subprocess.Popen(qemu_command(state, config), stdout=log, stderr=log,
                              start_new_session=True)
    qemu = process(vm.pid)
    if qemu is None:
        raise ValueError('QEMU exited; inspect private qemu.log')
    # Record QEMU before starting its serial companion, retaining recovery identity.
    save(state / 'processes.json', dict(qemu=qemu, controller=old['controller']))
    with (state / 'controller.log').open('ab') as log:
        controller = subprocess.Popen([sys.executable, str(Path(__file__).resolve()),
                                       '--state-dir', str(state), 'pump'],
                                      stdout=log, stderr=log, start_new_session=True)
    companion = process(controller.pid)
    if companion is None:
        raise ValueError('serial companion exited; inspect private controller.log')
    save(state / 'processes.json', dict(qemu=qemu, controller=companion))
    (state / 'qemu.pid').write_text(str(vm.pid))
    (state / 'controller.pid').write_text(str(controller.pid))
    print('Preview booting; use status, then connect when the guest is ready.')


def stop(state, config):
    data = records(state)
    if same_process(data['qemu']):
        subprocess.run(ssh_command(state, 'admin', config) + ['/sbin/shutdown -p now'],
                                capture_output=True, timeout=12, check=False)
        # Shutdown often closes SSH before its exit status arrives.
        deadline = time.monotonic() + 30
        while same_process(data['qemu']) and time.monotonic() < deadline:
            time.sleep(0.5)
        if same_process(data['qemu']):
            raise ValueError('guest did not power off; retained data/process, no forced kill')
    if same_process(data['controller']):
        os.kill(data['controller']['pid'], signal.SIGTERM)
    print('Preview powered off; disk, approved accounts, keys and saved work retained.')


def pump(state):
    """Serial companion for this prepared fixture's documented vmt boot workaround."""
    with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as stream:
        for _ in range(100):
            try:
                stream.connect(str(state / 'serial.sock'))
                break
            except (FileNotFoundError, ConnectionRefusedError):
                time.sleep(0.1)
        else:
            raise ValueError('preview serial socket unavailable')
        pending = b''
        stage = 0
        with (state / 'serial.log').open('ab', buffering=0) as log:
            while True:
                chunk = stream.recv(65536)
                if not chunk:
                    return
                log.write(chunk)
                pending = (pending + chunk)[-32768:]
                if b'uc> ' in pending and stage < 2:
                    pending = b''
                    command = (b'disable vmt\n', b'quit\n')[stage]
                    stage += 1
                    for byte in command:
                        stream.sendall(bytes([byte]))
                        time.sleep(0.04)


def main():
    os.umask(0o077)
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--state-dir', required=True, type=Path)
    commands = parser.add_subparsers(dest='command', required=True)
    registration = commands.add_parser('register', help='adopt a separately prepared running clone')
    registration.add_argument('--kernel', required=True, type=Path)
    for name in ('start', 'stop', 'status', 'connect', 'pump'):
        commands.add_parser(name)
    args = parser.parse_args()
    try:
        state = private_directory(args.state_dir)
        if args.command == 'pump':
            pump(state)
            return 0
        # Serialize lifecycle actions; never unlink a lock file.
        lock_path = state / 'control.lock'
        descriptor = os.open(lock_path, os.O_CREAT | os.O_RDWR | os.O_NOFOLLOW, 0o600)
        with os.fdopen(descriptor, 'r+') as lock:
            info = os.fstat(lock.fileno())
            if info.st_uid != os.getuid() or info.st_nlink != 1 or info.st_mode & 0o077:
                raise ValueError('unsafe control lock')
            fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
            if args.command == 'register':
                register(state, args.kernel)
                print('Prepared private preview registered; existing disk retained.')
                return 0
            config = read_config(state)
            if args.command == 'start':
                start(state, config)
            elif args.command == 'stop':
                stop(state, config)
            elif args.command == 'status':
                data = records(state)
                running = same_process(data['qemu'])
                print('Preview VM: ' + ('running' if running else 'stopped'))
                print(f'Member SSH: 127.0.0.1:{config["member_port"]} (host key pinned)')
                print('VM status does not prove BBS readiness; connect to verify.')
            elif args.command == 'connect':
                if not same_process(records(state)['qemu']):
                    raise ValueError('preview is stopped; start it first')
                command = ssh_command(state, 'member', config)
                fcntl.flock(lock, fcntl.LOCK_UN)
                return subprocess.run(command, check=False).returncode
        return 0
    except (OSError, ValueError, KeyError, IndexError, TypeError, subprocess.SubprocessError) as error:
        print(f'Preview: {error}', file=sys.stderr)
        return 1


if __name__ == '__main__':
    raise SystemExit(main())
