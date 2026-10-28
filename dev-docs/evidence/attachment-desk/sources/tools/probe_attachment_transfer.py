#!/usr/bin/env python3
"""Owned loopback TLS proof of automatic attachment APIs, not a home deployment.

Private fixture transcripts/keys remain in disposable directories and pipes. Only
bounded counters/status facts can be exported. The TLS peers authenticate two test
accounts and delegate every private-store IQ to the production Rust handler.
"""
import argparse
import base64
import hashlib
import json
import os
from pathlib import Path
import re
import select
import socket
import ssl
import struct
import subprocess
import tempfile
import threading
import time
from xml.etree import ElementTree as ET

from probe_xmpp_transport import BIND, FEATURE, SUCCESS, Reader, certificates

MAX_CIPHER = 1_048_592
NS = 'jabber:client'


class FixtureFailure(Exception):
    """Fixed local diagnostics never include peer/child output or private paths."""


def parse_progress(raw):
    match = re.fullmatch(rb'progress (0|[1-9][0-9]{0,6}) ([01]) ([01])\n', raw)
    if not match:
        raise FixtureFailure('invalid transfer progress')
    offset, complete, sent = map(int, match.groups())
    if offset > MAX_CIPHER or sent and not complete:
        raise FixtureFailure('invalid transfer progress')
    return offset, bool(complete), sent


def storage_kind(raw):
    # Only a test fault selector, never an identity or permission decision.
    try:
        if len(raw) > 16384:
            raise ValueError()
        element = ET.fromstring(raw).find('{urn:faction:blob:1}blob')
        if element is None or element.text is None or len(element.text) > 10924:
            raise ValueError()
        payload = base64.b64decode(element.text, validate=True)
        if not 9 <= len(payload) <= 8192 or payload[:8] != b'FBRQ0001' or payload[8] not in range(1, 7):
            raise ValueError()
        return payload[8]
    except (ValueError, ET.ParseError):
        raise FixtureFailure('invalid storage fault selector') from None


def stream_stanza(raw):
    # The qualified writer may inherit the stream's default namespace. This
    # fixture reconstructs that declared context, then delegates the same DOM.
    try:
        if len(raw) > 16384 or b'<!' in raw:
            raise ValueError()
        wrapper = ET.fromstring(b"<fixture xmlns='jabber:client' xmlns:stream='http://etherx.jabber.org/streams'>"+raw+b"</fixture>")
        if len(wrapper) != 1 or (wrapper.text or '').strip() or (wrapper[0].tail or '').strip():
            raise ValueError()
        element = wrapper[0]
        return element, ET.tostring(element)
    except (ValueError, ET.ParseError):
        raise FixtureFailure('invalid inherited fixture stanza') from None


def _read_exact(fd, count, deadline):
    data = bytearray()
    while len(data) < count:
        remaining = deadline-time.monotonic()
        if remaining <= 0 or not select.select([fd], [], [], remaining)[0]:
            raise FixtureFailure('handler deadline')
        part = os.read(fd, count-len(data))
        if not part:
            raise FixtureFailure('incomplete handler frame')
        data.extend(part)
    return bytes(data)


def read_frame(fd, cap, deadline):
    length = struct.unpack('!I', _read_exact(fd, 4, deadline))[0]
    if not 0 < length <= cap:
        raise FixtureFailure('handler frame bound')
    return _read_exact(fd, length, deadline)


def _write_all(fd, data, deadline):
    offset = 0
    while offset < len(data):
        remaining = deadline-time.monotonic()
        if remaining <= 0 or not select.select([], [fd], [], remaining)[1]:
            raise FixtureFailure('handler deadline')
        written = os.write(fd, data[offset:])
        if written <= 0:
            raise FixtureFailure('handler unavailable')
        offset += written


