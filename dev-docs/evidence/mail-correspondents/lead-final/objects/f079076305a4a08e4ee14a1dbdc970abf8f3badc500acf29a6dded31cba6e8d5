"""Reject drift and substituted correspondent-picker software observations."""
import copy
import hashlib
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest

SPEC = importlib.util.spec_from_file_location('correspondents_guard', Path(__file__).with_name('check_correspondents_evidence.py'))

class Evidence(unittest.TestCase):
    def setUp(self):
        self.guard = importlib.util.module_from_spec(SPEC)
        SPEC.loader.exec_module(self.guard)
        temp = tempfile.TemporaryDirectory()
        self.addCleanup(temp.cleanup)
        self.root = Path(temp.name)
        self.serial = 0
        def save(value):
            self.serial += 1
            data = value if isinstance(value, bytes) else (json.dumps(value) + '\n').encode()
            path = f'{self.serial}.raw'
            (self.root/path).write_bytes(data)
            return dict(path=path, sha256=hashlib.sha256(data).hexdigest())
        self.save = save
        source = save(b'executed fixture source\n')
        files = {name: source['sha256'] for name in ('crates/faction-client/src/checkpoint_ui.rs', 'crates/faction-client/src/checkpoint_ui_inbox.rs')}
        self.record = dict(schema=1, issue=235, result='pass', profile=self.guard.PROFILE.copy(), claims={key:False for key in self.guard.CLAIMS}, sources=[source], prior_records=[save(b'unchanged previous reply observations\n')], source_stages=[dict(label=label, manifest=save(files)) for label in ('host-final','native-final')], production=files, routes=dict(local={}, main={}), records={})
        associations = {}
        for kind in ('local','main'):
            for platform in ('host','netbsd'):
                for unicode in (False,True):
                    key = platform + ('_unicode' if unicode else '_classic')
                    data = dict(result='pass', scope=self.guard.SCOPES[kind], platform='Darwin' if platform=='host' else 'NetBSD', unicode=unicode, checks=sorted(self.guard.CONTROLS[kind]), count=len(self.guard.CONTROLS[kind]), source_sha256=source['sha256'])
                    if kind=='local':
                        data.update(private_fixture_removed=True, navigation_readonly_preserved=True, home_password_prompt_observed=False, configured_home_endpoint_was_closed=True)
                    else:
                        capture_id = f'{len(associations)+1:032x}'
                        hashes = dict(trace_sha256='1'*64, marker_set_sha256='2'*64, file_scan_input_sha256='3'*64)
                        data.update(capture_id=capture_id, private_input_hashes=hashes, exact_canonical_ack_invariants=True, marker_count=12, private_matches=0, file_protocol_matches=0, same_ssh_pid_until_normal_close=True, automatic_redial_observed=False, files_disabled=True, actual_home_transport_error='Connect', privacy_scan=dict(matches=0,symlinks=0,board_files=4,store_files=1,log_files=2), board_io_reconstruction=dict(records=10,streams=3,decoded_bytes=100,read_bytes=20,write_bytes=80))
                    ref = save(data)
                    self.record['routes'][kind][key] = ref
                    if kind=='main':
                        associations[key] = dict(capture_id=capture_id, source_sha256=source['sha256'], result_sha256=ref['sha256'], private_input_hashes=hashes, generated_before_capture=True, input_trace_unchanged=True)
        records = dict(associations=dict(result='pass',routes=associations), native=dict(result='pass', files=files, source_count=len(files), all_source_match=True, compile_memory_mib=2048,runtime_physmem=1054990336), cleanup=dict(result='pass',owned_removed=True,saved_stopped=True,saved_hashes_unchanged=True,saved_sizes_mtimes_unchanged=True,forwards_closed=True), tdd=dict(result='pass',captured_before_edit=True,red_exit=101,green_exit=0), publication=dict(result='pass',generated_private_values_scanned=12,private_matches=0,raw_private_key_or_vault_matches=0,explicit_manifests_verified=True), rust=dict(result='pass',runs={name:dict(passed=5,ignored=0,log=save(b'test result: ok. 5 passed; 0 failed; 0 ignored;\n')) for name in ('workspace','network','native','netbsd')}))
        for name,value in records.items():self.record['records'][name]=save(value)

    def change_record(self, name, change):
        value=json.loads((self.root/self.record['records'][name]['path']).read_bytes())
        change(value)
        self.record['records'][name]=self.save(value)

    def replace_route(self, kind, key, data):
        ref = self.save(data)
        self.record['routes'][kind][key] = ref
        if kind == 'main':
            # Keep the result/capture association coherent so an unrelated hash
            # mismatch cannot hide a missing check of the targeted route field.
            association = json.loads((self.root/self.record['records']['associations']['path']).read_bytes())
            association['routes'][key]['result_sha256'] = ref['sha256']
            self.record['records']['associations'] = self.save(association)

    def test_rejects_false_canonical_ack_with_coherent_capture_association(self):
        ref = self.record['routes']['main']['host_classic']
        data = json.loads((self.root/ref['path']).read_bytes())
        data['exact_canonical_ack_invariants'] = False
        self.replace_route('main', 'host_classic', data)
        with self.assertRaises(AssertionError):
            self.guard.validate(self.root, self.record)

    def test_complete_source_bound_matrix(self):
        self.guard.validate(self.root,self.record)

    def test_missing_or_host_substituted_native_route(self):
        for kind in ('local','main'):
            record=copy.deepcopy(self.record)
            record['routes'][kind]['netbsd_classic']=record['routes'][kind]['host_classic']
            with self.assertRaises(AssertionError):self.guard.validate(self.root,record)

    def test_changed_bytes_or_unsafe_paths(self):
        for path in ('../escape','/tmp/escape'):
            record=copy.deepcopy(self.record);record['sources'][0]['path']=path
            with self.assertRaises(AssertionError):self.guard.validate(self.root,record)
        (self.root/self.record['sources'][0]['path']).write_bytes(b'drift')
        with self.assertRaises(AssertionError):self.guard.validate(self.root,self.record)

    def test_missing_controls_and_private_network_regressions(self):
        original=copy.deepcopy(self.record)
        for kind,field,value in [('local','home_password_prompt_observed',True),('local','navigation_readonly_preserved',False),('main','private_matches',1),('main','same_ssh_pid_until_normal_close',False),('main','automatic_redial_observed',True),('main','actual_home_transport_error','Config'),('main','count',0)]:
            self.record=copy.deepcopy(original)
            ref=self.record['routes'][kind]['host_classic'];data=json.loads((self.root/ref['path']).read_bytes());data[field]=value
            self.replace_route(kind, 'host_classic', data)
            with self.assertRaises(AssertionError):self.guard.validate(self.root,self.record)

    def test_unbound_association_and_false_source_count(self):
        self.change_record('associations',lambda d:d['routes']['host_classic'].update(result_sha256='f'*64))
        with self.assertRaises(AssertionError):self.guard.validate(self.root,self.record)

    def test_rejects_source_count_lie(self):
        self.change_record('native',lambda d:d.update(source_count=1))
        with self.assertRaises(AssertionError):self.guard.validate(self.root,self.record)

    def test_rejects_cleanup_tdd_publication_and_count_lies(self):
        original=copy.deepcopy(self.record)
        for name,change in [('cleanup',lambda d:d.update(owned_removed=False)),('tdd',lambda d:d.update(red_exit=0)),('publication',lambda d:d.update(private_matches=1)),('rust',lambda d:d['runs']['netbsd'].update(passed=6))]:
            self.record=copy.deepcopy(original);self.change_record(name,change)
            with self.assertRaises(AssertionError):self.guard.validate(self.root,self.record)

    def test_no_relaxed_limits_or_new_authority_claim(self):
        self.record['profile']['normal_sessions']=250
        with self.assertRaises(AssertionError):self.guard.validate(self.root,self.record)
        self.record['profile']=self.guard.PROFILE.copy();self.record['claims']['automatic_send']=True
        with self.assertRaises(AssertionError):self.guard.validate(self.root,self.record)

    def test_ci_registers_guard_and_live_display_modes(self):
        workflow=(Path(__file__).resolve().parents[1]/'.github/workflows/documentation.yml').read_text()
        self.assertIn('python3 tools/check_correspondents_evidence.py',workflow)
        native=workflow.split('  native_mail:',1)[1].split('  documentation:',1)[0]
        commands=[line.strip() for line in native.splitlines() if line.strip().startswith('python3 tools/probe_mail_correspondents.py ')]
        self.assertEqual(len(commands),2)
        self.assertNotIn('--unicode',commands[0]);self.assertIn('--unicode',commands[1])
        for command in commands:
            self.assertIn('--fixture "$CARGO_TARGET_DIR/release/examples/mail_correspondents_fixture"',command)
        self.assertIn('--example mail_correspondents_fixture',native)

if __name__=='__main__':unittest.main()
