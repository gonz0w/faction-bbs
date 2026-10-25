#!/usr/bin/env python3
"""Verify immutable #208 software qualification; never start a client or VM."""
import hashlib,json,re
from evidence_sources import ROOT,source_bytes
RECORD_SHA256='12a94d6a0d50c4351b36dff5a022bbd0d82279f449ade74a82f1a00cfebcd500'
ORIGINAL_RECORD_SHA256='cbb314bd3f12008fc4b20dcd7b3e1646a91a907f66372ddcd444bde40b75a50b'
EXPECTED={
 'verified_tls_plain_auth_and_ciphertext_exchange':'ok',
 'verified_tls12_profile':'ok',
 'fragmented_xml_exchange':'ok',
 'coalesced_sasl_restart_retains_unread_bytes':'ok',
 'wrong_server_name':'tls',
 'untrusted_certificate':'tls',
 'changed_certificate_pin':'pin',
 'failed_account_authentication':'auth',
 'no_plain_mechanism':'mechanism',
 'malformed_xml_rejected':'xml_malformed',
 'excessive_depth_rejected_before_dom':'xml_limit',
 'excessive_events_rejected_before_dom':'xml_limit',
 'oversized_unfinished_text_rejected':'xml_limit',
 'oversized_stanza_rejected':'xml_limit',
 'root_whitespace_budget_is_bounded':'xml_limit',
 'connection_loss_has_no_retry':'lost',
 'absolute_deadline_bounds_trickling_peer':'timeout',
 'explicit_cancellation_bounds_stalled_peer':'cancelled'}
REQUIRED=set(EXPECTED)
TEARDOWN={'owned_vm_removed','owned_private_inputs_removed','ports_closed','saved_playground_stopped','saved_playground_unchanged'}
FALSE_CLAIMS={'product_adoption','live_mailbox','complete_omemo_codec','full_conformance','security_audit','physical_hardware','capacity','fresh_1g_release_build'}
def load_record(raw):
 assert hashlib.sha256(raw).hexdigest()==RECORD_SHA256,'saved transport qualification changed'
 return json.loads(raw)
def validate(d,resolve):
 assert d['schema']==1 and d['issue']==208 and d['result']=='qualified-research-direct-tls'
 assert d['profile']==dict(stack='tokio-xmpp6.0.0',tls='verified-direct',sasl='PLAIN',attempts=1)
 assert d['limits']==dict(xml_bytes=65536,interval_bytes=16384,depth=32,events=256,lifetime_seconds=10)
 assert set(d['runs'])=={'host','netbsd'}
 for route in d['runs'].values():
  assert route['rust_passed']==17
  m=route['matrix'];assert m['result']=='pass' and m['count']==len(m['checks'])==len(set(m['checks']))==18
  assert set(m['checks'])==REQUIRED and m['private_fixture_removed'] is True
  assert m['product_adopted'] is False and m['live_federation'] is False
  assert len(m['outcomes'])==18 and {x['check'] for x in m['outcomes']}==REQUIRED
  for row in m['outcomes']:
   assert row['result']=='pass' and row['status']==EXPECTED[row['check']] and row['attempts']==1
   assert row['tls_version'] in {None,'TLSv1.2','TLSv1.3'}
   if row['status']=='ok':assert row['tls_version'] is not None
   if row['check']=='verified_tls12_profile':assert row['tls_version']=='TLSv1.2'
 assert {'TLSv1.2','TLSv1.3'}<={x['tls_version'] for route in d['runs'].values() for x in route['matrix']['outcomes']}
 b=d['build'];assert b['complete_builder_mib']==2048 and b['complete_builder_tests']==16
 assert len(b['failed_1g'])==2 and {x['jobs'] for x in b['failed_1g']}=={1,2}
 assert all(x['result']=='compiler-sigkill' and x['exit']==101 for x in b['failed_1g'])
 assert b['kernel_confirmed_jobs']==[2]
 assert d['runtime']['configured_mib']==1024 and d['runtime']['incremental_build_exit']==0
 c=d['cost'];assert c['result']=='pass' and c['usage']['max_rss_kib']>0 and c['private_fixture_removed'] is True
 assert c['compiler_included'] is False and c['fixture_included'] is False and c['capacity_claim'] is False
 assert set(d['teardown'])==TEARDOWN and all(x is True for x in d['teardown'].values())
 assert set(d['claims'])==FALSE_CLAIMS and all(x is False for x in d['claims'].values())
 for group in ['execution_sources','frozen_sources']:
  assert d[group]
  for name,digest in d[group].items():
   assert re.fullmatch('[a-f0-9]{64}',digest)
   assert hashlib.sha256(resolve(name,digest)).hexdigest()==digest,name

def verify_observation_pin_correction(current,original_raw):
 c=current['observation_pin_correction'];assert hashlib.sha256(original_raw).hexdigest()==c['original_record_sha256']
 old=json.loads(original_raw);name=c['added_observation_path']
 assert name not in old['frozen_sources']
 assert set(current['frozen_sources'])==set(old['frozen_sources'])|{name}
 assert {n:h for n,h in current['frozen_sources'].items() if n!=name}==old['frozen_sources'],'prior source identity refreshed'
 assert {n:v for n,v in current.items() if n not in {'frozen_sources','observation_pin_correction'}}=={n:v for n,v in old.items() if n!='frozen_sources'},'measured observation refreshed'


def check():
 d=load_record((ROOT/'dev-docs/evidence/xmpp-transport/results.json').read_bytes());validate(d,source_bytes)
 raw=(ROOT/'dev-docs/evidence/xmpp-transport/initial-record.json').read_bytes()
 assert hashlib.sha256(raw).hexdigest()==ORIGINAL_RECORD_SHA256
 verify_observation_pin_correction(d,raw)
 print('Saved native XMPP research qualification verified: host/NetBSD bounds/TLS/auth/cancel, distinct2GiB build/1GiB runtime and owned cleanup; no product/live-mail/audit/capacity claim.')
if __name__=='__main__':check()
