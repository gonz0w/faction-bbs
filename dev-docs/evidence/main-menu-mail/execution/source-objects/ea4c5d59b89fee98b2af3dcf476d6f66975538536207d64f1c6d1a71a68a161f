import copy,hashlib,json,unittest
from unittest.mock import patch
from pathlib import Path
import check_checkpoint_evidence as guard
ROOT=Path(__file__).resolve().parents[1]
def fixture():
 old=json.loads((ROOT/'dev-docs/evidence/omemo-interop/results.json').read_text())
 row=dict(checkpoint_passed=17,checkpoint_failed=0,checkpoint_ignored_helper=1,helper_invoked_by_parent_tests=True,client_passed=21,native_passed=27,disabled_passed=1,checkpoint_checks=sorted(guard.REQUIRED_CHECKS),vault_count=13,picker_count=8)
 return dict(schema=1,issue=203,result='pass-experimental-native-checkpoints',default_feature_off=True,
  profile=dict(namespace='urn:xmpp:omemo:2',sce='urn:xmpp:sce:1',timestamp_subset='UTC-whole-seconds-1970-9999'),
  limits=dict(plain=524288,history=64,outbox=8,peers=1,snapshot=32768,skipped_keys=32,session_seconds=2592000),
  runs=dict(host=copy.deepcopy(row),netbsd=copy.deepcopy(row)),interop=copy.deepcopy(old['corrected']),
  claims={k:False for k in guard.FALSE_CLAIMS},teardown={k:True for k in guard.TEARDOWN},
  native_input_capture={'sample.rs':hashlib.sha256(b'executed').hexdigest()},frozen_sources={'frozen.rs':hashlib.sha256(b'frozen').hexdigest()})
def resolve(name,digest):return b'frozen' if name=='frozen.rs' else b'executed'
class CheckpointEvidence(unittest.TestCase):
 def test_complete_bounded_software_record(self):guard.validate(fixture(),resolve)
 def test_required_ci_checks_saved_checkpoint_result(self):self.assertIn('python3 tools/check_checkpoint_evidence.py',(ROOT/'.github/workflows/documentation.yml').read_text())
 def test_failed_native_or_missing_crash_recovery_never_passes(self):
  for mutate in [lambda d:d['runs']['netbsd'].update(checkpoint_failed=1),lambda d:d['runs']['netbsd'].update(native_passed=0),lambda d:d['runs']['host'].update(helper_invoked_by_parent_tests=False),lambda d:d['runs']['netbsd']['checkpoint_checks'].remove('process_death_after_delivery_before_ack_retries_identical_ciphertext')]:
   d=fixture();mutate(d)
   with self.assertRaises(AssertionError):guard.validate(d,resolve)
 def test_reference_failure_profile_and_limit_changes_refuse(self):
  for mutate in [lambda d:d['interop']['netbsd'].update(result='fail'),lambda d:d['interop']['host'].update(independent_two_way=False),lambda d:d['profile'].update(namespace='legacy'),lambda d:d['limits'].update(outbox=999)]:
   d=fixture();mutate(d)
   with self.assertRaises(AssertionError):guard.validate(d,resolve)
 def test_product_claims_tampered_source_and_missing_cleanup_refuse(self):
  with self.assertRaises(AssertionError):guard.validate(fixture(),lambda n,h:b'wrong')
  for name in guard.FALSE_CLAIMS:
   d=fixture();d['claims'][name]=True
   with self.assertRaises(AssertionError):guard.validate(d,resolve)
  d=fixture();d['teardown']['owned_vm_removed']=False
  with self.assertRaises(AssertionError):guard.validate(d,resolve)
 def test_capture_correction_cannot_refresh_runtime_hashes_or_outcomes(self):
  original=fixture();original['native_input_capture']['crates/.DS_Store']='a'*64
  raw=json.dumps(original).encode();current=copy.deepcopy(original)
  del current['native_input_capture']['crates/.DS_Store']
  current['capture_correction']=dict(original_record_sha256=hashlib.sha256(raw).hexdigest(),excluded_non_build_metadata=['crates/.DS_Store'])
  guard.verify_capture_correction(current,raw)
  for mutate in [lambda d:d['native_input_capture'].update({'sample.rs':'b'*64}),lambda d:d['runs']['netbsd'].update(checkpoint_passed=99),lambda d:d['capture_correction'].update(excluded_non_build_metadata=['sample.rs'])]:
   d=copy.deepcopy(current);mutate(d)
   with self.assertRaises(AssertionError):guard.verify_capture_correction(d,raw)
 def test_post_proof_build_fix_preserves_original_inputs_and_platform_scope(self):
  old=hashlib.sha256(b'old build').hexdigest();new=hashlib.sha256(b'new build').hexdigest()
  follow=dict(old_inputs={'build.rs':old},preserved_inputs={'build.rs':'frozen-build.rs'},current_inputs={'build.rs':new},platform_flags={'linux':['-D_DEFAULT_SOURCE'],'netbsd':['-D_NETBSD_SOURCE'],'macos':[]},runtime_code_changed=False,netbsd_flags_changed=False,vm_replayed_after_change=False,observations_refreshed=False)
  def resolve_build(name,digest):return b'old build' if name=='frozen-build.rs' else b'new build'
  guard.validate_build_followup(follow,resolve_build)
  self.assertEqual(guard.resolve_followup_source(follow,'build.rs',old,resolve_build),b'old build')
  for mutate in [lambda d:d.update(runtime_code_changed=True),lambda d:d.update(netbsd_flags_changed=True),lambda d:d.update(observations_refreshed=True),lambda d:d['preserved_inputs'].update({'build.rs':'wrong.rs'}),lambda d:d['current_inputs'].update({'build.rs':'a'*64})]:
   d=copy.deepcopy(follow);mutate(d)
   with self.assertRaises(AssertionError):guard.validate_build_followup(d,resolve_build)
 def test_immutable_record_rejects_even_valid_json_rewrites(self):
  raw=json.dumps(fixture()).encode()
  with patch.object(guard,'RECORD_SHA256',hashlib.sha256(raw).hexdigest()):
   self.assertEqual(guard.load_record(raw)['issue'],203)
   with self.assertRaises(AssertionError):guard.load_record(raw+b' ')
if __name__=='__main__':unittest.main()
