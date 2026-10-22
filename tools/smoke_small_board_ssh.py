#!/usr/bin/env python3
"""Rehearse the quickstart on a freshly installed, marked disposable NetBSD VM.

The two argv JSON files contain trusted local ssh command arrays including the
remote destination (never passwords). No endpoints/keys/transcripts are published.
Does NOT provision a host or run capacity/experimental-profile tests.
"""
import argparse
import json
from pathlib import Path
import secrets
import subprocess
import time
from check_restricted_ssh import SSHConsole


def smoke(args):
    assert args.disposable_fixture, 'only a fresh disposable VM is permitted'
    admin = json.loads(args.admin_argv.read_text())
    member = json.loads(args.member_argv.read_text())
    children, checks = [], []
    password = secrets.token_hex(24)

    def remote(command):
        result = subprocess.run(admin + ['set -e; PATH=/usr/sbin:/sbin:/usr/bin:/bin; export PATH; ' + command],
                                capture_output=True, timeout=30)
        assert result.returncode == 0, 'remote setup/control failed; inspect the disposable VM privately'
        return result.stdout.decode()

    def passed(name):
        checks.append(name)

    def owner(command):
        c = SSHConsole(admin[:-1] + ['-tt', admin[-1],
            "su - factiond -c '/usr/local/libexec/faction --state-dir /var/db/faction " + command + "'"])
        children.append(c)
        return c

    def enter(choice, handle):
        c = SSHConsole(member[:-1] + ['-tt', member[-1]])
        children.append(c)
        c.expect('[L]ogin [R]egister [Q]uit: ')
        c.send(choice)
        c.expect('Handle: ')
        c.send(handle)
        c.password(password)
        if choice == 'r':
            c.password(password, 'Confirm password (hidden): ')
        return c

    def menu(c):
        c.expect('Ctrl-C exits; Esc: Enter to recover.')

    def login(handle):
        c = enter('l', handle)
        c.expect('Quickstart Board')
        c.expect('Welcome to our board.')
        c.expect('Press Enter to continue: ')
        c.send('')
        menu(c)
        return c

    def pause(c):
        c.expect('Press Enter to return to menu: ')
        c.send('')
        menu(c)

    def logout(c):
        c.raw(b'q')
        c.expect('Until next time.')
        c.finish()

    try:
        remote('test -f /etc/faction-disposable-test; test ! -e /var/db/faction/faction.sqlite3; '
               'test "$(cat /etc/rc.conf.d/faction_broker)" = faction_broker=NO; '
               'test "$(cat /etc/rc.conf.d/faction_sshd)" = faction_sshd=NO; '
               'test ! -S /var/run/faction/terminal.sock; test ! -e /var/run/faction-sshd.pid; '
               'test ! -e /home/factiond/board-backup.json')
        passed('installed_without_starting_services')
        c = owner('init')
        c.expect('Handle: '); c.send('sysop')
        c.password(password, 'New password (12+ characters, hidden): ')
        c.password(password, 'Confirm password (hidden): ')
        c.expect('Sysop initialized. Lobby created.'); c.finish()
        passed('interactive_sysop_bootstrap')
        remote("su - factiond -c \"sed -e 's/Faction BBS/Quickstart Board/' -e 's/Make this board your own./Welcome to our board./' /var/db/faction/themes/base/theme.toml > /var/db/faction/themes/base/new.toml; mv /var/db/faction/themes/base/new.toml /var/db/faction/themes/base/theme.toml\"; "
               '/etc/rc.d/faction_broker onestart; /etc/rc.d/faction_sshd onestart')
        passed('ordinary_shared_backend_and_dedicated_ssh_start')
        remote('su - factionterm -c "test ! -r /var/db/faction/faction.sqlite3"; '
               'test ! -u /usr/local/libexec/faction-tty; '
               'test ! -u /usr/local/libexec/faction-service')
        passed('private_database_and_no_setuid_product_binaries')
        c = enter('r', 'alice')
        c.expect('Registration pending; sysop approval required.'); c.finish()
        c = enter('l', 'alice')
        c.expect('Sysop approval required. No community access.'); c.finish()
        assert b'Quickstart Board' not in c.transcript
        passed('registration_pending_cannot_see_board_identity_or_content')
        c = login('sysop')
        c.raw(b's'); c.expect('[C]reate room: '); c.send('a')
        c.expect('Member handle: '); c.send('alice')
        c.expect('Status updated; previous sessions expired.'); pause(c)
        logout(c)
        passed('sysop_approval_through_ansi_menu')
        alice = login('alice')
        # The initial menu focus is Rooms: down to New, then back to Rooms.
        alice.raw(b'\x1b[B'); alice.expect('New messages')
        alice.raw(b'\x1b[A'); alice.expect('Message rooms')
        alice.raw(b'\r'); alice.expect('lobby - Lobby'); pause(alice)
        passed('custom_identity_and_arrow_enter_navigation')
        alice.raw(b'p'); alice.expect('Room ID: '); alice.send('1')
        alice.expect('Reply to post ID (blank for new message): '); alice.send('')
        alice.expect('Subject (blank for generated/reply default): '); alice.send('')
        alice.expect('Message (one line, up to 1024 bytes): '); alice.send('Our first board message.')
        alice.expect('Posted #1.'); pause(alice)
        passed('hotkey_room_post')
        sysop = login('sysop')
        sysop.raw(b'n'); sysop.expect('Room ID: '); sysop.send('1')
        sysop.expect('Our first board message.'); sysop.expect('More? [y/N]: '); sysop.send('n'); pause(sysop)
        passed('peer_reads_durable_message')
        alice.raw(b'c'); sysop.raw(b'c')
        alice.expect('chat[#lobby]> '); sysop.expect('chat[#lobby]> ')
        alice.send('Hello from our small board.')
        sysop.expect('Hello from our small board.')
        for c in (alice, sysop):
            c.send('/quit'); menu(c); logout(c)
        passed('two_approved_members_exchange_local_chat')
        remote("printf 'faction_broker=YES\\nfaction_broker_backend=shared\\n' > /etc/rc.conf.d/faction_broker; "
               "printf 'faction_sshd=YES\\n' > /etc/rc.conf.d/faction_sshd; "
               '/etc/rc.d/faction_sshd onestop; /etc/rc.d/faction_broker onestop; '
               '/etc/rc.d/faction_broker onestart; /etc/rc.d/faction_sshd onestart')
        c = login('alice'); logout(c)
        passed('normal_restart_preserves_approved_access_and_theme')
        if args.reboot:
            subprocess.run(admin + ['/sbin/shutdown -r now'], capture_output=True, timeout=10)
            deadline = time.monotonic() + 90
            time.sleep(8)
            while time.monotonic() < deadline:
                try:
                    remote('/etc/rc.d/faction_broker onestatus; /etc/rc.d/faction_sshd onestatus')
                    break
                except (AssertionError, subprocess.TimeoutExpired):
                    time.sleep(1)
            else:
                raise AssertionError('normal boot services did not remain ready')
            # Ensure readiness survives boot-session teardown, rather than just
            # observing a socket during a transient startup window.
            time.sleep(3)
            c = login('alice'); logout(c)
            passed('fresh_boot_preserves_normal_service_and_login')
        remote('/etc/rc.d/faction_sshd onestop; /etc/rc.d/faction_broker onestop; '
               "printf 'faction_broker=YES\\nfaction_broker_backend=workers\\nfaction_broker_flags=\"--theme /var/db/faction/themes/base\"\\n' > /etc/rc.conf.d/faction_broker; "
               '/etc/rc.d/faction_broker onestart; /etc/rc.d/faction_sshd onestart')
        c = login('alice'); logout(c)
        passed('worker_rollback_uses_same_approved_account_and_theme')
        remote('/etc/rc.d/faction_sshd onestop; /etc/rc.d/faction_broker onestop; '
               "printf 'faction_broker=YES\\nfaction_broker_backend=shared\\n' > /etc/rc.conf.d/faction_broker; "
               '/etc/rc.d/faction_broker onestart; /etc/rc.d/faction_sshd onestart')
        c = login('alice'); logout(c)
        passed('return_to_shared_backend_preserves_member_access')
        remote("su - factiond -c '/usr/local/libexec/faction --state-dir /var/db/faction export /home/factiond/board-backup.json'; "
               "su - factiond -c '/usr/local/libexec/faction verify-export /home/factiond/board-backup.json'")
        passed('private_export_and_verify')
        for c in children:
            assert password.encode() not in c.transcript, 'a hidden credential was echoed'
        passed('credentials_never_echoed')
        return {'schema': 1, 'checks': checks, 'result': 'pass', 'reboot_exercised': args.reboot}
    finally:
        for c in children:
            c.close()


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--disposable-fixture', action='store_true')
    p.add_argument('--admin-argv', type=Path, required=True)
    p.add_argument('--member-argv', type=Path, required=True)
    p.add_argument('--output', type=Path, required=True)
    p.add_argument('--reboot', action='store_true', help='reboot only the marked disposable VM to check opt-in boot startup')
    args = p.parse_args()
    result = smoke(args)
    args.output.write_text(json.dumps(result, indent=2) + '\n')
    print(f"Small-board quickstart passed: {len(result['checks'])} native SSH/member/setup checks.")


if __name__ == '__main__':
    main()
