#!/usr/bin/env python3
"""Owned conversation compose/picker proof; synthetic loopback homes only.

The shared transport, inspectors and cancellation oracle retain their qualified
implementation. This driver does not provision a VM or a visited board.
"""
from probe_foreground_conversation import (
    Relay, LiveHome, certificates, native_command, canonical_snapshot,
    require_cancelled_confirmation,
    extension, element, ACCOUNTS, DEVICES,
    read_review, expect_one, owned_root,
    MARKER, arm_brief,
)
from probe_mail_reply import require_reply_commit
from probe_mail_conversation import require_conversation_screen as _require_retained_screen
import copy
import hashlib
import json
import os
from pathlib import Path
import re
import secrets
import ssl
import subprocess
import sys
import tempfile
import unicodedata
from xml.etree import ElementTree as ET


def require_composing_screen(screen, indices, selected, page, mode, active,
                             device=202, evidence_root=None):
    """Project only this eligible journey's exact combined W/Back footer.

    The row count stays exact. Every other raw scalar and row reaches the
    unchanged strict conversation oracle; the replaced row is fixed ASCII and
    independently bounded to 39 cells. Body text and earlier output stay exact.
    """
    footer = b'[W] Write fresh text  [B] Back'
    start = screen.rfind(b'[LOCAL CONVERSATION]')
    if start < 0:
        raise ValueError('current conversation title missing')
    rows = screen[start:].split(b'\r\n')
    positions = [index for index, row in enumerate(rows) if row == footer]
    if (not footer.isascii() or len(footer) > 39 or len(positions) != 1
            or b'[B] Back' in rows or any(rows[positions[0]+1:])):
        raise ValueError('one exact final combined W/Back footer required')
    rows[positions[0]] = b'[B] Back'
    projected = screen[:start]+b'\r\n'.join(rows)
    if evidence_root is not None:
        root = owned_root(evidence_root)
        path = root/('conversation-projection-'+secrets.token_hex(6)+'.private.json')
        path.write_text(json.dumps(dict(raw_hex=screen.hex(), projected_hex=projected.hex(),
            indices=indices, selected=selected, page=page, mode=mode, active=active,
            device=device), sort_keys=True)+'\n')
        path.chmod(0o600)
    _require_retained_screen(projected, indices, selected, page, mode, active, device)
    return projected


def capture_pending_wires(root, store, inspector, mail_fixture):
    """Inspect exact pending XML from the qualified read-only canonical copy."""
    root = owned_root(root); store = Path(store)
    previous = set(root.glob('inspection-copy-OWNED-*'))
    state = canonical_snapshot(root, store, inspector, mail_fixture)
    created = set(root.glob('inspection-copy-OWNED-*'))-previous
    if len(created) != 1:
        raise ValueError('one controlled canonical inspection copy required')
    copied = created.pop(); raw = (copied/'mail.fkv').read_bytes()
    if (store/'mail.fkv').read_bytes() != raw:
        raise ValueError('pending capture no longer matches the live canonical')
    argv = [str(inspector), 'pending-wires', str(copied)]
    result = subprocess.run(argv, input=b'checkpoint synthetic fixture password\n',
                            capture_output=True, timeout=15)
    report = root/('pending-wires-'+secrets.token_hex(6)+'.private.json')
    report.write_text(json.dumps(dict(argv=argv, exit=result.returncode,
        stdout_hex=result.stdout.hex(), stderr_hex=result.stderr.hex()), sort_keys=True)+'\n')
    report.chmod(0o600)
    if (result.returncode or result.stderr or len(result.stdout) > 1024*1024
            or (store/'mail.fkv').read_bytes() != raw or (copied/'mail.fkv').read_bytes() != raw):
        raise ValueError('private pending-wire inspection failed or changed canonical bytes')
    rows = json.loads(result.stdout)
    pending = {row['operation']:row for row in state['pending']}
    if (not isinstance(rows, list) or len(rows) != len(state['pending'])
            or len(pending) != len(state['pending']) or len(rows) > 8):
        raise ValueError('exact bounded pending-wire inventory required')
    wires = {}
    for row in rows:
        if (not isinstance(row, dict) or set(row) != {'operation', 'message'}
                or not isinstance(row['operation'], str)
                or not re.fullmatch('[a-f0-9]{32}', row['operation'])
                or row['operation'] == '00'*16 or row['operation'] in wires
                or row['operation'] not in pending or not isinstance(row['message'], str)
                or not 0 < len(row['message']) <= 32768 or len(row['message']) % 2
                or not re.fullmatch('[a-f0-9]+', row['message'])):
            raise ValueError('noncanonical or mismatched pending-wire inventory')
        wire = bytes.fromhex(row['message']); stanza = element(wire)
        if (stanza.get('id') != row['operation']
                or stanza.get('from') != state['archive']['home']
                or stanza.get('to') != pending[row['operation']]['address']):
            raise ValueError('pending wire differs from its canonical operation and route')
        wires[row['operation']] = wire
    return wires


