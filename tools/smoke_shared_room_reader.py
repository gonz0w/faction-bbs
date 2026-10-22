#!/usr/bin/env python3
"""Test shared room pages on a marked disposable NetBSD fixture via admin SSH.

Requires existing Faction service users/rc wrapper/native frontend. Installs built
binaries and a fresh synthetic board; never run on an operator's real system.
Uses a root-orchestrated UID-owned PTY, not a new dedicated-SSH admission test.
"""
import argparse
import json
import secrets
import shlex
import subprocess
import time
from pathlib import Path
from check_restricted_ssh import SSHConsole


def smoke(args):
    assert args.disposable_fixture
    admin = json.loads(args.admin_argv.read_text())
    password = secrets.token_hex(20)
    children = []
    checks = []
    state = '/var/db/faction-reader-fixture'

    def remote(command, data=None):
        r = subprocess.run(admin + ['set -e; PATH=/usr/sbin:/sbin:/usr/bin:/bin; export PATH; ' + command],
                           input=data, capture_output=True, timeout=30)
        assert r.returncode == 0, 'disposable shared fixture operation failed'
        return r.stdout.decode()

    def owner(command, handle='sysop'):
        c = SSHConsole(admin[:-1] + ['-tt', admin[-1],
            "su - factiond -c '/usr/local/libexec/faction --state-dir " + state + ' ' + command + "'"])
        children.append(c)
        c.expect('Handle: '); c.send(handle)
        if command in ['init', 'register']:
            c.password(password, 'New password (12+ characters, hidden): ')
            c.password(password, 'Confirm password (hidden): ')
            c.expect('Lobby created.' if command == 'init' else 'Registered pending sysop approval.')
            c.finish()
        else:
            c.password(password); c.expect('Command (then Enter): ')
        return c

    def status(action):
        c = owner('login'); c.send('s'); c.expect('[C]reate room: ')
        c.send(action); c.expect('Member handle: '); c.send('reader')
        c.expect('Status updated; previous sessions expired.')
        c.expect('Command (then Enter): '); c.send('q'); c.expect('Logged out.'); c.finish()

    member_calls = 0

    def member():
        nonlocal member_calls
        # Exercise the normal eight-launch burst/5-second refill; never tune it.
        if member_calls >= 7: time.sleep(5.1)
        member_calls += 1
        uid = remote('id -u factiond').strip()
        assert uid.isdecimal()
        command = 'set -e; PATH=/usr/sbin:/sbin:/usr/bin:/bin; export PATH; chown factionterm "$(tty)"; exec su -m factionterm -c ' + shlex.quote(
            '/usr/local/libexec/faction-tty --socket /var/run/faction/terminal.sock --owner-uid ' + uid)
        c = SSHConsole(admin[:-1] + ['-tt', admin[-1], command], columns=40, rows=18)
        children.append(c)
        c.expect('[L]ogin [R]egister [Q]uit: '); c.send('l')
        c.expect('Handle: '); c.send('reader'); c.password(password)
        c.expect('Press Enter to continue: '); c.send('')
        c.expect('Ctrl-C exits; Esc: Enter to recover.')
        return c

    def back(c):
        c.expect('Press Enter to return to menu: '); c.send('')
        c.expect('Ctrl-C exits; Esc: Enter to recover.')

    def unread(c, n):
        c.raw(b'r'); c.expect(f'Lobby ({n} unread)'); back(c)

    try:
        remote('test -f /etc/faction-disposable-test; test ! -e ' + state + '; '
               '/etc/rc.d/faction_broker onestop; '
               'install -d -o factiond -g factionipc -m 700 ' + state + '; '
               'install -o root -g wheel -m 755 ' + shlex.quote(str(args.bin_dir/'faction')) + ' /usr/local/libexec/faction; '
               'install -o root -g wheel -m 755 ' + shlex.quote(str(args.bin_dir/'faction-service')) + ' /usr/local/libexec/faction-service')
        uid = remote('id -u factionterm').strip(); assert uid.isdecimal()
        config = f'version=1\nstate_dir="{state}"\nsocket="/var/run/faction/terminal.sock"\nterminal_uid={uid}\nansi=true\n'
        remote('cat > /etc/faction/service.toml; chown root:factionipc /etc/faction/service.toml; chmod 640 /etc/faction/service.toml',config.encode())
        owner('init'); owner('register', 'reader'); status('a')
        body = ' '.join(f'word{i:03d}' for i in range(100)) + ' FINAL-MARKER'
        c = owner('login'); c.send('p'); c.expect('Room ID: '); c.send('1')
        c.expect('Reply to post ID (blank for new message): '); c.send('')
        c.expect('Subject (blank for generated/reply default): '); c.send('Midnight signal')
        c.expect('Message (one line, up to 1024 bytes): '); c.send(body)
        c.expect('Posted #1.'); c.expect('Command (then Enter): '); c.send('q'); c.expect('Logged out.'); c.finish()
        remote('/etc/rc.d/faction_broker onestart')
        c = member(); unread(c, 1)
        checks.append('approved_member_over_native_uid_checked_shared_frontend')
        c.raw(b'n'); c.expect('Room ID: '); c.send('1'); c.expect('Subject: Midnight signal'); c.expect('Posted: '); c.expect(' UTC'); c.expect('More? [y/N]: ')
        c.send('n'); back(c); unread(c, 1)
        checks.append('partial_message_does_not_advance_unread')
        c.raw(b'n'); c.expect('Room ID: '); c.send('1'); start = len(c.transcript)
        for _ in range(30):
            c.expect('More? [y/N]: ')
            if b'FINAL-MARKER' in c.transcript[start:]: break
            c.send('y')
        else: raise AssertionError('shared reader final page missing')
        text = c.transcript[start:]
        assert b'\x1b[2J\x1b[H' in text
        assert all(text.count(f'word{i:03d}'.encode()) == 1 for i in range(100))
        checks.append('wrapped_ansi_pages_preserve_every_word_once')
        c.send('n'); back(c); unread(c, 0)
        checks.append('complete_message_drain_advances_unread')
        for number, subject, expected in [(2, '', 'Re: Midnight signal'), (3, 'Side frequency', 'Side frequency')]:
            c.raw(b'p'); c.expect('Room ID: '); c.send('1')
            c.expect('Reply to post ID (blank for new message): '); c.send(str(number-1))
            c.expect('Subject (blank for generated/reply default): '); c.send(subject)
            c.expect('Message (one line, up to 1024 bytes): '); c.send('Shared reply body '+str(number))
            c.expect('Posted #'+str(number)+'.'); back(c)
            c.raw(b'n'); c.expect('Room ID: '); c.send('1')
            c.expect('Subject: '+expected); c.expect('Shared reply body '+str(number))
            c.expect('More? [y/N]: '); c.send('n'); back(c); unread(c, 0)
            checks.append('shared_reply_default_subject' if number==2 else 'shared_authored_subject_override')
        for number, subject, expected in [(4, 'Café 界', 'Caf\\u{e9} \\u{754c}'), (5, 'é'*80, '\\u{e9}'*4+'...'), (6, '', 'Re: Caf\\u{e9} \\u{754c}')]:
            c.raw(b'p'); c.expect('Room ID: '); c.send('1')
            c.expect('Reply to post ID (blank for new message): '); c.send('4' if number==6 else '')
            c.expect('Subject (blank for generated/reply default): ')
            if number == 4:
                c.raw(b'Caf\xc3'); c.raw(b'\xa9\x7f\xc3'); c.raw(b'\xa9\xe2\x7f'); c.send(' 界')
            else: c.send(subject)
            c.expect('Message (one line, up to 1024 bytes): '); c.send('Unicode shared fixture '+str(number))
            c.expect('Posted #'+str(number)+'.'); back(c)
            c.raw(b'n'); c.expect('Room ID: '); c.send('1')
            c.expect('Subject: '+expected); c.expect('Unicode shared fixture '+str(number))
            c.expect('More? [y/N]: '); c.send('n'); back(c)
            checks.append(['shared_split_utf8_and_backspace', 'shared_160_byte_subject_clipped_by_cells', 'shared_unicode_reply_default'][number-4])
        for bad in [b'\xff\n', b'\xe2\n', b'\x1b[2J\n', b'x'*161+b'\n']:
            other = member(); other.raw(b'p'); other.expect('Room ID: '); other.send('1')
            other.expect('Reply to post ID (blank for new message): '); other.send('')
            other.expect('Subject (blank for generated/reply default): '); other.raw(bad); other.finish(code=1)
        checks.append('shared_malformed_incomplete_control_and_overflow_close')
        c.raw(b't'); c.expect('[D]one: ')
        for option in ['u','2','3','b','a']:
            c.send(option); c.expect('[D]one: ')
        c.send('p'); c.expect('\x1b[?2026$p'); c.raw(b'\x1b[?20'); c.raw(b'26;2$y')
        c.expect('sync: true'); c.expect('[D]one: '); c.send('d')
        c.expect('Ctrl-C exits; Esc: Enter to recover.')
        c.raw(b'p'); c.expect('Room ID: '); c.send('1')
        c.expect('Reply to post ID (blank for new message): '); c.send('')
        c.expect('Subject (blank for generated/reply default): '); c.send('Café 界 🦀')
        c.expect('Message (one line, up to 1024 bytes): '); c.send('界🦀é '*70+'FINAL-MODERN')
        c.expect('Posted #7.'); back(c)
        c.raw(b'n'); c.expect('Room ID: '); c.send('1'); start=len(c.transcript)
        for _ in range(40):
            c.expect('Subject: Café 界 🦀'); c.expect('More? [y/N]: ')
            if b'FINAL-MODERN' in c.transcript[start:]:break
            c.send('y')
        else:raise AssertionError('shared modern page did not complete')
        output=c.transcript[start:]
        assert output.count('界🦀é'.encode())==70
        assert b'38;2;' in output and b'\x1b[?2026h' in output and b'\x1b[?2026l' in output
        c.send('n');back(c);checks.append('shared_modern_unicode_cells_rgb_and_synchronized_pages')
        c.raw(b't');c.expect('[D]one: ');c.send('c');c.expect('UTF-8: false');c.expect('[D]one: ');c.send('d');c.expect('Ctrl-C exits; Esc: Enter to recover.')
        checks.append('shared_classic_override_recovers_unknown_width_profile')
        other=member();other.raw(b't');other.expect('[D]one: ');other.send('p');other.expect('\x1b[?2026$p');other.expect('sync: false');other.expect('[D]one: ')
        other.raw(b'\x1b[?2026;2$yP\n');other.finish(code=1)
        for reply in [b'\x1b[?2026;2$yP\n',b'\x1b',b'\x1b]52;c;bad\x07']:
            other=member();other.raw(b't');other.expect('[D]one: ');other.send('p');other.expect('\x1b[?2026$p');other.raw(reply);other.finish(code=1)
        checks.append('shared_unknown_malformed_incomplete_late_probe_isolation')
        # Keep a Unicode draft open while the other connection is paging.
        draft = member(); draft.raw(b'p'); draft.expect('Room ID: '); draft.send('1')
        draft.expect('Reply to post ID (blank for new message): '); draft.send('')
        draft.expect('Subject (blank for generated/reply default): '); draft.send('Café')
        draft.expect('Message (one line, up to 1024 bytes): ')
        c.raw(b'h'); c.expect('Room ID: '); c.send('1'); c.expect('More? [y/N]: ')
        status('v'); c.send('y'); c.expect('session expired; log in again'); c.finish(code=1)
        checks.append('current_revocation_denies_next_page')
        draft.send('Revoked Unicode draft'); draft.expect('session expired; log in again'); draft.finish(code=1)
        checks.append('shared_revocation_denies_unicode_draft_submission')
        remote('/etc/rc.d/faction_broker onestop')
        if args.output:
            args.output.write_text(json.dumps({'checks': checks, 'count': len(checks), 'result': 'pass',
                'dedicated_ssh_admission_tested': False, 'capacity_claim': False}, indent=2)+'\n')
        print(f'Shared NetBSD reader passed {len(checks)} UID/PTY, paging, unread and revocation controls.')
    finally:
        for c in children: c.close()
        remote('/etc/rc.d/faction_broker onestop || true')


if __name__ == '__main__':
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--admin-argv', type=Path, required=True)
    p.add_argument('--bin-dir', type=Path, required=True)
    p.add_argument('--disposable-fixture', action='store_true')
    p.add_argument('--output', type=Path)
    smoke(p.parse_args())
