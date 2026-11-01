"""Finite composed-console fixture contracts; no Rust processes in these tests."""
import importlib,json,os,shutil,stat,tempfile,tomllib,unittest
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import patch

class ComposedFixtureContracts(unittest.TestCase):
    def setUp(self):
        self.fixture=importlib.import_module('probe_two_home_provider_console')
        parent=os.environ.get('FACTION322_CONSOLE_FIXTURES')
        self.root=Path(tempfile.mkdtemp(prefix='composed-contract-OWNED-',dir=parent)).resolve();self.root.chmod(0o700)
        if parent is None:self.addCleanup(shutil.rmtree,self.root)
        self.digests={}
        def home(label,name,account,device,identity,port,offset):
            root=self.root/label;root.mkdir(mode=0o700)
            h=SimpleNamespace(root=root,name=name,account=account,device=device,identity=identity,
                address='127.0.0.1:'+str(port),ca=root/'ca.pem',server=(root/'server.pem',root/'server.key'),
                relay=(root/'relay.pem',root/'relay.key'),inspector=(root/'inspector.pem',root/'inspector.key'),
                publisher=(root/'publisher.pem',root/'publisher.key'),
                device_pair=(self.root/(label+'-device.pem'),self.root/(label+'-device.key')))
            for i,path in enumerate([h.ca,h.server[0],h.relay[0],h.inspector[0],h.publisher[0],h.device_pair[0]]):
                path.write_bytes(b'synthetic public certificate');path.chmod(0o600);self.digests[path]=bytes([offset+i])*32
            h.device_pair[1].write_bytes(b'private synthetic device key');h.device_pair[1].chmod(0o600)
            return h
        self.a=home('home','home.test','alice@home.test',101,'01'*32,12345,11)
        self.b=home('away','away.test','bob@away.test',202,'02'*32,23456,21)

    def test_fixed_config_has_independent_roots_and_only_service_keys(self):
        with patch.object(self.fixture,'fingerprint',side_effect=lambda p:self.digests[p]):
            raw=self.fixture.composed_config(self.a,self.b)
        c=tomllib.loads(raw)
        self.assertEqual((c['version'],c['home'],c['endpoint']),(2,'home.test','127.0.0.1:12345'))
        self.assertEqual(c['policy_root'],str(self.a.root/'policy'))
        self.assertEqual(c['blob_root'],str(self.a.root/'blobs'))
        self.assertEqual(c['client_cas'],[str(self.a.ca),str(self.a.root/'peer-ca.pem')])
        self.assertEqual(c['local_relay']['address'],self.a.address)
        self.assertEqual(c['local_relay']['key'],str(self.a.relay[1]))
        self.assertEqual(c['publication_clients'],['publisher'])
        peer,=c['peers'];self.assertEqual(peer['home'],self.b.name)
        for role,credential in [('publication',self.a.publisher),('introspection',self.a.inspector),('resource',self.a.relay)]:
            self.assertEqual(peer[role]['address'],self.b.address)
            self.assertEqual(peer[role]['http_host'],self.b.name)
            self.assertEqual(peer[role]['server_name'],self.b.name)
            self.assertEqual(peer[role]['leaf_pin'],self.digests[self.b.server[0]].hex())
            self.assertEqual(peer[role]['ca'],str(self.a.root/'peer-ca.pem'))
            self.assertEqual(peer[role]['key'],str(credential[1]))
        self.assertNotIn(str(self.a.device_pair[1]),raw)
        self.assertNotIn(str(self.b.device_pair[1]),raw)
        self.assertLessEqual(len(raw.encode()),65536)

    def test_complete_policy_registers_only_local_device_and_exact_roles(self):
        with patch.object(self.fixture,'fingerprint',side_effect=lambda p:self.digests[p]):
            raw=self.fixture.composed_proposal(self.a,self.b,generation=1,reset_epoch=1,
                remote_generation=1,remote_reset=1,approved=True,peer_enabled=True,stamp=1000)
        p=tomllib.loads(raw);binding,=p['bindings']
        self.assertEqual(p['local_home'],self.a.name);self.assertEqual(p['issuer'],self.a.name)
        self.assertEqual(p['homes'],[self.a.name,self.b.name])
        self.assertEqual(binding,dict(certificate_sha256=self.digests[self.a.device_pair[0]].hex(),
            account=self.a.account,device=101,identity=self.a.identity,valid_from=940,valid_until=4600,
            approved=True,account_enabled=True,device_enabled=True,generation=1))
        self.assertNotIn(self.b.account,raw)
        roles={(s['home'],s['client_id'],s['purpose']):s for s in p['services']}
        self.assertEqual(set(roles),{(self.a.name,'relay','relay'),(self.b.name,'inspector','introspection'),
            (self.b.name,'relay','resource-client'),(self.a.name,'inspector','outbound-introspection'),
            (self.b.name,'endpoint','issuer-endpoint'),(self.a.name,'publisher','outbound-observation'),
            (self.b.name,'publisher','observation')})
        self.assertEqual(roles[(self.b.name,'publisher','observation')]['certificate_sha256'],self.digests[self.b.publisher[0]].hex())
        self.assertTrue(all(s['enabled'] and s['dns_san']==s['home'] for s in roles.values()))
        self.assertEqual(p['issuer_routes'][0]['audience'],'away.test/attachments')
        self.assertEqual(p['resources'][0]['audience'],'home.test/attachments')
        self.assertEqual(p['resources'][0]['ca_sha256'],self.digests[self.b.ca].hex())
        self.assertEqual(p['publications'][0]['ca_sha256'],self.digests[self.b.ca].hex())

    def test_revocation_and_remote_floors_are_explicit_complete_proposals(self):
        with patch.object(self.fixture,'fingerprint',side_effect=lambda p:self.digests[p]):
            p=tomllib.loads(self.fixture.composed_proposal(self.a,self.b,generation=8,reset_epoch=4,
                remote_generation=7,remote_reset=3,approved=False,peer_enabled=False,stamp=1000))
        self.assertEqual((p['generation'],p['reset_epoch']),(8,4))
        self.assertFalse(p['bindings'][0]['approved'])
        for association in [p['resources'][0],p['publications'][0]]:
            self.assertEqual((association['remote_generation'],association['remote_reset']),(7,3))
            self.assertEqual(association['address'],self.b.address)
        self.assertTrue(all(not s['enabled'] for s in p['services'] if s['home']==self.b.name))
        self.assertTrue(all(s['enabled'] for s in p['services'] if s['home']==self.a.name))

    def test_source_removal_waits_for_complete_canonical_confirmed_progress(self):
        source=self.root/'selected.bin';source.write_bytes(b'original selected source');source.chmod(0o600)
        chunks=[b'Confirmed ciphertext bytes: 16384\r\nSDK call: ',b'1 / 65; writes: 0\r\n',b'unchanged trailing bytes']
        expected=list(chunks)
        def read(fd,count):self.assertEqual((fd,count),(7,8192));return chunks.pop(0)
        observed=self.fixture.RemoveAfterProgress(source,1048592,read)
        self.assertEqual(observed(7,8192),expected[0]);self.assertTrue(source.exists())
        self.assertEqual(observed(7,8192),expected[1]);self.assertFalse(source.exists())
        self.assertEqual(observed(7,8192),expected[2]);self.assertTrue(observed.removed)

    def test_invalid_progress_cannot_remove_the_source(self):
        for raw in [b'Confirmed ciphertext bytes: 016384\r\nSDK call: 1 / 65; writes: 0\r\n',
                    b'Confirmed ciphertext bytes: 0\r\nSDK call: 1 / 65; writes: 0\r\n',
                    b'Confirmed ciphertext bytes: 16384\r\nSDK call: 2 / 65; writes: 0\r\n',
                    b'Confirmed ciphertext bytes: 16384\r\nSDK call: 1 / 65; writes: 1\r\n']:
            source=self.root/'selected-invalid.bin';source.write_bytes(b'source')
            observed=self.fixture.RemoveAfterProgress(source,1048592,lambda fd,count:raw)
            self.assertEqual(observed(7,8192),raw);self.assertTrue(source.exists());source.unlink()

    def test_owned_operator_uses_composed_cli_and_drains_its_child(self):
        script=self.root/'synthetic-operator'
        script.write_text("#!/usr/bin/env python3\nimport signal,sys\nif len(sys.argv)!=3 or sys.argv[1]!='serve-composed':sys.exit(3)\ndef stop(*args):\n print('stopped',flush=True)\n sys.exit(0)\nsignal.signal(signal.SIGTERM,stop)\nprint('ready 12345',flush=True)\nsignal.pause()\n")
        script.chmod(0o700)
        operator=self.fixture.ComposedOperator(script,self.a.root/'config.toml',self.a.root)
        try:
            self.assertEqual(operator.port,12345)
            self.assertIsNone(operator.process.poll())
            operator.close()
            self.assertEqual(operator.process.returncode,0)
            self.assertTrue(operator.closed)
        finally:
            if not operator.closed:operator.close()

    def privacy_inputs(self):
        import subprocess
        for h in [self.a,self.b]:
            (h.root/'blobs').mkdir(mode=0o700)
            for name,data in [('private-blobs.snapshot',b'opaque actual-store test bytes'),('private-blobs.lock',b'')]:
                p=h.root/'blobs'/name;p.write_bytes(data);p.chmod(0o600)
            p=h.root/'composed-stop-unit.private';p.write_bytes(b'stopped\n');p.chmod(0o600)
        (self.root/'alice').mkdir(mode=0o700)
        for name,data in [('alice/mail.fkv',b'encrypted member checkpoint'),('delivery.xml',b'opaque encrypted message'),('mail.private',b'opaque encrypted archive')]:
            p=self.root/name;p.write_bytes(data);p.chmod(0o600)
        return SimpleNamespace(root=self.root,homes=[self.a,self.b],fixture=self.root/'fixture',
            secret=b'private synthetic checkpoint password\n',local_password='private synthetic checkpoint password',
            home_passwords=['private synthetic home password one','private synthetic home password two'])

    def test_privacy_audit_binds_both_actual_roots_and_uses_unchanged_member_command(self):
        import subprocess,hashlib
        journey=self.privacy_inputs()
        done=subprocess.CompletedProcess([],0,stdout=b'audited\n',stderr=b'')
        with patch('subprocess.run',return_value=done) as run:
            result=self.fixture.audit_selected(journey,b'selected private plaintext')
        self.assertEqual(result['actual_stores'],2)
        self.assertTrue(result['descriptor_and_content_key_audit'])
        self.assertTrue(result['hidden_exact_tokens_unavailable'])
        self.assertEqual(run.call_args.args[0],[str(journey.fixture),'audit',str(self.root)])
        self.assertTrue(run.call_args.kwargs['input']==journey.secret,'member secret input changed')
        proof=json.loads((self.root/'privacy-audit.private.json').read_text())
        self.assertEqual(len(proof['sources']),8)
        self.assertTrue(proof['no_store_session_policy_authority_from_copies'])
        self.assertTrue(proof['member_audit_checkpoint_unchanged'])
        for item in proof['sources']:
            source,copy=Path(item['source']),Path(item['copy'])
            self.assertTrue(source.read_bytes()==copy.read_bytes(),'audit bytes differ')
            self.assertEqual(hashlib.sha256(source.read_bytes()).hexdigest(),item['sha256'])
            self.assertFalse(any(copy.is_relative_to(h.root) for h in journey.homes))
            self.assertEqual(stat.S_IMODE(copy.stat().st_mode),0o600)

    def test_actual_path_password_leak_stops_before_member_audit(self):
        journey=self.privacy_inputs()
        p=self.b.root/'blobs/private-blobs.snapshot';p.write_bytes(journey.home_passwords[1].encode())
        with patch('subprocess.run') as run,self.assertRaises(ValueError):
            self.fixture.audit_selected(journey,b'selected private plaintext')
        run.assert_not_called()
        self.assertTrue(p.read_bytes()==journey.home_passwords[1].encode(),'source bytes changed')

    def test_member_audit_output_must_be_exact_and_not_echo_needles(self):
        import subprocess
        journey=self.privacy_inputs()
        done=subprocess.CompletedProcess([],0,stdout=b'unexpected synthetic needle output\n',stderr=b'')
        with patch('subprocess.run',return_value=done),self.assertRaises(ValueError):
            self.fixture.audit_selected(journey,b'selected private plaintext')

    def test_byte_binding_is_rechecked_after_member_audit(self):
        import subprocess
        journey=self.privacy_inputs()
        def changed(*args,**kwargs):
            (self.a.root/'blobs/private-blobs.snapshot').write_bytes(b'changed observer input')
            return subprocess.CompletedProcess([],0,stdout=b'audited\n',stderr=b'')
        with patch('subprocess.run',side_effect=changed),self.assertRaises(ValueError):
            self.fixture.audit_selected(journey,b'selected private plaintext')
