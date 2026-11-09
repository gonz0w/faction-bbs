#!/usr/bin/env python3
"""Bounded #363/#364 production-console proof on owned synthetic TLS homes.

The reusable actor hooks are trusted test-fixture seams for a separately approved
native parked-SSH adapter. The CLI accepts only the ordinary local actor.
"""
from probe_foreground_conversation import (
    LiveHome, certificates, native_command, canonical_snapshot,
    require_cancelled_confirmation, require_live, ACCOUNTS, DEVICES,
    expect_one, read_review, owned_root, MARKER, arm_brief, connect_pair,
)
from probe_conversation_composing import (
    ReleaseRelay, capture_pending_wires, require_composing_screen,
)
from probe_mail_reply import require_reply_commit
import copy
import hashlib
import json
import re
import secrets
import ssl
import subprocess
import sys
import tempfile
import unicodedata
from pathlib import Path

NEXT = b'\r\n[Enter] Next page  [B] Return\r\n'
CONFIRM = b'\r\nType exchange to connect, or Enter: '
ACK = b'Type read to acknowledge locally, or Enter to keep current state: '
RESULT = b'[R] Review newly received text'
ARRIVALS = b'[LOCAL NEWLY RECEIVED TEXT]'
MENU = '[L] Lock and exit'
CONVERSATION = '[LOCAL CONVERSATION]'
HOME = 'HOME account password (hidden; never sent to a BBS): '


def displayed(text, unicode):
    """Render this proof's declared small alphabet; refuse unreviewed scalars."""
    result = []
    for char in text:
        if char == 'λ': result.append(char if unicode else '\\u{3bb}')
        elif char == '\u202e': result.append('\\u{202e}')
        elif char == '\n' or ' ' <= char <= '~': result.append(char)
        else: raise ValueError('fixture scalar outside the reviewed proof alphabet')
    return ''.join(result)


def safe_rows(raw, cap=39):
    text = raw.decode()
    rows = text.split('\r\n')
    for row in rows:
        if any(unicodedata.category(char) in ('Cc', 'Cf') for char in row):
            raise ValueError('unsafe raw terminal scalar')
        cells = sum(0 if unicodedata.combining(char) else
                    2 if unicodedata.east_asian_width(char) in 'WF' else 1 for char in row)
        if cells > cap: raise ValueError('proof pane exceeded its classic width')
    return rows


def require_body_review(raw, body, unicode):
    start = raw.find(b'[LOCAL BODY]')
    if start < 0: raise ValueError('complete body pages missing')
    # Only body rows have a pane prefix; prompts are checked by each caller.
    rows = [row[2:].decode() for row in raw[start:].split(b'\r\n') if row.startswith(b'| ')]
    expected = displayed(body, unicode).split('\n')
    if any(len(row) > 37 for row in expected):
        raise ValueError('fixture body exceeds the declared no-wrap oracle shape')
    pages = [tuple(map(int, match)) for match in
             re.findall(rb'\[LOCAL BODY\] Page ([0-9]+)/([0-9]+)', raw[start:])]
    total = (len(expected)+9)//10
    if rows != expected or pages != [(number, total) for number in range(1, total+1)]:
        raise ValueError('body rows or full canonical page sequence differ')
    for row in raw[start:].split(b'\r\n'):
        if row.startswith(b'| '): safe_rows(row)


def require_selected_review(raw, record, index, unicode):
    marker = b'[DEVICE-LOCAL SELECTED TEXT]'
    if (raw.count(marker) != 1 or raw.count(CONFIRM.strip(b'\r\n')) != 1
            or not raw.endswith(CONFIRM) or b'HOME account password' in raw):
        raise ValueError('one complete selected review before fresh confirmation required')
    metadata, body = raw.split(marker)
    panes = [row[2:].decode() for row in metadata.split(b'\r\n') if row.startswith(b'| ')]
    starts = [i for i, row in enumerate(panes) if row == 'Send selected text only.']
    if len(starts) != 1: raise ValueError('exact selected send review marker missing')
    expected = ('Send selected text only.'+'Retained record: '+str(index)+'Operation:'+
                record['operation']+'Subject: '+displayed(record['subject'], unicode))
    if ''.join(panes[starts[0]:]) != expected or record['direction'] != 'sent':
        raise ValueError('selected canonical record, operation or full subject differs')
    pages = [tuple(map(int, match)) for match in
             re.findall(rb'\[LOCAL BRIEF REVIEW\] Page ([0-9]+)/([0-9]+)', metadata)]
    if not pages or pages != [(number, pages[-1][1]) for number in range(1, pages[-1][1]+1)]:
        raise ValueError('selected metadata review pages incomplete')
    for row in metadata.split(b'\r\n'):
        if row.startswith(b'| '): safe_rows(row)
    require_body_review(body, record['body'], unicode)


