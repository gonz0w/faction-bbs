#!/usr/bin/env python3
"""Owned production-UI foreground proof with synthetic loopback TLS homes.

This fixture is not an XMPP S2S provider or independent-client interoperability.
Private captures stay in the marked owner root. A visited-board phase requires
separate marked fixture authority; this tool does not provision SSH or a VM.
"""
import base64
import copy
import hashlib
import json
import os
from pathlib import Path
import re
import select
import socket
import ssl
import stat
import subprocess
import secrets
import tempfile
import threading
import time
from collections import Counter
from xml.etree import ElementTree as ET

ACCOUNTS = ('alice@home.test', 'bob@away.test')
DEVICES = (101, 202)
NS = 'jabber:client'
OMEMO = 'urn:xmpp:omemo:2'
PUBSUB = 'http://jabber.org/protocol/pubsub'
MAM = 'urn:xmpp:mam:2'
RSM = 'http://jabber.org/protocol/rsm'
FORM = 'jabber:x:data'
MARKER = b'Owned issue356 foreground conversation fixture\n'


def owned_root(root):
    root = Path(root)
    info = root.lstat()
    if (not root.is_absolute() or root.is_symlink() or not stat.S_ISDIR(info.st_mode)
            or info.st_uid != os.getuid() or stat.S_IMODE(info.st_mode) != 0o700
            or (root/'OWNED').is_symlink() or (root/'OWNED').read_bytes() != MARKER):
        raise ValueError('marked private foreground fixture root required')
    return root


def element(raw):
    if not isinstance(raw, bytes) or not 0 < len(raw) <= 16384 or b'<!' in raw:
        raise ValueError('bounded fixture XML required')
    try:
        wrapper = ET.fromstring(b"<fixture xmlns='jabber:client' xmlns:stream='http://etherx.jabber.org/streams'>"+raw+b"</fixture>")
        if len(wrapper) != 1 or (wrapper.text or '').strip() or (wrapper[0].tail or '').strip():
            raise ValueError('one fixture stanza required')
        return wrapper[0]
    except ET.ParseError:
        raise ValueError('malformed fixture XML') from None


