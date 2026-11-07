"""Isolated foreground fixture/oracle contracts; production journeys are separate."""
import unittest
import json
import probe_foreground_conversation as proof


class Interface(unittest.TestCase):
    def test_isolated_fixture_module_is_available(self):
        self.assertTrue(proof.__doc__)

import copy
import tempfile
from pathlib import Path
from xml.etree import ElementTree as ET

A, B = 'alice@home.test', 'bob@away.test'
NS, OMEMO, MAM, RSM = 'jabber:client', 'urn:xmpp:omemo:2', 'urn:xmpp:mam:2', 'http://jabber.org/protocol/rsm'


def message(operation='11'*16, full=True):
    to = B + ('/faction202' if full else '')
    return (f"<message xmlns='{NS}' from='{A}' to='{to}' id='{operation}' type='chat'>"
            f"<encrypted xmlns='{OMEMO}'><header sid='101'><keys jid='{B}'><key rid='202'>AQID</key></keys></header>"
            "<payload>" + 'BQ=='*8 + "</payload></encrypted><store xmlns='urn:xmpp:hints'/></message>").encode()


def query(after=None, maximum=4, own=B):
    return (f"<iq xmlns='{NS}' type='set' id='iq-test' to='{own}'><query xmlns='{MAM}' queryid='query-test'>"
            f"<x xmlns='jabber:x:data' type='submit'><field var='FORM_TYPE' type='hidden'><value>{MAM}</value></field>"
            f"<field var='with'><value>{A}</value></field></x><set xmlns='{RSM}'><max>{maximum}</max>"
            + (f'<after>{after}</after>' if after else '') + '</set></query></iq>').encode()