class ReleaseRelay(Relay):
    """The qualified relay with one pre-captured immutable release expectation.

    Set expected_release only after bootstrap, using a wire inspected from an
    exact canonical ciphertext copy before dispatch. This does not decrypt or
    reinterpret the encrypted payload; only qualified outer routing may change.
    """
    def message(self, actor, raw):
        expectation = getattr(self, 'expected_release', None)
        if expectation is not None:
            if isinstance(raw, bytes) and 0 < len(raw) <= 16384:
                capture = self.root/('release-attempt-'+secrets.token_hex(6)+'.private')
                capture.write_bytes(raw); capture.chmod(0o600)
            expected_actor, operation, persisted = expectation
            expected, actual = element(persisted), element(raw)
            if expected_actor not in (0, 1) or actor != expected_actor:
                raise ValueError('release came from an unexpected synthetic actor')
            own, peer = ACCOUNTS[actor], ACCOUNTS[1-actor]
            own_full = own+'/faction'+str(DEVICES[actor])
            peer_full = peer+'/faction'+str(DEVICES[1-actor])
            if (expected.get('id') != operation or actual.get('id') != operation
                    or expected.get('from') not in (own, own_full)
                    or actual.get('from') not in (own, own_full)
                    or expected.get('to') not in (peer, peer_full)
                    or actual.get('to') != peer_full):
                raise ValueError('release differs from selected pending operation or full route')
            expected, actual = copy.deepcopy(expected), copy.deepcopy(actual)
            expected.set('from', own_full); actual.set('from', own_full)
            expected.set('to', peer_full)
            if ET.tostring(expected) != ET.tostring(actual):
                raise ValueError('release changed persisted encrypted content or metadata')
        return super().message(actor, raw)


def require_composed(before, after, address, device, subject, body):
    """Require one queue append in this warm, unread-seeded journey.

    The retained received seed provides the independently checked peer identity.
    Its unread state and nonempty archive token are fixture preconditions, not
    additional production W eligibility conditions.
    """
    require_reply_commit(before, after, address, device, subject, body)
    peers = [row for row in before['history']
             if row['direction'] == 'received' and row['address'] == address
             and row['device'] == device]
    identities = {row['identity'] for row in peers}
    row, pending = after['history'][-1], after['pending'][-1]
    operation = row['operation']
    if (not peers or not any(peer['read'] is False for peer in peers)
            or len(identities) != 1 or row['identity'] not in identities
            or not before['archive']['after']
            or not isinstance(operation, str)
            or not re.fullmatch('[a-f0-9]{32}', operation)
            or operation == '00'*16
            or any(previous['operation'] == operation for previous in before['history'])
            or pending['sent_at'] != row['sent_at']):
        raise ValueError('exact current-peer fresh queue association required')


