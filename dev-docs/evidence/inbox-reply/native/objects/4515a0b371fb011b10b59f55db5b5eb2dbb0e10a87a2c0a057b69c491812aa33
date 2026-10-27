#!/usr/bin/env python3
"""Selectable local inbox PTY controls. Only synthetic marked owned fixtures."""
import argparse
import json
import os
import re
import signal
import traceback
import unicodedata
import stat
import subprocess
import tempfile
from pathlib import Path
from smoke_native_client_ssh import NativeConsole

PASS = 'checkpoint synthetic fixture password'
UNLOCK = 'LOCAL checkpoint passphrase (hidden): '
MENU = '[L] Lock and exit'
ACK = 'Type read to acknowledge locally, or Enter to keep current state: '
INBOX = '[LOCAL INBOX]'
READY = '[B] Back'


def probe(client, fixture, owned_root, first_control_only=False, online=False):
    info = owned_root.lstat()
    assert stat.S_ISDIR(info.st_mode) and info.st_uid == os.getuid()
    assert info.st_mode & 0o777 == 0o700
    assert (owned_root/'OWNERSHIP').is_file()
    children, checks = [], []
    with tempfile.TemporaryDirectory(prefix='inbox-OWNED-', dir=owned_root) as temp:
        root = Path(temp).resolve()
        def seed(name, mode='unread'):
            path = root/name
            result = subprocess.run([str(fixture),str(path),mode],capture_output=True)
            assert result.returncode == 0, 'synthetic fixture failed'
            return path
        config = root/'home.toml'
        ca = root/'synthetic-ca.pem'
        ca.write_bytes(b'synthetic local-only CA; never used for a connection\n'); ca.chmod(0o600)
        config.write_text('version = 1\naccount = "alice@home.test"\nendpoint = "127.0.0.1:9"\ndomain = "home.test"\nca = '+json.dumps(str(ca))+'\npin = "'+'00'*32+'"\n')
        config.chmod(0o600)
        def inspect(path):
            result = subprocess.run([str(fixture),str(path),'inspect-unread'],capture_output=True)
            assert result.returncode == 0, 'synthetic inspection failed'
            return json.loads(result.stdout)
        def start(path, unicode=False, configured=False):
            args = ([str(client),'--mail-online',str(path),str(config)] if configured else
                    [str(client),'--mail-checkpoints',str(path),'open'])
            if unicode: args.append('--unicode')
            c = NativeConsole(args); children.append(c); c.resize(40,24)
            c.password(PASS, UNLOCK); c.expect(MENU)
            assert PASS.encode() not in c.transcript
            return c
        def screen(c):
            begin = len(c.transcript)
            c.expect(INBOX); c.expect(READY)
            output = c.transcript[begin:]+c.buffer
            assert b'\x1b' not in output and b'\x07' not in output
            assert b'\xc2\x9b' not in output and '\u202e'.encode() not in output
            lines = output.decode().splitlines()
            for line in lines:
                cells = sum(2 if unicodedata.east_asian_width(ch) in ('W','F') else 1 for ch in line)
                assert cells <= 40, 'compact inbox exceeds classic width'
            return [line for line in lines if re.match(r'^[> ] \d{2} ',line)]
        def inbox(c):
            c.raw(b'i'); return screen(c)
        def nav(c, key):
            c.raw(key); return screen(c)
        def indices(rows): return [int(row[2:4]) for row in rows]
        def selected(rows): return [int(row[2:4]) for row in rows if row.startswith('> ')]
        def back(c):
            c.raw(b'b'); c.expect(MENU)
        def open_record(c, number, received=True, unknown=False, key=b'o'):
            begin = len(c.transcript); c.raw(key)
            c.expect('[LOCAL INBOX RECORD]'); c.expect('local retained record '+str(number))
            c.expect('Peer: bob@away.test'); c.expect('Device: 202 / UTC ')
            c.expect('Subject: '); c.expect('Body: ')
            c.expect(ACK if received and not unknown else 'Enter to return: ')
            text = c.transcript[begin:]
            assert b'\x1b' not in text and b'\xc2\x9b' not in text and '\u202e'.encode() not in text
            if not received or unknown: assert ACK.encode() not in text
        def return_record(c, answer=None):
            if answer is not None:
                c.send(answer); c.expect('Enter to return: ')
            c.send(''); return screen(c)
        def stable(path, before):
            assert (path/'mail.fkv').read_bytes() == before
            assert {p.name for p in path.iterdir()} == {'mail.fkv','mail.lock'}
        def lock(c):
            c.raw(b'l'); c.finish()
        try:
            path = seed('entry')
            before = (path/'mail.fkv').read_bytes()
            c = start(path)
            # The unchanged first control fails on the frozen old client because
            # its trusted local menu has no I action. No source-only surrogate.
            assert b'[I] Local inbox' in c.transcript, 'inbox entry absent'
            inbox(c)
            c.raw(b'b'); c.expect(MENU); lock(c)
            assert (path/'mail.fkv').read_bytes() == before
            checks.append('explicit_offline_I_inbox_entry_and_back_preserve_checkpoint')
            if first_control_only:
                return dict(result='pass',count=len(checks),checks=checks,
                            private_fixture_removed=True,provider_contact=False,read_receipt_sent=False)

            # A single retained received record and empty checkpoint exercise the
            # actual model boundaries, including opening an empty selection.
            for count in [0,1]:
                path = seed('bounds-'+str(count),'inbox-'+str(count))
                status = inspect(path)
                assert status['retained'] == count, 'fixture retention count'
                before = (path/'mail.fkv').read_bytes()
                c = start(path); rows = inbox(c)
                assert indices(rows) == ([1] if count else [])
                for key in [b'k',b'j',b'n',b'p',b'\x1b[A',b'\x1b[B']:
                    rows = nav(c,key); assert indices(rows) == ([1] if count else [])
                if count:
                    open_record(c,1,key=b'\r'); return_record(c,'')
                else:
                    assert nav(c,b'o') == []
                back(c); lock(c); stable(path,before); assert inspect(path)==status
            checks.append('empty_and_single_real_checkpoint_bounds_never_wrap_or_acknowledge')

            path = seed('last-unread','inbox-1')
            c = start(path); inbox(c); nav(c,b'u'); open_record(c,1)
            rows = return_record(c,'read'); assert rows==[]
            assert b'No unread received records.' in c.transcript
            acknowledged = (path/'mail.fkv').read_bytes()
            assert nav(c,b'o')==[]
            rows = nav(c,b'a'); assert 'READ ' in rows[0]
            open_record(c,1); return_record(c,'read'); back(c); lock(c)
            stable(path,acknowledged); assert inspect(path)['unread']==0
            checks.append('last_unread_ack_returns_empty_filter_and_repeated_ack_is_idempotent')

            mixed = seed('mixed','inbox-7'); before = (mixed/'mail.fkv').read_bytes(); initial = inspect(mixed)
            assert initial['retained']==7 and initial['unread']==3 and initial['pending']==2
            c = start(mixed); rows = inbox(c)
            assert indices(rows)==[7,6,5,4,3,2] and selected(rows)==[7]
            assert any('READ ' in row and row[2:4]=='03' for row in rows)
            assert any('PENDING' in row and row[2:4]=='06' for row in rows)
            assert any('RETIRED' in row and row[2:4]=='04' for row in rows)
            assert selected(nav(c,b'k'))==[7]
            assert selected(nav(c,b'j'))==[6]
            assert selected(nav(c,b'\x1b[B'))==[5]
            assert selected(nav(c,b'\x1b[A'))==[6]
            rows = nav(c,b'n'); assert indices(rows)==[1] and selected(rows)==[1]
            assert selected(nav(c,b'n'))==[1]
            rows = nav(c,b'p'); assert indices(rows)==[7,6,5,4,3,2] and selected(rows)==[7]
            nav(c,b'j'); open_record(c,6,received=False); return_record(c)
            nav(c,b'j'); nav(c,b'j'); open_record(c,4,received=False); return_record(c)
            rows = nav(c,b'u'); assert indices(rows)==[7,5,1] and selected(rows)==[7]
            open_record(c,7); assert b'inbox body 07' in c.transcript
            return_record(c,''); back(c); lock(c); stable(mixed,before); assert inspect(mixed)==initial
            checks.append('seven_record_status_order_arrows_JK_pages_selected_only_and_unread_filter_readonly')

            c = start(mixed); inbox(c); nav(c,b'u'); open_record(c,7)
            rows = return_record(c,'read'); assert indices(rows)==[5,1] and selected(rows)==[5]
            rows = nav(c,b'a'); assert selected(rows)==[7]
            open_record(c,7); assert b'READ / local retained record 7' in c.transcript
            return_record(c,''); back(c); lock(c)
            after = (mixed/'mail.fkv').read_bytes(); assert after!=before
            changed = inspect(mixed); assert changed == dict(initial,unread=2)
            assert {p.name for p in mixed.iterdir()} == {'mail.fkv','mail.lock'}
            c = start(mixed); rows = inbox(c); assert 'READ ' in rows[0]
            back(c); lock(c); stable(mixed,after)
            checks.append('fresh_ack_marks_exact_original_received_record_and_refreshes_filter_selection_durably')

            path = seed('full','inbox-64'); initial = inspect(path); assert initial['retained']==64
            before = (path/'mail.fkv').read_bytes()
            c = start(path); rows = inbox(c); assert indices(rows)==[64,63,62,61,60,59]
            for _ in range(10): rows = nav(c,b'n')
            assert indices(rows)==[4,3,2,1]
            assert indices(nav(c,b'n'))==[4,3,2,1]
            for _ in range(8): rows = nav(c,b'j')
            assert selected(rows)==[1]
            for _ in range(10): rows = nav(c,b'p')
            assert selected(rows)==[64]
            back(c); lock(c); stable(path,before); assert inspect(path)==initial
            checks.append('all_64_retained_records_are_eleven_bounded_pages_with_partial_final_page')

            path = seed('unknown','pending'); before = (path/'mail.fkv').read_bytes()
            c = start(path); rows = inbox(c); assert 'UNKNOWN' in rows[0]
            rows = nav(c,b'u'); assert indices(rows)==[2,1] and 'UNKNOWN' in rows[0]
            assert b'Unread unknown; tracking disabled.' in c.transcript
            open_record(c,2,unknown=True); return_record(c)
            back(c); lock(c); stable(path,before); assert inspect(path)['unread'] is None
            checks.append('disabled_tracking_keeps_unknown_and_rejects_zero_unread_filter_without_upgrade')

            for configured in ([False,True] if online else [False]):
                for unicode in [False,True]:
                    path = seed('display-'+str(configured)+'-'+str(unicode),'inbox-7')
                    before = (path/'mail.fkv').read_bytes()
                    c = start(path,unicode=unicode,configured=configured); rows = inbox(c)
                    assert len(rows)==6 and indices(rows)==[7,6,5,4,3,2]
                    expected = 'λ界' if unicode else r'\u{3bb}\u{754c}'
                    assert expected in rows[0]
                    open_record(c,7); assert b'\\u{9}' in c.transcript and b'\\u{202e}' in c.transcript
                    return_record(c,''); back(c); lock(c); stable(path,before)
            checks.append('classic_and_explicit_unicode_subjects_are_inert_and_fit_40_columns')
            if online: checks.append('configured_online_I_uses_same_local_inbox_without_a_home_cycle')

            for key in [b'\x03',b'\x04']:
                path = seed('cancel-'+key.hex(),'unread'); before = (path/'mail.fkv').read_bytes()
                c = start(path); inbox(c); c.raw(key)
                c.expect('Checkpoint locked: '); c.finish(1); stable(path,before)
            checks.append('CtrlC_and_CtrlD_lock_from_inbox_and_restore_terminal_without_mutation')

            path = seed('typeahead','unread'); before = (path/'mail.fkv').read_bytes()
            c = start(path); inbox(c); c.raw(b'oread\n'); c.expect(ACK)
            c.raw(b'\x03'); c.expect('Checkpoint locked: '); c.finish(1); stable(path,before)
            c = start(path); inbox(c); open_record(c,2); c.raw(b'\x04')
            c.expect('Checkpoint locked: '); c.finish(1); stable(path,before)
            checks.append('queued_open_read_typeahead_and_cancel_after_body_never_acknowledge')

            path = seed('process-death','unread'); before = (path/'mail.fkv').read_bytes()
            c = start(path); inbox(c); open_record(c,2)
            os.kill(c.pid,signal.SIGKILL); os.waitpid(c.pid,0); c.done=True
            stable(path,before); assert inspect(path)['unread']==1
            c = start(path); inbox(c); back(c); lock(c); stable(path,before)
            checks.append('actual_pty_process_death_before_ack_preserves_unread_and_canonical_reopen')

            for mode in ['unread-expired','unread-revoked','unread-history-only']:
                path = seed(mode,mode); before = (path/'mail.fkv').read_bytes()
                c = start(path); inbox(c); open_record(c,2); return_record(c,''); back(c); lock(c); stable(path,before)
                c = start(path); inbox(c); open_record(c,2); return_record(c,'read'); back(c); lock(c)
                assert inspect(path)['unread']==0
            checks.append('expired_revoked_history_only_inbox_inspection_and_explicit_local_ack_stay_available')

            path = seed('write-failure','unread'); before = (path/'mail.fkv').read_bytes()
            c = start(path); inbox(c); open_record(c,2); path.chmod(0o500)
            try:
                c.send('read'); c.expect('Checkpoint locked: '); c.finish(1)
            finally: path.chmod(0o700)
            stable(path,before); assert inspect(path)['unread']==1
            checks.append('read_ack_write_failure_locks_poisoned_handle_and_preserves_canonical_checkpoint')
            assert all(PASS.encode() not in c.transcript for c in children)
        except Exception:
            (owned_root/'inbox-private-failure.log').write_text(traceback.format_exc())
            raise
        finally:
            for c in children: c.close()
    return dict(result='pass',count=len(checks),checks=checks,
                private_fixture_removed=True,provider_contact=False,read_receipt_sent=False)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--client',type=Path,required=True)
    parser.add_argument('--fixture',type=Path,required=True)
    parser.add_argument('--owned-root',type=Path,required=True)
    parser.add_argument('--first-control-only',action='store_true')
    parser.add_argument('--online',action='store_true')
    parser.add_argument('--output',type=Path)
    args = parser.parse_args()
    try:
        result = probe(args.client.resolve(),args.fixture.resolve(),args.owned_root.resolve(),args.first_control_only,args.online)
    except Exception:
        result = dict(result='fail',control='local_inbox_pty',private_output_suppressed=True)
    text = json.dumps(result,indent=2)+'\n'
    if args.output: args.output.write_text(text)
    print(text,end='')
    raise SystemExit(0 if result['result']=='pass' else 1)