class RelayBehavior(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory(prefix='foreground-relay-OWNED-')
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name).resolve(); self.root.chmod(0o700)
        (self.root/'OWNED').write_bytes(b'Owned issue356 foreground conversation fixture\n')
        self.relay = proof.Relay(self.root)

    def test_fresh_bound_generation_is_required_and_stale_unbind_is_inert(self):
        frames = []
        first = self.relay.register(1, B+'/faction202', frames.append)
        self.assertGreater(first, 0)
        self.assertEqual(self.relay.wait_bound(1, after=first-1, timeout=.01), first)
        with self.assertRaises(TimeoutError): self.relay.wait_bound(1, after=first, timeout=.01)
        self.relay.unregister(1, first)
        second = self.relay.register(1, B+'/faction202', frames.append)
        self.assertGreater(second, first)
        self.relay.unregister(1, first)
        self.assertEqual(self.relay.wait_bound(1, after=first, timeout=.01), second)

    def test_binding_rejects_wrong_account_resource_and_second_live_connection(self):
        for full in [A+'/faction202', B, B+'/faction999']:
            with self.subTest(full=full), self.assertRaises(ValueError): self.relay.register(1, full, lambda raw:None)
        self.relay.register(1, B+'/faction202', lambda raw:None)
        with self.assertRaises(ValueError): self.relay.register(1, B+'/faction202', lambda raw:None)

    def test_exact_full_route_delivers_opaque_ciphertext_with_qualified_own_sid(self):
        frames = []
        self.relay.register(1, B+'/faction202', frames.append)
        self.assertTrue(self.relay.message(0, message()))
        self.assertEqual(len(frames), 1)
        arrived = ET.fromstring(frames[0]); source = ET.fromstring(message())
        self.assertEqual(arrived.attrib, dict(source.attrib, **{'from':A+'/faction101'}))
        self.assertEqual(ET.tostring(arrived[0]), ET.tostring(source[0]))
        self.assertEqual([e.tag for e in arrived], ['{'+OMEMO+'}encrypted','{urn:xmpp:hints}store','{urn:xmpp:sid:0}stanza-id'])
        self.assertEqual(arrived[2].get('by'), B)
        self.assertEqual(arrived[2].get('id'), self.relay.inventory(1)[0][0])
        self.assertEqual(len(self.relay.inventory(0)), 1)
        self.assertEqual(len(self.relay.inventory(1)), 1)

    def test_bare_send_is_archive_only_and_exact_retry_preserves_inventory(self):
        frames = []
        self.relay.register(1, B+'/faction202', frames.append)
        self.assertFalse(self.relay.message(0, message(full=False)))
        self.assertEqual(frames, [])
        before = self.relay.inventory(1)
        self.assertTrue(self.relay.message(0, message()))
        self.assertEqual(self.relay.inventory(1), before)
        with self.assertRaises(ValueError): self.relay.message(0, message().replace(b'AQID', b'BAUG'))

    def test_wrong_routing_plaintext_metadata_presence_and_oversize_are_rejected(self):
        for raw in [message().replace(b'/faction202', b'/faction999'), message().replace(A.encode(), b'eve@home.test'),
                    message().replace(b'</message>', b'<body>private fallback</body></message>'),
                    message().replace(b'</message>', b"<stanza-id xmlns='urn:xmpp:sid:0' by='evil' id='x'/></message>"),
                    b"<presence xmlns='jabber:client'/>", b'x'*16385]:
            with self.subTest(raw=raw[:60]), self.assertRaises(ValueError): self.relay.message(0, raw)
        self.assertEqual(self.relay.inventory(1), [])

    def test_scoped_mam_pages_are_four_then_one_with_exact_original_wires(self):
        for number in range(1, 6): self.relay.message(0, message(f'{number:032x}', full=False))
        inventory = self.relay.inventory(1)
        raw = self.relay.request(1, query())
        wrapper = ET.fromstring(b'<fixture>'+raw+b'</fixture>')
        self.assertEqual(len(wrapper), 5)
        for index, frame in enumerate(wrapper[:4]):
            result = frame.find('{'+MAM+'}result')
            self.assertEqual(result.get('id'), inventory[index][0]);self.assertEqual(result.get('queryid'), 'query-test')
            forwarded = result.find('{urn:xmpp:forward:0}forwarded')
            self.assertEqual(ET.tostring(forwarded[1]), inventory[index][1])
        fin = wrapper[-1].find('{'+MAM+'}fin');self.assertEqual(fin.get('complete'), 'false')
        self.assertEqual(fin.find('{'+RSM+'}set/{'+RSM+'}last').text, inventory[3][0])
        later = ET.fromstring(b'<fixture>'+self.relay.request(1, query(inventory[3][0]))+b'</fixture>')
        self.assertEqual(len(later), 2)
        self.assertEqual(later[-1].find('{'+MAM+'}fin').get('complete'), 'true')
        with self.assertRaises(ValueError): self.relay.request(1, query('foreign-position'))
        with self.assertRaises(ValueError): self.relay.request(1, query(maximum=5))
        with self.assertRaises(ValueError): self.relay.request(1, query(own=A))


def record(subject='seed', number=1, read=True):
    return dict(direction='received',address=A,device=101,identity='01'*32,subject=subject,body=subject+' body',sent_at=42,
                operation='00'*16,received_id=f'{number:064x}',read=read)


def snapshot():
    return dict(history=[record()],pending=[dict(operation='aa'*16,address=A,device=101,sent_at=42)],
                archive=dict(home=B,local_device=202,epoch='12'*16,after='seed-position'))


def facts():
    return dict(unicode=False,tls_versions=['TLSv1.3','TLSv1.3'],selected_writes=1,live_accepted=1,live_duplicates=0,
                archive_accepted=2,archive_duplicates=1,cursor_unchanged_after_live=True,read_decisions_preserved=True,
                pending_preserved=True,exact_selected_operation=True,no_presence=True,children_reaped=True,
                listeners_closed=True,server_plaintext_matches=0)