def compact(text, unicode, cap):
    result = ''
    for char in text:
        scalar = displayed(char, unicode)
        if len(result)+len(scalar) > cap:
            return result[:cap-1]+'~' if len(result) >= cap else result+'~'
        result += scalar
    return result


def require_arrival_screen(raw, index, status, subject, unicode):
    rows = safe_rows(raw)
    expected = [ARRIVALS.decode(), 'Page 1/1; selection 1/1', f'> {index:02} {status}',
                compact(subject, unicode, 39), '[Up/Down J/K] Select  [N/P] Page',
                '[Enter/O] Open full record', '[B] Return', '']
    if rows != expected:
        raise ValueError('exact result-local canonical arrival selection required')


def require_local_read(before, after, index, frame):
    expected = copy.deepcopy(before)
    try: row = expected['history'][index]
    except (KeyError, IndexError, TypeError): raise ValueError('missing canonical read target') from None
    if (row['direction'] != 'received' or row['received_id'] != frame
            or row['read'] is not False or not re.fullmatch('[a-f0-9]{64}', frame)
            or frame == '00'*32):
        raise ValueError('fresh authenticated unread target required')
    row['read'] = True
    if after != expected: raise ValueError('read decision changed unrelated canonical state')


def require_arrival_interval(before_counts, after_counts, before, after, index, frame):
    require_cancelled_confirmation(before_counts, after_counts, None, None)
    require_local_read(before, after, index, frame)


class ActorLifecycle:
    """Record successful starts and bound the normal finish and failure cleanup."""
    def __init__(self, factory):
        self.factory = factory
        self.actors = {}
        self.finished = set()
        self.stopped = False

    def add(self, index, console):
        if index in self.actors: raise ValueError('actor already started')
        self.actors[index] = console

    def homes_stopped(self, closed, context):
        if self.stopped or closed != [True, True]:
            raise ValueError('one callback after both homes stop required')
        self.stopped = True
        self.factory.after_homes_stopped(context)

    def finish(self):
        failures = []
        for index, console in self.actors.items():
            if index not in self.finished:
                self.finished.add(index)
                try: self.factory.finish(index, console)
                except BaseException as error: failures.append(type(error).__name__)
        if failures: raise ValueError('actor finish failed: '+','.join(failures))

    def close(self):
        failures = []
        for console in self.actors.values():
            try: console.close()
            except BaseException as error: failures.append(type(error).__name__)
        if failures: raise ValueError('actor close failed: '+','.join(failures))


class LocalActors:
    def start(self, index, root, binaries, member, unicode):
        from smoke_native_client_ssh import NativeConsole
        console = NativeConsole([str(binaries['client']), '--mail-online',
            str(member['store']), str(member['config'])]+(['--unicode'] if unicode else []))
        try:
            console.resize(40, 24)
            console.password(member['local_password'], 'LOCAL checkpoint passphrase (hidden): ')
            console.expect(MENU)
            return console
        except BaseException:
            console.close()
            raise

    def after_homes_stopped(self, context):
        pass

    def finish(self, index, console):
        console.raw(b'l')
        console.finish()


def require_queued_screen(raw, candidates, selected, unicode):
    rows = safe_rows(raw)
    indices = [index for index, _ in candidates]
    if indices != sorted(set(indices)) or selected not in indices:
        raise ValueError('exact canonical queued candidate order required')
    position = indices.index(selected)
    start = position//2*2
    expected = ['[LOCAL QUEUED TEXT]',
        f'Page {position//2+1}/{(len(candidates)+1)//2}; selection {position+1}/{len(candidates)}']
    expected += [f'{">" if index == selected else " "} {index:02} '+compact(subject, unicode, 34)
                 for index, subject in candidates[start:start+2]]
    expected += ['[Up/Down J/K] Select  [N/P] Page', '[Enter/O] Review selected text', '[B] Return', '']
    if rows != expected: raise ValueError('queued selection differs from exact canonical candidates')


