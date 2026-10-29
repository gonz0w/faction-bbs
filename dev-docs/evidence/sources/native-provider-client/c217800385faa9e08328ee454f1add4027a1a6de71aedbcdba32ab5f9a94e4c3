#!/usr/bin/env python3
"""Owned #280 production-console proof; private credentials/captures never exported."""
import os
from pathlib import Path
import re
import secrets
import stat


def parse_ready(raw):
    match = re.fullmatch(rb'ready ([1-9][0-9]{0,4})\n', raw)
    if match is None or int(match[1]) > 65535:
        raise ValueError('invalid bounded provider readiness')
    return int(match[1])


def retain_capture(root, label, data):
    if not re.fullmatch(r'[a-z][a-z0-9-]{0,63}', label) or len(data) > 4*1024*1024:
        raise ValueError('private capture bound')
    root = Path(root)
    info = root.lstat()
    if root.is_symlink() or not stat.S_ISDIR(info.st_mode) or info.st_uid != os.getuid() or stat.S_IMODE(info.st_mode) != 0o700:
        raise ValueError('private capture root required')
    path = root/(label+'-'+secrets.token_hex(12)+'.private')
    fd = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
    with os.fdopen(fd, 'wb') as output:
        output.write(data)
    return path

import argparse
import errno
import signal
import base64
import copy
import hashlib
import json
import select
import shutil
import socket
import ssl
import subprocess
import tempfile
import time
from xml.etree import ElementTree as ET
from probe_attachment_transfer import HomePeer, _settings
from probe_xmpp_transport import certificates
from smoke_board_launcher import LauncherConsole

NS = 'jabber:client'
PUBSUB = 'http://jabber.org/protocol/pubsub'
OMEMO = 'urn:xmpp:omemo:2'
MAM = 'urn:xmpp:mam:2'
RSM = 'http://jabber.org/protocol/rsm'
UNLOCK = 'LOCAL checkpoint passphrase (hidden): '
MENU = '[L] Lock and exit'
DESK = '[R] Received attachment  [B] Back'
RECEIVED = '[S] Save verified file  [B] Back'
HOME = 'OWN HOME account password (hidden): '
ONLINE_HOME = 'HOME account password (hidden; never sent to a BBS): '
TRANSFER = 'Type transfer to connect to YOUR home: '


def private(path, data):
    with path.open('wb') as output:
        path.chmod(0o600)
        output.write(data)


def command(argv, *, secret=None, timeout=30):
    result = subprocess.run(list(map(str, argv)), input=secret, capture_output=True,
                            timeout=timeout)
    if result.returncode:
        raise ValueError('owned fixture command failed')
    return result.stdout


def wait_fact(predicate, seconds=5):
    deadline = time.monotonic()+seconds
    while not predicate():
        if time.monotonic() >= deadline:
            raise ValueError('bounded fixture observation unavailable')
        time.sleep(.01)


# These are observation limits from the existing production console contract.
# Generic Console.expect remains unchanged for every ordinary prompt/menu.
_TRANSFER_GAP = 15.0
_TRANSFER_WORK = 60.0
_TRANSFER_DRAIN = 5.0
_TRANSFER_OUTPUT = 16_384
_TRANSCRIPT_CAP = 4*1024*1024
_TRANSFER_COMPLETE = b'Foreground transfer complete.\r\n'
_TRANSFER_PROGRESS = re.compile(
    rb'Confirmed ciphertext bytes: ([0-9]{1,7})\r\n'
    rb'SDK call: ([0-9]{1,3}) / 65; writes: ([0-9]{1,3})\r\n')
_TRANSFER_MARKERS = [
    (b'Foreground time budget reached.', 'budget'),
    (b'Transfer stopped; no automatic retry.', 'transfer_stop'),
    (b'Stopped: progress or call bound.', 'progress_bound'),
    (b'Home mail locked:', 'locked'),
    (_TRANSFER_COMPLETE, 'completion'),
]


class TransferObservationError(ValueError):
    def __init__(self, observation):
        super().__init__('bounded transfer observation failed: '+observation['reason'])
        self.observation = observation


def _transfer_marker(buffer):
    found = [(buffer.find(text), name) for text, name in _TRANSFER_MARKERS
             if text in buffer]
    return min(found) if found else None