class ResultOracles(unittest.TestCase):
    def test_live_oracle_rejects_false_acceptance_read_change_cursor_or_outbox_change(self):
        before = snapshot(); after = copy.deepcopy(before); after['history'].append(record('live',2,False))
        proof.require_live(before, after, 'live', 'live body')
        candidates = [before, dict(after,history=after['history']+[record('extra',3,False)]),
                      dict(after,archive=dict(after['archive'],after='advanced')),dict(after,pending=[])]
        changed = copy.deepcopy(after);changed['history'][0]['read']=False;candidates.append(changed)
        changed = copy.deepcopy(after);changed['history'][-1]['read']=True;candidates.append(changed)
        changed = copy.deepcopy(after);changed['history'][-1]['identity']='ab'*32;candidates.append(changed)
        for bad in candidates:
            with self.subTest(bad=bad), self.assertRaises(ValueError): proof.require_live(before,bad,'live','live body')

    def test_recovery_oracle_requires_older_inventory_exact_dedup_and_original_read_decisions(self):
        before = snapshot();before['history'].append(record('live',4,False))
        after = copy.deepcopy(before);after['history'] += [record('old1',2,False),record('old2',3,False)]
        after['archive']['after']='live-position'
        proof.require_recovery(before,after,[('old1','old1 body'),('old2','old2 body')])
        candidates=[before,dict(after,history=after['history']+[record('live',4,False)]),dict(after,pending=[]),
                    dict(after,archive=dict(after['archive'],epoch='ef'*16))]
        changed=copy.deepcopy(after);changed['history'][0]['read']=False;candidates.append(changed)
        for bad in candidates:
            with self.subTest(bad=bad), self.assertRaises(ValueError):proof.require_recovery(before,bad,[('old1','old1 body'),('old2','old2 body')])

    def test_public_result_requires_all_observations_and_cannot_publish_secrets_or_interop_claims(self):
        good = facts(); result=proof.public_result(good)
        self.assertEqual(result['scope'],'production-foreground-ui-synthetic-two-home-tls')
        self.assertFalse(result['production_s2s']);self.assertFalse(result['netbsd_qualified'])
        bads = [dict(good,selected_writes=2),dict(good,live_accepted=0),dict(good,archive_duplicates=0),
                dict(good,children_reaped=False),dict(good,listeners_closed=False),dict(good,server_plaintext_matches=1),
                dict(good,tls_versions=['unobserved','TLSv1.3']),dict(good,password='private'),dict(good,exact_selected_operation=True,live_duplicates=True)]
        for bad in bads:
            with self.subTest(bad=bad), self.assertRaises(ValueError):proof.public_result(bad)

class AdditionalRefusalRed(unittest.TestCase):
    def test_public_success_false_positives_are_refused_before_public_mapping(self):
        good=facts()
        for bad in [dict(good,selected_writes=2),dict(good,live_accepted=0),dict(good,archive_duplicates=0),
                    dict(good,children_reaped=False),dict(good,listeners_closed=False),dict(good,server_plaintext_matches=1),
                    dict(good,tls_versions=['unobserved','TLSv1.3']),dict(good,password='private'),dict(good,live_duplicates=True)]:
            with self.subTest(field=[k for k in bad if bad[k]!=good.get(k)]),self.assertRaises(ValueError):proof.public_result(bad)

    def test_unbound_readiness_and_bad_mam_scope_are_refused(self):
        with tempfile.TemporaryDirectory(prefix='foreground-refusal-OWNED-') as name:
            root=Path(name).resolve();root.chmod(0o700);(root/'OWNED').write_bytes(b'Owned issue356 foreground conversation fixture\n')
            relay=proof.Relay(root)
            with self.assertRaises(TimeoutError):relay.wait_bound(1,after=0,timeout=.01)
            for raw in [query('unknown'),query(maximum=5),query(own=A)]:
                with self.subTest(raw=raw[:80]),self.assertRaises(ValueError):relay.request(1,raw)

    def test_mam_refuses_foreign_unknown_cursor_and_oversized_page_independently(self):
        with tempfile.TemporaryDirectory(prefix='foreground-mam-OWNED-') as name:
            root=Path(name).resolve();root.chmod(0o700);(root/'OWNED').write_bytes(b'Owned issue356 foreground conversation fixture\n')
            relay=proof.Relay(root)
            for raw in [query('unknown'),query(maximum=5),query(own=A)]:
                with self.subTest(raw=raw[:80]),self.assertRaises(ValueError):relay.request(1,raw)