def require_received(before, after, subject, body):
    """Check A's first authenticated input after its own outgoing seed.

    Read tracking can still be disabled on A. Existing read decisions, pending
    work and archive position must stay exact, while the new row is never read.
    """
    rows = extension(before, after)
    peers = {(row['address'], row['device'], row['identity'])
             for row in before['history'] if row['direction'] == 'sent'}
    if (len(rows) != 1 or len(peers) != 1
            or after['archive'] != before['archive']):
        raise ValueError('one exact received record with unchanged archive required')
    row = rows[0]
    if (row['direction'] != 'received' or row['read'] not in (None, False)
            or (row['address'], row['device'], row['identity']) not in peers
            or (row['subject'], row['body']) != (subject, body)):
        raise ValueError('authenticated input differs from exact current peer and text')


def connect_pair(receiver, sender, relay, confirm):
    """This journey receives on A before B may release its reviewed text."""
    previous = relay.generations[0]
    confirm(receiver)
    generation = relay.wait_bound(0, after=previous, timeout=2)
    confirm(sender)
    return generation


def arm_queued(console, own, peer, device, identity, operation):
    """Choose record 3 after explicit movement in this two-text fixture.

    Stop at the fresh literal confirmation. No confirmation or credential is
    supplied here. The caller observes canonical state/counters around cancels.
    """
    console.raw(b'e'); console.expect('[LOCAL BRIEF EXCHANGE]')
    review = read_review(console, b'\r\n[B] Return\r\n')
    panes = b''.join(line[2:] for line in review.splitlines() if line.startswith(b'| '))
    required = [('Home: '+own).encode(), ('Peer: '+peer).encode(),
                ('Peer device: '+str(device)).encode(), identity.encode()]
    if any(value not in panes for value in required) or b'[Q] Choose queued text' not in review:
        raise ValueError('queued selection lacks the exact current peer review')

    def picker(selected):
        _, raw = expect_one(console, [b'\r\n[B] Return\r\n'])
        text = raw.decode()
        lines = text.split('\r\n')
        if (b'[LOCAL QUEUED TEXT]' not in raw or len(lines) > 24
                or any(unicodedata.category(c) in ('Cc', 'Cf')
                       for c in text.replace('\r\n', ''))):
            raise ValueError('bounded inert queued picker required')
        for line in lines:
            cells = sum(0 if unicodedata.combining(c) else
                        2 if unicodedata.east_asian_width(c) in 'WF' else 1 for c in line)
            if cells > 39:
                raise ValueError('queued picker exceeded classic width')
        rows = re.findall(r'(?m)^([> ]) (\d{2}) [^\r\n]*\r?$', text)
        if ([int(number) for _, number in rows] != [2, 3]
                or [int(number) for mark, number in rows if mark == '>'] != [selected]
                or f'Page 1/1; selection {selected-1}/2' not in lines):
            raise ValueError('queued picker did not preserve the explicit original selection')
        return raw

    console.raw(b'q'); first = picker(2)
    console.raw(b'j'); second = picker(3)
    console.raw(b'o')
    selected = read_review(console, b'\r\nType exchange to connect, or Enter: ')
    panes = [line[2:] for line in selected.splitlines() if line.startswith(b'| ')]
    if (operation.encode() not in panes or b'Retained record: 3' not in panes
            or b'Send selected text only.' not in panes):
        raise ValueError('queued selection differs from the reviewed canonical operation')
    return review+first+second+selected