class Relay:
    """Two scoped opaque archives and live routes; no vault or crypto API.

    Retry deduplication here is a fixture choice, not a provider guarantee.
    Native authentication and retained read decisions are checked in the client.
    """
    def __init__(self, root):
        self.root = owned_root(root)
        self.condition = threading.Condition()
        self.generations = [0, 0]
        self.active = {}
        self.archives = [[], []]
        self.operations = {}
        self.public = {}
        self.messages = []
        self.queries = []

    def register(self, actor, full, send):
        if actor not in (0, 1) or full != f'{ACCOUNTS[actor]}/faction{DEVICES[actor]}' or not callable(send):
            raise ValueError('wrong synthetic bound resource')
        with self.condition:
            if actor in self.active:
                raise ValueError('duplicate live fixture binding')
            self.generations[actor] += 1
            generation = self.generations[actor]
            self.active[actor] = (generation, send)
            self.condition.notify_all()
            return generation

    def unregister(self, actor, generation):
        with self.condition:
            if self.active.get(actor, (None,))[0] == generation:
                del self.active[actor]
                self.condition.notify_all()

    def wait_bound(self, actor, after=0, timeout=2):
        if actor not in (0, 1) or not 0 < timeout <= 5:
            raise ValueError('bounded fixture readiness wait required')
        with self.condition:
            ready = self.condition.wait_for(
                lambda: actor in self.active and self.active[actor][0] > after, timeout)
            if not ready:
                raise TimeoutError('fresh authenticated fixture binding unavailable')
            return self.active[actor][0]

    def inventory(self, actor):
        with self.condition:
            return list(self.archives[actor])

    def message(self, actor, raw):
        stanza = element(raw)
        peer = 1-actor
        own, recipient = ACCOUNTS[actor], ACCOUNTS[peer]
        full = f'{recipient}/faction{DEVICES[peer]}'
        operation = stanza.get('id', '')
        children = list(stanza)
        if (actor not in (0, 1) or stanza.tag != '{'+NS+'}message'
                or stanza.get('from') not in (own, f'{own}/faction{DEVICES[actor]}')
                or stanza.get('to') not in (recipient, full) or stanza.get('type') != 'chat'
                or not re.fullmatch('[a-f0-9]{32}', operation)
                or [child.tag for child in children] != ['{'+OMEMO+'}encrypted', '{urn:xmpp:hints}store']
                or (stanza.text or '').strip() or children[1].attrib or list(children[1])
                or (children[1].text or '').strip()):
            raise ValueError('unexpected plaintext, metadata or message route')
        # Normalize only authenticated outer routing. All encrypted children and
        # operation labels remain exact; the fixture never opens their contents.
        canonical = copy.deepcopy(stanza)
        canonical.set('from', f'{own}/faction{DEVICES[actor]}')
        canonical.set('to', recipient)
        frame = ET.tostring(canonical)
        with self.condition:
            key = (actor, operation)
            previous = self.operations.get(key)
            if previous is not None and previous[0] != frame:
                raise ValueError('immutable queued ciphertext changed on retry')
            if previous is None:
                if any(len(self.archives[index]) >= 64 for index in (actor, peer)):
                    raise ValueError('bounded fixture archive full')
                ids = {}
                for index in (actor, peer):
                    ident = f'fixture{index}-{len(self.archives[index])+1}'
                    self.archives[index].append((ident, frame))
                    ids[index] = ident
                self.operations[key] = (frame, ids)
            ident = self.operations[key][1][peer]
            binding = self.active.get(peer) if stanza.get('to') == full else None
            self.messages.append(dict(actor=actor, operation=operation, to=stanza.get('to'),
                                      bound_generation=None if binding is None else binding[0]))
        if binding is None:
            return False
        arrived = copy.deepcopy(canonical)
        arrived.set('to', full)
        ET.SubElement(arrived, '{urn:xmpp:sid:0}stanza-id', {'by': recipient, 'id': ident})
        binding[1](ET.tostring(arrived))
        return True

    def request(self, actor, raw):
        request = element(raw)
        own = ACCOUNTS[actor]
        if request.tag != '{'+NS+'}iq' or not request.get('id'):
            raise ValueError('fixture IQ required')
        reply = ET.Element('{'+NS+'}iq', {'id': request.get('id'), 'type': 'result', 'from': own, 'to': f'{own}/faction{DEVICES[actor]}'})
        query = request.find('{'+MAM+'}query')
        if query is not None:
            if request.get('to') != own or request.get('type') != 'set' or not query.get('queryid'):
                raise ValueError('wrong own archive scope')
            fields = query.findall('{'+FORM+'}x/{'+FORM+'}field')
            form = {field.get('var'): field.findtext('{'+FORM+'}value') for field in fields}
            page = query.find('{'+RSM+'}set')
            if (len(fields) != 2 or form != {'FORM_TYPE': MAM, 'with': ACCOUNTS[1-actor]}
                    or page is None or page.findtext('{'+RSM+'}max') != '4'):
                raise ValueError('unsupported fixture archive query')
            after = page.findtext('{'+RSM+'}after')
            inventory = self.inventory(actor)
            if after is None:
                start = 0
            else:
                matches = [i for i, (ident, _) in enumerate(inventory) if ident == after]
                if len(matches) != 1:
                    raise ValueError('unknown scoped fixture archive position')
                start = matches[0]+1
            selected = inventory[start:start+4]
            self.queries.append(dict(actor=actor, after=after, ids=[ident for ident, _ in selected]))
            pieces = []
            for ident, frame in selected:
                outer = ET.Element('{'+NS+'}message', {'from': own, 'to': f'{own}/faction{DEVICES[actor]}'})
                result = ET.SubElement(outer, '{'+MAM+'}result', {'id': ident, 'queryid': query.get('queryid')})
                forwarded = ET.SubElement(result, '{urn:xmpp:forward:0}forwarded')
                ET.SubElement(forwarded, '{urn:xmpp:delay}delay', {'stamp': '2026-11-07T00:00:00Z'})
                forwarded.append(ET.fromstring(frame)); pieces.append(ET.tostring(outer))
            fin = ET.SubElement(reply, '{'+MAM+'}fin', {'complete': 'true' if start+len(selected) == len(inventory) else 'false'})
            rsm = ET.SubElement(fin, '{'+RSM+'}set')
            if selected:
                ET.SubElement(rsm, '{'+RSM+'}first').text = selected[0][0]
                ET.SubElement(rsm, '{'+RSM+'}last').text = selected[-1][0]
            return b''.join(pieces+[ET.tostring(reply)])
        pubsub = request.find('{'+PUBSUB+'}pubsub')
        if pubsub is None:
            raise ValueError('unsupported fixture IQ')
        publish = pubsub.find('{'+PUBSUB+'}publish')
        if publish is not None:
            node = publish.get('node')
            if node not in (OMEMO+':devices', OMEMO+':bundles'):
                raise ValueError('unsupported public fixture node')
            self.public[(own, node)] = copy.deepcopy(publish)
        else:
            items = pubsub.find('{'+PUBSUB+'}items')
            target = request.get('to', own)
            if items is None or target not in ACCOUNTS:
                raise ValueError('unsupported fixture discovery')
            node = items.get('node'); reply.set('from', target)
            public = self.public.get((target, node))
            if public is None:
                reply.set('type', 'error')
                error = ET.SubElement(reply, '{'+NS+'}error', {'type': 'cancel'})
                ET.SubElement(error, '{urn:ietf:params:xml:ns:xmpp-stanzas}item-not-found')
            else:
                result_items = ET.SubElement(ET.SubElement(reply, '{'+PUBSUB+'}pubsub'), '{'+PUBSUB+'}items', {'node': node})
                for child in public: result_items.append(copy.deepcopy(child))
        return ET.tostring(reply)


def extension(before, after):
    if set(before) != {'history', 'pending', 'archive'} or set(after) != set(before):
        raise ValueError('exact retained-state shape required')
    previous, current = before['history'], after['history']
    if not isinstance(previous, list) or not isinstance(current, list) or current[:len(previous)] != previous or after['pending'] != before['pending']:
        raise ValueError('original retained records, read decisions or pending work changed')
    ids = [row['received_id'] for row in current if row['direction'] == 'received']
    if len(ids) != len(set(ids)) or any(not isinstance(ident, str) or not re.fullmatch('[a-f0-9]{64}', ident) or ident == '0'*64 for ident in ids):
        raise ValueError('authenticated received records are duplicated or unidentified')
    return current[len(previous):]


def incoming(before, rows):
    peer = next((row for row in before['history'] if row['direction'] == 'received'), None)
    if peer is None or any(row['direction'] != 'received' or row['read'] is not False
                           or any(row[key] != peer[key] for key in ('address', 'device', 'identity')) for row in rows):
        raise ValueError('new records must be authenticated current-peer unread input')