import base64


def auth(password='synthetic-home-secret', account='alice'):
    return ("<auth xmlns='urn:ietf:params:xml:ns:xmpp-sasl' mechanism='PLAIN'>"+
            base64.b64encode(('\0'+account+'\0'+password).encode()).decode()+'</auth>').encode()


def bind(actor=0, resource=None):
    resource=resource or 'faction'+str((101,202)[actor])
    return f"<iq xmlns='{NS}' id='bind-209' type='set'><bind xmlns='urn:ietf:params:xml:ns:xmpp-bind'><resource>{resource}</resource></bind></iq>".encode()


class LiveHandshake(unittest.TestCase):
    def test_authentication_requires_exact_account_password_and_plain_profile(self):
        proof.LiveHome.verify_auth(0,'synthetic-home-secret',auth())
        for raw in [auth('wrong'),auth(account='eve'),auth().replace(b'PLAIN',b'OTHER'),auth().replace(b'</auth>',b'<extra/></auth>'),b'not xml']:
            with self.subTest(raw=raw[:60]),self.assertRaises(ValueError):proof.LiveHome.verify_auth(0,'synthetic-home-secret',raw)

    def test_binding_response_uses_only_the_requested_exact_device_resource(self):
        raw=proof.LiveHome.bind_response(1,bind(1))
        response=ET.fromstring(raw)
        self.assertEqual(response.get('id'),'bind-209')
        self.assertEqual(response.findtext('{urn:ietf:params:xml:ns:xmpp-bind}bind/{urn:ietf:params:xml:ns:xmpp-bind}jid'),B+'/faction202')

    def test_wrong_device_and_binding_shape_are_refused_before_ready(self):
        for raw in [bind(1,'faction999'),bind(1).replace(b"type='set'",b"type='get'"),bind(1).replace(b'</resource>',b'</resource><extra/>')]:
            with self.subTest(raw=raw),self.assertRaises(ValueError):proof.LiveHome.bind_response(1,raw)

import shutil
import ssl
import subprocess


class CertificateFixture(unittest.TestCase):
    def test_distinct_generated_roots_and_exact_home_SANs_are_private(self):
        with tempfile.TemporaryDirectory(prefix='foreground-cert-OWNED-') as name:
            root=Path(name).resolve();root.chmod(0o700)
            fingerprints=[]
            for domain in ['home.test','away.test']:
                directory=root/domain;directory.mkdir(mode=0o700)
                pair,ca=proof.certificates(directory,domain,shutil.which('openssl'))
                self.assertTrue(ca.is_file());self.assertTrue(all(path.is_file() for path in pair))
                self.assertEqual(pair[1].stat().st_mode&0o777,0o600)
                matched=subprocess.run([shutil.which('openssl'),'x509','-in',str(pair[0]),'-noout','-checkhost',domain],capture_output=True,timeout=5)
                self.assertEqual(matched.returncode,0)
                fingerprints.append(ca.read_bytes())
            self.assertNotEqual(*fingerprints)

class StreamNamespace(unittest.TestCase):
    def test_authenticated_stream_default_namespace_is_preserved_by_the_fixture(self):
        with tempfile.TemporaryDirectory(prefix='foreground-stream-OWNED-') as name:
            root=Path(name).resolve();root.chmod(0o700);(root/'OWNED').write_bytes(b'Owned issue356 foreground conversation fixture\n')
            relay=proof.Relay(root);frames=[];relay.register(1,B+'/faction202',frames.append)
            self.assertTrue(relay.message(0,message().replace(b" xmlns='jabber:client'",b'',1)))
            self.assertEqual(len(frames),1)


