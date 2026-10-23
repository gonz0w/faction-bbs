#!/usr/bin/env python3
"""Play unchanged original LORD on TWO marked private disposable NetBSD VMs.
Requires a fresh installed BBS and separately prepared original-game fixture.
Private argv, endpoint, tokens, game assets and transcripts never enter output.
This is a software oracle, not an installer or supported public game launcher.
"""
import argparse
import errno
import json
import os
from pathlib import Path
import re
import secrets
import select
import shlex
import subprocess
import time
from check_restricted_ssh import SSHConsole


def game_match(c, texts):
    gap = rb'(?:\x1b\[[0-?]*[ -/]*[@-~])*'
    patterns = [re.compile(gap.join(re.escape(bytes([b])) for b in text.encode()), re.I)
                for text in texts]
    deadline = time.monotonic() + 30
    while True:
        matches = [(m.start(), i, m) for i, pattern in enumerate(patterns)
                   if (m := pattern.search(c.buffer))]
        if matches:
            _, i, match = min(matches, key=lambda item: item[0])
            c.buffer = c.buffer[match.end():]
            return i
        assert time.monotonic() < deadline, 'original-game prompt timeout'
        if select.select([c.fd], [], [], .2)[0]:
            try:
                data = os.read(c.fd, 8192)
            except OSError as exc:
                if exc.errno != errno.EIO:
                    raise
                data = b''
            assert data, 'original-game SSH ended'
            c.buffer += data
            c.transcript += data


def game_expect(c, text):
    game_match(c, [text])


def game_until(c, target):
    for _ in range(12):
        i = game_match(c, [target, '<more>', '(C)ontinue'])
        if i == 0:
            return
        c.raw(b'\r' if i == 1 else b'c')
    raise AssertionError('too many original-game screens')


