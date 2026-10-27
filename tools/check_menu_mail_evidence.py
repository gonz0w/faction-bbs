#!/usr/bin/env python3
"""Validate immutable #230 main-menu mail observations; starts no runtime."""
import hashlib
import json
from pathlib import Path
import re
ROOT=Path(__file__).resolve().parents[1]
RESULT_SHA256='a6de523049a995792a234ec142a4f9a9af0349b3a7a1fca4423d324dbc8f8536'
PROFILE=dict(normal_sessions=8,launch_ms=5000,credential_permits=2,kernel_slots=2,owner_helpers=3,drain_seconds=5,mail_nonces=64,mail_payload_bytes=80)
CLAIMS={'background_polling','credential_lease','shared_mail_metadata','new_cryptography','physical_hardware','capacity','production_deployment'}
REQUIRED={'native-source-match','host-tests','native-tests','protocol','compatibility','legacy','runtime','cleanup','tdd','private-io-rescan'}
CONTROLS={
 'independent_approved_member_files_disabled_main_menu_entry',
 'ordinary_ssh_explanation_and_same_menu_return',
 'fixed_local_open_back_choice_without_unlock_or_home_contact',
 'queued_confirmation_discarded_and_fresh_local_confirmation_required',
 'wrong_unlock_and_canceled_unlock_lock_return_same_ssh',
 'hidden_prompt_and_private_console_do_not_render_hostile_board_output',
 'real_home_tcp_refusal_preserves_exact_checkpoint_history_pending_cursor',
 'existing_local_history_and_outbox_remain_device_private',
 'fresh_hidden_unlock_and_lock_return_to_same_ssh_main_menu',
 'private_view_disconnect_fails_closed_without_redial',
 'private_markers_absent_board_transcript_database_store_and_logs',
}
def raw(root,ref):
 path=Path(ref['path']);assert not path.is_absolute() and '..' not in path.parts
 assert re.fullmatch('[0-9a-f]{64}',ref['sha256'])
 target=root/path;assert not target.is_symlink() and target.resolve().is_relative_to(root.resolve())
 data=target.read_bytes();assert hashlib.sha256(data).hexdigest()==ref['sha256'],str(path)
 return data

def route(data,unicode,platform):
 assert data['platform']==platform and data['scope']=='main-menu-local-mail-files-disabled-same-ssh'
 assert data['result']=='pass' and data['unicode'] is unicode
 assert data['count']==len(data['checks'])==len(CONTROLS) and set(data['checks'])==CONTROLS
 for name in ('files_disabled','same_client_pid','same_ssh_pid_until_explicit_disconnect','checkpoint_history_pending_cursor_exact','actual_board_io_private_scan','no_file_protocol_in_board_io','terminal_restored','no_automatic_redial'):assert data[name] is True,name
 for name in ('transfer_authority_required','capacity_claim','physical_hardware'):assert data[name] is False,name
 assert type(data['private_transcript_intervals']) is int and data['private_transcript_intervals']>=4
 scan=data['privacy_scan'];assert scan['matches']==scan['symlinks']==0
 for name in ('board_files','store_files','log_files'):assert type(scan[name]) is int and scan[name]>0
 hostile=data['hostile_output'];assert hostile['written']==hostile['maximum']==16384
 assert hostile['writer_finished'] is True and hostile['owned_tty_verified'] is True

