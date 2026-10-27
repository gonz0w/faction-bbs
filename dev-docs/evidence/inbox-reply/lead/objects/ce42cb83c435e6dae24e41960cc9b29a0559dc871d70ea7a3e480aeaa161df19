"""Offline rejection tests for immutable selected-record reply evidence."""
import copy
import hashlib
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest

SPEC=importlib.util.spec_from_file_location('reply_evidence',Path(__file__).with_name('check_reply_evidence.py'))

class Evidence(unittest.TestCase):
    def setUp(self):
        self.m=importlib.util.module_from_spec(SPEC);SPEC.loader.exec_module(self.m)
        self.temp=tempfile.TemporaryDirectory();self.addCleanup(self.temp.cleanup);self.root=Path(self.temp.name)
        def save(name,value):
            raw=json.dumps(value).encode() if isinstance(value,dict) else value;p=self.root/name;p.write_bytes(raw)
            return dict(path=name,sha256=hashlib.sha256(raw).hexdigest())
        self.save=save;source=save('source.raw',b'actual executable input')
        self.record=dict(schema=1,issue=233,result='pass',profile=self.m.PROFILE.copy(),claims={k:False for k in self.m.CLAIMS},sources=[source],prior_records=[save('prior.raw',b'unchanged prior observation')],source_stages=[dict(label='native-final',manifest=save('native-map.json',{'crates/example.rs':source['sha256']}))],routes={'local':{},'main':{}},records={})
        for kind in ('local','main'):
            for platform in ('host','netbsd'):
                for mode in ('classic','unicode'):
                    data=dict(result='pass',platform='Darwin' if platform=='host' else 'NetBSD',scope=self.m.SCOPES[kind],unicode=mode=='unicode',checks=sorted(self.m.CONTROLS[kind]),count=len(self.m.CONTROLS[kind]))
                    if kind=='local':data.update(private_fixture_removed=True,configured_home_endpoint_was_closed=True,home_password_prompt_observed=False)
                    else:data.update(files_disabled=True,actual_home_transport_error='Connect',capture_id=f'{(0 if platform=="host" else 2)+(1 if mode=="unicode" else 0)+1:032x}',source_sha256=source['sha256'],marker_count=9,private_matches=0,file_protocol_matches=0,exact_canonical_reply_invariants=True,same_ssh_pid_until_normal_close=True,automatic_redial_observed=False,private_input_hashes=dict(trace_sha256='1'*64,marker_set_sha256='2'*64,file_scan_input_sha256='3'*64),privacy_scan=dict(board_files=4,store_files=1,log_files=2,matches=0,symlinks=0),board_io_reconstruction=dict(records=5,streams=3,decoded_bytes=90,read_bytes=40,write_bytes=50))
                    self.record['routes'][kind][platform+'_'+mode]=save(kind+'-'+platform+'-'+mode+'.json',data)
        values={k:dict(result='pass') for k in self.m.REQUIRED}
        values['native'].update(source_count=1,all_source_match=True,files={'crates/example.rs':source['sha256']},compile_memory_mib=2048,runtime_physmem=1054990336)
        values['cleanup'].update(owned_removed=True,saved_stopped=True,saved_hashes_unchanged=True,forwards_closed=True)
        values['tdd'].update(captured_before_edit=True,red_exit=101,green_exit=0)
        workspace=dict(result='pass',source_stable=True,checks=[dict(command=['cargo',name],returncode=0) for name in ('fmt','clippy','test')])
        summary=dict(final2=dict(network=dict(total_passed=139,total_ignored=4),native=dict(total_passed=90,total_ignored=4),sources={'crates/example.rs':source['sha256']}))
        values['host'].update(workspace=save('workspace.json',workspace),workspace_test_log=save('workspace.log',b'test result: ok. 473 passed; 0 failed; 0 ignored;\n'),client_summary=save('client-summary.json',summary),client_test_logs={key:save(key+'-test.log',f'test result: ok. {count} passed; 0 failed; 4 ignored;\n'.encode()) for key,count in [('native',90),('network',139)]})
        values['native']['test_log']=save('netbsd-test.log',b'test result: ok. 272 passed; 0 failed; 4 ignored;\n')
        values['compatibility']['routes']={}
        for platform in ('host','netbsd'):
            data=dict(result='pass',platform='Darwin' if platform=='host' else 'NetBSD',scope='same-ssh-parked-files-local-mail-restricted-sftp',count=14,checks=sorted(self.m.FILES_CONTROLS),unicode=False,approved_member=True,restricted_sftp=True,same_ssh_pid_until_explicit_disconnect=True,privacy_scan=dict(matches=0,symlinks=0),physical_hardware=False,capacity_claim=False)
            old=dict(result='pass',scope='actual-old-and-default-native-v1-files-rpc',count=2,checks=sorted(self.m.LEGACY_CONTROLS),private_mail_not_opened=True,physical_hardware=False,capacity_claim=False)
            values['compatibility']['routes'][platform]=dict(files=save(platform+'-files.json',data),legacy=save(platform+'-legacy.json',old))
        values['compatibility']['inbox']=save('inbox.json',dict(result='pass',count=14,checks=sorted(self.m.INBOX_CONTROLS),private_fixture_removed=True,provider_contact=False,read_receipt_sent=False))
        values['associations']=dict(result='pass',routes={})
        for key,ref in self.record['routes']['main'].items():
            data=json.loads((self.root/ref['path']).read_bytes())
            values['associations']['routes'][key]=dict(capture_id=data['capture_id'],source_sha256=data['source_sha256'],result_sha256=ref['sha256'],private_input_hashes=data['private_input_hashes'],generated_before_capture=True,input_trace_unchanged=True)
        for key,value in values.items():self.record['records'][key]=save(key+'.json',value)

    def mutate(self,record,ref,change):
        data=json.loads((self.root/ref['path']).read_bytes());change(data);return self.save('mutated-'+ref['path'],data)

    def test_rejects_unbound_capture_result_association(self):
        record=copy.deepcopy(self.record)
        record['records']['associations']=self.mutate(record,record['records']['associations'],lambda d:d['routes']['host_classic'].update(result_sha256='e'*64))
        with self.assertRaises(AssertionError):self.m.validate(self.root,record)

    def test_rejects_placeholder_compatibility_and_host_records(self):
        for key in ('host','compatibility'):
            record=copy.deepcopy(self.record);record['records'][key]=self.save('placeholder-'+key+'.json',dict(result='pass'))
            with self.assertRaises((AssertionError,KeyError)):self.m.validate(self.root,record)

    def test_rejects_native_source_count_mismatch(self):
        record=copy.deepcopy(self.record);record['records']['native']=self.mutate(record,record['records']['native'],lambda d:d.update(source_count=0))
        with self.assertRaises(AssertionError):self.m.validate(self.root,record)

    def test_accepts_complete_bounded_control_matrix(self):
        self.m.validate(self.root,self.record)

    def test_ci_runs_the_pinned_reply_guard(self):
        workflow=Path(__file__).resolve().parents[1]/'.github/workflows/documentation.yml'
        self.assertIn('python3 tools/check_reply_evidence.py',workflow.read_text())

    def test_ci_executes_both_live_local_reply_modes(self):
        workflow=(Path(__file__).resolve().parents[1]/'.github/workflows/documentation.yml').read_text()
        native=workflow.split('  native_mail:',1)[1].split('  documentation:',1)[0]
        commands=[line.strip() for line in native.splitlines() if line.strip().startswith('python3 tools/probe_mail_reply.py ')]
        self.assertEqual(len(commands),2)
        self.assertNotIn('--unicode',commands[0]);self.assertIn('--unicode',commands[1])
        for command in commands:
            self.assertIn('--fixture "$CARGO_TARGET_DIR/release/examples/mail_reply_fixture"',command)
            self.assertIn('--owned-root "$RUNNER_TEMP/reply-owned"',command)
        build=next(line for line in native.splitlines() if 'cargo build -p faction-client --features network-mail --bin faction-client' in line)
        self.assertIn('--example mail_reply_fixture',build)
        self.assertLess(native.index(build),native.index(commands[0]))

    def test_rejects_missing_or_host_substituted_native_route(self):
        for change in ('missing','host-as-native'):
            record=copy.deepcopy(self.record)
            if change=='missing':record['routes']['main'].pop('netbsd_unicode')
            else:record['routes']['local']['netbsd_classic']=record['routes']['local']['host_classic']
            with self.assertRaises((AssertionError,KeyError)):self.m.validate(self.root,record)

    def test_rejects_changed_or_missing_exact_control_and_scope(self):
        for change in [lambda d:d.update(scope='unrelated'),lambda d:d['checks'].pop(),lambda d:d['checks'].__setitem__(0,'unrelated')]:
            record=copy.deepcopy(self.record);record['routes']['main']['host_classic']=self.mutate(record,record['routes']['main']['host_classic'],change)
            with self.assertRaises(AssertionError):self.m.validate(self.root,record)

    def test_rejects_privacy_or_same_visit_regressions_with_coherent_new_hash(self):
        for key,value in [('private_matches',1),('file_protocol_matches',1),('marker_count',0),('same_ssh_pid_until_normal_close',False),('automatic_redial_observed',True),('actual_home_transport_error','Config'),('source_sha256','f'*64),('files_disabled',False),('board_io_reconstruction',dict(records=0,streams=0,decoded_bytes=0,read_bytes=0,write_bytes=0))]:
            record=copy.deepcopy(self.record);record['routes']['main']['host_classic']=self.mutate(record,record['routes']['main']['host_classic'],lambda d:d.update({key:value}))
            with self.assertRaises(AssertionError):self.m.validate(self.root,record)

    def test_rejects_local_mutation_network_or_unremoved_fixture(self):
        for key,value in [('private_fixture_removed',False),('configured_home_endpoint_was_closed',False),('home_password_prompt_observed',True)]:
            record=copy.deepcopy(self.record);record['routes']['local']['host_classic']=self.mutate(record,record['routes']['local']['host_classic'],lambda d:d.update({key:value}))
            with self.assertRaises(AssertionError):self.m.validate(self.root,record)

    def test_rejects_missing_record_source_drift_or_relaxed_resource_claim(self):
        for change in [lambda r:r['records'].pop('native'),lambda r:r['sources'][0].update(sha256='0'*64),lambda r:r['claims'].update(automatic_send=True),lambda r:r['profile'].update(normal_sessions=250)]:
            record=copy.deepcopy(self.record);change(record)
            with self.assertRaises((AssertionError,KeyError)):self.m.validate(self.root,record)

    def test_rejects_cleanup_tdd_or_build_scope_substitution(self):
        for key,field,value in [('cleanup','owned_removed',False),('tdd','red_exit',0),('tdd','captured_before_edit',False),('native','runtime_physmem',2*1024**3),('native','all_source_match',False),('native','files',{'other.rs':'e'*64})]:
            record=copy.deepcopy(self.record);record['records'][key]=self.mutate(record,record['records'][key],lambda d:d.update({field:value}))
            with self.assertRaises(AssertionError):self.m.validate(self.root,record)

if __name__=='__main__':unittest.main()