class Handler:
    def __init__(self, binary, root):
        self.process = subprocess.Popen([str(binary), 'handler', str(root)],
                                        stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                                        stderr=subprocess.PIPE)
        self.lock = threading.Lock()
        self.requests = 0
        self.transcript = root/'wire.private'

    def request(self, actor, xml):
        if actor not in (0, 1) or not 0 < len(xml) <= 16384:
            raise FixtureFailure('invalid handler request')
        with self.lock:
            deadline = time.monotonic()+5
            body = bytes([actor])+xml
            _write_all(self.process.stdin.fileno(), struct.pack('!I', len(body))+body, deadline)
            response = read_frame(self.process.stdout.fileno(), 16384, deadline)
            self.requests += 1
            before = self.transcript.stat().st_size if self.transcript.exists() else 0
            if before+len(xml)+len(response)+2 > 12*1024*1024:
                raise FixtureFailure('private fixture transcript bound')
            with self.transcript.open('ab') as capture:
                self.transcript.chmod(0o600)
                capture.write(xml+b'\n'+response+b'\n')
            return response

    def close(self):
        self.process.stdin.close()
        try:
            self.process.wait(timeout=5)
        except subprocess.TimeoutExpired:
            self.process.kill()
            self.process.wait(timeout=5)
            raise FixtureFailure('handler required termination')
        finally:
            self.process.stdout.close()
            error = self.process.stderr.read(16385)
            self.process.stderr.close()
        if self.process.returncode or error:
            raise FixtureFailure('handler failed')


