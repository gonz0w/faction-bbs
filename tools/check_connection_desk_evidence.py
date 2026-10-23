#!/usr/bin/env python3
"""Validate the bounded device-local connection-desk software record offline."""
import hashlib,json
from evidence_sources import ROOT,source_bytes,who_baseline_manifest

def check():
 d=json.loads((ROOT/'dev-docs/evidence/connection-desk-netbsd/results.json').read_text())
 assert d['schema']==1 and d['result']=='pass'
 assert d['native_guest']==dict(system='NetBSD',release='11.0',architecture='aarch64',cores=2,ram_mib=1024)
 assert d['native_guest_uname'].startswith('NetBSD 11.0')
 assert d['host_workspace_rust_tests']==361 and d['host_python_tests']==130
 assert d['host_client_tests']==d['native_client_tests']==21
 assert d['host_strict_workspace_clippy'] and d['native_frozen_build'] and d['new_native_project_before_build']
 assert len(d['client_execution_sources'])==61 and len(d['harness_sources'])==6 and d['selected_native_source_matches']==67
 assert len(d['unchanged_prior_nonclient_execution_sources'])==317
 for group in ['client_execution_sources','harness_sources','unchanged_prior_nonclient_execution_sources']:
  for name,digest in d[group].items():assert hashlib.sha256(source_bytes(name,digest)).hexdigest()==digest,name
 m=who_baseline_manifest();assert len(m['files'])==338 and len(m['records'])==1
 assert hashlib.sha256((ROOT/'dev-docs/evidence/who-baseline-manifest.json').read_bytes()).hexdigest()==d['prior_who_manifest_sha256']
 path='dev-docs/evidence/who-netbsd/results.json';old=json.loads((ROOT/path).read_text())
 assert m['records'][path]==d['prior_who_result_sha256']==hashlib.sha256((ROOT/path).read_bytes()).hexdigest()
 assert d['unchanged_prior_nonclient_execution_sources']=={n:h for n,h in old['execution_sources'].items() if not n.startswith('crates/faction-client/')}
 for group in ['host_connection_desk','native_connection_desk']:
  v=d[group];assert v['result']=='pass' and v['count']==len(v['checks'])==14 and len(set(v['checks']))==14
  assert v['plain_report'] and v['local_only'] and v['helper']=='/usr/bin/ssh-keygen'
  assert v['helper_seconds']==2 and v['helper_bytes']==32768 and v['fingerprints_per_endpoint']==4
  assert not any(v[k] for k in ['network_probe','host_verification','admission_claim','tor_support','capacity_claim','hardening_claim'])
  assert {'ctrl_c_cancels_an_active_pin_lookup_and_reaps_before_return','small_resize_uses_recovery_screen_and_reload_returns_to_bounded_pages','hashed_known_hosts_are_matched_by_base_openssh_without_mutating_pins','all_diagnostics_and_pin_checks_make_no_network_connection','lookup_output_is_bounded_and_failure_returns_to_the_desk'} <= set(v['checks'])
 for group in ['host_regressions','native_regressions']:
  assert set(d[group])=={'launcher','travel','vault_picker','vault'}
  for name,count in [('launcher',7),('travel',6),('vault_picker',8),('vault',13)]:
   v=d[group][name];assert v['result']=='pass' and v['count']==len(v['checks'])==count
 for key in ['host_client_sha256','native_client_sha256','host_pin_helper_sha256','native_pin_helper_sha256']:
  h=d[key];assert len(h)==64 and set(h)<=set('0123456789abcdef')
 assert d['budgets']==dict(normal_sessions=8,admission_refill_ms=5000,credential_permits=2,owner_helpers=3,new_registry_packages=0,new_server_helpers=0,new_schema=0,local_helpers_at_once=1,lookup_deadline_seconds=2,lookup_output_bytes=32768,fingerprints_per_endpoint=4,known_host_file_bytes=1048576)
 assert all(d['features'].values()) and all(d['teardown'].values()) and not any(d['claims'].values())
 print('Connection desk evidence passed: 21 client tests/14 local PTY controls per OS, 361 host workspace tests, 130 Python; 317 non-client source identities unchanged, no live authentication/network/capacity claim.')
if __name__=='__main__':check()
