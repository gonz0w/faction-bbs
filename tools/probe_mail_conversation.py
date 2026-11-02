#!/usr/bin/env python3
"""Bounded retained-conversation PTYs in explicit owned private fixtures.

Local mode opens no service. The optional settings route requires a separately
approved disposable NetBSD board and retains private trace/canonical captures.
The caller freezes source/binaries and owns final retained-fixture teardown.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import re
import secrets
import shlex
import socket
import subprocess
import tempfile
import termios
import time
import unicodedata

from probe_mail_correspondents import PICKER, GROUP, act, expect_screen, require_visible_indices
from probe_mail_reply import (PASS, UNLOCK, MENU, INBOX, ACK, REPLY, SUBJECT, BODY,
    QUEUE, private_root, require_reply_commit)
from probe_mail_search import search, require_capture_binding, save
from probe_menu_mail import (require_board_bound_private_absent, reconstruct_kdump_streams,
    TRACE_START, TRACE_STOP, TITLE, OPEN, BACK, CONFIRM_MAIL, MENU as BOARD_MENU)
from probe_visit_mail import (require_closed_endpoint, require_connection_refused,
    private_json, assert_same_visit, visit_processes, SCAN, validate_scan, HOME)
from smoke_native_client_ssh import NativeConsole
from check_restricted_ssh import SSHConsole
from smoke_board_launcher import LauncherConsole, PICK, CONFIRM, RETURN

CONVERSATION = '[LOCAL CONVERSATION]'
CHECKS = ['offline_online_exact_group_canonical_preview_order_and_movement',
    'original_record_open_and_parent_selection_after_movement',
    'unread_literal_query_intersection_and_online_reply_cancel_refresh',
    'preview_navigation_preserves_encrypted_and_logical_state',
    'bounded_inert_classic_unicode_rows_and_owned_local_teardown']
MAIN_CHECKS = ['actual_home_refusal_then_local_conversation_without_credential_cycle',
    'original_record_current_reply_cancel_and_filtered_preview_refresh',
    'locked_return_and_fresh_unlock_same_ssh_without_redial',
    'private_preview_markers_absent_strict_board_GIO_and_file_scan']


def sha(data):
    return hashlib.sha256(data).hexdigest()


def require_conversation_screen(screen, indices, selected, page, mode, active, device=202):
    start = screen.rfind(CONVERSATION.encode())
    assert start >= 0, 'current conversation title missing'
    text = screen[start:].decode()
    # Check raw text before splitlines can hide NEL or other line controls.
    assert not any(unicodedata.category(c) in {'Cc', 'Cf'} for c in text.replace('\r\n', '')), 'unsafe preview scalar'
    lines = text.splitlines()
    assert len(lines) <= 23, 'preview exceeds classic page'
    for line in lines:
        assert not any(unicodedata.category(c) in {'Cc', 'Cf'} for c in line), 'unsafe preview scalar'
        cells = sum(0 if unicodedata.combining(c) else 2 if unicodedata.east_asian_width(c) in 'WF' else 1 for c in line)
        assert cells <= 39, 'preview row exceeds classic width'
    assert f'D{device} / {mode}' + (' / Search' if active else '') in lines, 'group/filter/query state changed'
    assert f'Page {page[0]}/{page[1]}; ' in text, 'preview page changed'
    rows = [(i, re.fullmatch(r'([> ]) (\d{2}) (RECEIVED|SENT) (UNREAD|READ|UNKNOWN|PENDING|RETIRED)( @)?', line)) for i, line in enumerate(lines)]
    rows = [(i, match) for i, match in rows if match]
    assert [int(match[2]) for _, match in rows] == indices, 'preview crossed canonical group/order'
    assert indices == sorted(indices) and len(indices) <= 2, 'preview is not two chronological records'
    assert [int(match[2]) for _, match in rows if match[1] == '>'] == ([selected] if selected else []), 'selected original record changed'
    assert ('No eligible retained records.' in lines) == (not indices), 'empty scope is not truthful'
    if not indices:
        assert 'Page 0/0; selection 0/0' in lines
    for i, _ in rows:
        assert i + 6 < len(lines), 'incomplete message preview'
        assert re.fullmatch(r'UTC \d+', lines[i+1]), 'UTC field missing'
        assert lines[i+2].startswith('Subject: '), 'subject field missing'
        assert all(lines[i+j].startswith('| ') for j in [3, 4, 5]), 'body rows missing'
        assert lines[i+6] in {'[O] Full record', 'Body continues; O opens full record.'}, 'full-record continuation missing'
    assert sum(line.startswith('| ') for line in lines) == 3 * len(rows), 'body exceeds three rows'
    assert '[B] Back' in lines, 'parent return missing'


def require_readonly(raw_before, raw_after, before, after):
    assert raw_before == raw_after, 'preview changed encrypted canonical bytes'
    assert before == after, 'preview changed logical state'


PUBLIC_INPUTS = {PASS, 'recv 03', CONVERSATION, 'alice@home.test', 'CURRENT recipient:'}


def input_inventory(markers):
    return dict(private_values=[value for value in markers if value not in PUBLIC_INPUTS],
        public_input_provenance=dict(source_sha256=sha(Path(__file__).read_bytes()),
            values=[value for value in markers if value in PUBLIC_INPUTS]))


def inputs(root, client, fixture, extra=()):
    subject = 'conversation-' + secrets.token_hex(8)
    body = 'body-' + secrets.token_hex(16) + ' λ界 ' + 'x' * 160
    peer = 'peer-' + secrets.token_hex(6) + '@away.test'
    other = 'other-' + secrets.token_hex(6) + '@away.test'
    replacement = 'recv 03 reply-' + secrets.token_hex(8)
    markers = [subject, body, peer, other, replacement, PASS, 'recv 03', CONVERSATION,
        'alice@home.test', 'CURRENT recipient:'] + list(extra)
    return subject, body, peer, other, replacement, markers


def bind(root, client, fixture, markers, publication_extra=()):
    generated_at_ns = time.time_ns()
    marker_data = (json.dumps(list(dict.fromkeys(markers)), ensure_ascii=False) + '\n').encode()
    save(root, 'private-markers.json', marker_data)
    publication_data = (json.dumps(input_inventory(markers + list(publication_extra)), ensure_ascii=False) + '\n').encode()
    save(root, 'publication-inputs.private.json', publication_data)
    record = dict(capture_id=secrets.token_hex(16), source_sha256=sha(Path(__file__).read_bytes()),
        marker_set_sha256=sha(marker_data), publication_input_sha256=sha(publication_data), binary_sha256={'client':sha(client.read_bytes()), 'fixture':sha(fixture.read_bytes())},
        generated_at_ns=generated_at_ns)
    record['started_at_ns'] = time.time_ns()
    # Retain the actual candidate values even when the strict oracle refuses.
    save(root, 'capture-input.private.json', json.dumps(record))
    require_capture_binding(record, Path(__file__).read_bytes(), marker_data,
        {'client':client.read_bytes(), 'fixture':fixture.read_bytes()})
    assert record['publication_input_sha256'] == sha((root / 'publication-inputs.private.json').read_bytes())
    return record


def check(c, key, indices, selected, page, mode='All', active=False):
    screen = act(c, key, CONVERSATION)
    require_conversation_screen(screen, indices, selected, page, mode, active)
    return screen


def record_back(c, online):
    c.raw(b'o'); c.expect('[LOCAL INBOX RECORD]'); c.expect(ACK); c.send('')
    if online: c.expect(REPLY); c.raw(b'b')
    else: c.expect('Enter to return: '); c.send('')
    return expect_screen(c, CONVERSATION)


def compose(c, peer, identity, subject, body, queue):
    c.raw(b'o'); c.expect('[LOCAL INBOX RECORD]'); c.expect(ACK); c.send(''); c.expect(REPLY)
    c.raw(b'r'); c.expect('CURRENT recipient: ' + peer); c.expect('CURRENT device: 202 / ed25519:' + identity)
    c.expect(SUBJECT); c.send(subject); c.expect(BODY); c.send(body); c.expect(QUEUE); c.send('queue' if queue else '')
    return expect_screen(c, CONVERSATION)


def probe(client, fixture, owned_root, unicode=False):
    private_root(owned_root)
    root = Path(tempfile.mkdtemp(prefix='conversation-local-OWNED-', dir=owned_root)); root.chmod(0o700)
    subject, body, peer, other, replacement, markers = inputs(root, client, fixture)
    association = bind(root, client, fixture, markers)
    children = []
    try:
        with tempfile.TemporaryDirectory(prefix='fixture-OWNED-', dir=root) as name:
            fixture_root = Path(name)
            ca = save(fixture_root, 'unused-ca.pem', 'synthetic unused CA\n')
            with socket.socket() as unused:
                unused.bind(('127.0.0.1', 0)); endpoint = unused.getsockname()
            require_closed_endpoint(endpoint)
            config = save(fixture_root, 'home.toml', 'version=1\naccount="alice@home.test"\nendpoint=' + json.dumps('127.0.0.1:' + str(endpoint[1])) + '\ndomain="home.test"\nca=' + json.dumps(str(ca)) + '\npin="' + '00'*32 + '"\n')
            for online in [False, True]:
                store = fixture_root / ('vault-' + str(online))
                result = subprocess.run([str(fixture), 'seed', str(store), 'ready'], input=('\n'.join([subject,body,peer,other])+'\n').encode(),capture_output=True,timeout=90)
                assert result.returncode == 0, 'synthetic conversation fixture failed'
                def inspect():
                    result = subprocess.run([str(fixture),'inspect',str(store)],capture_output=True,timeout=30)
                    assert result.returncode == 0
                    return json.loads(result.stdout)
                before = inspect(); raw = (store/'mail.fkv').read_bytes()
                argv = [str(client),'--mail-online',str(store),str(config)] if online else [str(client),'--mail-checkpoints',str(store),'open']
                c = NativeConsole(argv + (['--unicode'] if unicode else [])); children.append(c); c.resize(40,24)
                c.password(PASS,UNLOCK); c.expect(MENU)
                act(c,b'i',INBOX); act(c,b'c',PICKER); act(c,b'o',GROUP)
                check(c,b't',[7,8],8,(2,2)); check(c,b'k',[7,8],7,(2,2)); check(c,b'\x1b[A',[1,2],2,(1,2))
                check(c,b'j',[7,8],7,(2,2)); check(c,b'\x1b[B',[7,8],8,(2,2)); check(c,b'p',[1,2],1,(1,2)); check(c,b'n',[7,8],7,(2,2))
                require_conversation_screen(record_back(c,online),[7,8],7,(2,2),'All',False)
                require_visible_indices(act(c,b'b',GROUP),[8,7,2,1],7)
                act(c,b'u',GROUP); require_visible_indices(search(c,'recv 03',GROUP),[7],7)
                check(c,b't',[7],7,(1,1),'Unread',True)
                if online:
                    require_conversation_screen(compose(c,peer,before['history'][6]['identity'],replacement,body,False),[7],7,(1,1),'Unread',True)
                require_visible_indices(act(c,b'b',GROUP),[7],7)
                act(c,b'b',PICKER); act(c,b'b',INBOX); c.raw(b'b'); c.expect(MENU); c.raw(b'l'); c.finish()
                require_readonly(raw,(store/'mail.fkv').read_bytes(),before,inspect())
                assert HOME.encode() not in c.transcript
                save(root,'terminal-'+str(online)+'.private',c.transcript)
            require_closed_endpoint(endpoint)
        assert not any(root.glob('fixture-OWNED-*'))
        result = dict(result='pass',platform=platform.system(),machine=platform.machine(),unicode=unicode,
            scope='device-local-mail-conversation',count=len(CHECKS),checks=CHECKS,**association,
            navigation_readonly_preserved=True,private_fixture_removed=True,children_reaped=all(c.done for c in children))
        save(root,'result.private.json',json.dumps(result,indent=2)+'\n')
        return result
    finally:
        for i,c in enumerate(children):
            save(root,'terminal-final-'+str(i)+'.private',c.transcript+c.buffer); c.close()


def probe_main_menu(a):
    private_root(a.owned_root)
    settings = private_json(a.settings)
    assert settings['disposable_fixture'] is True and settings['files_disabled'] is True
    assert settings['admin_argv'][0] == '/usr/bin/ssh' and settings['member_argv'][0] == '/usr/bin/ssh'
    root = Path(tempfile.mkdtemp(prefix='conversation-menu-OWNED-', dir=a.owned_root)); root.chmod(0o700)
    password, home_password = secrets.token_hex(24), secrets.token_hex(24)
    handle = settings['member_handle'] + secrets.token_hex(3)
    subject, body, peer, other, replacement, markers = inputs(root, a.client, a.fixture,
        [password,home_password,settings['sysop_password']])
    children, trace, visit = [], None, None

    def remote(code, data=None):
        result = subprocess.run(settings['admin_argv'] + ['/usr/pkg/bin/python3.13 -c ' + shlex.quote(code)], input=data,capture_output=True,timeout=45)
        if result.returncode: save(root,'admin-failure.private',result.stdout+result.stderr)
        assert result.returncode == 0, 'owned conversation board operation failed'
        return result.stdout
    def inspect():
        result = subprocess.run([str(a.fixture),'inspect',str(store)],capture_output=True,timeout=30)
        assert result.returncode == 0
        return json.loads(result.stdout)
    def login(c, handle, secret, register=False):
        c.expect('[L]ogin [R]egister [Q]uit: '); c.send('r' if register else 'l'); c.expect('Handle: '); c.send(handle); c.password(secret)
        if register: c.password(secret,'Confirm password (hidden): ')
        else: c.expect('Press Enter to continue: '); c.send(''); c.expect(BOARD_MENU)
    def board():
        c = SSHConsole(settings['member_argv']); children.append(c)
        return c
    def same():
        assert_same_visit(visit,visit_processes(c.pid))
    def enter():
        c.raw(b'm'); c.expect(TITLE); c.expect(OPEN); c.expect(BACK); c.raw(b'o'); c.expect(CONFIRM_MAIL); c.send('y'); c.expect(UNLOCK)
        assert not termios.tcgetattr(c.fd)[3] & termios.ECHO
        c.send(PASS); c.expect(MENU)
    def leave_group():
        act(c,b'b',PICKER); act(c,b'b',INBOX); c.raw(b'b'); c.expect(MENU); c.raw(b'l'); c.expect(BOARD_MENU); same()
    try:
        store = root/'mail'
        result = subprocess.run([str(a.fixture),'seed',str(store),'ready'],input=('\n'.join([subject,body,peer,other])+'\n').encode(),capture_output=True,timeout=90)
        assert result.returncode == 0
        before = inspect(); canonical = (store/'mail.fkv').read_bytes()
        for row in before['history']: markers.extend([row['identity'],row['subject'],row['body'],row['address']])
        markers = list(dict.fromkeys(markers))
        association = bind(root,a.client,a.fixture,markers,publication_extra=[handle])
        marker_data = (root/'private-markers.json').read_bytes()
        # The registration handle is deliberately board-visible. Retain it only
        # in the private publication inventory, outside the board-exclusion set.
        with socket.socket() as unused:
            unused.bind(('127.0.0.1',0)); endpoint=unused.getsockname()
        require_closed_endpoint(endpoint)
        ca = save(root,'public-ca.pem',a.public_ca.read_bytes())
        config = save(root,'home.toml','version=1\naccount="alice@home.test"\nendpoint='+json.dumps('127.0.0.1:'+str(endpoint[1]))+'\ndomain="home.test"\nca='+json.dumps(str(ca))+'\npin="'+'ab'*32+'"\n')
        catalog = save(root,'boards.toml','version=1\nhome_mail={store='+json.dumps(str(store))+',config='+json.dumps(str(config))+'}\n[[boards]]\nid="approved"\nlabel="Owned conversation board"\nprofile='+json.dumps(settings['profile'])+'\n')
        outage = subprocess.run([str(a.mail_fixture),'sync',str(store),str(config)],input=(PASS+'\n'+home_password+'\n').encode(),capture_output=True,timeout=30)
        save(root,'outage.private.log',outage.stdout+outage.stderr)
        require_connection_refused(outage.returncode,outage.stdout,outage.stderr)
        require_readonly(canonical,(store/'mail.fkv').read_bytes(),before,inspect())
        registration = board(); login(registration,handle,password,True); registration.expect('Registration pending; sysop approval required.'); registration.finish()
        sysop = board(); login(sysop,'sysop',settings['sysop_password']); sysop.raw(b's'); sysop.expect('[C]reate room: '); sysop.send('a'); sysop.expect('Member handle: '); sysop.send(handle)
        sysop.expect('Status updated; previous sessions expired.'); sysop.expect('Press Enter to return to menu: '); sysop.send(''); sysop.expect(BOARD_MENU); sysop.raw(b'q'); sysop.expect('Logged out.'); sysop.finish()
        c = LauncherConsole([str(a.client),'--boards',str(catalog)]+(['--unicode'] if a.unicode else [])); children.append(c)
        c.expect(PICK); c.raw(b'a'); c.expect(CONFIRM); c.send('y'); login(c,handle,password); visit=visit_processes(c.pid)
        trace = json.loads(remote(TRACE_START.replace('issue230-board-','issue339-board-')))
        save(root,'trace-input.private.json',json.dumps(dict(trace=trace,**association)))
        enter(); c.raw(b'n'); c.password(home_password,HOME); c.expect('Home cycle stopped; pending ciphertext retained.'); c.expect('Enter to return: '); c.send(''); c.expect(MENU)
        start = len(c.transcript)
        act(c,b'i',INBOX); act(c,b'c',PICKER); act(c,b'o',GROUP)
        check(c,b't',[7,8],8,(2,2)); check(c,b'p',[1,2],1,(1,2)); check(c,b'n',[7,8],7,(2,2))
        require_conversation_screen(record_back(c,True),[7,8],7,(2,2),'All',False)
        require_visible_indices(act(c,b'b',GROUP),[8,7,2,1],7)
        act(c,b'u',GROUP); require_visible_indices(search(c,'recv 03',GROUP),[7],7)
        check(c,b't',[7],7,(1,1),'Unread',True)
        require_conversation_screen(compose(c,peer,before['history'][6]['identity'],replacement,body,False),[7],7,(1,1),'Unread',True)
        require_visible_indices(act(c,b'b',GROUP),[7],7)
        assert HOME.encode() not in c.transcript[start:], 'preview acquired home credentials'
        leave_group(); require_readonly(canonical,(store/'mail.fkv').read_bytes(),before,inspect())
        enter(); act(c,b'i',INBOX); act(c,b'c',PICKER); act(c,b'o',GROUP)
        check(c,b't',[7,8],8,(2,2)); act(c,b'b',GROUP); leave_group()
        require_readonly(canonical,(store/'mail.fkv').read_bytes(),before,inspect())
        same(); c.raw(b'q'); c.expect('Logged out.'); c.expect(RETURN); c.send(''); c.expect(PICK); c.raw(b'q'); c.finish()
        scan_input = json.dumps(dict(markers=markers,board=settings['scan_board'],store=settings['scan_store'],log=settings['scan_logs'])).encode()
        save(root,'board-file-scan-input.private.json',scan_input)
        scan = validate_scan(json.loads(remote(SCAN,scan_input)))
        board_io = remote(TRACE_STOP.replace('issue230-board-','issue339-board-'),json.dumps(trace).encode()); trace=None
        save(root,'board-io.private.log',board_io)
        reconstruction = require_board_bound_private_absent(board_io,[m.encode() for m in markers])
        streams,_ = reconstruct_kdump_streams(board_io)
        assert all(any(token in value for key,value in streams.items() if key[2]=='read') for token in [b'@MAIL1 park ',b'@MAIL1 back '])
        assert all(b'FACTION1;' not in value and b'@FACTION1' not in value for value in streams.values())
        require_closed_endpoint(endpoint)
        assert all(m.encode() not in (store/'mail.fkv').read_bytes() for m in markers)
        hashes=dict(trace_sha256=sha(board_io),marker_set_sha256=sha(marker_data),file_scan_input_sha256=sha(scan_input))
        save(root,'canonical-before.private.json',json.dumps(before)); save(root,'canonical-after.private.json',json.dumps(inspect()))
        result=dict(result='pass',platform=platform.system(),machine=platform.machine(),unicode=a.unicode,
            scope='same-ssh-main-menu-device-local-mail-conversation',count=len(MAIN_CHECKS),checks=MAIN_CHECKS,
            **association,private_input_hashes=hashes,board_io_reconstruction=reconstruction,privacy_scan=scan,
            private_matches=scan['matches'],marker_count=len(markers),files_disabled=True,actual_home_transport_error='Connect',
            canonical_navigation_unchanged=True,same_ssh_pid_until_normal_close=True,automatic_redial_observed=False,private_fixture_retained_for_lead=True)
        save(root,'result.private.json',json.dumps(result,indent=2)+'\n')
        return result
    finally:
        if trace is not None:
            try: save(root,'board-io-failure.private.log',remote(TRACE_STOP.replace('issue230-board-','issue339-board-'),json.dumps(trace).encode()))
            except (AssertionError,subprocess.SubprocessError): pass
        if visit is not None:
            try:
                if visit_processes(visit['client']) == visit: os.kill(visit['ssh'],15)
            except (AssertionError,ChildProcessError,ProcessLookupError): pass
        for i,child in enumerate(children):
            save(root,'terminal-'+str(i)+'.private',child.transcript+child.buffer); child.close()


if __name__ == '__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--client',type=Path,required=True); parser.add_argument('--fixture',type=Path,required=True)
    parser.add_argument('--owned-root',type=Path,required=True); parser.add_argument('--unicode',action='store_true'); parser.add_argument('--output',type=Path)
    parser.add_argument('--settings',type=Path); parser.add_argument('--mail-fixture',type=Path); parser.add_argument('--public-ca',type=Path)
    args=parser.parse_args()
    if args.settings:
        assert args.mail_fixture and args.public_ca
        result=probe_main_menu(args)
    else: result=probe(args.client.resolve(),args.fixture.resolve(),args.owned_root.resolve(),args.unicode)
    if args.output: save(args.output.parent,args.output.name,json.dumps(result,indent=2)+'\n')
    print('Retained-conversation PTY controls passed:',result['count'],result['platform'],result['machine'])