class HomePeer:
    def __init__(self, root, cert, account, device, password, handler, actor):
        self.root, self.account, self.device = root, account, device
        self.password, self.handler, self.actor = password, handler, actor
        self.domain = account.split('@')[1]
        self.stop = threading.Event()
        self.failure = None
        self.phase = 'accept'
        self.connections = self.authenticated = self.messages = 0
        self.max_requests = self.max_input = 0
        self.drop_response = False
        self.dropped = False
        self.delay_replies = False
        self.delay_phases = set()
        self.revoke_after_get = False
        self.revoked_after_get = False
        self.context = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
        self.context.minimum_version = ssl.TLSVersion.TLSv1_2
        self.context.load_cert_chain(*map(str, cert))
        self.listener = socket.socket()
        self.listener.bind(('127.0.0.1', 0))
        self.listener.listen(2)
        self.listener.settimeout(.1)
        self.port = self.listener.getsockname()[1]
        self.thread = threading.Thread(target=self.run, name='faction-attachment-owned-home', daemon=True)
        self.thread.start()

    def run(self):
        while not self.stop.is_set():
            try:
                plain, _ = self.listener.accept()
            except socket.timeout:
                continue
            except OSError:
                break
            self.connections += 1
            plain.settimeout(5)
            try:
                with self.context.wrap_socket(plain, server_side=True) as channel:
                    self.serve(channel)
            except (OSError, ssl.SSLError):
                if not self.stop.is_set() and not self.delay_replies:
                    self.failure = 'unexpected TLS connection failure'
            except Exception as error:
                if not self.stop.is_set():
                    self.failure = 'owned home protocol failure: '+type(error).__name__+':'+self.phase

    def serve(self, channel):
        self.phase = 'stream'
        reader = Reader(channel)
        reader.stream()
        opening = ("<stream:stream xmlns='jabber:client' xmlns:stream='http://etherx.jabber.org/streams' "
                   "from='"+self.domain+"' id='attachment-proof' version='1.0'>").encode()
        channel.sendall(opening+FEATURE)
        self.phase = 'auth'
        auth = ET.fromstring(reader.element('auth'))
        expected = b'\0'+self.account.split('@')[0].encode()+b'\0'+self.password.encode()
        if auth.get('mechanism') != 'PLAIN' or base64.b64decode(auth.text or '', validate=True) != expected:
            raise FixtureFailure('fixture authentication failed')
        self.authenticated += 1
        channel.sendall(SUCCESS)
        reader.stream()
        channel.sendall(opening+BIND)
        self.phase = 'bind'
        request = ET.fromstring(reader.element('iq'))
        ident = request.get('id', '')
        if not re.fullmatch('[A-Za-z0-9_.:-]{1,128}', ident):
            raise FixtureFailure('invalid binding id')
        jid = self.account+'/faction'+str(self.device)
        channel.sendall(("<iq xmlns='jabber:client' type='result' id='"+ident+
                         "'><bind xmlns='urn:ietf:params:xml:ns:xmpp-bind'><jid>"+jid+
                         "</jid></bind></iq>").encode())
        count = 0
        while not self.stop.is_set():
            try:
                raw = reader.take(rb'(?:</(?:[A-Za-z_][\w.-]*:)?(?:iq|message|presence)\s*>|<(?:[A-Za-z_][\w.-]*:)?presence\b[^>]*/>)').strip()
            except Exception:
                # Dropping Session after a bounded cycle is normal. Counts and
                # successful client outcomes distinguish this from acceptance.
                break
            stanza, normalized = stream_stanza(raw)
            if stanza.tag == '{'+NS+'}presence':
                continue
            if stanza.tag == '{'+NS+'}message':
                if self.actor != 0 or stanza.find('{urn:xmpp:omemo:2}encrypted') is None:
                    raise FixtureFailure('unexpected message')
                if stanza.find('{'+NS+'}body') is not None or stanza.find('{'+NS+'}subject') is not None:
                    raise FixtureFailure('plaintext message escaped')
                path = self.root/'delivery.xml'
                if path.exists() and path.read_bytes() != normalized:
                    raise FixtureFailure('queued wire changed across retry')
                path.write_bytes(normalized)
                path.chmod(0o600)
                self.messages += 1
                continue
            if stanza.tag != '{'+NS+'}iq':
                raise FixtureFailure('unexpected stanza')
            count += 1
            self.phase = 'handler'
            response = self.handler.request(self.actor, normalized)
            is_blob = stanza.find('{urn:faction:blob:1}blob') is not None
            if self.drop_response and not self.dropped and is_blob and storage_kind(raw) == 3:
                self.dropped = True
                break
            if self.revoke_after_get and not self.revoked_after_get and is_blob and storage_kind(raw) == 5:
                (self.root/'server-revoked').write_text('explicit fixture revocation\n')
                self.revoked_after_get = True
            phase = 'begin' if is_blob and storage_kind(raw) == 1 else stanza.get('id')
            if self.delay_replies and phase in {'begin', 'peer-devices'}:
                self.delay_phases.add(phase)
                if self.stop.wait(6):
                    break
            channel.sendall(response)
        self.max_requests = max(self.max_requests, count)
        self.max_input = max(self.max_input, reader.total)

    def close(self):
        self.stop.set()
        self.listener.close()
        self.thread.join(timeout=5)
        if self.thread.is_alive():
            raise FixtureFailure('owned home did not stop')
        if self.failure:
            raise FixtureFailure(self.failure)


def _run(binary, root, action, secret, *arguments, allow_failure=False):
    process = subprocess.run([str(binary), action, str(root), *map(str, arguments)],
                             input=secret, capture_output=True, timeout=20)
    if process.stderr or len(process.stdout) > 4096:
        raise FixtureFailure('client emitted invalid diagnostic output')
    if process.returncode and not allow_failure:
        raise FixtureFailure('client action failed: '+action)
    return process


def _settings(root, name, peer, cert):
    pin = hashlib.sha256(ssl.PEM_cert_to_DER_cert(cert.read_text())).hexdigest()
    path = root/(name+'-home.toml')
    ca = root/'ca.pem'
    ca.chmod(0o600)
    path.write_text(f'version = 1\naccount = "{peer.account}"\nendpoint = "127.0.0.1:{peer.port}"\ndomain = "{peer.domain}"\nca = "{ca}"\npin = "{pin}"\n')
    path.chmod(0o600)
    return path


