#!/usr/bin/env python3
"""Offline synthetic OMEMO-2 qualification; no product runtime or live accounts."""
import argparse
import asyncio
import copy
import hashlib
import json
import os
import re
import secrets
import select
import subprocess
import tempfile
from datetime import datetime, timezone
from pathlib import Path
from xml.etree import ElementTree as ET

OMEMO = 'urn:xmpp:omemo:2'
SCE = 'urn:xmpp:sce:1'
CLIENT = 'jabber:client'
PROFILE = {'omemo': 'XEP-0384 0.9.1', 'sce': 'XEP-0420 0.5.0',
           'timestamp': 'XEP-0082 1.1.1', 'namespace': OMEMO, 'sce_namespace': SCE}
STAMP = '2026-10-24T12:00:00Z'
LIMIT = 8192


def stamp(value):
    if not isinstance(value, str) or not re.fullmatch(r'\d{4}-\d\d-\d\dT\d\d:\d\d:\d\d(?:\.\d+)?(?:Z|[+-]\d\d:\d\d)', value):
        raise ValueError('timestamp requires an explicit zone')
    try:
        result = datetime.fromisoformat(value.replace('Z', '+00:00'))
    except ValueError:
        raise ValueError('invalid timestamp') from None
    return result


def pack_sce(sender, recipient, subject, body, sent_at=STAMP, padding=None):
    stamp(sent_at)
    if not subject or len(subject.encode()) > 128 or not body or len(body.encode()) > 1024:
        raise ValueError('bounded nonempty subject/body required')
    root = ET.Element('{'+SCE+'}envelope')
    content = ET.SubElement(root, '{'+SCE+'}content')
    ET.SubElement(content, '{'+CLIENT+'}subject').text = subject
    ET.SubElement(content, '{'+CLIENT+'}body').text = body
    ET.SubElement(root, '{'+SCE+'}from', jid=sender)
    ET.SubElement(root, '{'+SCE+'}to', jid=recipient)
    ET.SubElement(root, '{'+SCE+'}time', stamp=sent_at)
    pad = ET.SubElement(root, '{'+SCE+'}rpad')
    if padding is None:
        # Random-content padding; not an encoding requirement on received padding.
        minimum = max(1, 2048-len(ET.tostring(root)))
        padding = ''.join(secrets.choice('abcdefghijklmnopqrstuvwxyz0123456789!')
                          for _ in range(minimum+secrets.randbelow(201)))
    pad.text = padding
    raw = ET.tostring(root, encoding='utf-8')
    if len(raw) > LIMIT:
        raise ValueError('envelope exceeds proof cap')
    return raw


def unpack_sce(raw, sender, recipient, sent_at=STAMP):
    if len(raw) > LIMIT:
        raise ValueError('envelope exceeds proof cap')
    try:
        text = raw.decode('utf-8')
        if '<!' in text or '<?' in text:
            raise ValueError('unsafe envelope declaration')
        root = ET.fromstring(text)
    except (UnicodeError, ET.ParseError):
        raise ValueError('malformed envelope') from None
    names = {'{'+SCE+'}'+n for n in ['content', 'from', 'to', 'time', 'rpad']}
    if root.tag != '{'+SCE+'}envelope' or root.attrib or len(root) != 5 or {x.tag for x in root} != names:
        raise ValueError('unexpected envelope shape')
    for name, jid in [('from', sender), ('to', recipient)]:
        node = root.find('{'+SCE+'}'+name)
        if node.attrib != {'jid': jid} or len(node) or (node.text or '').strip():
            raise ValueError('wrong sender/recipient binding')
    node = root.find('{'+SCE+'}time')
    if set(node.attrib) != {'stamp'} or len(node) or (node.text or '').strip():
        raise ValueError('unexpected timestamp shape')
    if abs((stamp(node.get('stamp'))-stamp(sent_at)).total_seconds()) > 300:
        raise ValueError('timestamp disagrees with enclosing stanza')
    pad = root.find('{'+SCE+'}rpad')
    if pad.attrib or len(pad) or not pad.text:
        raise ValueError('missing padding')
    content = root.find('{'+SCE+'}content')
    if content.attrib or len(content) != 2 or {n.tag for n in content} != {'{'+CLIENT+'}subject', '{'+CLIENT+'}body'}:
        raise ValueError('unexpected private content')
    if any(n.attrib or len(n) for n in content):
        raise ValueError('nested private content')
    subject, body = content.findtext('{'+CLIENT+'}subject'), content.findtext('{'+CLIENT+'}body')
    if not subject or len(subject.encode()) > 128 or not body or len(body.encode()) > 1024:
        raise ValueError('bounded nonempty subject/body required')
    return subject, body