def _append_transfer_output(console, data):
    transcript_room = max(0, _TRANSCRIPT_CAP-len(console.transcript))
    buffer_room = max(0, _TRANSFER_OUTPUT-len(console.buffer))
    console.transcript += data[:transcript_room]
    console.buffer += data[:buffer_room]
    return len(data) <= min(transcript_room, buffer_room)


def _cancel_transfer(console, *, observe_stop=False, clock=None, poll=None,
                     read=None, reap=None, kill=None):
    """Failure-only grace, never extra work or a late success opportunity.

    At the work deadline first observe the producer's own outcome for at most
    four seconds. Record it before our cancellation; absent that evidence the
    timeout remains an observation failure, not a proved producer failure.
    """
    clock = time.monotonic if clock is None else clock
    poll = select.select if poll is None else poll
    read = os.read if read is None else read
    reap = os.waitpid if reap is None else reap
    kill = os.kill if kill is None else kill
    began = clock()
    deadline = began+_TRANSFER_DRAIN
    cancel_at = None if observe_stop else began
    killed = False
    marker_before_cancel = None
    cancelled = False
    result = dict(reaped=False, killed=False, marker_before_cancel=None,
                  cancelled=False, seconds=0.0)
    while True:
        now = clock()
        if now >= deadline:
            break
        if not cancelled:
            marker = _transfer_marker(console.buffer)
            if marker is not None:
                marker_before_cancel = marker[1]
            if observe_stop and (marker is not None or now >= began+4):
                cancel_at = now
            if cancel_at is not None:
                try:console.raw(b'\x03')
                except OSError:pass
                cancelled = True
        # Immediate cancellation gets four seconds to cooperate; after passive
        # observation reserve part of the final second for kill and reap.
        kill_at = (min(cancel_at+0.8, deadline-0.2) if observe_stop and cancelled
                   else began+4)
        if cancelled and not killed and now >= kill_at:
            try:kill(console.pid, signal.SIGKILL)
            except ProcessLookupError:pass
            killed = True
        try:
            pid, _ = reap(console.pid, os.WNOHANG)
        except ChildProcessError:
            pid = console.pid
        if pid:
            console.done = True
            result['reaped'] = True
            break
        next_boundary = (kill_at if cancelled and not killed else
                         began+4 if not cancelled else deadline)
        timeout = min(0.05, deadline-now, max(0.0, next_boundary-now))
        try:
            ready, _, _ = poll([console.fd], [], [], timeout)
            if ready:
                try:data = read(console.fd, 8192)
                except OSError as error:
                    if error.errno != errno.EIO:raise
                    data = b''
                if data:_append_transfer_output(console, data)
        except OSError:
            # Still reach the fixed kill/reap boundaries after a broken PTY.
            time.sleep(min(0.01, max(0.0, deadline-clock())))
    result.update(killed=killed, marker_before_cancel=marker_before_cancel,
                  cancelled=cancelled, seconds=round(clock()-began, 6))
    return result


