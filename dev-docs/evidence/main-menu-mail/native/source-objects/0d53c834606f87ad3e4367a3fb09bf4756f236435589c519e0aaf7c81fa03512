import copy
import hashlib
import json
from pathlib import Path
import tempfile
import unittest
import check_roaming_evidence as e

class RoamingEvidenceTests(unittest.TestCase):
    def setUp(self):
        self.tmp=tempfile.TemporaryDirectory();self.addCleanup(self.tmp.cleanup)
        self.root=Path(self.tmp.name)
        def ref(name,data):
            raw=data if isinstance(data,bytes) else json.dumps(data).encode()
            (self.root/name).write_bytes(raw)
            return dict(path=name,sha256=hashlib.sha256(raw).hexdigest())
        src=ref('source',b'executed source')
        self.result=dict(schema=1,issue=223,result='pass',sources=[src],source_stages=[dict(label='native-final',files=[dict(name='crates/example.rs',sha256=src['sha256'])])],proof_profile=dict(boards=2,homes=2,bindings=4,certificate_seconds=300,sequential_boards=True,default_off=True),claims=dict(production=False,wire_replay=False,leaf_key_pin=False,physical_hardware=False,capacity=False),records={})
        values={name:dict(result='pass') for name in e.REQUIRED}
        for name in ('host-workspace','native-workspace'):values[name]['passed']=1
        values['native-source-match'].update(all_match=True,files={'crates/example.rs':src['sha256']})
        values['host-oracle'].update(checks=7,wire_replay_claim=False)
        values['native-oracle'].update(checks=7,wire_replay_claim=False)
        values['runtime'].update(normal_sessions=8,admission_refill_ms=5000,credential_permits=2,kernel_slots=2,owner_helpers=3,drain_seconds=5,physmem=1054990336)
        values['cleanup'].update(owned_removed=True,saved_stopped=True,saved_hashes_unchanged=True,forwards_closed=True)
        for name,data in values.items():self.result['records'][name]=ref(name,data)
    def test_committed_record_and_all_bound_inputs(self):
        e.check()
    def test_complete_source_bound_result(self):
        e.validate(self.root,self.result)
    def test_tampering_and_path_escape_fail(self):
        for change in ('digest','escape','bytes'):
            bad=copy.deepcopy(self.result)
            if change=='digest':bad['sources'][0]['sha256']='0'*64
            elif change=='escape':bad['sources'][0]['path']='../outside'
            else:(self.root/'source').write_bytes(b'changed')
            with self.assertRaises((AssertionError,ValueError)):e.validate(self.root,bad)
    def test_missing_proof_or_unsafe_claim_fails(self):
        for change in ('missing','claim','profile'):
            bad=copy.deepcopy(self.result)
            if change=='missing':del bad['records']['native-workspace']
            elif change=='claim':bad['claims']['production']=True
            else:bad['proof_profile']['bindings']=5
            with self.assertRaises((AssertionError,ValueError)):e.validate(self.root,bad)
    def test_native_source_mismatch_fails(self):
        bad=copy.deepcopy(self.result);bad['source_stages'][0]['files'][0]['sha256']='0'*64
        with self.assertRaises((AssertionError,ValueError)):e.validate(self.root,bad)

if __name__=='__main__':unittest.main()