def current_peer_review(console, own, peer, device, identity):
    console.raw(b'e'); console.expect('[LOCAL BRIEF EXCHANGE]')
    raw = read_review(console, b'\r\n[B] Return\r\n')
    # This unchanged trusted footer is exactly 40 ASCII cells. It does not
    # enlarge the 39-cell bound of peer panes, bodies or any other output row.
    footer = b'At most 10 seconds; idle wait 2 seconds.'
    rows = raw.split(b'\r\n')
    if rows.count(footer) != 1: raise ValueError('one exact immutable brief footer required')
    for row in rows:
        if row != footer: safe_rows(row)
    panes = b''.join(row[2:] for row in raw.split(b'\r\n') if row.startswith(b'| '))
    expected = ('Home: '+own+'Peer: '+peer+'Peer device: '+str(device)+'Current identity:'+identity).encode()
    if panes != expected: raise ValueError('reviewed current peer context differs')
    if b'[Q] Choose queued text' not in raw: raise ValueError('queued route unavailable')
    return raw


def arm_selected(console, own, peer, device, identity, state, selected, unicode, cancel=None):
    current_peer_review(console, own, peer, device, identity)
    pending = {row['operation'] for row in state['pending']}
    candidates = [(index+1, row['subject']) for index, row in enumerate(state['history'])
                  if row['direction'] == 'sent' and row['operation'] in pending]
    if not candidates or selected not in [index for index, _ in candidates]:
        raise ValueError('selected canonical pending operation missing')
    console.raw(b'q')
    for index, _ in candidates:
        _, raw = expect_one(console, [b'\r\n[B] Return\r\n'])
        raw = raw[raw.index(b'[LOCAL QUEUED TEXT]'):]
        require_queued_screen(raw, candidates, index, unicode)
        if cancel == 'picker':
            console.raw(b'b'); console.expect('[B] Return'); console.raw(b'b')
            return raw
        if index == selected: break
        console.raw(b'j')
    console.raw(b'o')
    if cancel == 'body':
        pieces = []
        for _ in range(8):
            token, raw = expect_one(console, [NEXT, CONFIRM]); pieces.append(raw)
            combined = b''.join(pieces)
            if token == CONFIRM: raise ValueError('multiline review did not offer body cancellation')
            if b'[DEVICE-LOCAL SELECTED TEXT]' in combined:
                if b'[LOCAL BODY] Page 1/' not in combined or b'HOME account password' in combined:
                    raise ValueError('body cancellation skipped the selected body')
                console.raw(b'b')
                return combined
            console.send('')
        raise ValueError('bounded selected metadata review did not reach body')
    raw = read_review(console, CONFIRM)
    require_selected_review(raw, state['history'][selected-1], selected, unicode)
    return raw


def read_body(console):
    pieces = []
    for _ in range(8):
        token, raw = expect_one(console, [NEXT, ACK]); pieces.append(raw)
        if token == ACK: return b''.join(pieces)
        console.send('')
    raise ValueError('bounded ordinary body did not reach its fresh read decision')


def write_text(console, peer, device, identity, subject, body, unicode):
    console.raw(b'w')
    _, raw = expect_one(console, [b'Subject (128 UTF-8 bytes): '])
    recipient = ('CURRENT recipient: '+peer+'\r\n').encode()
    current = (f'CURRENT device: {device} / ed25519:'+identity+'\r\n').encode()
    if recipient not in raw or current not in raw: raise ValueError('writer current recipient differs')
    console.send(subject); console.expect('write> ')
    for line in body.split('\n'):
        console.send('.'+line if line.startswith('.') else line)
        console.expect('Accepted:'); console.expect('write> ')
    console.send('.done')
    pieces = []
    for _ in range(8):
        token, raw = expect_one(console, [NEXT, b'Type queue to persist encrypted ciphertext: '])
        pieces.append(raw)
        if token != NEXT: break
        console.send('')
    else: raise ValueError('bounded writer review did not finish')
    raw = b''.join(pieces)
    if (recipient not in raw or current not in raw
            or ('Subject: '+displayed(subject, unicode)+'\r\n').encode() not in raw):
        raise ValueError('writer final current recipient or full subject differs')
    require_body_review(raw, body, unicode)
    console.send('queue'); console.expect('Queued durably;')


