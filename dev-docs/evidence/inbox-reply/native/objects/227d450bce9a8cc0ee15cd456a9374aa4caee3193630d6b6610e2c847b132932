#!/usr/bin/env python3
"""Validate the bounded read-only board-guide software record offline."""
import hashlib,json
from evidence_sources import ROOT,source_bytes,passport_baseline_manifest

def check():
 d=json.loads((ROOT/'dev-docs/evidence/board-guide-netbsd/results.json').read_text())
 assert d['schema']==1 and d['result']=='pass' and d['database_schema']==18
 assert d['native_guest']==dict(system='NetBSD',release='11.0',architecture='aarch64',cores=2,ram_mib=1024)
 assert d['native_guest_uname'].startswith('NetBSD 11.0')
 assert d['host_rust_tests']==d['native_rust_tests']==369 and d['host_python_tests']==130
 assert all(d[k] for k in ['host_strict_workspace_clippy','host_maintained_rust_terminal_workflow','native_frozen_workspace_build','new_owned_native_project','native_custom_menu_reader_console_and_export_restore'])
 assert len(d['execution_sources'])==d['selected_native_source_matches'] and len(d['execution_sources'])==340
 assert len(d['harness_sources'])==14 and len(d['unchanged_dependency_access_credential_sources'])==13
 for group in ['execution_sources','harness_sources','unchanged_dependency_access_credential_sources']:
  for n,h in d[group].items():assert hashlib.sha256(source_bytes(n,h)).hexdigest()==h,n
 assert {'crates/faction-terminal/src/guide.rs','crates/faction-cli/src/guide.rs','crates/faction-service/src/guide_ui.rs','tools/smoke_board_guide.py','tools/smoke_shared_board_guide.py','crates/faction-client/src/passport.rs','docs/images/nightwire-menu.svg'}<=set(d['execution_sources'])
 m=passport_baseline_manifest();assert len(m['files'])==335 and len(m['records'])==1
 assert hashlib.sha256((ROOT/'dev-docs/evidence/passport-baseline-manifest.json').read_bytes()).hexdigest()==d['prior_passport_manifest_sha256']
 n='dev-docs/evidence/passport-netbsd/results.json';assert hashlib.sha256((ROOT/n).read_bytes()).hexdigest()==d['prior_passport_result_sha256']==m['records'][n]
 for k,count in [('host_guide',17),('native_guide',17),('shared_guide',10)]:
  v=d[k];assert v['result']=='pass' and v['count']==len(v['checks'])==len(set(v['checks']))==count and v['schema_version']==18
  assert all(v[n] for n in ['actual_registration_and_approval','current_menu_keys','current_enabled_topics','classic_plain_fallback','question_mark_refresh_preserved','read_only'])
  assert not v['capacity_claim'] and not v['hardening_claim']
 assert d['shared_guide']['actual_ssh_and_native'] and d['shared_guide']['normal_sessions']==8 and d['shared_guide']['admission_refill_ms']==5000
 assert {'actual_live_module_disable_removes_current_topic_without_stale_authority','actual_native_compact_help_is_plain_and_requires_no_new_remote_frame','actual_revocation_ends_a_held_guide_without_a_cached_approval_bypass'}<=set(d['shared_guide']['checks'])
 for name,count in [('host_who',21),('host_idle_notifications',20)]:
  v=d[name];assert v['result']=='pass' and v['count']==len(v['checks'])==count
 counts=dict(who=21,idle_notifications=20,standalone=12,social=8,editorial=10,magazine=7,requests=6,library=8,travel=6,connection=14,passport=16,gallery=8,discussion=8,catchup=8,polls=8,gatherings=8,pages=8,wall=7,full=7,writing=13,browser=10,operator=8,launcher=7,directory=11,nightwire=7,picker=8,vault=13)
 assert set(d['native_regressions'])==set(counts)|{'art'}
 for n,count in counts.items():
  v=d['native_regressions'][n];assert v['result']=='pass' and v['count']==len(v['checks'])==count,n
 assert d['native_regressions']['art']['profile_count']==18
 for k,count in [('host_binaries_sha256',5),('native_binaries_sha256',5),('installed_native_binaries_sha256',3)]:
  assert len(d[k])==count and all(len(h)==64 and set(h)<=set('0123456789abcdef') for h in d[k].values())
 assert all(d['native_binaries_sha256'][n]==h for n,h in d['installed_native_binaries_sha256'].items())
 assert d['budgets']==dict(normal_sessions=8,admission_refill_ms=5000,credential_permits=2,owner_helpers=3,shared_output_bytes=32768,frame_bytes=8192,drain_seconds=5,new_registry_packages=0,new_schema=0,new_server_helpers=0,guide_notes_bytes=1024,guide_notes_lines=16,guide_title_bytes=24,guide_min_columns=40,guide_min_rows=18,guide_max_columns=120,guide_max_rows=40)
 assert all(d['features'].values()) and all(d['teardown'].values()) and not any(d['claims'].values())
 print('Board-guide evidence passed: 369 Rust per OS, 17 local/worker controls per OS, 10 actual shared SSH/native controls; current themed reference and classic fallback, no capacity/hardening claim.')
if __name__=='__main__':check()