def require_live(before, after, subject, body):
    rows = extension(before, after)
    if len(rows) != 1 or (rows[0]['subject'], rows[0]['body']) != (subject, body) or after['archive'] != before['archive']:
        raise ValueError('one exact live record with an unchanged archive token required')
    incoming(before, rows)
    return after


def require_recovery(before, after, older):
    rows = extension(before, after)
    incoming(before, rows)
    if (Counter((row['subject'], row['body']) for row in rows) != Counter(older)
            or len(rows) != len(older) or set(after['archive']) != set(before['archive'])
            or any(after['archive'][key] != before['archive'][key] for key in ('home', 'local_device', 'epoch'))
            or after['archive']['after'] is None or after['archive']['after'] == before['archive']['after']):
        raise ValueError('complete controlled older inventory and scoped cursor progress required')
    return after


def public_result(facts):
    booleans = ('cursor_unchanged_after_live','read_decisions_preserved','pending_preserved',
                'exact_selected_operation','no_presence','children_reaped','listeners_closed')
    counts = dict(selected_writes=1,live_accepted=1,live_duplicates=0,archive_accepted=2,
                  archive_duplicates=1,server_plaintext_matches=0)
    if (set(facts) != set(booleans)|set(counts)|{'unicode','tls_versions'}
            or type(facts['unicode']) is not bool
            or any(facts[key] is not True for key in booleans)
            or any(type(facts[key]) is not int or facts[key] != value for key, value in counts.items())
            or not isinstance(facts['tls_versions'], list) or len(facts['tls_versions']) != 2
            or any(value not in ('TLSv1.2','TLSv1.3') for value in facts['tls_versions'])):
        raise ValueError('incomplete or unsafe foreground qualification record')
    return dict(facts, result='pass', scope='production-foreground-ui-synthetic-two-home-tls',
                production_s2s=False, netbsd_qualified=False, independent_interoperability=False)