def _away_certificate(root, openssl):
    key, csr, pem = (root/('away.'+suffix) for suffix in ('key', 'csr', 'pem'))
    ext = root/'away.ext'
    ext.write_text('basicConstraints=critical,CA:FALSE\nsubjectAltName=DNS:away.test\nextendedKeyUsage=serverAuth\nkeyUsage=critical,digitalSignature,keyEncipherment\n')
    commands = [
        [openssl, 'req', '-config', str(root/'openssl-fixture.cnf'), '-new', '-newkey', 'rsa:2048', '-nodes', '-sha256', '-subj', '/CN=away.test', '-keyout', str(key), '-out', str(csr)],
        [openssl, 'x509', '-req', '-in', str(csr), '-CA', str(root/'ca.pem'), '-CAkey', str(root/'ca.key'), '-set_serial', '261', '-days', '2', '-sha256', '-extfile', str(ext), '-out', str(pem)],
    ]
    for command in commands:
        result = subprocess.run(command, stdin=subprocess.DEVNULL, capture_output=True, timeout=30)
        if result.returncode:
            raise FixtureFailure('away certificate failed')
    key.chmod(0o600)
    return pem, key


def probe(binary, openssl='openssl'):
    started = time.monotonic()
    facts = {}
    with tempfile.TemporaryDirectory(prefix='faction261-owned-tls-') as temporary:
        root = Path(temporary).resolve()
        root.chmod(0o700)
        (root/'OWNED').write_text('Owned disposable attachment TLS proof\n')
        import secrets
        secret = (secrets.token_hex(20)+'\n'+secrets.token_hex(20)+'\n').encode()
        pairs = certificates(root, openssl)
        away_cert = _away_certificate(root, openssl)
        _run(binary, root, 'init', secret)
        _run(binary, root, 'prepare', secret, 1_048_576)
        handler = Handler(binary, root)
        alice = bob = None
        try:
            home_password = secret.splitlines()[1].decode()
            alice = HomePeer(root, pairs['good'], 'alice@home.test', 101, home_password, handler, 0)
            bob = HomePeer(root, away_cert, 'bob@away.test', 202, home_password, handler, 1)
            ac = _settings(root, 'alice', alice, pairs['good'][0])
            bc = _settings(root, 'bob', bob, away_cert[0])
            # Ordinary Sync must not send managed attachments before storage.
            _run(binary, root, 'sync', secret, ac)
            if alice.messages:
                raise FixtureFailure('ordinary sync bypassed upload')
            # Lose a response after the real handler's durable mutation.
            alice.drop_response = True
            result = _run(binary, root, 'upload', secret, ac, allow_failure=True)
            if not alice.dropped or result.returncode == 0:
                raise FixtureFailure('lost-response control was not observed')
            uploaded = False
            for cycle in range(80):
                result = _run(binary, root, 'upload', secret, ac)
                offset, complete, sent = parse_progress(result.stdout)
                if complete:
                    if offset != MAX_CIPHER or sent != 1 or not alice.messages:
                        raise FixtureFailure('incomplete release proof')
                    uploaded = True
                    facts['upload_cycles_after_loss'] = cycle+1
                    break
            if not uploaded:
                raise FixtureFailure('upload did not finish within bound')
            # One production call shares its ten-second outer deadline across
            # storage and the fresh peer-verifying release Session.
            before_messages = alice.messages
            alice.delay_replies = True
            began = time.monotonic()
            result = _run(binary, root, 'upload', secret, ac, allow_failure=True)
            elapsed = time.monotonic()-began
            if result.returncode == 0 or not 9 <= elapsed < 12 or alice.messages != before_messages or alice.delay_phases != {'begin', 'peer-devices'}:
                raise FixtureFailure('whole-call deadline control failed')
            facts['two_session_timeout_seconds'] = round(elapsed, 3)
            facts['timeout_sent_no_message'] = True
            facts['sender_connections'] = alice.connections
            facts['sender_max_requests'] = alice.max_requests
            facts['sender_max_physical_input'] = alice.max_input
            alice.close()
            alice = None
            # The sender is offline before native receipt and bulk fetch.
            _run(binary, root, 'receive', secret)
            before_connections = bob.connections
            for action, expected in [('fetch-uninvited', b'refused route\n'), ('fetch-wrong-frame', b'refused selection\n')]:
                result = _run(binary, root, action, secret, bc, allow_failure=True)
                if result.returncode == 0 or result.stdout != expected or bob.connections != before_connections:
                    raise FixtureFailure('local selection or route bypass')
            bob.revoke_after_get = True
            result = _run(binary, root, 'fetch', secret, bc, allow_failure=True)
            if result.returncode == 0 or not bob.revoked_after_get:
                raise FixtureFailure('between-chunk revocation was not observed')
            result = _run(binary, root, 'verify', secret, allow_failure=True)
            if result.returncode == 0:
                raise FixtureFailure('partial fetch released plaintext')
            (root/'server-revoked').unlink()
            facts['uninvited_and_wrong_frame_preflight_refused'] = True
            facts['between_chunk_revocation_and_resume'] = True
            fetched = False
            for cycle in range(80):
                result = _run(binary, root, 'fetch', secret, bc)
                offset, complete, sent = parse_progress(result.stdout)
                if sent:
                    raise FixtureFailure('fetch sent a mail message')
                if complete:
                    if offset != MAX_CIPHER:
                        raise FixtureFailure('incomplete fetched bytes')
                    fetched = True
                    facts['fetch_cycles'] = cycle+1
                    break
            if not fetched:
                raise FixtureFailure('fetch did not finish within bound')
            _run(binary, root, 'verify', secret)
            _run(binary, root, 'audit', secret)
            facts['server_files_and_wire_private_metadata_absent'] = True
            save_dir = root/'selected-save'
            save_dir.mkdir(mode=0o700)
            destination = save_dir/'explicit.bin'
            _run(binary, root, 'save', secret, destination)
            before = destination.read_bytes()
            if len(before) != 1_048_576:
                raise FixtureFailure('saved plaintext size mismatch')
            result = _run(binary, root, 'save', secret, destination, allow_failure=True)
            if not result.returncode or destination.read_bytes() != before:
                raise FixtureFailure('existing save was overwritten')
            facts['recipient_connections'] = bob.connections
            facts['recipient_max_requests'] = bob.max_requests
            facts['recipient_max_physical_input'] = bob.max_input
            facts['handler_requests'] = handler.requests
            facts['lost_response_observed'] = True
            facts['full_plaintext_bytes'] = 1_048_576
            facts['sender_offline_at_fetch'] = True
            facts['explicit_no_overwrite_save'] = True
            if max(facts['sender_max_physical_input'], facts['recipient_max_physical_input']) > 65536:
                raise FixtureFailure('physical cycle budget exceeded')
            if facts['recipient_max_requests'] > 8:
                raise FixtureFailure('fetch request budget exceeded')
        finally:
            failures = []
            for resource in (alice, bob, handler):
                if resource is not None:
                    try:
                        resource.close()
                    except Exception as error:
                        failures.append(type(resource).__name__+':'+str(error))
            if failures:
                raise FixtureFailure('owned fixture cleanup failed: '+','.join(failures))
    if root.exists():
        raise FixtureFailure('private fixture retained')
    return dict(result='pass', facts=facts, private_fixture_removed=True,
                actual_home_sessions=True, actual_rust_store=True,
                provider_deployment=False, device_attestation=False,
                wall_seconds=round(time.monotonic()-started, 3))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path, required=True)
    parser.add_argument('--openssl', default='openssl')
    parser.add_argument('--output', type=Path)
    arguments = parser.parse_args()
    result = probe(arguments.binary.resolve(), arguments.openssl)
    if arguments.output:
        arguments.output.write_text(json.dumps(result, indent=2)+'\n')
    print('Automatic encrypted attachment TLS software proof passed; owned fixtures removed.')