def pin_bytes(value):
    if not isinstance(value, str) or not re.fullmatch(r'ed25519:[0-9a-f]{64}', value):
        raise ValueError('explicit Ed25519 wire identity required; no legacy migration')
    return bytes.fromhex(value[8:])


def parse_reply(raw):
    if len(raw) > 131072 or not raw.endswith(b'\n'):
        raise ValueError('bad native response size')
    try:
        line = raw[:-1].decode('ascii')
    except UnicodeError:
        raise ValueError('non-ASCII native response') from None
    if not re.fullmatch(r'ok(?: [a-z0-9:._-]+)*', line):
        raise ValueError('native operation refused or protocol mismatch')
    return line.split()[1:]


class Native:
    """One serial device in an owned pipe; secret state never enters argv/logs."""
    def __init__(self, argv, diagnostic):
        self.argv, self.diagnostic = list(argv), diagnostic
        self.log = open(diagnostic, 'ab')
        os.chmod(diagnostic, 0o600)
        self.process = subprocess.Popen(self.argv, stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                                        stderr=self.log, bufsize=0)
        self.buffer = bytearray()
        self.bundle = None
        self.peer = None

    def call(self, *fields):
        line = ' '.join(str(f) for f in fields).encode('ascii')+b'\n'
        if len(line) > 131072:
            raise ValueError('native request cap')
        self.process.stdin.write(line); self.process.stdin.flush()
        while b'\n' not in self.buffer:
            if not select.select([self.process.stdout], [], [], 15)[0]:
                raise TimeoutError('native fixture timeout')
            chunk = os.read(self.process.stdout.fileno(), 65536)
            if not chunk:
                raise ValueError('native fixture exited')
            self.buffer.extend(chunk)
            if len(self.buffer) > 131072:
                raise ValueError('native response cap')
        line, _, tail = self.buffer.partition(b'\n'); self.buffer = bytearray(tail)
        return parse_reply(bytes(line)+b'\n')

    def export(self):
        result = self.call('export')
        if len(result) != 1:
            raise ValueError('native snapshot shape')
        return result[0]

    def close(self):
        if self.process.poll() is None:
            self.process.stdin.close()
            try:self.process.wait(timeout=5)
            except subprocess.TimeoutExpired:self.process.kill();self.process.wait(timeout=5)
        self.process.stdout.close();self.log.close()


def bundle_fields(fields):
    if len(fields) != 6:
        raise ValueError('native bundle shape')
    ik,spk,sig=map(bytes.fromhex,fields[:3]);pk=bytes.fromhex(fields[4])
    if [len(ik),len(spk),len(sig),len(pk)] != [32,32,64,32]:
        raise ValueError('native bundle bounds')
    spkid,pkid=int(fields[3]),int(fields[5])
    if not (1 <= spkid <= 2147483647 and 1 <= pkid <= 2147483647):
        raise ValueError('native bundle identifier bounds')
    return dict(ik=ik,spk=spk,sig=sig,spkid=spkid,pk=pk,pkid=pkid)


def bundle_tokens(b):
    return [b['ik'].hex(),b['spk'].hex(),b['sig'].hex(),str(b['spkid']),b['pk'].hex(),str(b['pkid'])]


def enroll_native(native,jid,device,pin):
    pin_bytes(pin)
    native.call('trust',pin)
    native.peer=(jid,device,pin)


def native_seal(native,raw,identity):
    fields=native.call('seal',raw.hex())
    if len(fields)!=3 or fields[1] not in ['0','1']:
        raise ValueError('native sealed shape')
    return dict(namespace=OMEMO,sender='native@a.test',recipient='reference@b.test',device=101,
                identity=identity,key=bytes.fromhex(fields[0]),prekey=fields[1]=='1',
                payload=bytes.fromhex(fields[2]),sent_at=STAMP)