class TLSWorker(unittest.TestCase):
    def setUp(self):
        self.tmp=tempfile.TemporaryDirectory(prefix='foreground-tls-OWNED-');self.addCleanup(self.tmp.cleanup)
        self.root=Path(self.tmp.name).resolve();self.root.chmod(0o700);(self.root/'OWNED').write_bytes(b'Owned issue356 foreground conversation fixture\n')
        self.relay=proof.Relay(self.root);self.homes=[];self.channels=[]
        self.addCleanup(self.cleanup)
        self.pairs=[];self.cas=[]
        for domain in ['home.test','away.test']:
            directory=self.root/domain;directory.mkdir(mode=0o700)
            pair,ca=proof.certificates(directory,domain,shutil.which('openssl'));self.pairs.append(pair);self.cas.append(ca)

    def cleanup(self):
        for channel in self.channels:channel.close()
        for home in self.homes:home.close()

    def start(self,actor):
        home=proof.LiveHome(self.root,actor,self.relay,self.pairs[actor],'synthetic-home-secret');self.homes.append(home)
        self.assertGreater(home.port,0,'new TLS home interface must expose an actual bound port')
        return home

    def connect(self,actor,wrong=False):
        from probe_xmpp_transport import Reader
        import socket
        home=self.homes[actor]
        context=ssl.create_default_context(cafile=str(self.cas[actor]))
        channel=context.wrap_socket(socket.create_connection(('127.0.0.1',home.port),timeout=2),server_hostname=['home.test','away.test'][actor])
        channel.settimeout(2);self.channels.append(channel);reader=Reader(channel)
        opening=b"<stream:stream xmlns='jabber:client' xmlns:stream='http://etherx.jabber.org/streams' to='home.test' version='1.0'>"
        channel.sendall(opening);reader.take(rb'</stream:features>')
        channel.sendall(auth(password='wrong' if wrong else 'synthetic-home-secret',account=['alice','bob'][actor]))
        if wrong:return channel,reader
        reader.take(rb'<success\b[^>]*/>')
        channel.sendall(opening);reader.take(rb'</stream:features>')
        channel.sendall(bind(actor));response=ET.fromstring(reader.element('iq'))
        self.assertEqual(response.findtext('{urn:ietf:params:xml:ns:xmpp-bind}bind/{urn:ietf:params:xml:ns:xmpp-bind}jid'),[A,B][actor]+'/faction'+str((101,202)[actor]))
        return channel,reader

    def test_two_actual_tls_bindings_route_then_serve_matching_archive_ciphertext(self):
        self.start(0);self.start(1)
        a,ar=self.connect(0);b,br=self.connect(1)
        generation=self.relay.wait_bound(1,after=0,timeout=1);self.assertGreater(generation,0)
        a.sendall(message())
        live=ET.fromstring(br.element('message'));self.assertEqual(live.get('to'),B+'/faction202')
        self.assertEqual(live.find('{urn:xmpp:sid:0}stanza-id').get('by'),B)
        b.sendall(query())
        block=ET.fromstring(b'<fixture>'+br.element('iq')+b'</fixture>')
        self.assertEqual(len(block),2)
        recovered,fin=block
        forwarded=recovered.find('{'+MAM+'}result/{urn:xmpp:forward:0}forwarded')
        self.assertEqual(ET.tostring(live[0]),ET.tostring(forwarded[1][0]))
        self.assertEqual(fin.find('{'+MAM+'}fin').get('complete'),'true')
        self.assertTrue(all(home.tls_versions[-1] in ['TLSv1.2','TLSv1.3'] for home in self.homes))
        self.cleanup()
        for home in self.homes:
            self.assertFalse(home.thread.is_alive());self.assertTrue(home.closed);self.assertIsNone(home.failure)
            import socket
            with self.assertRaises(ConnectionRefusedError):socket.create_connection(('127.0.0.1',home.port),timeout=1)

    def test_wrong_authentication_still_closes_owned_worker_and_listener(self):
        home=self.start(0);self.connect(0,wrong=True)
        self.assertTrue(home.failure_event.wait(2))
        self.assertEqual(home.authenticated,0);self.assertEqual(home.bindings,0)
        self.assertIsNotNone(home.failure)
        self.cleanup();self.assertFalse(home.thread.is_alive());self.assertTrue(home.closed)
        import socket
        with self.assertRaises(ConnectionRefusedError):socket.create_connection(('127.0.0.1',home.port),timeout=1)

