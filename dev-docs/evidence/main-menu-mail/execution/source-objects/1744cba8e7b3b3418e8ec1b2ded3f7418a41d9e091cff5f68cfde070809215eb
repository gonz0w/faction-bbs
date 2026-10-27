"""Independent guards for the same-SSH parked-mail acceptance oracle."""
import copy
import unittest
import probe_visit_mail as proof

class VisitProofOracle(unittest.TestCase):
    def test_requires_identical_live_client_and_ssh_identity(self):
        before={'client':41,'ssh':42}
        proof.assert_same_visit(before,dict(before))
        for after in [{'client':41,'ssh':43},{'client':43,'ssh':42},{'client':41,'ssh':0}]:
            with self.assertRaises(AssertionError):proof.assert_same_visit(before,after)

    def test_private_interval_exclusion_does_not_hide_remote_leak(self):
        transcript=b'BOARD START\nPRIVATE BODY\nBOARD END'
        spans=[(0,12),(25,len(transcript))]
        proof.require_private_absent(transcript,spans,[b'PRIVATE BODY'])
        with self.assertRaises(AssertionError):proof.require_private_absent(transcript,[(0,len(transcript))],[b'PRIVATE BODY'])
        with self.assertRaises(AssertionError):proof.require_private_absent(transcript,[],[b'PRIVATE BODY'])

    def test_local_read_cannot_change_history_pending_or_cursor(self):
        before={'canonical':b'old','history':[{'body':'private'}],'pending':[{'operation':'01'*16}],'archive':{'checkpoint':{'after':'cursor','epoch':'a'}}}
        after=copy.deepcopy(before);after['canonical']=b'new'
        proof.require_only_read_metadata_changed(before,after)
        for field in ['history','pending','archive']:
            bad=copy.deepcopy(after);bad[field]=None
            with self.assertRaises(AssertionError):proof.require_only_read_metadata_changed(before,bad)
        with self.assertRaises(AssertionError):proof.require_only_read_metadata_changed(before,before)

if __name__=='__main__':unittest.main()