def native_open(native,frame,pin):
    jid,device,value=pin
    if native.peer!=pin or frame['namespace']!=OMEMO or (frame['sender'],frame['device'],frame['identity'])!=(jid,device,pin_bytes(value)) or frame['recipient']!='native@a.test':
        raise ValueError('unverified device or wrong profile/destination')
    fields=native.call('open',frame['key'].hex() or '-',int(frame['prekey']),frame['payload'].hex() or '-')
    try:
        if len(fields)!=1:
            raise ValueError('native plaintext shape')
        result=unpack_sce(bytes.fromhex(fields[0]),frame['sender'],frame['recipient'],frame['sent_at'])
    except Exception:
        native.call('abort')
        raise
    native.call('commit')
    return result


class Reference:
    """Research-only independent implementation; copies state before acceptance."""
    def __init__(self,jid,device,data=None):
        self.jid,self.device,self.data=jid,device,copy.deepcopy(data or {})
        self.pins={};self.peer_bundle=None;self.identity=None

    def backend(self):
        from omemo.storage import Storage,Just,Nothing
        from twomemo import twomemo
        class Memory(Storage):
            def __init__(self,data):
                super().__init__(disable_cache=True);self.data=copy.deepcopy(data)
            async def _load(self,key):
                return Just(copy.deepcopy(self.data[key])) if key in self.data else Nothing()
            async def _store(self,key,value):self.data[key]=copy.deepcopy(value)
            async def _delete(self,key):self.data.pop(key,None)
        storage=Memory(self.data)
        return twomemo.Twomemo(storage,max_num_per_session_skipped_keys=32),storage

    @classmethod
    async def create(cls,jid,device):
        self=cls(jid,device);backend,storage=self.backend()
        await backend.generate_pre_keys(100)
        bundle=await backend.get_bundle(jid,device)
        self.identity=bundle.identity_key;self.data=storage.data
        return self

    async def public_bundle(self):
        backend,storage=self.backend();b=await backend.get_bundle(self.jid,self.device)
        self.data=storage.data
        pk=min(b.pre_key_ids,key=lambda k:b.pre_key_ids[k])
        return dict(ik=b.identity_key,spk=b.bundle.signed_pre_key,sig=b.bundle.signed_pre_key_sig,
                    spkid=b.signed_pre_key_id,pk=pk,pkid=b.pre_key_ids[pk])

    def enroll(self,jid,device,value):
        key=pin_bytes(value)
        previous=self.pins.get((jid,device))
        if previous is not None and previous!=key:raise ValueError('changed verified device')
        self.pins[jid,device]=key

    def snapshot(self):return json.dumps(self.data,sort_keys=True,separators=(',',':'))

    async def restart(self):
        # New backend/cache on every call; JSON round trip exercises persisted state.
        self.data=json.loads(self.snapshot())
        backend,_=self.backend();assert await backend.load_session('native@a.test',101) is not None

    async def seal(self,raw):
        import x3dh
        from twomemo import twomemo as t
        backend,storage=self.backend()
        session=await backend.load_session('native@a.test',101)
        content,pkey=await backend.encrypt_plaintext(raw)
        if session is None:
            b=self.peer_bundle
            if self.pins.get(('native@a.test',101))!=b['ik']:raise ValueError('unverified bundle')
            bundle=t.BundleImpl('native@a.test',101,x3dh.Bundle(b['ik'],b['spk'],b['sig'],{b['pk']}),b['spkid'],{b['pk']:b['pkid']})
            session,key=await backend.build_session_active('native@a.test',101,bundle,pkey)
        else:key=await backend.encrypt_key_material(session,pkey)
        prekey=session.initiation==t.Initiation.ACTIVE and not session.confirmed
        wire=key.serialize()
        if prekey:wire=session.key_exchange.serialize(wire)
        await backend.store_session(session);self.data=storage.data
        return dict(namespace=OMEMO,sender=self.jid,recipient='native@a.test',device=self.device,
                    identity=self.identity,key=wire,prekey=prekey,payload=content.ciphertext,sent_at=STAMP)

    async def open(self,frame):
        from twomemo import twomemo as t
        if frame['namespace']!=OMEMO or frame['recipient']!=self.jid or self.pins.get((frame['sender'],frame['device']))!=frame['identity']:
            raise ValueError('unverified sender/device/profile/destination')
        backend,storage=self.backend()
        session=await backend.load_session(frame['sender'],frame['device'])
        if frame['prekey']:
            exchange,wire=t.KeyExchangeImpl.parse(frame['key'])
            if exchange.identity_key!=frame['identity']:raise ValueError('prekey identity differs from verified device')
            encrypted=t.EncryptedKeyMaterialImpl.parse(wire,frame['sender'],frame['device'])
            if session is not None and session.key_exchange.builds_same_session(exchange):
                pkey=await backend.decrypt_key_material(session,encrypted)
            else:session,pkey=await backend.build_session_passive(frame['sender'],frame['device'],exchange,encrypted)
        else:
            if session is None:raise ValueError('missing session')
            encrypted=t.EncryptedKeyMaterialImpl.parse(frame['key'],frame['sender'],frame['device'])
            pkey=await backend.decrypt_key_material(session,encrypted)
        raw=await backend.decrypt_plaintext(t.ContentImpl(frame['payload']),pkey)
        result=unpack_sce(raw,frame['sender'],frame['recipient'],frame['sent_at'])
        await backend.store_session(session);self.data=storage.data
        return result


