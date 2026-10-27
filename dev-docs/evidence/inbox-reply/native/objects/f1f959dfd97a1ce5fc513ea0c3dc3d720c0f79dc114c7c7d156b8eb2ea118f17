"""Protect source-bound directory observations and honest execution/cleanup claims."""
import json,tempfile,unittest
from pathlib import Path
from check_directory_submissions_evidence import validate, match_inputs

class EvidenceChecks(unittest.TestCase):
    def test_native_match_binds_names_and_digests(self):
        stage={'label':'candidate03','files':[{'name':'core.rs','sha256':'a'*64}]}
        match_inputs([stage], {'files':{'core.rs':'a'*64}})
        with self.assertRaises(AssertionError):
            match_inputs([stage], {'files':{'wrong.rs':'a'*64}})
        with self.assertRaises(AssertionError):
            match_inputs([stage], {'files':{'core.rs':'b'*64}})
    def fixture(self):
        self.tmp=tempfile.TemporaryDirectory();self.addCleanup(self.tmp.cleanup);root=Path(self.tmp.name)
        record={'schema':1,'issue':184,'result':'pass','sources':[],'records':{},'native':{'matched_inputs':0,'compile_memory_mib':2048,'runtime_memory_bytes':1054990336},'teardown':{'owned_removed':True,'saved_stopped':True,'saved_hashes_unchanged':True,'forwards_closed':True},'claims':{'capacity':False,'hardening':False,'physical_hardware':False}}
        return root,record
    def test_incomplete_observations_cannot_claim_success(self):
        root,r=self.fixture()
        with self.assertRaises((AssertionError,ValueError)):validate(root,r)
    def test_published_record_verifies(self):
        root=Path(__file__).resolve().parents[1]
        r=json.loads((root/'dev-docs/evidence/directory-submissions/results.json').read_text())
        validate(root,r)
    def test_cleanup_and_source_tampering_are_rejected(self):
        root=Path(__file__).resolve().parents[1];r=json.loads((root/'dev-docs/evidence/directory-submissions/results.json').read_text())
        r['teardown']['saved_stopped']=False
        with self.assertRaises(AssertionError):validate(root,r)
        r['teardown']['saved_stopped']=True
        r['sources'][0]['sha256']='0'*64
        with self.assertRaises(AssertionError):validate(root,r)

if __name__=='__main__':unittest.main()
