#!/usr/bin/env python3
"""Check the exact bounded device-local passport software record offline."""
import hashlib,json
from evidence_sources import ROOT,source_bytes,connection_desk_baseline_manifest

def check():
 d=json.loads((ROOT/'dev-docs/evidence/passport-netbsd/results.json').read_text())
 assert d['schema']==1 and d['result']=='pass'
 assert d['native_guest']==dict(system='NetBSD',release='11.0',architecture='aarch64',cores=2,ram_mib=1024)
 assert d['native_guest_uname'].startswith('NetBSD 11.0')
 assert d['host_workspace_rust_tests']==365 and d['host_python_tests']==130 and d['host_client_tests']==d['native_client_tests']==25
 assert d['host_strict_workspace_clippy'] and d['native_frozen_build'] and d['new_native_project_before_build']
 assert len(d['client_execution_sources'])==65 and len(d['harness_sources'])==7 and d['selected_native_source_matches']==72
 assert len(d['unchanged_prior_nonclient_execution_sources'])==317
 for group in ['client_execution_sources','harness_sources','unchanged_prior_nonclient_execution_sources']:
  for n,h in d[group].items():assert hashlib.sha256(source_bytes(n,h)).hexdigest()==h,n
 m=connection_desk_baseline_manifest();assert len(m['files'])==330 and len(m['records'])==1
 assert hashlib.sha256((ROOT/'dev-docs/evidence/connection-desk-baseline-manifest.json').read_bytes()).hexdigest()==d['prior_connection_desk_manifest_sha256']
 path='dev-docs/evidence/connection-desk-netbsd/results.json';old=json.loads((ROOT/path).read_text())
 assert m['records'][path]==d['prior_connection_desk_result_sha256']==hashlib.sha256((ROOT/path).read_bytes()).hexdigest()
 assert d['unchanged_prior_nonclient_execution_sources']==old['unchanged_prior_nonclient_execution_sources']
 for group in ['host_passport','native_passport']:
  v=d[group];assert v['result']=='pass' and v['count']==len(v['checks'])==len(set(v['checks']))==16
  assert v['device_local'] and v['plaintext_notes']
  assert not any(v[k] for k in ['automatic_visit_tracking','verified_board_identity','network_io','mail_change','capacity_claim','hardening_claim'])
  assert {'original_nightwire_calling_card_imports_with_required_rights_and_inert_art','too_small_terminal_cannot_confirm_unseen_content_and_resize_preserves_review','private_fixed_storage_has_no_pending_files_network_io_or_travel_history','removed_contact_notes_remain_viewable_exportable_and_explicitly_deletable','changed_profile_is_never_silently_rebound_and_seal_import_requires_review','local_share_preview_defaults_private_fields_off_and_requires_explicit_toggles'}<=set(v['checks'])
 for group in ['host_connection_desk','native_connection_desk']:
  v=d[group];assert v['result']=='pass' and v['count']==len(v['checks'])==14 and v['local_only']
  assert not v['network_probe'] and not v['host_verification']
 for group in ['host_regressions','native_regressions']:
  for name,count in [('launcher',7),('travel',6),('vault_picker',8),('vault',13)]:
   v=d[group][name];assert v['result']=='pass' and v['count']==len(v['checks'])==count
 for key in ['host_client_sha256','native_client_sha256','host_pin_helper_sha256','native_pin_helper_sha256']:
  h=d[key];assert len(h)==64 and set(h)<=set('0123456789abcdef')
 assert d['budgets']==dict(normal_sessions=8,admission_refill_ms=5000,credential_permits=2,owner_helpers=3,new_registry_packages=0,new_server_helpers=0,new_schema=0,notebook_entries=32,notebook_bytes=65536,seal_file_bytes=4096,seal_rows=8,seal_columns=38,seal_bytes=311,note_bytes=512,favorite_bytes=96)
 assert all(d['features'].values()) and all(d['teardown'].values()) and not any(d['claims'].values())
 print('Passport evidence passed: 25 client tests/16 local PTYs per OS, 365 host Rust/130 Python, 72 exact native inputs; explicit local souvenirs/notes and unchanged server, no tracking/identity/capacity claim.')
if __name__=='__main__':check()