async def run_matrix(argv, private_dir):
    checks = []
    native = None
    import xeddsa
    vectors=[('d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a','d85e07ec22b0ad881537c2f44d662d1a143cf830c57aca4305d85c7a90f6b62e'),('03a107bff3ce10be1d70dd18e74bc09967e4d6309ba50d5f1ddc8664125531b8','4701d08488451f545a409fb58ae3e58581ca40ac3f7f114698cd71deac73ca01')]
    for ed,cv in vectors:
        n=Native(argv,private_dir/'native-diagnostic.log')
        try:
            assert n.call('convert',ed)==[cv]
            assert xeddsa.ed25519_pub_to_curve25519_pub(bytes.fromhex(ed)).hex()==cv
            assert xeddsa.curve25519_pub_to_ed25519_pub(bytes.fromhex(cv),bool(int(ed[-2:],16)&128)).hex()==ed
            flipped=bytearray.fromhex(ed);flipped[-1]^=128
            assert n.call('convert',flipped.hex())==[cv]
            n.call('new');n.call('trust','ed25519:'+ed)
            before=n.export()
            for value in ['ed25519:'+flipped.hex(),'curve25519:'+cv]:
                try:n.call('trust',value)
                except ValueError:assert n.export()==before
                else:raise AssertionError('ambiguous legacy/changed-sign identity trusted')
            checks.append('independent_ed_curve_sign_'+str(int(ed[-2:],16)>>7)+'_vector_and_no_implicit_migration')
        finally:n.close()
    async def check_rejection(label, operation, snapshot):
        before = snapshot()
        try:
            await operation()
        except Exception:
            if snapshot() != before:
                raise AssertionError('rejected operation changed committed state')
        else:
            raise AssertionError('negative input was accepted')
        checks.append(label)

    for initiator in ['native', 'reference']:
        native = Native(argv, private_dir/'native-diagnostic.log')
        try:
            nb = bundle_fields(native.call('new'))
            reference = await Reference.create('reference@b.test', 202)
            native_pin = ('native@a.test', 101, 'ed25519:'+nb['ik'].hex())
            reference.enroll(*native_pin)
            rb = await reference.public_bundle()
            peer_pin = ('reference@b.test', 202, 'ed25519:'+rb['ik'].hex())
            enroll_native(native, *peer_pin)
            expected = ('Café <&>', '雪 & a late signal')
            envelope = pack_sce('native@a.test', 'reference@b.test', *expected)
            if initiator == 'native':
                native.call('start', *bundle_tokens(rb))
                first = native_seal(native, envelope, nb['ik'])
                assert await reference.open(first) == expected
                reply = await reference.seal(pack_sce('reference@b.test','native@a.test',*expected))
                assert native_open(native, reply, peer_pin) == expected
            else:
                reference.peer_bundle = nb
                first = await reference.seal(pack_sce('reference@b.test','native@a.test',*expected))
                assert native_open(native, first, peer_pin) == expected
                reply = native_seal(native, envelope, nb['ik'])
                assert await reference.open(reply) == expected
            checks.append(initiator+'_initiates_first_and_reciprocal_reply_sce')
            # Both directions now have an established, confirmed ratchet.
            nframes = [native_seal(native,envelope,nb['ik']) for _ in range(3)]
            for index in [2,0,1]:assert await reference.open(nframes[index]) == expected
            checks.append(initiator+'_reference_accepts_out_of_order_2_0_1')
            rframes = [await reference.seal(pack_sce('reference@b.test','native@a.test',*expected)) for _ in range(3)]
            # Preserve private state through an actual native process restart.
            snapshot = native.export();native.close();native=Native(argv, private_dir/'native-diagnostic.log')
            native.call('import',snapshot);native.peer=peer_pin
            for index in [2,0,1]:assert native_open(native,rframes[index],peer_pin) == expected
            checks.append(initiator+'_native_restart_accepts_out_of_order_2_0_1')
            await reference.restart()
            checks.append(initiator+'_reference_storage_restart')
            for index in [2,0,1]:
                await check_rejection(initiator+'_native_replay_'+str(index),
                    async_call(lambda:native_open(native,rframes[index],peer_pin)), native.export)
                await check_rejection(initiator+'_reference_replay_'+str(index),
                    lambda index=index:reference.open(nframes[index]), reference.snapshot)
            # Tamper/truncate authenticated bytes; every rejection preserves state.
            valid_r=await reference.seal(pack_sce('reference@b.test','native@a.test',*expected))
            valid_n=native_seal(native,envelope,nb['ik'])
            for target,valid,snapshot,operation in [('native',valid_r,native.export,lambda f:native_open(native,f,peer_pin)),
                                                     ('reference',valid_n,reference.snapshot,reference.open)]:
                for field in ['key','payload']:
                    for mutation in ['tamper','truncate']:
                        changed=dict(valid);raw=changed[field]
                        changed[field]=(bytes([raw[0]^1])+raw[1:]) if mutation=='tamper' else raw[:1]
                        op=async_call(lambda changed=changed,operation=operation:operation(changed)) if target=='native' else lambda changed=changed:reference.open(changed)
                        await check_rejection(initiator+'_'+target+'_'+field+'_'+mutation,op,snapshot)
                for field,value in [('namespace','urn:xmpp:omemo:0'),('sender','other@b.test' if target=='native' else 'other@a.test'),
                                    ('device',203),('identity',bytes(32))]:
                    changed=dict(valid);changed[field]=value
                    op=async_call(lambda changed=changed,operation=operation:operation(changed)) if target=='native' else lambda changed=changed:reference.open(changed)
                    await check_rejection(initiator+'_'+target+'_refuses_changed_'+field,op,snapshot)
            assert native_open(native,valid_r,peer_pin)==expected
            assert await reference.open(valid_n)==expected
            checks.append(initiator+'_valid_after_rejected_crypto_metadata')
            # Validly encrypted but semantically invalid SCE must abort proposed state.
            for target in ['native','reference']:
                sender='reference@b.test' if target=='native' else 'native@a.test'
                recipient='native@a.test' if target=='native' else 'reference@b.test'
                for defect in ['sender','recipient','time','namespace']:
                    raw=pack_sce(sender,recipient,*expected)
                    raw=raw.replace(sender.encode(),b'evil@x.test') if defect=='sender' else raw
                    raw=raw.replace(recipient.encode(),b'evil@y.test') if defect=='recipient' else raw
                    raw=raw.replace(b'12:00:00Z',b'12:30:00Z') if defect=='time' else raw
                    raw=raw.replace(b'urn:xmpp:sce:1',b'urn:xmpp:sce:0') if defect=='namespace' else raw
                    frame=await reference.seal(raw) if target=='native' else native_seal(native,raw,nb['ik'])
                    op=async_call(lambda:native_open(native,frame,peer_pin)) if target=='native' else lambda:reference.open(frame)
                    await check_rejection(initiator+'_'+target+'_sce_'+defect+'_aborts_state',op,
                                          native.export if target=='native' else reference.snapshot)
                follow=pack_sce(sender,recipient,*expected)
                frame=await reference.seal(follow) if target=='native' else native_seal(native,follow,nb['ik'])
                assert (native_open(native,frame,peer_pin) if target=='native' else await reference.open(frame))==expected
            checks.append(initiator+'_valid_successors_after_sce_aborts')
            # Explicit legacy or changed enrollment cannot overwrite the verified pin.
            for pin in ['curve25519:'+rb['ik'].hex(),'ed25519:'+bytes(32).hex()]:
                await check_rejection(initiator+'_rejects_pin_'+pin.split(':')[0],
                                      async_call(lambda pin=pin:native.call('trust',pin)),native.export)
            # More than 32 skipped keys must not consume the receiver transaction.
            distant=[await reference.seal(pack_sce('reference@b.test','native@a.test',*expected)) for _ in range(34)]
            await check_rejection(initiator+'_native_skip_cap_rejects_without_state_change',
                                  async_call(lambda:native_open(native,distant[-1],peer_pin)),native.export)
            assert native_open(native,distant[0],peer_pin)==expected
            checks.append(initiator+'_valid_low_counter_after_skip_cap')
        finally:
            if native:native.close();native=None
    return checks