def wait_transfer(console, total, *, started=None, clock=None, poll=None,
                  read=None, cancel=None):
    """Observe confirmed foreground progress within the unchanged 60 seconds.

    Only a whole validated progress record renews the 15-second gap. Noise,
    fragments, duplicates and later output never extend the absolute deadline.
    Keep trailing menu bytes available to the ordinary Console.expect caller.
    """
    if type(total) is not int or not 1 <= total <= 1_048_592:
        raise ValueError('invalid expected transfer size')
    clock = time.monotonic if clock is None else clock
    poll = select.select if poll is None else poll
    read = os.read if read is None else read
    started = clock() if started is None else started
    deadline = started+_TRANSFER_WORK
    last_valid = started
    records = []
    offset = 0
    seen = len(console.buffer)

    def observation(reason, now):
        return dict(reason=reason, calls=len(records), offset=offset,
                    seconds=round(now-started, 6), progress=records)

    def fail(reason):
        result = observation(reason, clock())
        console.transfer_observation = result
        result['drain'] = (cancel(console) if cancel is not None else
            _cancel_transfer(console, observe_stop=reason == 'work_deadline',
                             clock=clock, poll=poll, read=read))
        marker = result['drain'].get('marker_before_cancel')
        result['classification'] = (
            'product_stop_observed' if marker in {'budget','transfer_stop','progress_bound'}
            else 'late_completion_observed' if marker == 'completion'
            else 'observer_timeout_ambiguous' if reason == 'work_deadline'
            else 'observer_rejection')
        raise TransferObservationError(result)

    if seen > _TRANSFER_OUTPUT or len(console.transcript) > _TRANSCRIPT_CAP:
        fail('output_bound')
    while True:
        now = clock()
        # Work has no success grace, including bytes returned exactly at 60s.
        if now >= deadline:fail('work_deadline')
        if now > last_valid+_TRANSFER_GAP:fail('idle_gap')
        progress = _TRANSFER_PROGRESS.search(console.buffer)
        marker = _transfer_marker(console.buffer)
        # Parsing cannot turn a timestamp sampled before a deadline into a
        # successful record/completion after that deadline.
        now = clock()
        if now >= deadline:fail('work_deadline')
        if now > last_valid+_TRANSFER_GAP:fail('idle_gap')
        if marker is not None and (progress is None or marker[0] < progress.start()):
            if marker[1] != 'completion':fail('product_stop')
            if not records or offset != total:fail('early_completion')
            console.buffer = console.buffer[marker[0]+len(_TRANSFER_COMPLETE):]
            result = observation('complete', now)
            console.transfer_observation = result
            return result
        if progress is not None:
            values = tuple(map(int, progress.groups()))
            confirmed, call, writes = values
            if (any(str(value).encode() != raw for value, raw in zip(values, progress.groups()))
                    or call != len(records)+1 or call > 65
                    or not offset < confirmed <= total or writes not in (0,1)
                    or writes and confirmed != total):
                fail('invalid_progress')
            offset = confirmed
            last_valid = now
            records.append(dict(call=call, offset=confirmed, writes=writes,
                                seconds=round(now-started, 6)))
            console.buffer = console.buffer[progress.end():]
            continue
        if now >= last_valid+_TRANSFER_GAP:fail('idle_gap')
        remaining = min(deadline-now, last_valid+_TRANSFER_GAP-now)
        try:
            ready, _, _ = poll([console.fd], [], [], remaining)
            if not ready:continue
            try:data = read(console.fd, 8192)
            except OSError as error:
                if error.errno != errno.EIO:raise
                data = b''
        except OSError:fail('io_error')
        if not data:fail('eof')
        seen += len(data)
        if not _append_transfer_output(console, data) or seen > _TRANSFER_OUTPUT:
            fail('output_bound')


