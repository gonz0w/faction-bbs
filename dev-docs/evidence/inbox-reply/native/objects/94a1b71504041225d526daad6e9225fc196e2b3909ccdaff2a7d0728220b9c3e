import copy,hashlib,json,unittest
from unittest.mock import patch
import check_checkpoint_console_evidence as guard

def fixture():
 row=dict(result='pass',count=len(guard.REQUIRED),checks=sorted(guard.REQUIRED),fixture_children_removed=True,network_mail=False,hardware_claim=False)
 return dict(schema=1,issue=205,result='pass-local-checkpoint-console',default_feature_off=True,
  runs=dict(host=dict(row,platform='Darwin'),netbsd=dict(row,platform='NetBSD')),
  execution_sources={'sample.rs':hashlib.sha256(b'executed').hexdigest()},
  frozen_sources={'frozen.rs':hashlib.sha256(b'executed').hexdigest()},
  rust=dict(host_passed=72,netbsd_passed=72,intentional_ignored_helper=1,helper_invoked_by_parent=True),
  default_off={route:dict(result='pass',unsupported_command_rejected=True,store_not_created=True) for route in ['host','netbsd']},
  compatibility={route+'_'+kind:dict(result='pass',count=count) for route in ['host','netbsd'] for kind,count in [('vault',13),('picker',8)]},
  teardown={k:True for k in guard.TEARDOWN},claims={k:False for k in guard.FALSE_CLAIMS})
class ConsoleEvidence(unittest.TestCase):
 def test_complete_source_matched_local_console_record(self):guard.validate(fixture(),lambda n,h:b'executed')
 def test_missing_or_failed_route_and_omitted_acceptance_refuse(self):
  for change in [lambda d:d['runs']['netbsd'].update(result='fail'),lambda d:d['runs']['host']['checks'].pop(),lambda d:d['runs']['host'].update(fixture_children_removed=False),lambda d:d.update(default_feature_off=False)]:
   d=fixture();change(d)
   with self.assertRaises(AssertionError):guard.validate(d,lambda n,h:b'executed')
 def test_tampered_sources_and_overclaims_and_incomplete_teardown_refuse(self):
  with self.assertRaises(AssertionError):guard.validate(fixture(),lambda n,h:b'wrong')
  for field,keys in [('claims',guard.FALSE_CLAIMS),('teardown',guard.TEARDOWN)]:
   for key in keys:
    d=fixture();d[field][key]=field=='claims'
    with self.assertRaises(AssertionError):guard.validate(d,lambda n,h:b'executed')
 def test_default_off_and_existing_vault_picker_regressions_are_required(self):
  for change in [lambda d:d['default_off']['host'].update(store_not_created=False),lambda d:d['compatibility']['netbsd_vault'].update(count=0),lambda d:d['rust'].update(helper_invoked_by_parent=False)]:
   d=fixture();change(d)
   with self.assertRaises(AssertionError):guard.validate(d,lambda n,h:b'executed')
 def test_record_rewrite_and_missing_required_ci_guard_refuse(self):
  raw=json.dumps(fixture()).encode()
  with patch.object(guard,'RECORD_SHA256',hashlib.sha256(raw).hexdigest()):
   self.assertEqual(guard.load_record(raw)['issue'],205)
   with self.assertRaises(AssertionError):guard.load_record(raw+b' ')
  from pathlib import Path
  self.assertIn('python3 tools/check_checkpoint_console_evidence.py',(Path(__file__).resolve().parents[1]/'.github/workflows/documentation.yml').read_text())
if __name__=='__main__':unittest.main()