async def run_baseline(argv,private_dir):
    outcomes=[]
    for initiator in ['native','reference']:
        n=Native(argv,private_dir/'native-diagnostic.log')
        try:
            nb=bundle_fields(n.call('new'));r=await Reference.create('reference@b.test',202)
            r.enroll('native@a.test',101,'ed25519:'+nb['ik'].hex());r.peer_bundle=nb
            rb=await r.public_bundle();pin=('reference@b.test',202,'ed25519:'+rb['ik'].hex());enroll_native(n,*pin)
            expected=('Independent first message','Reply qualification')
            if initiator=='native':
                n.call('start',*bundle_tokens(rb))
                first=native_seal(n,pack_sce('native@a.test','reference@b.test',*expected),nb['ik'])
                assert await r.open(first)==expected
                reply=await r.seal(pack_sce('reference@b.test','native@a.test',*expected))
                before=n.export()
                try:native_open(n,reply,pin)
                except ValueError:assert n.export()==before
                else:raise AssertionError('baseline unexpectedly accepted reference reply')
            else:
                first=await r.seal(pack_sce('reference@b.test','native@a.test',*expected))
                assert native_open(n,first,pin)==expected
                reply=native_seal(n,pack_sce('native@a.test','reference@b.test',*expected),nb['ik'])
                before=r.snapshot()
                try:await r.open(reply)
                except Exception:assert r.snapshot()==before
                else:raise AssertionError('baseline unexpectedly accepted native reply')
            outcomes.append(initiator+'_first_sce_decrypts_but_reply_authentication_rejects_without_commit')
        finally:n.close()
    return outcomes