class PublicMail:
    """Ordinary account-authenticated relay; public bundles and opaque mail only.

    No private store API, transport credential or member-vault operation exists.
    The inherited HomePeer authenticates each ordinary XMPP account separately.
    """
    def __init__(self, root):
        self.root = root
        self.blob_requests = 0
        self.archive_pages = 0
        self.requests = 0

    def request(self, actor, raw):
        request = ET.fromstring(raw)
        self.requests += 1
        if request.find('{urn:faction:blob:1}blob') is not None:
            self.blob_requests += 1
            raise ValueError('device-provider attachment fallback attempted')
        own = ['alice@home.test', 'bob@home.test'][actor]
        reply = ET.Element('{'+NS+'}iq', {'type': 'result', 'id': request.attrib['id'],
                                         'from': own, 'to': own})
        query = request.find('{'+MAM+'}query')
        if query is not None:
            self.archive_pages += 1
            after = query.find('{'+RSM+'}set/{'+RSM+'}after')
            available = (self.root/'delivery.xml').exists() and after is None
            pieces = []
            if available:
                outer = ET.Element('{'+NS+'}message', {'from': own, 'to': own})
                result = ET.SubElement(outer, '{'+MAM+'}result',
                                       {'id': 'attachment-1', 'queryid': query.attrib['queryid']})
                forwarded = ET.SubElement(result, '{urn:xmpp:forward:0}forwarded')
                ET.SubElement(forwarded, '{urn:xmpp:delay}delay', {'stamp': '2026-10-29T00:00:00Z'})
                forwarded.append(ET.fromstring((self.root/'delivery.xml').read_bytes()))
                pieces.append(ET.tostring(outer))
            fin = ET.SubElement(reply, '{'+MAM+'}fin', {'complete': 'true'})
            page = ET.SubElement(fin, '{'+RSM+'}set')
            if available:
                ET.SubElement(page, '{'+RSM+'}first').text = 'attachment-1'
                ET.SubElement(page, '{'+RSM+'}last').text = 'attachment-1'
            pieces.append(ET.tostring(reply))
            response = b''.join(pieces)
        else:
            pubsub = request.find('{'+PUBSUB+'}pubsub')
            if pubsub is None:
                raise ValueError('unsupported ordinary mail fixture IQ')
            items = pubsub.find('{'+PUBSUB+'}items')
            if items is None:
                reply.set('from', 'home.test')
            else:
                account = request.get('to', own)
                if account not in ['alice@home.test', 'bob@home.test']:
                    raise ValueError('unknown ordinary mail account')
                reply.set('from', account)
                node = items.attrib['node']
                result_items = ET.SubElement(ET.SubElement(reply, '{'+PUBSUB+'}pubsub'),
                                             '{'+PUBSUB+'}items', {'node': node})
                if node == OMEMO+':devices':
                    item = ET.SubElement(result_items, '{'+PUBSUB+'}item', {'id': 'current'})
                    devices = ET.SubElement(item, '{'+OMEMO+'}devices')
                    ET.SubElement(devices, '{'+OMEMO+'}device',
                                  {'id': '101' if account.startswith('alice@') else '202'})
                elif node == OMEMO+':bundles':
                    name = account.split('@')[0]
                    export = ET.parse(self.root/(name+'-bundle.xml')).getroot()
                    item = export.find('{'+PUBSUB+'}pubsub/{'+PUBSUB+'}publish/{'+PUBSUB+'}item')
                    if item is None:
                        raise ValueError('public bundle export unavailable')
                    result_items.append(copy.deepcopy(item))
                else:
                    raise ValueError('unsupported public node')
            response = ET.tostring(reply)
        capture = self.root/'mail.private'
        old = capture.stat().st_size if capture.exists() else 0
        if old+len(raw)+len(response)>4*1024*1024:
            raise ValueError('ordinary mail capture bound')
        with capture.open('ab') as output:
            capture.chmod(0o600)
            output.write(raw+b'\n'+response+b'\n')
        return response


class Provider:
    def __init__(self, binary, root):
        self.root = root
        self.process = subprocess.Popen([str(binary), 'serve', str(root)],
                                        stdin=subprocess.DEVNULL, stdout=subprocess.PIPE,
                                        stderr=subprocess.PIPE)
        self.closed = False
        deadline = time.monotonic()+5
        raw = bytearray()
        try:
            while b'\n' not in raw:
                remaining = deadline-time.monotonic()
                if remaining <= 0 or not select.select([self.process.stdout], [], [], remaining)[0]:
                    raise ValueError('provider readiness deadline')
                part = os.read(self.process.stdout.fileno(), 64)
                if not part:
                    raise ValueError('provider readiness unavailable')
                raw.extend(part)
                if len(raw)>64:
                    raise ValueError('provider readiness unavailable')
            self.port = parse_ready(bytes(raw))
        except BaseException:
            try:
                self.process.terminate()
                try:self.process.wait(timeout=1)
                except subprocess.TimeoutExpired:
                    self.process.kill();self.process.wait(timeout=4)
                retain_capture(self.root,'provider-startup-failed',bytes(raw)+self.process.stderr.read(4097))
            finally:
                self.closed=True
                self.process.stdout.close();self.process.stderr.close()
            raise

    def status(self):
        try:
            data = (self.root/'status').read_bytes()
            match = re.fullmatch(rb'([0-9]{1,4}) ([0-9]{1,5}) ([01])\n', data)
            if match is None:
                return (0, 0, 0)
            return tuple(map(int, match.groups()))
        except FileNotFoundError:
            return (0, 0, 0)

    def close(self):
        if self.closed:
            return
        private(self.root/'stop', b'explicit owned fixture stop\n')
        try:
            self.process.wait(timeout=5)
        except subprocess.TimeoutExpired:
            self.process.kill()
            self.process.wait(timeout=5)
            raise ValueError('provider did not drain')
        finally:
            self.closed = True
        extra = self.process.stdout.read(65)
        error = self.process.stderr.read(4097)
        self.process.stdout.close()
        self.process.stderr.close()
        if self.process.returncode or extra or error:
            raise ValueError('provider fixture failed')