class LiveHome:
    def __init__(self, root, actor, relay, pair, password):
        self.root = owned_root(root)
        if actor not in (0,1) or not password or len(password) > 128:
            raise ValueError('fixed synthetic TLS home required')
        self.actor, self.relay, self.password = actor, relay, password
        self.stop = threading.Event();self.failure_event = threading.Event()
        self.closed = False;self.failure = None;self.channel = None
        self.connections = self.authenticated = self.bindings = 0
        self.tls_versions = []
        self.context = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
        self.context.minimum_version = ssl.TLSVersion.TLSv1_2
        self.context.load_cert_chain(*map(str,pair))
        self.listener = socket.socket()
        try:
            self.listener.bind(('127.0.0.1',0));self.listener.listen(2);self.listener.settimeout(.1)
        except BaseException:
            self.listener.close();raise
        self.port = self.listener.getsockname()[1]
        self.thread = threading.Thread(target=self.run,name='foreground-owned-home-'+str(actor),daemon=True)
        self.thread.start()

    def capture(self, raw):
        path = self.root/('home-'+str(self.actor)+'.private')
        if len(raw) > 65536 or (path.stat().st_size if path.exists() else 0)+len(raw)+1 > 4*1024*1024:
            raise ValueError('bounded private home capture exceeded')
        with path.open('ab') as output:
            path.chmod(0o600);output.write(raw+b'\n')

    def run(self):
        while not self.stop.is_set():
            try:
                plain,_ = self.listener.accept()
            except socket.timeout:
                continue
            except OSError:
                break
            self.connections += 1
            plain.settimeout(5)
            try:
                with self.context.wrap_socket(plain,server_side=True) as channel:
                    self.channel = channel;self.tls_versions.append(channel.version())
                    self.serve(channel)
            except Exception as error:
                if not self.stop.is_set():
                    self.failure = type(error).__name__+':'+getattr(self,'phase','handshake')
                    self.failure_event.set()
            finally:
                self.channel = None
                plain.close()

    def serve(self, channel):
        from probe_xmpp_transport import Reader, FEATURE, SUCCESS, BIND
        reader = Reader(channel)
        own = ACCOUNTS[self.actor];domain = own.split('@')[1]
        opening = ("<stream:stream xmlns='jabber:client' xmlns:stream='http://etherx.jabber.org/streams' from='"+domain+"' id='foreground-fixture' version='1.0'>").encode()
        self.phase='stream';reader.stream();channel.sendall(opening+FEATURE)
        self.phase='auth';self.verify_auth(self.actor,self.password,reader.element('auth'))
        self.authenticated += 1;channel.sendall(SUCCESS)
        self.phase='restart';reader.stream();channel.sendall(opening+BIND)
        self.phase='bind';response=self.bind_response(self.actor,reader.element('iq'))
        channel.sendall(response)
        # One worker owns both directions of each TLS socket. A local pipe wakes
        # select for queued ciphertext; another home never writes this SSL object.
        read_fd,write_fd=os.pipe();os.set_blocking(read_fd,False);os.set_blocking(write_fd,False)
        queued=[];queue_lock=threading.Lock();generation=None
        def send(raw):
            if not 0 < len(raw) <= 16384:
                raise ValueError('bounded live fixture output required')
            with queue_lock:
                if len(queued) >= 32:raise ValueError('bounded live fixture queue full')
                queued.append(raw)
            os.write(write_fd,b'x')
        try:
            generation=self.relay.register(self.actor,f'{own}/faction{DEVICES[self.actor]}',send)
            self.bindings += 1
            deadline=time.monotonic()+10;count=0
            pattern=rb'(?:</(?:[A-Za-z_][\w.-]*:)?(?:iq|message|presence)\s*>|<(?:[A-Za-z_][\w.-]*:)?presence\b[^>]*/>)'
            self.phase='application'
            while not self.stop.is_set():
                matched=re.search(pattern,reader.buffer)
                if matched:
                    raw,reader.buffer=reader.buffer[:matched.end()].strip(),reader.buffer[matched.end():]
                    stanza=element(raw);count+=1
                    if count > 32:raise ValueError('bounded fixture stanza count exceeded')
                    if stanza.tag == '{'+NS+'}message':
                        self.relay.message(self.actor,raw)
                    elif stanza.tag == '{'+NS+'}iq':
                        reply=self.relay.request(self.actor,raw);channel.sendall(reply);self.capture(reply)
                    else:
                        raise ValueError('foreground fixture never permits presence')
                    self.capture(raw)
                    continue
                remaining=deadline-time.monotonic()
                if remaining <= 0:break
                ready,_,_=select.select([channel,read_fd],[],[],min(remaining,.2))
                if read_fd in ready:
                    os.read(read_fd,4096)
                    with queue_lock:pending,queued[:]=list(queued),[]
                    for raw in pending:channel.sendall(raw);self.capture(raw)
                if channel in ready or channel.pending():
                    part=channel.recv(4096)
                    if not part:break
                    reader.total+=len(part)
                    if reader.total > 65536 or len(reader.buffer)+len(part) > 16384 or b'<!' in part:
                        raise ValueError('bounded fixture XML input exceeded')
                    reader.buffer+=part
        finally:
            if generation is not None:self.relay.unregister(self.actor,generation)
            os.close(read_fd);os.close(write_fd)

    def close(self):
        if self.closed:return
        self.stop.set();self.listener.close()
        if self.channel is not None:
            try:self.channel.shutdown(socket.SHUT_RDWR)
            except OSError:pass
        self.thread.join(timeout=5)
        self.closed=True
        if self.thread.is_alive():raise ValueError('owned TLS worker failed to stop')

    @staticmethod
    def verify_auth(actor, password, raw):
        auth = element(raw)
        expected = b'\0'+ACCOUNTS[actor].split('@')[0].encode()+b'\0'+password.encode()
        try:
            supplied = base64.b64decode(auth.text or '', validate=True)
        except ValueError:
            raise ValueError('invalid fixture authentication encoding') from None
        if (auth.tag != '{urn:ietf:params:xml:ns:xmpp-sasl}auth'
                or auth.attrib != {'mechanism': 'PLAIN'} or list(auth) or supplied != expected):
            raise ValueError('fixture account authentication refused')

    @staticmethod
    def bind_response(actor, raw):
        request = element(raw)
        bind = request.find('{urn:ietf:params:xml:ns:xmpp-bind}bind')
        if (request.tag != '{'+NS+'}iq' or request.get('type') != 'set'
                or not re.fullmatch('[A-Za-z0-9_.:-]{1,128}', request.get('id',''))
                or len(request) != 1 or bind is None or len(bind) != 1
                or bind[0].tag != '{urn:ietf:params:xml:ns:xmpp-bind}resource'
                or bind[0].text != 'faction'+str(DEVICES[actor]) or list(bind[0])):
            raise ValueError('fixture device binding refused')
        reply = ET.Element('{'+NS+'}iq', {'type':'result', 'id':request.get('id')})
        ET.SubElement(ET.SubElement(reply, '{urn:ietf:params:xml:ns:xmpp-bind}bind'),
                      '{urn:ietf:params:xml:ns:xmpp-bind}jid').text = f'{ACCOUNTS[actor]}/faction{DEVICES[actor]}'
        return ET.tostring(reply)


def certificates(root, domain, openssl):
    root = Path(root)
    if domain not in ('home.test','away.test') or not root.is_absolute() or root.is_symlink() or stat.S_IMODE(root.stat().st_mode) != 0o700:
        raise ValueError('private fixed-domain certificate fixture required')
    def run(arguments):
        result = subprocess.run([str(openssl),*map(str,arguments)],stdin=subprocess.DEVNULL,capture_output=True,timeout=15)
        if result.returncode:
            raise ValueError('private fixture certificate command failed')
    config=root/'openssl.cnf';config.write_text('[req]\ndistinguished_name=dn\n[dn]\n');config.chmod(0o600)
    ca,ca_key=root/'ca.pem',root/'ca.key'
    certificate,key,csr=root/'server.pem',root/'server.key',root/'server.csr'
    run(['req','-config',config,'-x509','-newkey','ec','-pkeyopt','ec_paramgen_curve:P-256','-nodes','-sha256','-days','2',
         '-subj','/CN='+domain+' synthetic CA','-addext','basicConstraints=critical,CA:TRUE','-addext','keyUsage=critical,keyCertSign,cRLSign','-addext','subjectKeyIdentifier=hash','-keyout',ca_key,'-out',ca])
    run(['req','-config',config,'-new','-newkey','ec','-pkeyopt','ec_paramgen_curve:P-256','-nodes','-sha256',
         '-subj','/CN='+domain,'-keyout',key,'-out',csr])
    extension=root/'server.ext';extension.write_text('basicConstraints=critical,CA:FALSE\nsubjectAltName=DNS:'+domain+'\nextendedKeyUsage=serverAuth\nkeyUsage=critical,digitalSignature\nsubjectKeyIdentifier=hash\nauthorityKeyIdentifier=keyid,issuer\n');extension.chmod(0o600)
    run(['x509','-req','-in',csr,'-CA',ca,'-CAkey',ca_key,'-set_serial',str(secrets.randbits(96) or 1),'-days','2','-sha256','-extfile',extension,'-out',certificate])
    for path in (ca,ca_key,certificate,key,csr):path.chmod(0o600)
    return (certificate,key),ca