def run(a):
    assert a.disposable_fixture
    admin = json.loads(a.admin_argv.read_text())
    member = json.loads(a.member_argv.read_text())
    game_admin = json.loads(a.game_admin_argv.read_text())
    tokens = json.loads(a.game_tokens.read_text())
    password = secrets.token_hex(24)
    if a.private_debug_dir:
        debug = a.private_debug_dir.resolve()
        assert not debug.is_relative_to(Path(__file__).resolve().parents[1])
        debug.mkdir(mode=0o700, parents=True, exist_ok=True)
        debug.chmod(0o700)
        secret = debug / 'generated-bbs-password'
        secret.write_text(password)
        secret.chmod(0o600)
    children, checks, samples = [], [], {}

    def remote(argv, command, data=None, ok=True):
        x = subprocess.run(argv + ['set -e; PATH=/usr/sbin:/sbin:/usr/bin:/bin:/usr/pkg/bin; export PATH; ' + command],
                           input=data, capture_output=True, timeout=35)
        assert not ok or x.returncode == 0, 'private guest fixture action failed'
        return x

    def bbs(command, **kw): return remote(admin, command, **kw)
    def host(command, **kw): return remote(game_admin, command, **kw)
    def counts(): return json.loads(host('cat /home/factionclassic/runtime/counts.json').stdout)
    def menu(c): c.expect('Ctrl-C exits; Esc: Enter to recover.')

    def drained():
        deadline = time.monotonic() + 6
        while time.monotonic() < deadline:
            value = counts()
            if value['runs'] == value['reaped']:
                assert not host("pgrep -u factionclassic -f '^/opt/faction-dosbox/dosbox( |$)'", ok=False).stdout.strip()
                assert not host('find /home/factionclassic/c -name DOOR.SYS').stdout.strip()
                return value
            time.sleep(.1)
        raise AssertionError('original game not reaped')

    def spawn(argv):
        c = SSHConsole(argv)
        children.append(c)
        return c

    def enter(choice, handle):
        c = spawn(member[:-1] + ['-tt', member[-1]])
        c.expect('[L]ogin [R]egister [Q]uit: ')
        c.send(choice)
        c.expect('Handle: ')
        c.send(handle)
        c.password(password)
        if choice == 'r':
            c.password(password, 'Confirm password (hidden): ')
        return c

    def login(handle):
        c = enter('l', handle)
        c.expect('Press Enter to continue: ')
        c.send('')
        menu(c)
        return c

    def status(handle, action):
        sysop.raw(b's')
        sysop.expect('[C]reate room: ')
        sysop.send(action)
        sysop.expect('Member handle: ')
        sysop.send(handle)
        sysop.expect('Status updated; previous sessions expired.')
        sysop.expect('Press Enter to return to menu: ')
        sysop.send('')
        menu(sysop)

    def connect(c, user='gameone', token=None):
        c.raw(b'd')
        c.expect('Classic game servers')
        c.expect('Ctrl-C returns to the BBS.')
        c.raw(b'a')
        c.expect('Never reuse your BBS password.')
        c.expect('Ctrl-C returns to the BBS.')
        c.raw(b'c')
        c.expect('Game login: ')
        c.send(user)
        c.password(tokens[user] if token is None else token, 'Game access token (hidden): ')

    def entry(c, user='gameone'):
        connect(c, user)
        # Original title art rotates and may stream slowly. Wait for the actual
        # pause prompt rather than mistaking a temporary serial pause for EOF.
        for _ in range(4):
            i = game_match(c, ['Your choice, warrior?', '<more>', 'Press A Key', 'BY:  Seth Able'])
            if i == 0:
                return
            # Captions can occur before the final ANSI tail. LORD flushes keys
            # after drawing; allow the remaining serial output to finish first.
            deadline, last = time.monotonic() + 12, time.monotonic()
            while time.monotonic() < deadline:
                if select.select([c.fd], [], [], .1)[0]:
                    data = os.read(c.fd, 8192)
                    assert data
                    c.buffer += data
                    c.transcript += data
                    last = time.monotonic()
                if time.monotonic() - last >= 2:
                    break
            c.raw(b'\r')
        raise AssertionError('too many original title pauses')

    def realm(c):
        entry(c)
        c.raw(b'e')
        game_until(c, 'Your command')

    def bank(c, expected):
        c.raw(b'v')
        game_expect(c, "Rogueone's Stats")
        game_expect(c, 'Gold In Bank')
        game_expect(c, ': ' + str(expected))
        game_until(c, 'Your command')

    def quit_game(c):
        c.raw(b'q')
        game_expect(c, 'Quit game?')
        c.raw(b'y')
        menu(c)
        drained()

    def quit_bbs(c):
        c.raw(b'q')
        c.expect('Logged out.')
        c.finish()

    def passed(name):
        checks.append(name)
        print('Passed:', name, flush=True)

    try:
        bbs('test -f /etc/faction-disposable-test; test ! -e /var/db/faction/faction.sqlite3; test ! -S /var/run/faction/terminal.sock')
        host('test -f /etc/faction-disposable-test; test -f /home/factionclassic/c/lord/LORD.DAT; test ! -e /home/factionclassic/c/lord/PLAYER.DAT; test ! -e /home/factionclassic/c/lord/player.dat')
        config = bbs('cat /etc/faction/service.toml').stdout.decode()
        assert '[[classic_games]]' not in config
        config += '\n[[classic_games]]\nslug="lord"\ntitle="Original LORD 4.06"\naddress=' + json.dumps(a.game_endpoint) + '\n'
        bbs('cat > /etc/faction/service.toml', data=config.encode())
        c = spawn(admin[:-1] + ['-tt', admin[-1], "su - factiond -c '/usr/local/libexec/faction --state-dir /var/db/faction init'"])
        c.expect('Handle: ')
        c.send('sysop')
        c.password(password, 'New password (12+ characters, hidden): ')
        c.password(password, 'Confirm password (hidden): ')
        c.expect('Lobby created.')
        c.finish()
        bbs('/etc/rc.d/faction_broker onestart; /etc/rc.d/faction_sshd onestart')
        initial = counts()
        before = initial['runs']
        for name in ('alice', 'bob'):
            c = enter('r', name)
            c.expect('Registration pending; sysop approval required.')
            c.finish()
        c = enter('l', 'alice')
        c.expect('Sysop approval required. No community access.')
        c.finish()
        assert counts()['runs'] == before
        sysop = login('sysop')
        status('alice', 'a')
        status('bob', 'a')
        alice, bob = login('alice'), login('bob')
        passed('pending_local_members_have_no_game_access')
        connect(alice, token='wrong-independent-fixture-token')
        alice.expect('Game login denied.')
        menu(alice)
        assert counts()['runs'] == before
        passed('independent_game_authentication_denies_wrong_token')
        entry(alice)
        assert any(glyph.encode() in alice.transcript for glyph in '░▒▓█▄▀')
        alice.raw(b'e')
        game_expect(alice, 'Your choice?')
        alice.raw(b'y')
        game_expect(alice, 'Name: ')
        alice.send('RogueOne')
        game_expect(alice, 'Rogueone?')
        alice.raw(b'y')
        game_expect(alice, 'And your gender?')
        alice.raw(b'm')
        game_expect(alice, 'Pick one.')
        alice.raw(b'k')
        game_until(alice, 'Your command')
        alice.raw(b'y')  # Original LORD's Ye Olde Bank hotkey.
        game_expect(alice, 'Gold In Hand')
        game_expect(alice, 'Your command')
        alice.raw(b'd')
        game_expect(alice, 'AMOUNT:')
        alice.send('100')
        game_expect(alice, 'Your command')
        alice.raw(b'r')
        game_expect(alice, 'Your command')
        bank(alice, 100)
        passed('original_ansi_cp437_character_creation_and_real_bank_gameplay')
        rows = host('ps -ax -o user,pid,rss,vsz,pcpu,comm').stdout.decode().splitlines()
        samples['active_processes'] = [{'rss_kib': int(p[2]), 'vsz_kib': int(p[3]), 'program': 'dosbox' if 'dosbox' in line else 'fixture-python'}
                                     for line in rows if (p := line.split()) and p[0] == 'factionclassic']
        connect(bob, 'gametwo')
        bob.expect('One game node is enabled.')
        menu(bob)
        drop = host('cat /home/factionclassic/c/NODE1/DOOR.SYS').stdout.decode().splitlines()
        assert drop[9] == 'Traveler One' and drop[13] == 'UNUSED' and len(drop) == 52
        assert password not in '\n'.join(drop) and tokens['gameone'] not in '\n'.join(drop)
        assert host('test -e /home/factionclassic/c/NODE2/DOOR.SYS', ok=False).returncode != 0
        passed('one_original_node_locked_without_overwriting_active_identity')
        quit_game(alice)
        entry(bob, 'gametwo')
        bob.raw(b'e')
        game_expect(bob, 'You have never')
        drop = host('cat /home/factionclassic/c/NODE2/DOOR.SYS').stdout.decode().splitlines()
        assert drop[9] == 'Traveler Two' and drop[3] == '2'
        bob.raw(b'\x1d')
        menu(bob)
        drained()
        passed('second_game_identity_and_dropfile_are_separate')
        quit_bbs(alice)
        alice = login('alice')
        realm(alice)
        bank(alice, 100)
        passed('original_character_and_balance_survive_fresh_ssh_bbs_game_login')
        alice.raw(b'\x1d')
        menu(alice)
        drained()
        realm(alice)
        bank(alice, 100)
        passed('local_escape_reaps_game_and_saved_character_reopens')
        alice.resize(100, 30)
        alice.expect('Terminal resized; game closed.')
        menu(alice)
        drained()
        realm(alice)
        bank(alice, 100)
        passed('resize_closes_game_and_next_original_login_works')
        status('alice', 'v')
        alice.expect('Action failed:')
        alice.finish(1)
        drained()
        status('alice', 'a')
        alice = login('alice')
        realm(alice)
        bank(alice, 100)
        passed('local_revocation_closes_original_game_and_reapproval_recovers')
        pid = host("pgrep -u factionclassic -f '^/opt/faction-dosbox/dosbox( |$)'").stdout.decode().strip()
        assert re.fullmatch(r'[0-9]+', pid)
        host('kill -KILL ' + pid)
        menu(alice)
        drained()
        realm(alice)
        bank(alice, 100)
        quit_game(alice)
        passed('original_emulator_failure_reaped_and_saved_character_recovers')
        for c in (sysop, alice):
            c.raw(b'c')
            c.expect('chat[#lobby]> ')
        sysop.send('Welcome back from the original game.')
        alice.expect('Welcome back from the original game.')
        for c in (sysop, alice):
            c.send('/quit')
            menu(c)
        for c in (alice, bob, sysop): quit_bbs(c)
        passed('local_chat_and_normal_logout_remain_usable')
        summary = bbs("su - factiond -c '/usr/local/libexec/faction --state-dir /var/db/faction export /home/factiond/original-game-export.sqlite3'").stdout
        assert b'0 game score entries' in summary
        bbs("su - factiond -c '/usr/local/libexec/faction verify-export /home/factiond/original-game-export.sqlite3'")
        passed('original_game_state_does_not_enter_local_arcade_export')
        final = drained()
        assert final['denied'] - initial['denied'] == 1 and final['busy'] - initial['busy'] == 1
        return {'schema': 1, 'result': 'pass', 'checks': checks, 'game_host': {k: final[k] - initial[k] for k in final}, 'samples': samples}
    finally:
        if a.private_debug_dir:
            for i, c in enumerate(children):
                path = a.private_debug_dir / ('transcript-' + str(i) + '.bin')
                path.write_bytes(c.transcript)
                path.chmod(0o600)
        for c in children: c.close()


def main():
    p = argparse.ArgumentParser(description=__doc__)
    for name in ('admin-argv', 'member-argv', 'game-admin-argv', 'game-tokens', 'output'):
        p.add_argument('--' + name, type=Path, required=True)
    p.add_argument('--game-endpoint', required=True)
    p.add_argument('--private-debug-dir', type=Path)
    p.add_argument('--disposable-fixture', action='store_true')
    a = p.parse_args()
    result = run(a)
    a.output.write_text(json.dumps(result, indent=2) + '\n')
    print('Original LORD SSH software-fixture checks passed:', len(result['checks']))


if __name__ == '__main__': main()