def device_certificate(root, name, openssl):
    key, csr, pem, ext = [root/(name+suffix) for suffix in ['.key', '.csr', '.pem', '.ext']]
    command([openssl, 'req', '-config', root/'openssl-fixture.cnf', '-new', '-newkey', 'ec',
             '-pkeyopt', 'ec_paramgen_curve:P-256', '-nodes', '-sha256', '-subj', '/CN=synthetic-device',
             '-keyout', key, '-out', csr])
    private(ext, b'basicConstraints=critical,CA:FALSE\nextendedKeyUsage=clientAuth\nkeyUsage=critical,digitalSignature\n')
    command([openssl, 'x509', '-req', '-in', csr, '-CA', root/'ca.pem', '-CAkey', root/'ca.key',
             '-set_serial', str(secrets.randbits(64) or 1), '-days', '2', '-sha256', '-extfile', ext, '-out', pem])
    key.chmod(0o600)
    return pem, key


def fingerprint(path):
    return hashlib.sha256(ssl.PEM_cert_to_DER_cert(path.read_text())).digest()


def profile(root, name, identity, port, server_cert, device_pair):
    directory = root/(name+'-provider')
    directory.mkdir(mode=0o700)
    for source, target in [(root/'ca.pem', 'ca.pem'), (device_pair[0], 'device.pem'),
                           (device_pair[1], 'device.key')]:
        private(directory/target, source.read_bytes())
    values = dict(version=1, home='home.test', endpoint='127.0.0.1:'+str(port),
                  name='home.test', pin=fingerprint(server_cert).hex(),
                  account=name+'@home.test', device=101 if name=='alice' else 202,
                  identity=identity, ca=str(directory/'ca.pem'),
                  certificate=str(directory/'device.pem'), key=str(directory/'device.key'))
    text = ''.join(key+'='+json.dumps(value)+'\n' for key, value in values.items())
    path = directory/'profile.toml'
    private(path, text.encode())
    return path


def require_root(path):
    path = Path(path)
    info = path.lstat()
    if not path.is_absolute() or path.is_symlink() or not stat.S_ISDIR(info.st_mode) or info.st_uid != os.getuid() or stat.S_IMODE(info.st_mode) != 0o700 or (path/'OWNED').read_bytes() != b'Owned authenticated-provider console proof\n':
        raise ValueError('marked owned private console root required')
    return path