def run_journey(arguments, actor_factory=None):
    """One classic OR Unicode journey. No retries, VM setup or external service."""
    # Load all qualified runtime helpers before binding their executed source.
    import probe_home_mail
    import probe_xmpp_transport
    import smoke_native_client_ssh
    from probe_visit_mail import require_closed_endpoint
    parent = owned_root(arguments.owned_root)
    root = Path(tempfile.mkdtemp(prefix='brief-review-OWNED-', dir=parent)); root.chmod(0o700)
    (root/'OWNED').write_bytes(MARKER)

    def save(name, value):
        path = root/name
        path.write_bytes(value if isinstance(value, bytes) else
            (json.dumps(value, ensure_ascii=False, sort_keys=True, indent=2)+'\n').encode())
        path.chmod(0o600)
        return path

    token = 'seed-'+secrets.token_hex(5)
    local_password = 'checkpoint synthetic fixture password'
    passwords = [secrets.token_hex(20), secrets.token_hex(20)]
    seed = (token, 'private seed-'+secrets.token_hex(5))
    subject = 'twins-λ\u202e-'+secrets.token_hex(4)+'-'+'subject-'*8
    first = (subject, '\n'.join('other-'+secrets.token_hex(3)+f' {number:02} λ\u202e' for number in range(15)))
    second = (subject, '\n'.join('picked-'+secrets.token_hex(3)+f' {number:02} λ\u202e' for number in range(15)))
    inputs = save('inputs.private.json', dict(local_password=local_password, home_passwords=passwords,
        query=token, seed=seed, first=first, second=second, unicode=arguments.unicode))
    binary_root = root/'binaries'; binary_root.mkdir(mode=0o700)
    objects = root/'source-objects'; objects.mkdir(mode=0o700)
    binaries = {}; binary_bindings = {}; sources = {}
    for name in ('client', 'mail_fixture', 'inspector'):
        original = Path(getattr(arguments, name)).resolve(); raw = original.read_bytes()
        copied = binary_root/name; copied.write_bytes(raw); copied.chmod(0o700)
        binaries[name] = copied
        binary_bindings[name] = dict(original=str(original), executed=str(copied), sha256=hashlib.sha256(raw).hexdigest())
    for name, module in tuple(sys.modules.items()):
        location = getattr(module, '__file__', None)
        if location is None: continue
        path = Path(location).resolve()
        if path.parent != Path(__file__).resolve().parent or path.suffix != '.py': continue
        raw = path.read_bytes(); digest = hashlib.sha256(raw).hexdigest(); copied = objects/digest
        if not copied.exists(): copied.write_bytes(raw); copied.chmod(0o600)
        sources[name] = dict(path=str(path), object=str(copied), sha256=digest)
    save('bindings.private.json', dict(binaries=binary_bindings, sources=sources,
        inputs_sha256=hashlib.sha256(inputs.read_bytes()).hexdigest(), python=sys.version))
    factory = actor_factory if actor_factory is not None else LocalActors()
    actors = ActorLifecycle(factory); relay = ReleaseRelay(root); homes = []; members = []
    failures = []; facts = None

    def call(index, action, *args, password=False, lines=()):
        return native_command(root, binaries['mail_fixture'], action, members[index],
                              *args, home_password=password, lines=lines)

    def snapshot(index, label):
        state = canonical_snapshot(root, members[index]['store'], binaries['inspector'], binaries['mail_fixture'])
        save(label+f'-{index}.state.private.json', state)
        save(label+f'-{index}.canonical.private', (members[index]['store']/'mail.fkv').read_bytes())
        return state

    def counts(): return tuple((home.connections, home.authenticated) for home in homes)

    def screen(console):
        _, raw = expect_one(console, [b'\r\n[W] Write fresh text  [B] Back\r\n'])
        return raw[raw.rindex(CONVERSATION.encode()):]

    def conversation(console, unread):
        for key, title in ((b'i', '[LOCAL INBOX]'), (b'c', '[LOCAL CORRESPONDENTS]'),
                           (b'o', '[LOCAL CORRESPONDENT INBOX]')):
            console.raw(key); console.expect(title); console.expect('[B] Back')
        if unread:
            console.raw(b'u'); console.expect('[LOCAL CORRESPONDENT INBOX]'); console.expect('[B] Back')
        console.raw(b's'); console.expect('Search (64 bytes; blank clears): '); console.send(token)
        console.expect('Local search active.'); console.expect('[B] Back'); console.raw(b't')
        return screen(console)

    def unchanged(index, label, before_counts, before_state, original, returned):
        after = snapshot(index, label)
        require_cancelled_confirmation(before_counts, counts(), before_state, after)
        if returned != original: raise ValueError('cancel changed exact original filtered view')
        return after

    def confirm(console):
        index = next(index for index, value in actors.actors.items() if value is console)
        console.send('exchange'); console.password(passwords[index], HOME)
        console.expect('Connecting directly to your home.')

    def result(console, write, accepted):
        _, raw = expect_one(console, [b'Enter to return: '])
        if (write.encode() not in raw or f'Accepted on this device: {accepted}.'.encode() not in raw
                or b'Duplicates: 0.' not in raw or (RESULT in raw) != bool(accepted)
                or ARRIVALS in raw):
            raise ValueError('explicit result count, uncertainty or arrival choice differs')
        return raw

    def return_menu(console):
        for title in ('[LOCAL CORRESPONDENT INBOX]', '[LOCAL CORRESPONDENTS]', '[LOCAL INBOX]'):
            console.raw(b'b'); console.expect(title); console.expect('[B] Back')
        console.raw(b'b'); console.expect(MENU)

    def arrival_screen(console, index, status):
        _, raw = expect_one(console, [b'\r\n[B] Return\r\n'])
        raw = raw[raw.index(ARRIVALS):]
        require_arrival_screen(raw, index, status, subject, arguments.unicode)
        return raw

    try:
        identities = []
        for index, domain in enumerate(('home.test', 'away.test')):
            tls = root/domain; tls.mkdir(mode=0o700)
            pair, ca = certificates(tls, domain, arguments.openssl)
            home = LiveHome(root, index, relay, pair, passwords[index]); homes.append(home)
            config = root/f'home-{index}.toml'; der = ssl.PEM_cert_to_DER_cert(pair[0].read_text())
            values = dict(version=1, account=ACCOUNTS[index], endpoint='127.0.0.1:'+str(home.port),
                domain=domain, ca=str(ca), pin=hashlib.sha256(der).hexdigest())
            config.write_text(''.join(key+'='+json.dumps(value)+'\n' for key, value in values.items())); config.chmod(0o600)
            members.append(dict(store=root/('alice' if index == 0 else 'bob'), config=config,
                local_password=local_password, password=passwords[index]))
            identity = call(index, 'init', ACCOUNTS[index], DEVICES[index]).decode().strip()
            if not re.fullmatch('[a-f0-9]{64}', identity): raise ValueError('native identity unavailable')
            identities.append(identity)
        if identities[0] == identities[1]: raise ValueError('independent device identities required')
        for index in (0, 1):
            call(index, 'enroll', ACCOUNTS[1-index], DEVICES[1-index], identities[1-index])
            call(index, 'enable-archive')
        call(1, 'publish', members[1]['config'], password=True)
        call(0, 'prepare', members[0]['config'], password=True)
        call(0, 'queue', lines=seed); call(0, 'sync', members[0]['config'], password=True)
        for index, expected in ((1, (1, 0)), (0, (0, 1))):
            page = json.loads(call(index, 'catchup', members[index]['config'], password=True))
            if page != dict(entries=1, accepted=expected[0], duplicates=expected[1], complete=True):
                raise ValueError('qualified warm seed archive differs')
        for index in (0, 1):
            console = factory.start(index, root, binaries, members[index], arguments.unicode)
            actors.add(index, console)
            console.raw(b'e'); console.expect('Type enable to add encrypted local read tracking: ')
            console.send('enable'); console.expect('Enter to return: '); console.send(''); console.expect(MENU)
        sender, receiver = actors.actors[0], actors.actors[1]
        originals = [conversation(sender, False), conversation(receiver, True)]
        for index, mode in ((0, 'All'), (1, 'Unread')):
            require_composing_screen(originals[index], [1], 1, (1, 1), mode, True,
                                     DEVICES[1-index], evidence_root=root)
        sender_state = snapshot(0, 'sender-seed')
        for label, content in (('first', first), ('second', second)):
            before_counts = counts()
            write_text(sender, ACCOUNTS[1], DEVICES[1], identities[1], *content, arguments.unicode)
            returned = screen(sender); after = snapshot(0, label+'-queued')
            require_reply_commit(sender_state, after, ACCOUNTS[1], DEVICES[1], *content)
            if after['history'][-1]['identity'] != identities[1] or returned != originals[0] or counts() != before_counts:
                raise ValueError('local twin queue changed context, view or connected')
            sender_state = after
        selected = len(sender_state['history']); operation = sender_state['history'][-1]['operation']
        sender_bytes = (members[0]['store']/'mail.fkv').read_bytes()
        wires = capture_pending_wires(root, members[0]['store'], binaries['inspector'], binaries['mail_fixture'])
        save('pending-wires.private.json', {key: value.hex() for key, value in wires.items()})
        for cancel in ('picker', 'body', 'confirmation'):
            before_counts = counts()
            raw = arm_selected(sender, ACCOUNTS[0], ACCOUNTS[1], DEVICES[1], identities[1],
                sender_state, selected, arguments.unicode, cancel if cancel != 'confirmation' else None)
            save('selected-'+cancel+'.private', raw)
            if cancel == 'confirmation': sender.send('')
            unchanged(0, cancel+'-cancel', before_counts, sender_state, originals[0], screen(sender))
            if (members[0]['store']/'mail.fkv').read_bytes() != sender_bytes:
                raise ValueError('cancel changed encrypted pending/read/cursor bytes')
        receiver_before = snapshot(1, 'receiver-before')
        arm_brief(receiver, ACCOUNTS[1], ACCOUNTS[0], DEVICES[0], identities[0])
        save('selected-complete.private', arm_selected(sender, ACCOUNTS[0], ACCOUNTS[1], DEVICES[1],
            identities[1], sender_state, selected, arguments.unicode))
        relay.expected_release = (0, operation, wires[operation])
        before_counts = counts(); messages = len(relay.messages); queries = list(relay.queries)
        generation = connect_pair(receiver, sender, relay, confirm)
        result(sender, 'Selected text local write completed.', 0); sender.send('')
        if screen(sender) != originals[0]: raise ValueError('selected send changed original sender view')
        save('receiver-result.private', result(receiver, 'No selected text write attempted.', 1))
        receiver_after = snapshot(1, 'receiver-accepted')
        require_live(receiver_before, receiver_after, *second)
        if ((members[0]['store']/'mail.fkv').read_bytes() != sender_bytes
                or relay.queries != queries or relay.messages[messages:] != [dict(actor=0,
                    operation=operation, to=ACCOUNTS[1]+'/faction202', bound_generation=generation)]
                or any(after != (before[0]+1, before[1]+1) for before, after in zip(before_counts, counts()))):
            raise ValueError('exact selected ciphertext, connection count or manual-only archive changed')
        require_cancelled_confirmation(None, None, sender_state, snapshot(0, 'sender-after'))
        new_index = len(receiver_after['history']); frame = receiver_after['history'][-1]['received_id']
        arrival_bytes = (members[1]['store']/'mail.fkv').read_bytes()
        review_counts = counts()
        receiver.raw(b'r'); save('arrival-list.private', arrival_screen(receiver, new_index, 'UNREAD'))
        receiver.raw(b'o'); _, partial = expect_one(receiver, [NEXT])
        if ACK in partial: raise ValueError('partial body offered acknowledgement')
        receiver.raw(b'b'); arrival_screen(receiver, new_index, 'UNREAD')
        if snapshot(1, 'body-cancel') != receiver_after: raise ValueError('body cancellation acknowledged')
        receiver.raw(b'o'); full = read_body(receiver)
        require_body_review(full, second[1], arguments.unicode); save('arrival-skip-review.private', full)
        receiver.send(''); receiver.expect('Enter to return: '); receiver.send('')
        arrival_screen(receiver, new_index, 'UNREAD')
        if (snapshot(1, 'read-skip') != receiver_after
                or (members[1]['store']/'mail.fkv').read_bytes() != arrival_bytes):
            raise ValueError('listing, paging or skipped read changed canonical decisions')
        receiver.raw(b'o'); full = read_body(receiver); require_body_review(full, second[1], arguments.unicode)
        receiver.send('read'); receiver.expect('Acknowledged on this device only; no read receipt sent.')
        receiver.expect('Enter to return: '); receiver.send(''); arrival_screen(receiver, new_index, 'READ')
        acknowledged = snapshot(1, 'acknowledged'); require_local_read(receiver_after, acknowledged, new_index-1, frame)
        receiver.raw(b'b'); receiver.expect('Enter to return: '); receiver.send('')
        returned = screen(receiver)
        require_composing_screen(returned, [1], 1, (1, 1), 'Unread', True, DEVICES[0], evidence_root=root)
        if returned != originals[1]: raise ValueError('arrival review changed exact original search/filter/selection')
        save('arrival-review.interval.private.json', dict(before_counts=review_counts, after_counts=counts(),
            before_state=receiver_after, after_state=acknowledged, index=new_index-1, received_id=frame,
            before_view_hex=originals[1].hex(), after_view_hex=returned.hex()))
        require_arrival_interval(review_counts, counts(), receiver_after, acknowledged, new_index-1, frame)
        for home in homes: home.close(); require_closed_endpoint(('127.0.0.1', home.port))
        before_counts = counts()
        arm_brief(receiver, ACCOUNTS[1], ACCOUNTS[0], DEVICES[0], identities[0]); confirm(receiver)
        outage = result(receiver, 'No selected text write attempted.', 0)
        if b'Brief exchange stopped.' not in outage: raise ValueError('home outage was not reported honestly')
        receiver.send('')
        unchanged(1, 'home-outage', before_counts, acknowledged, originals[1], screen(receiver))
        for console in actors.actors.values(): return_menu(console)
        context = dict(root=root, binaries=binaries, members=members, actors=actors.actors,
                       snapshot=snapshot, counts=counts, save=save, arguments=arguments)
        actors.homes_stopped([home.closed for home in homes], context)
        actors.finish()
        credentials = [local_password, *passwords]
        markers = credentials+[token, *(value for content in (seed, first, second) for value in content)]
        if any(value.encode() in console.transcript for value in credentials for console in actors.actors.values()):
            raise ValueError('hidden credential reached a terminal')
        server = b''.join(path.read_bytes() for path in root.glob('home-*.private'))
        server += b''.join(frame for actor in (0, 1) for _, frame in relay.inventory(actor))
        if any(value.encode() in server for value in markers): raise ValueError('private plaintext reached home capture')
        if any(value.encode() in (member['store']/'mail.fkv').read_bytes() for value in markers for member in members):
            raise ValueError('private marker found in encrypted canonical')
        for binding in sources.values():
            if hashlib.sha256(Path(binding['path']).read_bytes()).hexdigest() != binding['sha256']:
                raise ValueError('executed proof helper changed during journey')
        facts = dict(result='pass', unicode=arguments.unicode, selected_writes=1, live_accepted=1,
            complete_selected_body=True, same_subject_twins=True, cancellation_no_connection=True,
            explicit_hidden_arrival_review=True, fresh_local_read_only=True, exact_original_view=True,
            pending_and_cursor_preserved=True, server_plaintext_matches=0,
            tls_versions=[home.tls_versions[-1] for home in homes], netbsd_qualified=False,
            production_s2s=False, independent_interoperability=False)
    except BaseException:
        import traceback
        save('failure.private.txt', traceback.format_exc().encode())
        raise
    finally:
        try: actors.close()
        except BaseException as error: failures.append(type(error).__name__)
        for index, console in actors.actors.items(): save(f'terminal-{index}.private', console.transcript+console.buffer)
        refused = []
        for home in homes:
            try: home.close(); require_closed_endpoint(('127.0.0.1', home.port)); refused.append(True)
            except BaseException as error: failures.append(type(error).__name__); refused.append(False)
            if home.failure is not None: failures.append(home.failure)
        teardown = dict(workers_joined=all(not home.thread.is_alive() for home in homes),
            listeners_closed=all(home.closed for home in homes), started_actors=len(actors.actors),
            actors_finished=len(actors.finished), child_processes_reaped=not failures,
            closed_ports_refused=refused, failures=failures)
        save('teardown.private.json', teardown)
        save('relay.private.json', dict(messages=relay.messages, queries=relay.queries,
            archives=[[(ident, wire.hex()) for ident, wire in relay.inventory(actor)] for actor in (0, 1)]))
    if failures or not all(teardown[key] for key in ('workers_joined', 'listeners_closed', 'child_processes_reaped')):
        raise ValueError('owned brief-review teardown or protocol failed')
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
    facts = run_journey(parser.parse_args())
    print('Brief review synthetic TLS journey passed:', 'Unicode' if facts['unicode'] else 'classic')


if __name__ == '__main__':
    try: main()
    except (ValueError, OSError, AssertionError, TimeoutError, subprocess.SubprocessError):
        raise SystemExit('Brief review fixture failed; private diagnostics retained.') from None