EXPECTED_CONTROLS={'protocol': {'actual_versioned_park_and_offer_cancel_same_connection_without_files', 'stale_nonce_reuse_after_completed_return_closes_connection', 'current_approval_revocation_before_park_denied', 'duplicate_park_closes_connection', 'wrong_parked_return_closes_connection', 'trailing_request_field_closes_connection', 'malformed_uppercase_nonce_closes_connection', 'out_of_state_mail_control_closes_connection', 'current_approval_revocation_after_park_denied', 'old_native_explanation_same_ssh_return', 'default_build_explanation_same_ssh_return', 'network_unconfigured_explanation_same_ssh_return'}, 'hostile': {'approval_revoked_during_private_view_locked_closed_without_remote_render_or_redial', 'bounded_output_overflow_locked_closed_without_remote_render_or_redial', 'forged_frame_locked_closed_without_remote_render_or_redial'}, 'compact': {'faction-compact_mail_action_explanation_and_same_connection_return', 'dumb_mail_action_explanation_and_same_connection_return', 'actual_compact_native_client_routes_offer_and_returns_same_ssh'}, 'fallback': {'inherited_ansi_explanation_same_menu_no_handoff', 'standalone_plain_explanation_same_menu_no_handoff', 'standalone_ansi_explanation_same_menu_no_handoff', 'inherited_plain_explanation_same_menu_no_handoff'}, 'classic': {'private_view_ssh_disconnect_fails_closed_without_redial', 'independent_registration_pending_and_explicit_approval', 'normal_return_to_bbs_and_same_ssh_files_reentry', 'fresh_hidden_unlock_and_locked_return_same_ssh', 'same_ssh_approved_files_restricted_sftp_review_download', 'explicit_encrypted_tracking_upgrade_and_skip_preserve_unread', 'actual_home_tcp_refusal_preserves_encrypted_state', 'private_markers_absent_remote_transcript_database_store_logs', 'opt_in_generic_locked_stale_notices_only_at_files', 'home_outage_history_pending_cursor_exact_same_ssh', 'explicit_post_body_read_acknowledgement_persists_locally', 'bounded_hostile_remote_output_quiescent_during_hidden_prompt', 'wrong_unlock_and_canceled_unlock_lock_return_same_ssh', 'local_confirmation_cancel_and_dropped_typeahead_same_ssh'}, 'unicode': {'private_view_ssh_disconnect_fails_closed_without_redial', 'independent_registration_pending_and_explicit_approval', 'normal_return_to_bbs_and_same_ssh_files_reentry', 'fresh_hidden_unlock_and_locked_return_same_ssh', 'same_ssh_approved_files_restricted_sftp_review_download', 'explicit_encrypted_tracking_upgrade_and_skip_preserve_unread', 'actual_home_tcp_refusal_preserves_encrypted_state', 'private_markers_absent_remote_transcript_database_store_logs', 'opt_in_generic_locked_stale_notices_only_at_files', 'home_outage_history_pending_cursor_exact_same_ssh', 'explicit_post_body_read_acknowledgement_persists_locally', 'bounded_hostile_remote_output_quiescent_during_hidden_prompt', 'wrong_unlock_and_canceled_unlock_lock_return_same_ssh', 'local_confirmation_cancel_and_dropped_typeahead_same_ssh'}, 'legacy': {'older-native-v1_files_rpc_same_ssh_return', 'current-default-native-v1_files_rpc_same_ssh_return'}}
EXPECTED_SCOPES={'protocol': 'actual-ssh-mail-protocol-and-compatibility-files-disabled', 'hostile': 'actual-ssh-private-transition-hostile-controls', 'compact': 'actual-classic-and-compact-ssh-mail', 'fallback': 'local-and-inherited-real-pty-main-menu-mail-explanation', 'classic': 'same-ssh-parked-files-local-mail-restricted-sftp', 'unicode': 'same-ssh-parked-files-local-mail-restricted-sftp', 'legacy': 'actual-old-and-default-native-v1-files-rpc'}
def supplemental(root,records):
 specs={
  'protocol':{'protocol':(12,'actual-ssh-mail-protocol-and-compatibility-files-disabled'),'hostile':(3,None)},
  'compatibility':{'compact':(3,None),'fallback':(4,None)},
  'legacy':{'classic':(14,'same-ssh-parked-files-local-mail-restricted-sftp'),'unicode':(14,'same-ssh-parked-files-local-mail-restricted-sftp'),'legacy':(2,'actual-old-and-default-native-v1-files-rpc')},
 }
 for category,kinds in specs.items():
  routes=records[category]['routes'];assert set(routes)=={'host','netbsd'}
  for platform,entries in routes.items():
   assert set(entries)==set(kinds)
   for name,(count,scope) in kinds.items():
    data=json.loads(raw(root,entries[name]));assert data['result']=='pass'
    assert data['count']==len(data['checks'])==len(set(data['checks']))==count
    assert data['capacity_claim'] is False and data['physical_hardware'] is False
    assert set(data['checks'])==EXPECTED_CONTROLS[name] and data['scope']==EXPECTED_SCOPES[name]
    if 'platform' in data:assert data['platform']==('Darwin' if platform=='host' else 'NetBSD')
    if name in ('classic','unicode'):assert data['unicode'] is (name=='unicode') and data['restricted_sftp'] is True
    if name=='protocol':assert data['files_disabled'] is True and data['no_private_unlock'] is True and data['normal_limits_unchanged'] is True
    if name=='hostile':assert data['private_store_unchanged'] is True and data['normal_output_cap']==32768
    if name=='compact':assert data['dumb_output_no_escape'] is True and data['compact_offer_preserved'] is True
    if name=='fallback':assert data['no_shared_handoff_claim'] is True
    if name=='legacy':assert data['private_mail_not_opened'] is True