def async_call(call):
    async def invoke():return call()
    return invoke


async def main(a):
    argv=[str(a.native.resolve(strict=True))] if a.native else json.loads(a.native_argv_json.read_text())
    if not isinstance(argv,list) or not argv or not all(isinstance(x,str) for x in argv):
        raise ValueError('trusted native argv required')
    with tempfile.TemporaryDirectory(prefix='faction200-interop-') as tmp:
        path=Path(tmp);path.chmod(0o700)
        checks=await (run_baseline(argv,path) if a.rejected_baseline else run_matrix(argv,path))
        diagnostic=path/'native-diagnostic.log'
        reports=re.findall(r'(\d+)\s+maximum resident set size',diagnostic.read_text(errors='replace')) if diagnostic.exists() else []
    record={'schema':1,'result':'rejected' if a.rejected_baseline else 'pass','checks':checks,'count':len(checks),'profile':PROFILE,
            'generated_native_library':True,'independent_two_way':not a.rejected_baseline,
            'product_adoption':False,'live_xmpp':False,'synthetic_fixture_removed':True,
            'native_maximum_resident_size_raw':list(map(int,reports)),
            'rss_unit':a.rss_unit}
    a.output.write_text(json.dumps(record,indent=2)+'\n')
    print('Offline native/reference result:',record['result'],len(checks),'controls.')


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    group=parser.add_mutually_exclusive_group(required=True)
    group.add_argument('--native',type=Path);group.add_argument('--native-argv-json',type=Path)
    parser.add_argument('--output',required=True,type=Path)
    parser.add_argument('--rejected-baseline',action='store_true')
    parser.add_argument('--rss-unit',choices=['bytes','KiB','not-measured'],default='not-measured')
    parser.add_argument('--disposable-synthetic-fixture',action='store_true',required=True)
    args=parser.parse_args()
    try:asyncio.run(main(args))
    except Exception as error:
        import traceback
        args.output.with_suffix('.private-error').write_text(traceback.format_exc())
        os.chmod(args.output.with_suffix('.private-error'),0o600)
        print('Offline interoperability failed:',type(error).__name__)
        raise SystemExit(1)
