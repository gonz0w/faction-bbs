import copy,hashlib,json,unittest
from unittest.mock import patch
import check_xmpp_transport_evidence as guard

def fixture():
 names=sorted(guard.REQUIRED)
 matrix=dict(result='pass',count=len(names),checks=names,private_fixture_removed=True,product_adopted=False,live_federation=False,outcomes=[dict(check=n,result='pass',status=guard.EXPECTED[n],attempts=1,tls_version='TLSv1.2' if n=='verified_tls12_profile' else 'TLSv1.3') for i,n in enumerate(names)])
 return dict(schema=1,issue=208,result='qualified-research-direct-tls',profile=dict(stack='tokio-xmpp6.0.0',tls='verified-direct',sasl='PLAIN',attempts=1),limits=dict(xml_bytes=65536,interval_bytes=16384,depth=32,events=256,lifetime_seconds=10),runs={n:dict(rust_passed=17,matrix=copy.deepcopy(matrix)) for n in ['host','netbsd']},
  build=dict(failed_1g=[dict(jobs=j,result='compiler-sigkill',exit=101) for j in [1,2]],kernel_confirmed_jobs=[2],complete_builder_mib=2048,complete_builder_tests=16),runtime=dict(configured_mib=1024,incremental_build_exit=0),
  cost=dict(result='pass',usage=dict(max_rss_kib=9988),private_fixture_removed=True,compiler_included=False,fixture_included=False,capacity_claim=False),
  execution_sources={'probe.rs':hashlib.sha256(b'executed').hexdigest()},frozen_sources={'old.rs':hashlib.sha256(b'executed').hexdigest()},teardown={k:True for k in guard.TEARDOWN},claims={k:False for k in guard.FALSE_CLAIMS})
class TransportEvidence(unittest.TestCase):
 def test_complete_bounded_source_matched_qualification(self):guard.validate(fixture(),lambda n,h:b'executed')
 def test_observed_tls_versions_are_backend_specific_not_invented(self):
  d=fixture()
  for row in d['runs']['host']['matrix']['outcomes']:row['tls_version']='TLSv1.2'
  guard.validate(d,lambda n,h:b'executed')
 def test_failed_or_missing_transport_tls_and_stanza_controls_refuse(self):
  for change in [lambda d:d['runs']['netbsd'].update(rust_passed=0),lambda d:d['runs']['host']['matrix']['checks'].pop(),lambda d:d['runs']['netbsd']['matrix']['outcomes'][0].update(tls_version='TLSv1')]:
   d=fixture();change(d)
   with self.assertRaises(AssertionError):guard.validate(d,lambda n,h:b'executed')
 def test_compiler_runtime_conflation_and_capacity_claims_refuse(self):
  for change in [lambda d:d['build'].update(complete_builder_mib=1024),lambda d:d['build'].update(failed_1g=[]),lambda d:d['runtime'].update(configured_mib=2048),lambda d:d['cost'].update(compiler_included=True)]:
   d=fixture();change(d)
   with self.assertRaises(AssertionError):guard.validate(d,lambda n,h:b'executed')
 def test_tampered_source_unremoved_fixture_and_overclaims_refuse(self):
  with self.assertRaises(AssertionError):guard.validate(fixture(),lambda n,h:b'wrong')
  for field,keys in [('teardown',guard.TEARDOWN),('claims',guard.FALSE_CLAIMS)]:
   for key in keys:
    d=fixture();d[field][key]=field=='claims'
    with self.assertRaises(AssertionError):guard.validate(d,lambda n,h:b'executed')
 def test_rejected_observation_pin_correction_cannot_refresh_measurements(self):
  original=fixture();raw=json.dumps(original).encode();d=copy.deepcopy(original)
  name='intermediate.json';digest=hashlib.sha256(b'old observation').hexdigest()
  d['frozen_sources'][name]=digest
  d['observation_pin_correction']=dict(original_record_sha256=hashlib.sha256(raw).hexdigest(),added_observation_path=name)
  guard.verify_observation_pin_correction(d,raw)
  for change in [lambda x:x['cost']['usage'].update(max_rss_kib=1),lambda x:x['frozen_sources'].update({'old.rs':'a'*64})]:
   changed=copy.deepcopy(d);change(changed)
   with self.assertRaises(AssertionError):guard.verify_observation_pin_correction(changed,raw)
 def test_result_is_immutable_and_required_ci_checks_it(self):
  raw=json.dumps(fixture()).encode()
  with patch.object(guard,'RECORD_SHA256',hashlib.sha256(raw).hexdigest()):
   self.assertEqual(guard.load_record(raw)['issue'],208)
   with self.assertRaises(AssertionError):guard.load_record(raw+b' ')
  from pathlib import Path
  self.assertIn('python3 tools/check_xmpp_transport_evidence.py',(Path(__file__).resolve().parents[1]/'.github/workflows/documentation.yml').read_text())
if __name__=='__main__':unittest.main()
