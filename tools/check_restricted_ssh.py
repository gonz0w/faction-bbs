#!/usr/bin/env python3
"""Actual application SSH oracles. ONLY a fresh, explicitly marked disposable guest.

Requires the NetBSD fixture in dev-docs/research/0024-restricted-ssh-netbsd.md.
Does not provision users/SSH or alter host settings. Generated board credentials,
private transcript and connection arguments are never printed in sanitized evidence.
"""
import argparse
import fcntl
import json
import os
import pty
import secrets
import shlex
import struct
import subprocess
import termios
import time
from pathlib import Path
from smoke_local_console import Console


class SSHConsole(Console):
    def __init__(self, argv, columns=80, rows=24):
        self.pid, self.fd = pty.fork()
        if self.pid == 0:
            fcntl.ioctl(0, termios.TIOCSWINSZ, struct.pack('HHHH', rows, columns, 0, 0))
            os.environ['TERM'] = 'xterm-256color'
            os.closerange(3, os.sysconf('SC_OPEN_MAX'))
            os.execv('/usr/bin/ssh', argv)
        self.buffer, self.transcript, self.done = b'', b'', False


def run(args):
    assert args.disposable_fixture, 'explicit disposable-fixture flag required'
    password = secrets.token_hex(24)
    children, results, observations = [], [], {}
    common = ['/usr/bin/ssh', '-F', '/dev/null', '-o', 'IdentitiesOnly=yes',
              '-o', 'BatchMode=yes', '-o', 'ConnectTimeout=5', '-o', 'LogLevel=ERROR',
              '-o', 'StrictHostKeyChecking=yes', '-o', 'UserKnownHostsFile=' + str(args.known_hosts)]
    admin = common + ['-p', str(args.admin_port), '-i', str(args.admin_identity), 'root@' + args.host]
    member = common + ['-p', str(args.port), '-i', str(args.identity), 'factionterm@' + args.host]

    def remote(command, ok=True):
        p = subprocess.run(admin + [command], capture_output=True, timeout=20)
        assert not ok or p.returncode == 0, 'guest administrative fixture action failed'
        return p

    def call(options=(), command=(), identity=None):
        argv = list(member)
        if identity:
            argv[argv.index('-i')+1] = str(identity)
        # SSH options must precede destination.
        return subprocess.run(argv[:-1] + list(options) + [argv[-1]] + list(command),
                              capture_output=True, timeout=10)

    def passed(name):
        results.append({'oracle': name, 'result': 'pass'})

    def owner(command):
        c = SSHConsole(admin[:-1] + ['-tt', admin[-1],
                       "su - factiond -c '/usr/local/libexec/faction --state-dir /var/db/faction " + command + "'"])
        children.append(c)
        c.expect('Handle: '); c.send('sysop')
        if command == 'init':
            c.password(password, 'New password (12+ characters, hidden): ')
            c.password(password, 'Confirm password (hidden): ')
            c.expect('Lobby created.'); c.finish()
        else:
            c.password(password); c.expect('Command (then Enter): ')
        return c

    def status(action, name):
        c = owner('login'); c.send('s'); c.expect('[C]reate room: ')
        c.send(action); c.expect('Member handle: '); c.send(name)
        c.expect('Status updated; previous sessions expired.'); c.expect('Command (then Enter): ')
        c.send('q'); c.expect('Logged out.'); c.finish()

    def enter(choice, handle=None):
        c = SSHConsole(member[:-1] + ['-tt', member[-1]])
        children.append(c)
        if choice is None:
            return c
        c.expect('[L]ogin [R]egister [Q]uit: '); c.send(choice)
        if handle:
            c.expect('Handle: '); c.send(handle)
            if choice == 'r':
                c.password(password, 'New password (12+ characters, hidden): ')
                c.password(password, 'Confirm password (hidden): ')
            else:
                c.password(password)
        return c

    def approved(handle):
        c = enter('l', handle); c.expect('Press Enter to continue: '); c.send('')
        c.expect('Ctrl-C exits; Esc: Enter to recover.'); return c

    def pause_rate():
        # Default broker refills one admission token every five seconds. Tests do
        # not disable the gate, and deliberately space valid workflow batches.
        time.sleep(5.1)

    try:
        remote("test -f /etc/faction-disposable-test && test ! -e /var/db/faction/faction.sqlite3")
        owner('init')
        terminal_uid = remote('id -u factionterm').stdout.decode().strip()
        assert terminal_uid.isdecimal()
        worker_command = 'nohup /usr/local/libexec/faction-broker --socket /var/run/faction/terminal.sock --terminal-uid ' + terminal_uid + ' --worker /usr/local/libexec/faction-member --state-dir /var/db/faction > /home/factiond/broker.log 2>&1 < /dev/null & echo $! > /home/factiond/broker.pid'
        remote('su - factiond -c ' + shlex.quote(worker_command))
        time.sleep(1)
        remote('test -S /var/run/faction/terminal.sock')
        assert remote('su - factionterm -c "test -r /var/db/faction/faction.sqlite3"', ok=False).returncode != 0
        passed('separate_terminal_uid_cannot_read_database')
        broker_pid = int(remote('cat /home/factiond/broker.pid').stdout)
        assert remote(f'ps -p {broker_pid} -o comm=').stdout.strip() == b'/usr/local/libexec/faction-broker'
        before = len(remote(f'/usr/bin/fstat -p {broker_pid}').stdout.splitlines())
        assert before >= 4, 'broker descriptor baseline was not observable' 
        probe = """import array,os,socket
for count in [1,2,80]:
    s=socket.socket(socket.AF_UNIX,socket.SOCK_STREAM)
    s.settimeout(5);s.connect('/var/run/faction/terminal.sock')
    fd=os.open('/dev/null',os.O_RDONLY)
    packet=b'FBSTTY01'+bytes([5])+b'xterm'+bytes(58)
    s.sendmsg([packet],[(socket.SOL_SOCKET,socket.SCM_RIGHTS,array.array('i',[fd]*count))])
    os.close(fd);s.shutdown(socket.SHUT_WR)
    assert s.recv(1)==b''
    s.close()
"""
        remote('su - factionterm -c ' + shlex.quote('/usr/pkg/bin/python3.13 -c ' + shlex.quote(probe)))
        passed('malformed_or_extra_local_descriptors_cannot_open_worker')
        time.sleep(1.2)
        after = len(remote(f'/usr/bin/fstat -p {broker_pid}').stdout.splitlines())
        observations['rejected_handoff_descriptor_rows'] = {'before':before, 'after':after}
        assert before == after, 'rejected handoffs leaked broker descriptors'
        passed('rejected_handoffs_preserve_broker_descriptor_count')
        pause_rate()
        baseline=enter('q'); baseline.finish()
        passed('ordinary_ssh_shell_channel_enters_only_bbs')
        assert call(['-T']).returncode != 0
        passed('no_pty_denied_before_board_access')
        p = call(['-T'], ['printf FACTION_ESCAPE_SENTINEL'])
        assert p.returncode != 0 and b'FACTION_ESCAPE_SENTINEL' not in p.stdout
        passed('exec_denied_without_host_command')
        assert call(['-tt'], [' ']).returncode != 0
        passed('blank_exec_denied')
        assert call(['-s'], ['sftp']).returncode != 0
        passed('sftp_denied')
        for options, name in [
            (['-W', '127.0.0.1:22'], 'direct_tcp_forward_denied'),
            (['-N', '-o', 'ExitOnForwardFailure=yes', '-R', '0:127.0.0.1:22'], 'remote_tcp_forward_denied'),
            (['-N', '-o', 'ExitOnForwardFailure=yes', '-R', '/tmp/faction-denied.sock:127.0.0.1:22'], 'remote_unix_forward_denied')]:
            assert call(options).returncode != 0
            passed(name)
        assert call(['-T'], identity=args.unknown_identity).returncode == 255
        passed('unknown_transport_key_denied')
        c = enter('r', 'alice'); c.expect('Registered pending sysop approval.'); c.finish()
        c = enter('l', 'alice'); c.expect('Account Pending; sysop approval required. No community access.'); c.finish()
        assert b'Press Enter to continue' not in c.transcript
        passed('real_registration_pending_before_identity_and_menu')
        c = enter('r', 'bobby'); c.expect('Registered pending sysop approval.'); c.finish()
        status('a', 'alice'); status('a', 'bobby')
        alice, bob = approved('alice'), approved('bobby')
        alice.raw(b'p'); alice.expect('Room ID: '); alice.send('1')
        alice.expect('Reply to post ID (blank for new message): '); alice.send('')
        alice.expect('Message (one line, up to 1024 bytes): '); alice.send('SSH fixture durable room post')
        alice.expect('Posted #1.'); alice.expect('Press Enter to return to menu: '); alice.send('')
        alice.expect('Ctrl-C exits; Esc: Enter to recover.')
        passed('approved_ansi_hotkey_room_post')
        alice.raw(b'c'); bob.raw(b'c')
        alice.expect('chat[#lobby]> '); bob.expect('chat[#lobby]> ')
        snapshot = remote("ps -ax -o user= -o rss= -o vsz= -o comm=").stdout.decode()
        sample = {}
        for line in snapshot.splitlines():
            fields = line.split(None, 3)
            if len(fields) != 4:
                continue
            user, rss, vsz, name = fields
            role = name.strip().rsplit('/', 1)[-1]
            if role in ['faction-broker', 'faction-member', 'faction-ssh']:
                sample.setdefault(role, []).append({'rss_kib': int(rss), 'vsz_kib': int(vsz)})
        assert len(sample.get('faction-member', [])) == 2
        assert len(sample.get('faction-broker', [])) == 1
        observations['two_member_chat_process_sample'] = sample
        bob.raw(b'kept draft'); alice.send('SSH simultaneous arrival')
        bob.expect('SSH simultaneous arrival'); bob.expect('chat[#lobby]> kept draft')
        passed('simultaneous_ssh_chat_preserves_draft')
        status('v', 'alice'); alice.send('/poll')
        alice.expect('session expired; log in again'); alice.finish(1)
        passed('active_ssh_chat_revocation_closes_session')
        bob.resize(30, 10); bob.expect('Chat closed after resize');
        bob.expect('Press Enter to return to menu: '); bob.send('')
        bob.expect('Command (then Enter): '); bob.send('q'); bob.expect('Logged out.'); bob.finish()
        passed('ssh_window_resize_plain_fallback_and_logout')
        pause_rate()
        c = enter('l', 'alice'); c.expect('Account Revoked; sysop approval required. No community access.'); c.finish()
        passed('revoked_reconnect_cannot_reenter')
        status('a', 'alice'); pause_rate(); c = approved('alice'); c.raw(b'c'); c.expect('chat[#lobby]> ')
        assert remote("ps -ax -o comm= | awk '$1==\"/usr/local/libexec/faction-member\" {n++} END {print n+0}'").stdout.strip() == b'1', 'live worker was not observable before disconnect'
        os.kill(c.pid, 9); os.close(c.fd); os.waitpid(c.pid, 0); c.done=True; children.remove(c)
        time.sleep(2)
        count = remote("ps -ax -o comm= | awk '$1==\"/usr/local/libexec/faction-member\" {n++} END {print n+0}'").stdout.strip()
        assert count == b'0', 'worker survived disconnected SSH transport'
        passed('disconnect_stops_trusted_worker')
        # Structural snapshots + real fresh login are covered again by native PTY
        # recovery tests; this oracle checks operator-only command syntax at SSH.
        p = call(['-T'], ['/usr/local/libexec/faction --state-dir /var/db/faction export /tmp/leak'])
        assert p.returncode != 0
        remote('test ! -e /tmp/leak')
        passed('owner_export_unreachable_from_ssh_exec')
        if args.lifecycle_output:
            from check_service_lifecycle import exercise
            evidence = exercise(remote, approved, enter, children)
            args.lifecycle_output.write_text(json.dumps(evidence, indent=2)+'\n')
        for c in children:
            assert password.encode() not in c.transcript, 'BBS credential echoed'
    finally:
        for c in children:
            c.close()
    args.output.write_text(json.dumps({'schema':1, 'scope':'NetBSD disposable real application SSH', 'results':results, 'observations':observations},indent=2)+'\n')
    print(f'{len(results)} actual application SSH oracles passed; no credentials or transcripts emitted.')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--disposable-fixture', action='store_true')
    parser.add_argument('--host', required=True)
    parser.add_argument('--port', required=True, type=int)
    parser.add_argument('--admin-port', required=True, type=int)
    parser.add_argument('--identity', required=True, type=Path)
    parser.add_argument('--admin-identity', required=True, type=Path)
    parser.add_argument('--unknown-identity', required=True, type=Path)
    parser.add_argument('--known-hosts', required=True, type=Path)
    parser.add_argument('--output', required=True, type=Path)
    parser.add_argument('--lifecycle-output', type=Path)
    run(parser.parse_args())