class BindFailureCleanup(unittest.TestCase):
    def test_listener_is_closed_if_sandbox_or_OS_refuses_bind(self):
        from unittest.mock import Mock,patch
        with tempfile.TemporaryDirectory(prefix='foreground-bind-failure-OWNED-') as name:
            root=Path(name).resolve();root.chmod(0o700);(root/'OWNED').write_bytes(b'Owned issue356 foreground conversation fixture\n')
            tls=root/'tls';tls.mkdir(mode=0o700);pair,_=proof.certificates(tls,'home.test',shutil.which('openssl'))
            listener=Mock();listener.bind.side_effect=PermissionError('synthetic denied bind')
            with patch.object(proof.socket,'socket',return_value=listener):
                with self.assertRaises(PermissionError):proof.LiveHome(root,0,proof.Relay(root),pair,'synthetic-secret')
            listener.close.assert_called_once()

from types import SimpleNamespace
from unittest.mock import patch


class OrchestrationBoundaries(unittest.TestCase):
    def setUp(self):
        self.tmp=tempfile.TemporaryDirectory(prefix='foreground-orchestration-OWNED-');self.addCleanup(self.tmp.cleanup)
        self.root=Path(self.tmp.name).resolve();self.root.chmod(0o700);(self.root/'OWNED').write_bytes(b'Owned issue356 foreground conversation fixture\n')

    def test_native_command_sends_credentials_only_over_stdin_and_retains_exact_outcome(self):
        home=dict(store=self.root/'alice',local_password='local-private',password='home-private')
        completed=subprocess.CompletedProcess([],0,b'ok\n',b'')
        with patch.object(proof.subprocess,'run',return_value=completed) as call:
            self.assertEqual(proof.native_command(self.root,Path('/owned/binary'),'sync',home,Path('/owned/config'),home_password=True),b'ok\n')
        argv=call.call_args.args[0];options=call.call_args.kwargs
        self.assertEqual(argv,['/owned/binary','sync',str(home['store']),'/owned/config'])
        self.assertNotIn('local-private',str(argv));self.assertNotIn('home-private',str(argv));self.assertEqual(options['input'],b'local-private\nhome-private\n')
        captures=list(self.root.glob('command-*.private.json'));self.assertEqual(len(captures),1)
        data=json.loads(captures[0].read_text());self.assertEqual(data['stdout_hex'],b'ok\n'.hex());self.assertEqual(data['exit'],0)

    def test_failed_or_noisy_native_command_never_yields_a_false_success(self):
        home=dict(store=self.root/'alice',local_password='local-private',password='home-private')
        for completed in [subprocess.CompletedProcess([],1,b'secret-out',b'secret-err'),subprocess.CompletedProcess([],0,b'ok\n',b'secret-err')]:
            with patch.object(proof.subprocess,'run',return_value=completed),self.assertRaisesRegex(ValueError,'^native fixture action failed$'):
                proof.native_command(self.root,Path('/owned/binary'),'sync',home,Path('/owned/config'),home_password=True)

    def test_prompt_reader_consumes_earliest_complete_prompt_without_extra_input(self):
        console=SimpleNamespace(buffer=b'prefix\r\n[Enter] Next page  [B] Return\r\ntail',transcript=b'original',fd=-1)
        next_page=b'\r\n[Enter] Next page  [B] Return\r\n';final=b'\r\n[B] Return\r\n'
        token,raw=proof.expect_one(console,[next_page,final])
        self.assertEqual(token,next_page);self.assertEqual(raw,b'prefix'+next_page)
        self.assertEqual(console.buffer,b'tail');self.assertEqual(console.transcript,b'original')

    def test_prompt_reader_timeout_and_capture_bounds_fail_closed(self):
        console=SimpleNamespace(buffer=b'',transcript=b'',fd=-1)
        with patch.object(proof.time,'monotonic',side_effect=[0,1]),self.assertRaises(TimeoutError):proof.expect_one(console,[b'prompt'],seconds=.5)
        console.buffer=b'x'*(4*1024*1024+1)
        with self.assertRaises(ValueError):proof.expect_one(console,[b'prompt'])

    def test_review_only_advances_after_each_observed_page_and_requires_complete_sequence(self):
        final=b'\r\n[B] Return\r\n'
        frames=[b'[LOCAL BRIEF REVIEW] Page 1/2\r\n| Home: alice@home.test\r\n[Enter] Next page  [B] Return\r\n',
                b'[LOCAL BRIEF REVIEW] Page 2/2\r\n| Peer: bob@away.test\r\n[R] Receive briefly\r\n[B] Return\r\n']
        class Console:
            def __init__(self):self.buffer=frames[0];self.transcript=b'';self.fd=-1;self.sent=[]
            def send(self,text):
                self.sent.append(text);self.buffer+=frames[len(self.sent)]
        console=Console();text=proof.read_review(console,final)
        self.assertIn(b'Home: alice@home.test',text);self.assertIn(b'Peer: bob@away.test',text);self.assertEqual(console.sent,[''])
        malformed=SimpleNamespace(buffer=frames[1],transcript=b'',fd=-1)
        with self.assertRaises(ValueError):proof.read_review(malformed,final)

    def test_snapshot_inspects_an_exact_owned_copy_without_touching_live_store(self):
        store=self.root/'live';store.mkdir(mode=0o700);canonical=store/'mail.fkv';canonical.write_bytes(b'exact synthetic encrypted canonical');canonical.chmod(0o600)
        before=canonical.read_bytes();observed=[]
        state=snapshot();archive=dict(enabled=True,cursor_present=True,ready=True,checkpoint=state['archive'])
        def run(argv,**options):
            observed.append(argv)
            if argv[0]=='/owned/inspector':
                copied=Path(argv[2]);self.assertNotEqual(copied,store);self.assertEqual((copied/'mail.fkv').read_bytes(),before)
                return subprocess.CompletedProcess(argv,0,json.dumps(dict(state,archive={})).encode(),b'')
            return subprocess.CompletedProcess(argv,0,json.dumps(archive).encode(),b'')
        with patch.object(proof.subprocess,'run',side_effect=run):
            result=proof.canonical_snapshot(self.root,store,Path('/owned/inspector'),Path('/owned/mail-fixture'))
        self.assertEqual(result,state);self.assertEqual(canonical.read_bytes(),before)
        self.assertEqual(len(observed),2)

