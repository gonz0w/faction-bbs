"""Offline controls for the source-bound main-menu mail proof."""
import copy
import hashlib
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest

SPEC=importlib.util.spec_from_file_location('menu_mail_evidence',Path(__file__).with_name('check_menu_mail_evidence.py'))

class Evidence(unittest.TestCase):
    def setUp(self):
        self.module=importlib.util.module_from_spec(SPEC);SPEC.loader.exec_module(self.module)
        self.temp=tempfile.TemporaryDirectory();self.addCleanup(self.temp.cleanup);self.root=Path(self.temp.name)
        self.record=dict(schema=1,issue=230,result='pass',profile=self.module.PROFILE.copy(),claims={k:False for k in self.module.CLAIMS},routes={},records={},sources=[],source_stages=[],prior_records=[])
        def save(name,data):
            raw=(json.dumps(data).encode() if isinstance(data,dict) else data);path=self.root/name;path.write_bytes(raw)
            ref=dict(path=name,sha256=hashlib.sha256(raw).hexdigest());self.record['sources'].append(ref);return ref
        self.save=save
        source=save('source.rs',b'measured executable input')
        self.record['source_stages']=[dict(label='native-final',files=[dict(name='crates/example.rs',sha256=source['sha256'])])]
        self.record['prior_records']=[save('prior.json',b'unchanged prior result')]
        for route in ('host','netbsd'):
            self.record['routes'][route]={}
            for name,unicode in [('classic',False),('unicode',True)]:
                data=dict(result='pass',platform='Darwin' if route=='host' else 'NetBSD',scope='main-menu-local-mail-files-disabled-same-ssh',unicode=unicode,count=len(self.module.CONTROLS),checks=sorted(self.module.CONTROLS),files_disabled=True,transfer_authority_required=False,same_client_pid=True,same_ssh_pid_until_explicit_disconnect=True,checkpoint_history_pending_cursor_exact=True,actual_board_io_private_scan=True,no_file_protocol_in_board_io=True,terminal_restored=True,no_automatic_redial=True,capacity_claim=False,physical_hardware=False,private_transcript_intervals=6,privacy_scan=dict(board_files=4,store_files=1,log_files=2,matches=0,symlinks=0),hostile_output=dict(written=16384,maximum=16384,writer_finished=True,owned_tty_verified=True))
                self.record['routes'][route][name]=save(route+'-'+name+'.json',data)
        values={name:dict(result='pass') for name in self.module.REQUIRED}
        values['native-source-match'].update(files={'crates/example.rs':source['sha256']},all_match=True)
        values['runtime'].update(physmem=1054990336,**self.module.PROFILE)
        values['cleanup'].update(owned_removed=True,saved_stopped=True,saved_hashes_unchanged=True,forwards_closed=True)
        values['tdd'].update(red_exit=101,green_exit=0,captured_before_edit=True)
        specs={'protocol':{'protocol':(12,'actual-ssh-mail-protocol-and-compatibility-files-disabled'),'hostile':(3,None)},'compatibility':{'compact':(3,None),'fallback':(4,None)},'legacy':{'classic':(14,'same-ssh-parked-files-local-mail-restricted-sftp'),'unicode':(14,'same-ssh-parked-files-local-mail-restricted-sftp'),'legacy':(2,'actual-old-and-default-native-v1-files-rpc')}}
        for category,kinds in specs.items():
            values[category]['routes']={}
            for platform in ('host','netbsd'):
                entries={};values[category]['routes'][platform]=entries
                for name,(count,scope) in kinds.items():
                    data=dict(result='pass',count=count,checks=sorted(self.module.EXPECTED_CONTROLS[name]),scope=self.module.EXPECTED_SCOPES[name],platform='Darwin' if platform=='host' else 'NetBSD',capacity_claim=False,physical_hardware=False,unicode=name=='unicode',restricted_sftp=True,files_disabled=True,no_private_unlock=True,normal_limits_unchanged=True,private_store_unchanged=True,normal_output_cap=32768,dumb_output_no_escape=True,compact_offer_preserved=True,no_shared_handoff_claim=True,private_mail_not_opened=True)
                    entries[name]=save(category+'-'+platform+'-'+name+'.json',data)
        values['private-io-rescan']=dict(result='pass',scope='postrun-retained-ktrace-hex-stream-reconstruction',checker_sha256=source['sha256'],fresh_ui_execution=False,stream_key=['pid','fd','direction'],routes={})
        for platform in ('host','netbsd'):
            for mode in ('classic','unicode'):
                values['private-io-rescan']['routes'][platform+'_'+mode]=dict(original_route_result_sha256=self.record['routes'][platform][mode]['sha256'],generated_marker_count=5,private_marker_matches=0,file_protocol_prefix_matches=0,input_trace_unchanged=True)
        for name,value in values.items():self.record['records'][name]=save(name+'.json',value)

    def test_ci_runs_the_immutable_record_guard(self):
        workflow=Path(__file__).resolve().parents[1]/'.github/workflows/documentation.yml'
        self.assertIn('python3 tools/check_menu_mail_evidence.py',workflow.read_text())

    def test_rejects_host_results_reused_as_native(self):
        record=copy.deepcopy(self.record);record['routes']['netbsd']=copy.deepcopy(record['routes']['host'])
        with self.assertRaises(AssertionError):self.module.validate(self.root,record)

    def test_rejects_empty_supplemental_pass_metadata(self):
        for name in ('protocol','compatibility','legacy'):
            record=copy.deepcopy(self.record);record['records'][name]=self.save('empty-'+name+'.json',dict(result='pass'))
            with self.assertRaises((AssertionError,KeyError)):self.module.validate(self.root,record)

    def test_rejects_substituted_supplemental_control(self):
        record=copy.deepcopy(self.record);aggregate=json.loads((self.root/record['records']['protocol']['path']).read_bytes());ref=aggregate['routes']['host']['protocol'];path=self.root/ref['path'];data=json.loads(path.read_bytes());data['checks'][0]='unrelated';payload=json.dumps(data).encode();path.write_bytes(payload)
        newsha=hashlib.sha256(payload).hexdigest();ref['sha256']=newsha
        for item in record['sources']:
            if item['path']==str(path.relative_to(self.root)):item['sha256']=newsha
        newaggregate=self.save('mutated-protocol.json',aggregate);record['records']['protocol']=newaggregate
        with self.assertRaises(AssertionError):self.module.validate(self.root,record)

    def test_rejects_missing_or_unbound_corrected_privacy_rescan(self):
        for change in ('remove','leak','wrong-route'):
            record=copy.deepcopy(self.record)
            if change=='remove':record['records'].pop('private-io-rescan')
            else:
                ref=record['records']['private-io-rescan'];data=json.loads((self.root/ref['path']).read_bytes())
                data['routes']['netbsd_unicode']['private_marker_matches' if change=='leak' else 'original_route_result_sha256']=1 if change=='leak' else '0'*64
                record['records']['private-io-rescan']=self.save('mutated-rescan-'+change+'.json',data)
            with self.assertRaises((AssertionError,KeyError)):self.module.validate(self.root,record)

    def publication_fixture(self):
        def store(name,value):
            data=json.dumps(value).encode() if isinstance(value,dict) else value;path=self.root/name;path.write_bytes(data);return dict(path=name,sha256=hashlib.sha256(data).hexdigest())
        previous=copy.deepcopy(self.record);source=store('snapshot.md',b'frozen relative Markdown');previous['sources'].append(source)
        current=copy.deepcopy(previous);moved=store('snapshot.raw',b'frozen relative Markdown');current['sources'][-1]=moved
        current['publication_layout']=dict(kind='frozen-markdown-path-relocation',no_measurements_refreshed=True,original_record=store('original.raw',previous),relocations=[dict(original_path=source['path'],new_path=moved['path'],sha256=source['sha256'])],sources=[])
        return current,previous,store

    def test_accepts_exact_frozen_markdown_relocation(self):
        current,_,_=self.publication_fixture();self.module.validate(self.root,current)

    def test_rejects_publication_measurement_refresh(self):
        current,previous,store=self.publication_fixture();previous['profile']['normal_sessions']=250
        current['publication_layout']['original_record']=store('changed-original.raw',previous)
        with self.assertRaises(AssertionError):self.module.validate(self.root,current)

    def test_accepts_complete_bounded_observations(self):
        self.module.validate(self.root,self.record)

    def test_rejects_changed_sources_or_extra_claims(self):
        for change in [lambda r:r['sources'][0].update(sha256='0'*64),lambda r:r['source_stages'][0]['files'][0].update(sha256='1'*64),lambda r:r['claims'].update(background_polling=True),lambda r:r['claims'].update(extra=False),lambda r:r['profile'].update(normal_sessions=250)]:
            record=copy.deepcopy(self.record);change(record)
            with self.assertRaises(AssertionError):self.module.validate(self.root,record)

    def test_rejects_missing_route_or_required_record(self):
        for change in [lambda r:r['routes']['netbsd'].pop('unicode'),lambda r:r['records'].pop('protocol'),lambda r:r['routes'].pop('host')]:
            record=copy.deepcopy(self.record);change(record)
            with self.assertRaises((AssertionError,KeyError)):self.module.validate(self.root,record)

    def test_rejects_incomplete_private_or_file_separation_controls(self):
        for key,value in [('checks',[]),('files_disabled',False),('same_ssh_pid_until_explicit_disconnect',False),('actual_board_io_private_scan',False),('no_file_protocol_in_board_io',False),('terminal_restored',False),('unicode',True),('transfer_authority_required',True),('privacy_scan',dict(board_files=4,store_files=1,log_files=2,matches=1,symlinks=0)),('hostile_output',dict(written=0,maximum=0,writer_finished=False,owned_tty_verified=True))]:
            record=copy.deepcopy(self.record);ref=record['routes']['host']['classic'];path=self.root/ref['path'];original=path.read_bytes();data=json.loads(original);data[key]=value;payload=json.dumps(data).encode();path.write_bytes(payload);ref['sha256']=hashlib.sha256(payload).hexdigest()
            # The underlying reference also appears in sources: preserve coherent
            # hashes so rejection must come from the changed proof assertion.
            next(x for x in record['sources'] if x['path']==ref['path'])['sha256']=ref['sha256']
            try:
                with self.assertRaises(AssertionError):self.module.validate(self.root,record)
            finally:path.write_bytes(original)

    def test_rejects_retained_leaks_or_missing_cleanup_red(self):
        for name,key,value in [('cleanup','owned_removed',False),('tdd','red_exit',0),('tdd','captured_before_edit',False),('runtime','physmem',2*1024**3)]:
            record=copy.deepcopy(self.record);ref=record['records'][name];path=self.root/ref['path'];original=path.read_bytes();data=json.loads(original);data[key]=value;payload=json.dumps(data).encode();path.write_bytes(payload);ref['sha256']=hashlib.sha256(payload).hexdigest();next(x for x in record['sources'] if x['path']==ref['path'])['sha256']=ref['sha256']
            try:
                with self.assertRaises(AssertionError):self.module.validate(self.root,record)
            finally:path.write_bytes(original)

if __name__=='__main__':unittest.main()