def write_conversation(console, peer, identity, subject, body, queue, unicode=False):
    """Drive B's existing empty-start writer; review every finite body row."""
    subject_prompt = b'Subject (128 UTF-8 bytes): '
    queue_prompt = b'Type queue to persist encrypted ciphertext: '
    next_page = b'[Enter] Next page  [B] Return\r\n'
    expected_peer = ('CURRENT recipient: '+peer+'\r\n').encode()
    expected_device = ('CURRENT device: 101 / ed25519:'+identity+'\r\n').encode()
    console.raw(b'w')
    _, initial = expect_one(console, [subject_prompt])
    if expected_peer not in initial or expected_device not in initial or b'Suggested subject:' in initial:
        raise ValueError('fresh conversation writer lacks the exact current recipient')
    console.send(subject); console.expect('[LOCAL MULTILINE WRITER]'); console.expect('write> ')
    for line in body.split('\n'):
        console.send('.'+line if line.startswith('.') else line)
        console.expect('Accepted:'); console.expect('write> ')
    console.send('.done')
    pieces = []
    for _ in range(4):
        token, raw = expect_one(console, [next_page, queue_prompt])
        pieces.append(raw)
        if token == queue_prompt: break
        console.send('')
    else:
        raise ValueError('bounded writer review did not finish')
    review = b''.join(pieces)
    displayed = body if unicode else body.replace('λ', '\\u{3bb}')
    rows = [line[2:].decode() for line in review.splitlines() if line.startswith(b'| ')]
    pages = [tuple(map(int, match)) for match in
             re.findall(rb'\[LOCAL BODY\] Page ([0-9]+)/([0-9]+)', review)]
    if (expected_peer not in review or expected_device not in review
            or ('Subject: '+subject+'\r\n').encode() not in review
            or rows != displayed.split('\n') or not pages
            or pages != [(number, pages[-1][1]) for number in range(1, pages[-1][1]+1)]):
        raise ValueError('complete writer review differs from the exact current intent')
    console.send('queue' if queue else '')
    console.expect('Queued durably;' if queue else 'Writing cancelled;')
    return review