class AdditionalOrchestrationRefusals(unittest.TestCase):
    def test_oversize_prompt_capture_and_incomplete_review_cannot_pass(self):
        console=SimpleNamespace(buffer=b'x'*(4*1024*1024+1),transcript=b'',fd=-1)
        with self.assertRaises(ValueError):proof.expect_one(console,[b'prompt'])
        console.buffer=b'[LOCAL BRIEF REVIEW] Page 2/2\r\n| incomplete\r\n[B] Return\r\n'
        with self.assertRaises(ValueError):proof.read_review(console,b'\r\n[B] Return\r\n')

    def test_snapshot_refuses_symlink_before_executing_any_inspector(self):
        with tempfile.TemporaryDirectory(prefix='foreground-copy-refusal-OWNED-') as name:
            root=Path(name).resolve();root.chmod(0o700);(root/'OWNED').write_bytes(b'Owned issue356 foreground conversation fixture\n')
            store=root/'live';store.mkdir(mode=0o700);(store/'mail.fkv').symlink_to('/not-owned')
            with patch.object(proof.subprocess,'run') as command,self.assertRaises(ValueError):proof.canonical_snapshot(root,store,Path('/inspector'),Path('/mail-fixture'))
            command.assert_not_called()

    def test_incomplete_review_sequence_is_refused_independently(self):
        console=SimpleNamespace(buffer=b'[LOCAL BRIEF REVIEW] Page 2/2\r\n| incomplete\r\n[B] Return\r\n',transcript=b'',fd=-1)
        with self.assertRaises(ValueError):proof.read_review(console,b'\r\n[B] Return\r\n')