def native_command(root, binary, action, home, *arguments, home_password=False, lines=()):
    from probe_home_mail import _invocation
    root = owned_root(root)
    argv,options = _invocation(binary,action,home,arguments,home_password,lines)
    completed = subprocess.run(argv,**options)
    if len(completed.stdout) > 4*1024*1024 or len(completed.stderr) > 4*1024*1024:
        raise ValueError('native fixture output exceeded capture bound')
    record = dict(argv=argv,stdin_sha256=hashlib.sha256(options['input']).hexdigest(),
                  exit=completed.returncode,stdout_hex=completed.stdout.hex(),stderr_hex=completed.stderr.hex())
    path=root/('command-'+secrets.token_hex(12)+'.private.json')
    path.write_text(json.dumps(record,sort_keys=True)+'\n');path.chmod(0o600)
    if completed.returncode or completed.stderr:
        raise ValueError('native fixture action failed')
    return completed.stdout


def expect_one(console, tokens, seconds=15):
    deadline=time.monotonic()+seconds
    while True:
        if len(console.buffer) > 4*1024*1024 or len(console.transcript) > 4*1024*1024:
            raise ValueError('bounded private terminal capture exceeded')
        positions=[(console.buffer.find(token),token) for token in tokens if token in console.buffer]
        if positions:
            position,token=min(positions,key=lambda pair:pair[0])
            end=position+len(token);raw,console.buffer=console.buffer[:end],console.buffer[end:]
            return token,raw
        remaining=deadline-time.monotonic()
        if remaining <= 0:raise TimeoutError('expected local foreground prompt unavailable')
        if not select.select([console.fd],[],[],min(remaining,.2))[0]:continue
        data=os.read(console.fd,8192)
        if not data:raise ValueError('local terminal ended before its expected prompt')
        console.buffer+=data;console.transcript+=data


def read_review(console, final):
    next_page=b'\r\n[Enter] Next page  [B] Return\r\n'
    pieces=[]
    for _ in range(8):
        token,raw=expect_one(console,[next_page,final]);pieces.append(raw)
        if token == final:break
        console.send('')
    else:raise ValueError('bounded local review did not finish')
    text=b''.join(pieces)
    pages=[tuple(map(int,match)) for match in re.findall(rb'\[LOCAL BRIEF REVIEW\] Page ([0-9]+)/([0-9]+)',text)]
    if not pages or pages != [(number,pages[-1][1]) for number in range(1,pages[-1][1]+1)]:
        raise ValueError('local review page sequence incomplete')
    return text


def canonical_snapshot(root, store, inspector, mail_fixture):
    root=owned_root(root);store=Path(store)
    path=store/'mail.fkv';info=path.lstat()
    if (not store.is_absolute() or not store.resolve().is_relative_to(root) or path.is_symlink()
            or not stat.S_ISREG(info.st_mode) or info.st_uid != os.getuid() or info.st_nlink != 1
            or stat.S_IMODE(info.st_mode) != 0o600 or not 0 < info.st_size <= 1024*1024):
        raise ValueError('owned regular encrypted canonical required')
    raw=path.read_bytes()
    copy_root=Path(tempfile.mkdtemp(prefix='inspection-copy-OWNED-',dir=root));copy_root.chmod(0o700)
    copied=copy_root/'mail.fkv';copied.write_bytes(raw);copied.chmod(0o600)
    (copy_root/'SCOPE').write_text('Exact synthetic canonical copy for read-only inspection; never restored or connected.\n')
    inspected=subprocess.run([str(inspector),'inspect',str(copy_root)],capture_output=True,timeout=15)
    if inspected.returncode or inspected.stderr or len(inspected.stdout) > 1024*1024:
        raise ValueError('owned copy inspection failed')
    state=json.loads(inspected.stdout)
    home=dict(store=copy_root,local_password='checkpoint synthetic fixture password')
    archive=json.loads(native_command(root,mail_fixture,'archive-status',home))
    if not archive.get('ready') or archive.get('checkpoint') is None:
        raise ValueError('warm archive token missing from canonical copy')
    state['archive']=archive['checkpoint']
    (copy_root/'inspection.private.json').write_text(json.dumps(state,sort_keys=True)+'\n')
    if path.read_bytes() != raw or copied.read_bytes() != raw:
        raise ValueError('canonical changed during controlled inspection')
    return state


def arm_brief(console, own, peer, device, identity, operation=None):
    console.raw(b'e');console.expect('[LOCAL BRIEF EXCHANGE]')
    review=read_review(console,b'\r\n[B] Return\r\n')
    panes=b''.join(line[2:] for line in review.splitlines() if line.startswith(b'| '))
    required=[('Home: '+own).encode(),('Peer: '+peer).encode(),
              ('Peer device: '+str(device)).encode(),identity.encode()]
    if any(value not in panes for value in required) or b'[R] Receive briefly' not in review:
        raise ValueError('current full peer review did not match trusted fixture context')
    confirm=b'\r\nType exchange to connect, or Enter: '
    if operation is not None:
        if b'[S] Send selected text' not in review:
            raise ValueError('selected ordinary ciphertext action unavailable')
        console.raw(b's')
        selected=read_review(console,confirm)
        panes=b''.join(line[2:] for line in selected.splitlines() if line.startswith(b'| '))
        if operation.encode() not in panes or b'Send selected text only.' not in panes:
            raise ValueError('selected immutable operation review mismatched')
        return review+selected
    console.raw(b'r');console.expect('Receive briefly; no text release.')
    console.expect('Type exchange to connect, or Enter: ')
    return review