def run_journey(arguments):
    """One fresh classic OR Unicode production journey, without retries."""
    from smoke_native_client_ssh import NativeConsole
    from probe_visit_mail import require_closed_endpoint
    import probe_home_mail
    import probe_xmpp_transport
    parent = owned_root(arguments.owned_root)
    root = Path(tempfile.mkdtemp(prefix='composing-unicode-OWNED-' if arguments.unicode
                                 else 'composing-classic-OWNED-', dir=parent))
    root.chmod(0o700); (root/'OWNED').write_bytes(MARKER)

    def save(name, value):
        path = root/name
        path.write_bytes(value if isinstance(value, bytes) else
                         (json.dumps(value, ensure_ascii=False, sort_keys=True, indent=2)+'\n').encode())
        path.chmod(0o600)
        return path

    token = 'compose-'+secrets.token_hex(5)
    local_password = 'checkpoint synthetic fixture password'
    home_passwords = [secrets.token_hex(20), secrets.token_hex(20)]
    seed = (token+' seed', 'seed-'+secrets.token_hex(8))
    first = (token+' first', 'first-'+secrets.token_hex(6)+'\n\n.done\nlast λ')
    second = ('chosen-'+secrets.token_hex(6), 'second-'+secrets.token_hex(6)+'\n\n.done\nlast λ')
    inputs = save('inputs.private.json', dict(local_password=local_password,
        home_passwords=home_passwords, query=token, seed=seed, first=first, second=second))
    objects = root/'source-objects'; objects.mkdir(mode=0o700)
    binary_root = root/'binaries'; binary_root.mkdir(mode=0o700)
    binaries = {}; binary_bindings = {}
    for name in ('client', 'mail_fixture', 'inspector'):
        source = Path(getattr(arguments, name)).resolve(); raw = source.read_bytes()
        copied = binary_root/name; copied.write_bytes(raw); copied.chmod(0o700)
        binaries[name] = copied
        binary_bindings[name] = dict(path=str(source), executed=str(copied),
                                     sha256=hashlib.sha256(raw).hexdigest())
    repo = Path(__file__).resolve().parent
    imports = {}
    for name, module in tuple(sys.modules.items()):
        location = getattr(module, '__file__', None)
        if location is None: continue
        source = Path(location).resolve()
        if source.parent != repo or source.suffix != '.py': continue
        raw = source.read_bytes(); digest = hashlib.sha256(raw).hexdigest()
        target = objects/digest
        if not target.exists(): target.write_bytes(raw); target.chmod(0o600)
        imports[name] = dict(path=str(source), object=str(target), sha256=digest)
    raw = Path(__file__).read_bytes(); digest = hashlib.sha256(raw).hexdigest()
    target = objects/digest
    if not target.exists(): target.write_bytes(raw); target.chmod(0o600)
    save('bindings.private.json', dict(producer=dict(path=str(Path(__file__).resolve()),
        object=str(target), sha256=digest), imports=imports, binaries=binary_bindings,
        input_sha256=hashlib.sha256(inputs.read_bytes()).hexdigest(), python=sys.version))

    relay = ReleaseRelay(root); homes = []; members = []; consoles = []
    facts = None; failures = []; controls = {}
    HOME = 'HOME account password (hidden; never sent to a BBS): '
    MENU = '[L] Lock and exit'; CONVERSATION = '[LOCAL CONVERSATION]'

    def call(index, action, *args, password=False, lines=()):
        return native_command(root, binaries['mail_fixture'], action, members[index],
                              *args, home_password=password, lines=lines)

    def snapshot(index, label):
        state = canonical_snapshot(root, members[index]['store'], binaries['inspector'],
                                   binaries['mail_fixture'])
        save(label+'-'+str(index)+'.state.private.json', state)
        save(label+'-'+str(index)+'.canonical.private', (members[index]['store']/'mail.fkv').read_bytes())
        return state

    def counts():
        return tuple((home.connections, home.authenticated) for home in homes)

    def screen(console):
        start = len(console.transcript)-len(console.buffer)
        console.expect(CONVERSATION); console.expect('[B] Back')
        captured = console.transcript[start:len(console.transcript)-len(console.buffer)]
        return captured[captured.rfind(CONVERSATION.encode()):]

    def start(index):
        console = NativeConsole([str(binaries['client']), '--mail-online',
            str(members[index]['store']), str(members[index]['config'])]+
            (['--unicode'] if arguments.unicode else []))
        consoles.append(console); console.resize(40, 24)
        console.password(local_password, 'LOCAL checkpoint passphrase (hidden): ')
        console.expect(MENU)
        return console

    def conversation(console, unread):
        console.raw(b'i'); console.expect('[LOCAL INBOX]'); console.expect('[B] Back')
        console.raw(b'c'); console.expect('[LOCAL CORRESPONDENTS]'); console.expect('[B] Back')
        console.raw(b'o'); console.expect('[LOCAL CORRESPONDENT INBOX]'); console.expect('[B] Back')
        if unread:
            console.raw(b'u'); console.expect('[LOCAL CORRESPONDENT INBOX]'); console.expect('[B] Back')
        console.raw(b's'); console.expect('Search (64 bytes; blank clears): '); console.send(token)
        console.expect('Local search active.'); console.expect('[B] Back')
        console.raw(b't')
        return screen(console)

    def retained_view(view, expected):
        require_composing_screen(view, [1], 1, (1, 1), 'Unread', True, 101, evidence_root=root)
        if view != expected:
            raise ValueError('hidden queue or exchange changed the retained conversation view')

    def local_interval(label, before_counts, before, after, original, returned, readonly):
        after_counts = counts()
        save(label+'.interval.private.json', dict(before_counts=before_counts,
            after_counts=after_counts, before_state=before, after_state=after,
            before_view_hex=original.hex(), after_view_hex=returned.hex()))
        require_cancelled_confirmation(before_counts, after_counts,
                                       before if readonly else None, after if readonly else None)
        retained_view(returned, original)

    def confirm(console):
        index = 1 if console is sender else 0
        console.send('exchange'); console.password(home_passwords[index], HOME)
        console.expect('Connecting directly to your home.')

    def result(console, write, accepted, stopped=False):
        start = len(console.transcript)-len(console.buffer)
        console.expect(write); console.expect(f'Accepted on this device: {accepted}.')
        console.expect('Duplicates: 0.'); console.expect('Enter to return: ')
        captured = console.transcript[start:len(console.transcript)-len(console.buffer)]
        if (b'Brief exchange stopped.' in captured) != stopped:
            raise ValueError('foreground result stopped-state mismatch')
        console.send('')
        return screen(console)

    def return_menu(console):
        for label in ('[LOCAL CORRESPONDENT INBOX]', '[LOCAL CORRESPONDENTS]', '[LOCAL INBOX]'):
            console.raw(b'b'); console.expect(label); console.expect('[B] Back')
        console.raw(b'b'); console.expect(MENU)

    try:
        identities = []
        for index, domain in enumerate(('home.test', 'away.test')):
            tls = root/domain; tls.mkdir(mode=0o700)
            pair, ca = certificates(tls, domain, arguments.openssl)
            home = LiveHome(root, index, relay, pair, home_passwords[index]); homes.append(home)
            config = root/('home-'+str(index)+'.toml')
            der = ssl.PEM_cert_to_DER_cert(pair[0].read_text())
            values = dict(version=1, account=ACCOUNTS[index], endpoint='127.0.0.1:'+str(home.port),
                          domain=domain, ca=str(ca), pin=hashlib.sha256(der).hexdigest())
            config.write_text(''.join(key+'='+json.dumps(value)+'\n' for key, value in values.items()))
            config.chmod(0o600)
            members.append(dict(store=root/('alice' if index == 0 else 'bob'),
                local_password=local_password, password=home_passwords[index], config=config))
            identity = call(index, 'init', ACCOUNTS[index], DEVICES[index]).decode().strip()
            if not re.fullmatch('[a-f0-9]{64}', identity):
                raise ValueError('native public identity unavailable')
            identities.append(identity)
        if identities[0] == identities[1]:
            raise ValueError('independent native identity fixtures required')
        for index in (0, 1):
            call(index, 'enroll', ACCOUNTS[1-index], DEVICES[1-index], identities[1-index])
            call(index, 'enable-archive')
        call(1, 'publish', members[1]['config'], password=True)
        call(0, 'prepare', members[0]['config'], password=True)
        call(0, 'queue', lines=seed); call(0, 'sync', members[0]['config'], password=True)
        for index, expected in ((1, (1, 0)), (0, (0, 1))):
            page = json.loads(call(index, 'catchup', members[index]['config'], password=True))
            if page != dict(entries=1, accepted=expected[0], duplicates=expected[1], complete=True):
                raise ValueError('warm seed archive did not match the controlled inventory')

        sender = start(1)
        sender.raw(b'e'); sender.expect('Type enable to add encrypted local read tracking: ')
        sender.send('enable'); sender.expect('Encrypted local read tracking enabled.')
        sender.expect('Enter to return: '); sender.send(''); sender.expect(MENU)
        before = snapshot(1, 'seed')
        if (len(before['history']) != 1 or before['history'][0]['read'] is not False
                or before['history'][0]['identity'] != identities[0]
                or not before['archive']['after'] or before['pending']):
            raise ValueError('warm current-peer unread seed fixture required')
        original = conversation(sender, True)
        require_composing_screen(original, [1], 1, (1, 1), 'Unread', True, 101, evidence_root=root)
        save('conversation-original.private', original)

        baseline_counts = counts()
        write_conversation(sender, ACCOUNTS[0], identities[0], *first, False, arguments.unicode)
        returned = screen(sender); after_cancel = snapshot(1, 'compose-cancel')
        local_interval('compose-cancel', baseline_counts, before, after_cancel, original, returned, True)
        if (root/'seed-1.canonical.private').read_bytes() != (members[1]['store']/'mail.fkv').read_bytes():
            raise ValueError('cancelled writer changed encrypted canonical bytes')
        for label, content in (('first-queue', first), ('second-queue', second)):
            baseline_counts = counts()
            write_conversation(sender, ACCOUNTS[0], identities[0], *content, True, arguments.unicode)
            returned = screen(sender); queued = snapshot(1, label)
            require_composed(before, queued, ACCOUNTS[0], DEVICES[0], *content)
            local_interval(label, baseline_counts, before, queued, original, returned, False)
            before = queued
        selected = before['history'][2]; selected_operation = selected['operation']
        if (selected['subject'], selected['body']) != second or len(before['pending']) != 2:
            raise ValueError('exact second queued multiline intent missing')
        sender_bytes = (members[1]['store']/'mail.fkv').read_bytes()
        wires = capture_pending_wires(root, members[1]['store'], binaries['inspector'], binaries['mail_fixture'])
        save('persisted-wires.private.json', {operation:wire.hex() for operation, wire in wires.items()})
        if set(wires) != {row['operation'] for row in before['pending']}:
            raise ValueError('captured wires differ from the exact pending inventory')

        # Enter Q and Back without ever selecting, confirming or obtaining a credential.
        baseline_counts = counts()
        sender.raw(b'e'); sender.expect('[LOCAL BRIEF EXCHANGE]')
        review = read_review(sender, b'\r\n[B] Return\r\n')
        if b'[Q] Choose queued text' not in review: raise ValueError('queued action unavailable')
        sender.raw(b'q'); _, picker = expect_one(sender, [b'\r\n[B] Return\r\n'])
        save('queued-back-screen.private', picker)
        if b'[LOCAL QUEUED TEXT]' not in picker: raise ValueError('queued picker missing')
        sender.raw(b'b'); sender.expect('[Q] Choose queued text'); sender.expect('[B] Return')
        sender.raw(b'b'); returned = screen(sender)
        local_interval('picker-back', baseline_counts, before, snapshot(1, 'picker-back'),
                       original, returned, True)
        baseline_counts = counts()
        arm_queued(sender, ACCOUNTS[1], ACCOUNTS[0], DEVICES[0], identities[0], selected_operation)
        sender.send(''); returned = screen(sender)
        local_interval('picker-blank', baseline_counts, before, snapshot(1, 'picker-blank'),
                       original, returned, True)
        if (members[1]['store']/'mail.fkv').read_bytes() != sender_bytes:
            raise ValueError('picker navigation or cancellation changed canonical ciphertext')

        receiver = start(0); receiver_before = snapshot(0, 'receiver-before')
        receiver_view = conversation(receiver, False)
        require_composing_screen(receiver_view, [1], 1, (1, 1), 'All', True, 202, evidence_root=root)
        arm_brief(receiver, ACCOUNTS[0], ACCOUNTS[1], DEVICES[1], identities[1])
        selected_review = arm_queued(sender, ACCOUNTS[1], ACCOUNTS[0], DEVICES[0], identities[0], selected_operation)
        displayed_subject = second[0] if arguments.unicode else second[0].replace('λ', '\\u{3bb}')
        if ('Subject: '+displayed_subject).encode() not in selected_review:
            raise ValueError('selected review omitted the exact fresh subject')
        relay.expected_release = (1, selected_operation, wires[selected_operation])
        message_start = len(relay.messages); queries_before = list(relay.queries)
        foreground_counts = counts()
        generation = connect_pair(receiver, sender, relay, confirm)
        sender_view = result(sender, 'Selected text local write completed.', 0)
        arrived_view = result(receiver, 'No selected text write attempted.', 1)
        retained_view(sender_view, original)
        if arrived_view != receiver_view:
            raise ValueError('nonmatching live text changed the retained A search view')
        expected_release = [dict(actor=1, operation=selected_operation,
                                to=ACCOUNTS[0]+'/faction101', bound_generation=generation)]
        if relay.messages[message_start:] != expected_release or relay.queries != queries_before:
            raise ValueError('foreground exchange wrote other text or automatically fetched archive')
        if (members[1]['store']/'mail.fkv').read_bytes() != sender_bytes:
            raise ValueError('selected write changed or retired sender pending ciphertext')
        sender_after = snapshot(1, 'sender-after'); receiver_after = snapshot(0, 'receiver-after')
        require_cancelled_confirmation(None, None, before, sender_after)
        require_received(receiver_before, receiver_after, *second)
        save('foreground.interval.private.json', dict(before_counts=foreground_counts,
            after_counts=counts(), selected_operation=selected_operation, messages=expected_release,
            before_receiver=receiver_before, after_receiver=receiver_after))
        if any(after != (prior[0]+1, prior[1]+1) for prior, after in zip(foreground_counts, counts())):
            raise ValueError('foreground attempt did not use exactly one fresh connection and authentication per actor')

        homes[1].close(); require_closed_endpoint(('127.0.0.1', homes[1].port))
        baseline_counts = counts()
        arm_brief(sender, ACCOUNTS[1], ACCOUNTS[0], DEVICES[0], identities[0]); confirm(sender)
        returned = result(sender, 'No selected text write attempted.', 0, stopped=True)
        local_interval('home-outage', baseline_counts, sender_after, snapshot(1, 'home-outage'),
                       original, returned, True)
        for console in (sender, receiver): return_menu(console); console.raw(b'l'); console.finish()
        credentials = [local_password, *home_passwords]
        if any(value.encode() in console.transcript for value in credentials for console in consoles):
            raise ValueError('hidden credential reached a terminal capture')
        markers = credentials+[token, *(value for content in (seed, first, second) for value in content)]
        server_bytes = b''.join(path.read_bytes() for path in root.glob('home-*.private'))
        server_bytes += b''.join(frame for actor in (0, 1) for _, frame in relay.inventory(actor))
        if any(value.encode() in server_bytes for value in markers):
            raise ValueError('private plaintext reached home application capture or archive')
        if any(value.encode() in (member['store']/'mail.fkv').read_bytes()
               for value in markers for member in members):
            raise ValueError('plaintext marker found in encrypted canonical')
        for binding in imports.values():
            if hashlib.sha256(Path(binding['path']).read_bytes()).hexdigest() != binding['sha256']:
                raise ValueError('executed helper source changed during the journey')
        controls = dict(compose_cancel=True, queued_two_in_UI=True, unread_seed_preserved=True,
            hidden_filter_and_view_return=True, picker_back_no_connection=True,
            picker_blank_no_connection=True, exact_persisted_ciphertext=True,
            authenticated_exact_multiline=True, first_queued_text_unsent=True,
            no_automatic_archive=True, home_outage_return=True, server_plaintext_matches=0)
        facts = dict(result='pass', unicode=arguments.unicode,
            scope='conversation-composing-queued-selection-production-ui-synthetic-two-home-tls',
            production_s2s=False, netbsd_qualified=False, independent_interoperability=False,
            selected_writes=1, live_accepted=1, tls_versions=[home.tls_versions[-1] for home in homes],
            controls=controls)
    except BaseException:
        import traceback
        save('failure.private.txt', traceback.format_exc().encode())
        raise
    finally:
        reaped = []
        for index, console in enumerate(consoles):
            save('terminal-'+str(index)+'.private', console.transcript+console.buffer)
            try: console.close(); reaped.append(True)
            except BaseException as error: reaped.append(False); failures.append(type(error).__name__)
        for home in homes:
            try: home.close()
            except BaseException as error: failures.append(type(error).__name__)
            if home.failure is not None: failures.append(home.failure)
        refused = []
        for home in homes:
            try: require_closed_endpoint(('127.0.0.1', home.port)); refused.append(True)
            except (AssertionError, OSError): refused.append(False); failures.append('listener-not-refused')
        teardown = dict(workers_joined=all(not home.thread.is_alive() for home in homes),
            listeners_closed=all(home.closed for home in homes), child_processes_reaped=all(reaped),
            closed_ports_refused=refused, failures=failures)
        save('teardown.private.json', teardown)
        save('relay.private.json', dict(messages=relay.messages, queries=relay.queries,
            archives=[[(ident, wire.hex()) for ident, wire in relay.inventory(actor)] for actor in (0, 1)]))
    if failures or not all(teardown[key] for key in ('workers_joined', 'listeners_closed', 'child_processes_reaped')):
        raise ValueError('owned composing fixture teardown or protocol failed')
    facts['teardown'] = teardown
    save('result.private.json', facts)
    return facts


def main():
    import argparse
    import shutil
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('owned-root', 'client', 'mail-fixture', 'inspector'):
        parser.add_argument('--'+name, type=Path, required=True)
    parser.add_argument('--openssl', default=shutil.which('openssl'))
    parser.add_argument('--unicode', action='store_true')
    result = run_journey(parser.parse_args())
    print('Conversation compose/queued selection synthetic TLS journey passed:',
          'Unicode' if result['unicode'] else 'classic')


if __name__ == '__main__':
    try: main()
    except (ValueError, OSError, AssertionError, TimeoutError, subprocess.SubprocessError):
        raise SystemExit('Composing fixture failed; private diagnostics retained.') from None