class PairDispatch(unittest.TestCase):
    def test_sender_confirmation_waits_for_fresh_receiver_binding_without_sleep(self):
        events=[]
        class Relay:
            generations=[3,7]
            def wait_bound(self,actor,after,timeout):
                self.assertion=(actor,after,timeout);events.append('fresh-bound');return 8
        relay=Relay()
        def confirm(value):
            if value=='sender':self.assertIn('fresh-bound',events)
            events.append(value)
        self.assertEqual(proof.connect_pair('receiver','sender',relay,confirm),8)
        self.assertEqual(events,['receiver','fresh-bound','sender']);self.assertEqual(relay.assertion,(1,7,2))

    def test_missing_fresh_receiver_bind_never_releases_sender(self):
        events=[]
        relay=SimpleNamespace(generations=[3,7],wait_bound=lambda *args,**kwargs:(_ for _ in ()).throw(TimeoutError()))
        with self.assertRaises(TimeoutError):proof.connect_pair('receiver','sender',relay,events.append)
        self.assertEqual(events,['receiver'])

    def test_review_binds_full_current_peer_and_exact_selected_operation(self):
        identity='ab'*32;operation='cd'*16;events=[]
        first=(b'[LOCAL BRIEF EXCHANGE]\r\n[LOCAL BRIEF REVIEW] Page 1/1\r\n| Home: '+A.encode()+b'\r\n| Peer: '+B.encode()+b'\r\n| Peer device: 202\r\n| Current identity:\r\n| '+identity.encode()+b'\r\n[R] Receive briefly\r\n[S] Send selected text\r\n[B] Return\r\n')
        second=b'[LOCAL BRIEF REVIEW] Page 1/1\r\n| Send selected text only.\r\n| Retained record: 4\r\n| Operation:\r\n| '+operation.encode()+b'\r\n| Subject: chosen\r\nType exchange to connect, or Enter: '
        class Console:
            def __init__(self):self.buffer=b'';self.transcript=b'';self.fd=-1
            def raw(self,key):events.append(key);self.buffer+=first if key==b'e' else second
            def send(self,text):raise AssertionError('one-page review requires no queued Enter')
            def expect(self,text):
                token=text.encode();self.assertion=token
                if token not in self.buffer:raise AssertionError('missing explicit prompt')
                self.buffer=self.buffer.split(token,1)[1]
        proof.arm_brief(Console(),A,B,202,identity,operation)
        self.assertEqual(events,[b'e',b's'])
        bad=Console()
        with self.assertRaises(ValueError):proof.arm_brief(bad,A,B,202,'ef'*32,operation)
        self.assertEqual(events,[b'e',b's',b'e'])

    def test_wrong_current_review_is_refused_before_selecting_or_entering_credentials(self):
        events=[]
        class Console:
            buffer=b'';transcript=b'';fd=-1
            def raw(self,key):
                events.append(key);self.buffer=b'[LOCAL BRIEF EXCHANGE]\r\n[LOCAL BRIEF REVIEW] Page 1/1\r\n| Home: mallory@home.test\r\n| Peer: '+B.encode()+b'\r\n| Peer device: 202\r\n| Current identity:\r\n| '+b'ab'*32+b'\r\n[R] Receive briefly\r\n[B] Return\r\n'
            def expect(self,text):self.buffer=self.buffer.split(text.encode(),1)[1]
        with self.assertRaises(ValueError):proof.arm_brief(Console(),A,B,202,'ab'*32)
        self.assertEqual(events,[b'e'])