def connect_pair(receiver, sender, relay, confirm):
    previous=relay.generations[1]
    confirm(receiver)
    generation=relay.wait_bound(1,after=previous,timeout=2)
    confirm(sender)
    return generation


def require_cancelled_confirmation(before_counts, after_counts, before_state, after_state):
    if after_counts != before_counts or after_state != before_state:
        raise ValueError('cancelled confirmation changed state or connected')


def run_journey(arguments):
    """One fresh classic OR Unicode journey; no automatic retry or remote board."""
    from smoke_native_client_ssh import NativeConsole
    from probe_mail_conversation import require_conversation_screen
    import probe_home_mail
    import probe_xmpp_transport
    import smoke_native_client_ssh
    parent=owned_root(arguments.owned_root)
    root=Path(tempfile.mkdtemp(prefix='unicode-OWNED-' if arguments.unicode else 'classic-OWNED-',dir=parent))
    root.chmod(0o700);(root/'OWNED').write_bytes(MARKER)
    local_password='checkpoint synthetic fixture password'
    home_passwords=[secrets.token_hex(20),secrets.token_hex(20)]
    token='fg-'+secrets.token_hex(5)
    seed=(token+' seed','seed-'+secrets.token_hex(6))
    older=[(token+' old'+str(index),'older-'+secrets.token_hex(6)) for index in (1,2)]
    live=(token+' live','live-'+secrets.token_hex(6)+' λ')
    other=('unselected-'+secrets.token_hex(6),'never-release-'+secrets.token_hex(6))
    private_inputs=dict(local_password=local_password,home_passwords=home_passwords,seed=seed,older=older,live=live,unselected=other)
    inputs_path=root/'inputs.private.json';inputs_path.write_text(json.dumps(private_inputs,ensure_ascii=False,indent=2)+'\n');inputs_path.chmod(0o600)
    binaries={name:Path(getattr(arguments,name)).resolve() for name in ('client','mail_fixture','inspector')}
    bindings=dict(producer=dict(path=str(Path(__file__).resolve()),sha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest()),
                  imports={module.__name__:hashlib.sha256(Path(module.__file__).read_bytes()).hexdigest() for module in (probe_home_mail,probe_xmpp_transport,smoke_native_client_ssh)},
                  binaries={name:dict(path=str(path),sha256=hashlib.sha256(path.read_bytes()).hexdigest()) for name,path in binaries.items()},
                  private_input_sha256=hashlib.sha256(inputs_path.read_bytes()).hexdigest())
    bound=root/'bindings.private.json';bound.write_text(json.dumps(bindings,sort_keys=True,indent=2)+'\n');bound.chmod(0o600)
    relay=Relay(root);homes=[];members=[];consoles=[];controls={};facts=None
    HOME='HOME account password (hidden; never sent to a BBS): '
    MENU='[L] Lock and exit';CONVERSATION='[LOCAL CONVERSATION]'
    def call(index,action,*args,password=False,lines=()):
        return native_command(root,binaries['mail_fixture'],action,members[index],*args,home_password=password,lines=lines)
    def snapshot(index):return canonical_snapshot(root,members[index]['store'],binaries['inspector'],binaries['mail_fixture'])
    def screen(console):
        start=len(console.transcript)-len(console.buffer)
        console.expect(CONVERSATION);console.expect('[B] Back')
        return console.transcript[start:len(console.transcript)-len(console.buffer)]
    def start(index):
        console=NativeConsole([str(binaries['client']),'--mail-online',str(members[index]['store']),str(members[index]['config'])]+(['--unicode'] if arguments.unicode else []))
        consoles.append(console);console.resize(40,24)
        console.password(local_password,'LOCAL checkpoint passphrase (hidden): ');console.expect(MENU)
        return console
    def conversation(console):
        console.raw(b'i');console.expect('[LOCAL INBOX]');console.expect('[B] Back')
        console.raw(b'c');console.expect('[LOCAL CORRESPONDENTS]');console.expect('[B] Back')
        console.raw(b'o');console.expect('[LOCAL CORRESPONDENT INBOX]');console.expect('[B] Back')
        console.raw(b's');console.expect('Search (64 bytes; blank clears): ');console.send(token)
        console.expect('Local search active.');console.expect('[B] Back')
        console.raw(b't');return screen(console)
    def return_menu(console):
        for label in ('[LOCAL CORRESPONDENT INBOX]','[LOCAL CORRESPONDENTS]','[LOCAL INBOX]'):
            console.raw(b'b');console.expect(label);console.expect('[B] Back')
        console.raw(b'b');console.expect(MENU)
    def confirm(console):
        console.send('exchange');console.password(home_passwords[1 if console is receiver else 0],HOME)
        console.expect('Connecting directly to your home.')
    def result(console,write,accepted,duplicates,stopped=False):
        start=len(console.transcript)-len(console.buffer)
        console.expect(write);console.expect(f'Accepted on this device: {accepted}.');console.expect(f'Duplicates: {duplicates}.')
        console.expect('Enter to return: ')
        raw=console.transcript[start:len(console.transcript)-len(console.buffer)]
        if (b'Brief exchange stopped.' in raw) != stopped:
            raise ValueError('foreground result stopped-state mismatch')
        console.send('');returned=screen(console)
        return raw,returned
    try:
        identities=[]
        for index,domain in enumerate(('home.test','away.test')):
            tls=root/domain;tls.mkdir(mode=0o700)
            pair,ca=certificates(tls,domain,arguments.openssl)
            home=LiveHome(root,index,relay,pair,home_passwords[index]);homes.append(home)
            config=root/('home-'+str(index)+'.toml')
            der=ssl.PEM_cert_to_DER_cert(pair[0].read_text())
            values=dict(version=1,account=ACCOUNTS[index],endpoint='127.0.0.1:'+str(home.port),domain=domain,ca=str(ca),pin=hashlib.sha256(der).hexdigest())
            config.write_text(''.join(key+'='+json.dumps(value)+'\n' for key,value in values.items()));config.chmod(0o600)
            members.append(dict(store=root/('alice' if index==0 else 'bob'),local_password=local_password,password=home_passwords[index],config=config))
            identity=call(index,'init',ACCOUNTS[index],DEVICES[index]).decode().strip()
            if not re.fullmatch('[a-f0-9]{64}',identity):raise ValueError('native public identity unavailable')
            identities.append(identity)
        if identities[0]==identities[1]:raise ValueError('independent native identity fixture required')
        for index in (0,1):
            call(index,'enroll',ACCOUNTS[1-index],DEVICES[1-index],identities[1-index])
            call(index,'enable-archive')
        call(1,'publish',members[1]['config'],password=True)
        call(0,'prepare',members[0]['config'],password=True)
        call(0,'queue',lines=seed);call(0,'sync',members[0]['config'],password=True)
        for index,expected in ((1,(1,0)),(0,(0,1))):
            page=json.loads(call(index,'catchup',members[index]['config'],password=True))
            if page != dict(entries=1,accepted=expected[0],duplicates=expected[1],complete=True):raise ValueError('warm native archive seed did not match controlled inventory')
        receiver=start(1)
        receiver.raw(b'e');receiver.expect('Type enable to add encrypted local read tracking: ');receiver.send('enable')
        receiver.expect('Encrypted local read tracking enabled.');receiver.expect('Enter to return: ');receiver.send('');receiver.expect(MENU)
        receiver.raw(b'i');receiver.expect('[LOCAL INBOX]');receiver.expect('[B] Back');receiver.raw(b'o')
        receiver.expect('Type read to acknowledge locally, or Enter to keep current state: ');receiver.send('read')
        receiver.expect('Acknowledged on this device only; no read receipt sent.');receiver.expect('[R] Reply locally  [B/Enter] Back to inbox')
        receiver.raw(b'b');receiver.expect('[LOCAL INBOX]');receiver.expect('[B] Back');receiver.raw(b'b');receiver.expect(MENU)
        before=snapshot(1)
        if len(before['history'])!=1 or before['history'][0]['read'] is not True:raise ValueError('explicit seed read decision missing')
        for content in older:call(0,'queue',lines=content)
        call(0,'sync',members[0]['config'],password=True)
        call(0,'queue',lines=live);call(0,'queue',lines=other)
        sender_before=snapshot(0);sender_bytes=(members[0]['store']/'mail.fkv').read_bytes()
        selected=next(row for row in sender_before['history'] if (row['subject'],row['body'])==live)
        if selected['direction']!='sent' or selected['device']!=202 or selected['identity']!=identities[1] or not any(row['operation']==selected['operation'] for row in sender_before['pending']):raise ValueError('fresh ordinary selected operation missing before attempt')
        sender=start(0)
        a_view=conversation(sender);b_view=conversation(receiver)
        require_conversation_screen(a_view,[3,4],4,(2,2),'All',True,202)
        require_conversation_screen(b_view,[1],1,(1,1),'All',True,101)
        # Blank confirmation cannot acquire a credential or start a connection.
        cancel_counts=(homes[1].connections,homes[1].authenticated)
        arm_brief(receiver,ACCOUNTS[1],ACCOUNTS[0],101,identities[0]);receiver.send('')
        require_conversation_screen(screen(receiver),[1],1,(1,1),'All',True,101)
        require_cancelled_confirmation(cancel_counts,(homes[1].connections,homes[1].authenticated),before,snapshot(1))
        controls['blank_confirmation_no_connection']=True
        arm_brief(receiver,ACCOUNTS[1],ACCOUNTS[0],101,identities[0])
        arm_brief(sender,ACCOUNTS[0],ACCOUNTS[1],202,identities[1],selected['operation'])
        message_start=len(relay.messages)
        generation=connect_pair(receiver,sender,relay,confirm)
        _,a_view=result(sender,'Selected text local write completed.',0,0)
        _,b_view=result(receiver,'No selected text write attempted.',1,0)
        require_conversation_screen(a_view,[3,4],4,(2,2),'All',True,202)
        require_conversation_screen(b_view,[1,2],1,(1,1),'All',True,101)
        if relay.messages[message_start:]!=[dict(actor=0,operation=selected['operation'],to=ACCOUNTS[1]+'/faction202',bound_generation=generation)]:raise ValueError('conversation sent more than its exact reviewed selected operation')
        if (members[0]['store']/'mail.fkv').read_bytes()!=sender_bytes:raise ValueError('sender pending ciphertext changed during foreground write')
        live_state=snapshot(1);require_live(before,live_state,*live)
        receiver.raw(b'x');receiver.expect('[LOCAL HOME EXCHANGE]');receiver.expect('[B] Return');receiver.raw(b'a')
        receiver.password(home_passwords[1],HOME)
        receiver.expect('Archive page saved. Entries: 3; accepted: 2; duplicates or known sent copies: 1.')
        receiver.expect('Enter to return: ');receiver.send('')
        require_conversation_screen(screen(receiver),[1,2],1,(1,2),'All',True,101)
        recovered=snapshot(1);require_recovery(live_state,recovered,older)
        if relay.queries[-1] != dict(actor=1,after=before['archive']['after'],ids=[ident for ident,_ in relay.inventory(1)[1:]]):raise ValueError('post-live explicit MAM scope or exact controlled inventory changed')
        # A closed own home must leave this local conversation useful and intact.
        homes[1].close();connections=homes[1].connections
        try:
            socket.create_connection(('127.0.0.1',homes[1].port),timeout=1)
        except ConnectionRefusedError:pass
        else:raise ValueError('closed owned home unexpectedly accepts connections')
        arm_brief(receiver,ACCOUNTS[1],ACCOUNTS[0],101,identities[0]);confirm(receiver)
        _,b_view=result(receiver,'No selected text write attempted.',0,0,stopped=True)
        require_conversation_screen(b_view,[1,2],1,(1,2),'All',True,101)
        if snapshot(1)!=recovered or homes[1].connections!=connections:raise ValueError('home outage changed retained state or accepted a connection')
        controls['home_outage_retained_state_and_view']=True
        for console in (sender,receiver):return_menu(console);console.raw(b'l');console.finish()
        markers=[local_password,*home_passwords,*(value for content in [seed,*older,live,other] for value in content)]
        if any(password.encode() in console.transcript for password in [local_password,*home_passwords] for console in consoles):raise ValueError('hidden credential reached a terminal capture')
        server_bytes=b''.join(path.read_bytes() for path in root.glob('home-*.private'))+b''.join(frame for index in (0,1) for _,frame in relay.inventory(index))
        if any(marker.encode() in server_bytes for marker in markers):raise ValueError('private plaintext reached home application capture or archive')
        for index in (0,1):
            if any(marker.encode() in (members[index]['store']/'mail.fkv').read_bytes() for marker in markers):raise ValueError('private marker found in encrypted canonical')
        facts=dict(unicode=arguments.unicode,tls_versions=[home.tls_versions[-1] for home in homes],selected_writes=1,
                   live_accepted=len(live_state['history'])-len(before['history']),live_duplicates=0,
                   archive_accepted=len(recovered['history'])-len(live_state['history']),archive_duplicates=1,
                   cursor_unchanged_after_live=live_state['archive']==before['archive'],read_decisions_preserved=recovered['history'][:len(live_state['history'])]==live_state['history'],
                   pending_preserved=recovered['pending']==before['pending'],exact_selected_operation=True,no_presence=all(home.failure is None for home in homes),
                   children_reaped=all(console.done for console in consoles),listeners_closed=False,server_plaintext_matches=0)
    except BaseException:
        import traceback
        failed=root/'failure.private.txt';failed.write_text(traceback.format_exc());failed.chmod(0o600)
        raise
    finally:
        for index,console in enumerate(consoles):
            path=root/('terminal-'+str(index)+'.private');path.write_bytes(console.transcript+console.buffer);path.chmod(0o600);console.close()
        failures=[]
        for home in homes:
            try:home.close()
            except BaseException as error:failures.append(type(error).__name__)
            if home.failure is not None:failures.append(home.failure)
        from probe_visit_mail import require_closed_endpoint
        closed_ports=[]
        for home in homes:
            try:require_closed_endpoint(('127.0.0.1',home.port));closed_ports.append(True)
            except (AssertionError,OSError):closed_ports.append(False);failures.append('listener-not-refused')
        teardown=dict(workers_joined=all(not home.thread.is_alive() for home in homes),listeners_closed=all(home.closed for home in homes),
                      child_processes_reaped=all(console.done for console in consoles),closed_ports_refused=closed_ports,failures=failures)
        path=root/'teardown.private.json';path.write_text(json.dumps(teardown,sort_keys=True)+'\n');path.chmod(0o600)
    if failures or not teardown['workers_joined'] or not teardown['listeners_closed']:raise ValueError('owned foreground fixture teardown or protocol failed')
    facts['listeners_closed']=True
    result=public_result(facts)
    (root/'controls.private.json').write_text(json.dumps(controls,sort_keys=True)+'\n')
    output=root/'result.private.json';output.write_text(json.dumps(result,sort_keys=True,indent=2)+'\n');output.chmod(0o600)
    return result


def main():
    import argparse
    import shutil
    parser=argparse.ArgumentParser(description=__doc__)
    for name in ('owned-root','client','mail-fixture','inspector'):parser.add_argument('--'+name,type=Path,required=True)
    parser.add_argument('--openssl',default=shutil.which('openssl'));parser.add_argument('--unicode',action='store_true')
    arguments=parser.parse_args()
    result=run_journey(arguments)
    print('Foreground production UI synthetic TLS journey passed:', 'Unicode' if result['unicode'] else 'classic')


if __name__=='__main__':
    try:main()
    except (ValueError,OSError,AssertionError,TimeoutError,subprocess.SubprocessError):
        raise SystemExit('Foreground fixture failed; private diagnostics retained.') from None