def one_journey(client, fixture, owned_root, openssl, unicode):
    root = Path(tempfile.mkdtemp(prefix='journey-OWNED-', dir=owned_root))
    root.chmod(0o700)
    private(root/'OWNED', b'Owned authenticated-provider member seed\n')
    children, peers = [], []
    provider = None
    facts = []
    local_password, home_password = [secrets.token_hex(20) for _ in range(2)]
    secret = (local_password+'\n').encode()
    try:
        command([fixture, 'seed', root], secret=secret)
        pairs = certificates(root, openssl)
        device_pairs = [device_certificate(root, name, openssl) for name in ['alice', 'bob']]
        identities = (root/'public-identities.txt').read_text().splitlines()
        if len(identities)!=2 or any(not re.fullmatch('[0-9a-f]{64}', value) for value in identities):
            raise ValueError('public identity exports invalid')
        server = root/'server'
        server.mkdir(mode=0o700)
        private(server/'OWNED', b'Owned public-policy provider fixture\n')
        for source, name in [(root/'public-principals.bin','public-principals.bin'),
                             (pairs['good'][0],'server.pem'),(pairs['good'][1],'server.key'),
                             (root/'ca.pem','ca.pem')]:
            private(server/name, source.read_bytes())
        private(server/'certificate-digests.bin', b''.join(fingerprint(pair[0]) for pair in device_pairs))
        provider = Provider(fixture, server)
        public_mail = PublicMail(root)
        alice = HomePeer(root, pairs['good'], 'alice@home.test', 101, home_password, public_mail, 0)
        peers.append(alice)
        bob = HomePeer(root, pairs['good'], 'bob@home.test', 202, home_password, public_mail, 1)
        peers.append(bob)
        configs = [_settings(root,name,peer,pairs['good'][0]) for name,peer in [('alice',alice),('bob',bob)]]
        profiles = [profile(root,name,identity,provider.port,pairs['good'][0],pair)
                    for name,identity,pair in zip(['alice','bob'],identities,device_pairs)]
        def start(member, selected=None, storage=None):
            args = [client,'--mail-online',(root if storage is None else storage)/member,configs[0 if member=='alice' else 1]]
            if unicode: args.append('--unicode')
            if selected is not None: args.extend(['--device-provider',selected])
            c = LauncherConsole(args, columns=40, rows=18)
            children.append(c)
            c.password(local_password, UNLOCK)
            c.expect(MENU)
            return c
        def desk(c): c.raw(b'm');c.expect(DESK)
        def lock(c, received=False):
            if received: c.raw(b'b');c.expect(DESK)
            c.raw(b'b');c.expect(MENU);c.raw(b'l');c.finish()
        def upload(c, operation):
            c.raw(b'u');c.expect('Full operation ID (blank cancels): ');c.send(operation)
            c.expect(TRANSFER);c.send('transfer');c.password(home_password,HOME)
            return time.monotonic()
        def select_received(c):
            c.raw(b'r');c.expect('Original record number (blank cancels): ');c.send('1');c.expect(RECEIVED)
        def save(c, destination):
            c.raw(b's');c.expect('Absolute save path (blank cancels): ');c.send(str(destination))
            c.expect('Type save to verify and publish: ');c.send('save');c.expect(RECEIVED)
        # Invalid profile is lazy and refuses preparation before a spool exists.
        invalid = profiles[0].parent/'invalid.toml'
        private(invalid, profiles[0].read_bytes().replace(b'device=101',b'device=202'))
        c=start('alice',invalid);desk(c);c.raw(b'c');c.expect('Selected attachment profile refused.');c.expect(DESK)
        assert not (root/'alice/attachments').exists() and provider.status()[0]==0
        lock(c)
        facts.append('lazy_invalid_profile_refuses_before_prepare_spool_or_connection')
        # A foreign peer is only a separate member-side negative fixture. It is
        # never installed in the provider's registry or used for the main journey.
        foreign=root/'foreign-member';foreign.mkdir(mode=0o700)
        private(foreign/'OWNED',b'Owned authenticated-provider member seed\n')
        command([fixture,'seed',foreign,'foreign'],secret=secret)
        private(foreign/'ca.pem',(root/'ca.pem').read_bytes())
        foreign_identity=(foreign/'public-identities.txt').read_text().splitlines()[0]
        foreign_profile=profile(foreign,'alice',foreign_identity,provider.port,pairs['good'][0],device_pairs[0])
        c=start('alice',foreign_profile,foreign);desk(c);c.raw(b'c');c.expect('Attachment action unavailable.');c.expect(DESK)
        assert not (foreign/'alice/attachments').exists() and provider.status()[0]==0
        lock(c)
        facts.append('foreign_peer_is_refused_before_prepare_spool_or_provider_connection')
        # Production prepare selects a snapshot, then source deletion proves retry
        # uses the durable ciphertext spool rather than reopening the plaintext.
        source=root/'selected-source.bin'
        payload=bytes((i*73+19)%251 for i in range(1_048_576))
        private(source,payload);source.chmod(0o644)
        c=start('alice',profiles[0]);desk(c);c.raw(b'c')
        for prompt,answer in [
            ('Absolute LOCAL source path (blank cancels): ',str(source)),
            ('Private display name (255 UTF-8 bytes): ','private-selected-name.bin\u202e'),
            ('Media type (127 ASCII bytes): ','application/octet-stream'),
            ('Subject (128 UTF-8 bytes): ','private-selected-subject'),
            ('Body (1024 UTF-8 bytes, one line): ','private-selected-body')]:
            c.expect(prompt);c.send(answer)
        c.expect('Type queue to encrypt and persist attachment: ');c.send('queue');c.expect(DESK)
        operations=re.findall(rb'Operation: ([0-9a-f]{32})',c.transcript)
        assert len(operations)==1
        operation=operations[0].decode();source.unlink();lock(c)
        assert not (root/'delivery.xml').exists()
        facts.append('maximum_size_prepare_is_device_local_and_source_independent')
        # Selected provider outage leaves actual ordinary XMPP publication usable.
        with socket.socket() as unavailable:
            unavailable.bind(('127.0.0.1',0))
            bad=profiles[0].parent/'outage.toml'
            private(bad,profiles[0].read_bytes().replace(('127.0.0.1:'+str(provider.port)).encode(),('127.0.0.1:'+str(unavailable.getsockname()[1])).encode()))
            c=start('alice',bad);desk(c);upload(c,operation)
            c.expect('Transfer stopped; no automatic retry.');c.expect(DESK)
            assert not (root/'delivery.xml').exists() and provider.status()[0]==0
            c.raw(b'b');c.expect(MENU);c.raw(b'u');c.password(home_password,ONLINE_HOME)
            c.expect('Manual cycle complete.');c.expect('Enter to return: ');c.send('');c.expect(MENU)
            c.raw(b'l');c.finish()
        assert alice.authenticated>=1 and public_mail.blob_requests==0
        facts.append('provider_outage_preserves_ordinary_mail_without_attachment_fallback')
        # A wrong server pin and a different approved device's actual key must
        # not authorize Alice's pending object or open account-only attachment IQ.
        snapshot=server/'serverstore/private-blobs.snapshot'
        pristine=snapshot.read_bytes()
        wrong_pin=profiles[0].parent/'wrong-pin.toml'
        private(wrong_pin,profiles[0].read_bytes().replace(fingerprint(pairs['good'][0]).hex().encode(),b'01'*32))
        c=start('alice',wrong_pin);desk(c);upload(c,operation)
        c.expect('Transfer stopped; no automatic retry.');c.expect(DESK);lock(c)
        assert snapshot.read_bytes()==pristine and not (root/'delivery.xml').exists()
        directory=profiles[0].parent
        private(directory/'other-device.pem',device_pairs[1][0].read_bytes())
        private(directory/'other-device.key',device_pairs[1][1].read_bytes())
        wrong_device=directory/'wrong-device.toml'
        private(wrong_device,profiles[0].read_bytes().replace(b'device.pem',b'other-device.pem').replace(b'device.key',b'other-device.key'))
        c=start('alice',wrong_device);desk(c);upload(c,operation)
        c.expect('Transfer stopped; no automatic retry.');c.expect(DESK);lock(c)
        assert snapshot.read_bytes()==pristine and not (root/'delivery.xml').exists() and public_mail.blob_requests==0
        facts.append('wrong_server_pin_and_actual_wrong_device_key_cannot_store_or_release_mail')
        # Stall after a real authenticated operation. Cancellation can leave
        # durable progress; exact reopen/retry must resolve it without mail release.
        initial_operations=provider.status()[1]
        private(server/'stall',b'pause after real served operation\n')
        c=start('alice',profiles[0]);desk(c);upload(c,operation)
        wait_fact(lambda:provider.status()[1]>initial_operations)
        began=time.monotonic();c.raw(b'\x03');c.expect('Home mail locked: local operation stopped.');c.finish(code=1)
        assert time.monotonic()-began<5 and not (root/'delivery.xml').exists()
        (server/'stall').unlink();wait_fact(lambda:provider.status()[2]==0)
        facts.append('cancel_after_possible_durable_progress_locks_and_joins_before_reopen')
        c=start('alice',profiles[0]);desk(c);submitted=upload(c,operation)
        upload_observation=wait_transfer(c,1_048_592,started=submitted);c.expect(DESK)
        wait_fact(lambda:alice.messages==1)
        assert re.search(rb'SDK call: (?:[1-9]|[1-5][0-9]|6[0-5]) / 65',c.transcript)
        lock(c)
        assert public_mail.blob_requests==0
        facts.append('reopened_exact_retry_commits_before_actual_encrypted_mail_release')
        # Bob's actual production archive action authenticates and persists receipt.
        c=start('bob',profiles[1]);c.raw(b'a');c.password(home_password,ONLINE_HOME)
        c.expect('Archive page saved. Entries: 1; accepted: 1;');c.expect('Enter to return: ');c.send('');c.expect(MENU)
        desk(c);select_received(c)
        assert not (root/'bob/attachments').exists()
        before=c.transcript.count(HOME.encode())
        c.raw(b'f');c.expect(TRANSFER);c.send('transfer');submitted=time.monotonic()
        fetch_observation=wait_transfer(c,1_048_592,started=submitted);c.expect(RECEIVED)
        assert c.transcript.count(HOME.encode())==before
        c.raw(b'v');c.expect('Verified locally: 1048576 plaintext bytes.');c.expect(RECEIVED)
        destination=root/'saved'/'explicit.bin';destination.parent.mkdir(mode=0o700)
        save(c,destination)
        assert destination.read_bytes()==payload and stat.S_IMODE(destination.stat().st_mode)==0o600
        save(c,destination)
        assert b'Save failed before publication.' in c.transcript and destination.read_bytes()==payload
        assert b'\\u{202e}' in c.transcript and '\u202e'.encode() not in c.transcript
        lock(c,True)
        facts.append('actual_authenticated_mail_receipt_fetch_without_password_verify_private_save_no_overwrite')
        before_revocation=snapshot.read_bytes()
        delivered=(root/'delivery.xml').read_bytes()
        private(server/'revoke',b'explicit synthetic current-policy revocation\n')
        wait_fact(lambda:(server/'revoked').exists())
        c=start('alice',profiles[0]);desk(c);upload(c,operation)
        c.expect('Transfer stopped; no automatic retry.');c.expect(DESK);lock(c)
        assert snapshot.read_bytes()==before_revocation and (root/'delivery.xml').read_bytes()==delivered
        assert alice.messages==1 and public_mail.blob_requests==0
        facts.append('current_provider_revocation_refuses_retry_without_store_or_mail_mutation')
        assert public_mail.archive_pages>=1 and public_mail.blob_requests==0
        command([fixture,'audit',root],secret=secret)
        assert all(local_password.encode() not in c.transcript and home_password.encode() not in c.transcript for c in children)
        facts.append('server_ciphertext_and_public_mail_exclude_member_private_metadata_keys_plaintext')
        return dict(mode='unicode' if unicode else 'classic',checks=facts,plaintext_bytes=len(payload),
                    provider_sessions=provider.status()[0],provider_operations=provider.status()[1],
                    ordinary_mail_connections=alice.connections+bob.connections,mail_received=True,
                    transfer_observations=dict(upload=upload_observation,fetch=fetch_observation))
    finally:
        failures=[]
        for index,c in enumerate(children):
            try:
                retain_capture(owned_root,'unicode-console' if unicode else 'classic-console',c.transcript)
                if hasattr(c,'transfer_observation'):
                    retain_capture(owned_root,'transfer-observation',
                                   (json.dumps(c.transfer_observation,sort_keys=True)+'\n').encode())
            except BaseException as error: failures.append(error)
            try:c.close()
            except BaseException as error:failures.append(error)
        for peer in peers:
            try:peer.close()
            except BaseException as error:failures.append(error)
        if provider is not None:
            try:provider.close()
            except BaseException as error:failures.append(error)
        for capture in [root/'mail.private',root/'delivery.xml']:
            if capture.exists():
                try:retain_capture(owned_root,'private-mail-capture',capture.read_bytes())
                except BaseException as error:failures.append(error)
        # Keep failed fixture bytes privately for diagnosis; successful fixtures
        # are removed after every process/socket was closed and joined.
        if not failures and __import__('sys').exc_info()[0] is None:
            shutil.rmtree(root)
            assert not root.exists()
        if failures:
            raise ValueError('owned console fixture cleanup failed') from failures[0]


def probe(client, fixture, owned_root, openssl):
    owned_root=require_root(owned_root)
    modes=[one_journey(client,fixture,owned_root,openssl,unicode) for unicode in [False,True]]
    return dict(result='pass',count=sum(len(mode['checks']) for mode in modes),modes=modes,
                private_fixture_removed=True,children_reaped=True,provider_deployment=False,
                independent_interoperability=False,physical_hardware=False,capacity_claim=False)


def make_parser():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--client',type=Path,required=True)
    parser.add_argument('--fixture',type=Path,required=True)
    parser.add_argument('--owned-root',type=Path,required=True)
    parser.add_argument('--openssl',default='openssl')
    parser.add_argument('--output',type=Path,required=True)
    return parser


if __name__=='__main__':
    args=make_parser().parse_args()
    result=probe(args.client.resolve(),args.fixture.resolve(),args.owned_root,args.openssl)
    private(args.output,(json.dumps(result,indent=2,sort_keys=True)+'\n').encode())
    print('Authenticated-provider production console passed',result['count'],'controls.')