def publication(root,record):
 if 'publication_layout' not in record:return
 layout=record['publication_layout'];assert layout['kind']=='frozen-markdown-path-relocation' and layout['no_measurements_refreshed'] is True
 previous=json.loads(raw(root,layout['original_record']));assert 'publication_layout' not in previous
 assert layout['relocations'] and len({r['original_path'] for r in layout['relocations']})==len(layout['relocations'])
 for change in layout['relocations']:
  assert change['original_path'].endswith('.md') and change['new_path']!=change['original_path']
  matches=[s for s in previous['sources'] if s['path']==change['original_path']];assert len(matches)==1
  assert matches[0]['sha256']==change['sha256'];matches[0]['path']=change['new_path'];raw(root,matches[0])
 for source in layout['sources']:raw(root,source)
 current=dict(record);current.pop('publication_layout');assert current==previous,'publication changed a measurement or source identity'

def validate(root,record):
 publication(root,record)
 assert record['schema']==1 and record['issue']==230 and record['result']=='pass'
 assert record['profile']==PROFILE and set(record['claims'])==CLAIMS and all(v is False for v in record['claims'].values())
 assert record['sources'] and record['prior_records'] and REQUIRED<=record['records'].keys()
 for source in record['sources']+record['prior_records']:raw(root,source)
 assert set(record['routes'])=={'host','netbsd'}
 for platform in ('host','netbsd'):
  assert set(record['routes'][platform])=={'classic','unicode'}
  for name,unicode in [('classic',False),('unicode',True)]:route(json.loads(raw(root,record['routes'][platform][name])),unicode,'Darwin' if platform=='host' else 'NetBSD')
 records={name:json.loads(raw(root,ref)) for name,ref in record['records'].items()}
 assert all(value['result']=='pass' for value in records.values())
 supplemental(root,records)
 final=[s for s in record['source_stages'] if s['label']=='native-final'];assert len(final)==1 and final[0]['files']
 expected={f['name']:f['sha256'] for f in final[0]['files']};assert len(expected)==len(final[0]['files'])
 assert records['native-source-match']['all_match'] is True and records['native-source-match']['files']==expected
 available={s['sha256'] for s in record['sources']}
 for stage in record['source_stages']:
  assert stage['files']
  for source in stage['files']:assert source['sha256'] in available
 for name,value in PROFILE.items():assert records['runtime'][name]==value
 assert 1000*1024**2<records['runtime']['physmem']<1024**3
 assert all(records['cleanup'][name] is True for name in ('owned_removed','saved_stopped','saved_hashes_unchanged','forwards_closed'))
 scan=records['private-io-rescan'];assert scan['scope']=='postrun-retained-ktrace-hex-stream-reconstruction'
 assert scan['fresh_ui_execution'] is False and scan['stream_key']==['pid','fd','direction']
 assert scan['checker_sha256'] in available and set(scan['routes'])=={p+'_'+m for p in ('host','netbsd') for m in ('classic','unicode')}
 for platform in ('host','netbsd'):
  for mode in ('classic','unicode'):
   item=scan['routes'][platform+'_'+mode]
   assert item['original_route_result_sha256']==record['routes'][platform][mode]['sha256']
   assert item['generated_marker_count']>0 and item['private_marker_matches']==item['file_protocol_prefix_matches']==0
   assert item['input_trace_unchanged'] is True
 assert records['tdd']['red_exit']!=0 and records['tdd']['green_exit']==0 and records['tdd']['captured_before_edit'] is True
 return records

def check():
 data=(ROOT/'dev-docs/evidence/main-menu-mail/results.json').read_bytes()
 assert RESULT_SHA256 and hashlib.sha256(data).hexdigest()==RESULT_SHA256,'immutable main-menu mail result changed'
 validate(ROOT,json.loads(data))
 print('Main-menu mail evidence passed: source-bound host/NetBSD classic/Unicode, Files separation, private board-I/O exclusion and cleanup; no production claim.')
if __name__=='__main__':check()
