#!/usr/bin/env python3
"""Validate immutable #230 main-menu mail observations; starts no runtime."""
import hashlib
import json
from pathlib import Path
import re
ROOT=Path(__file__).resolve().parents[1]
RESULT_SHA256=None
PROFILE=dict(normal_sessions=8,launch_ms=5000,credential_permits=2,kernel_slots=2,owner_helpers=3,drain_seconds=5,mail_nonces=64,mail_payload_bytes=80)
CLAIMS={'background_polling','credential_lease','shared_mail_metadata','new_cryptography','physical_hardware','capacity','production_deployment'}
REQUIRED={'native-source-match','host-tests','native-tests','protocol','compatibility','legacy','runtime','cleanup','tdd'}
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

def route(data,unicode):
 assert data['result']=='pass' and data['unicode'] is unicode
 assert data['count']==len(data['checks'])==len(CONTROLS) and set(data['checks'])==CONTROLS
 for name in ('files_disabled','same_client_pid','same_ssh_pid_until_explicit_disconnect','checkpoint_history_pending_cursor_exact','actual_board_io_private_scan','no_file_protocol_in_board_io','terminal_restored','no_automatic_redial'):assert data[name] is True,name
 for name in ('transfer_authority_required','capacity_claim','physical_hardware'):assert data[name] is False,name
 assert type(data['private_transcript_intervals']) is int and data['private_transcript_intervals']>=4
 scan=data['privacy_scan'];assert scan['matches']==scan['symlinks']==0
 for name in ('board_files','store_files','log_files'):assert type(scan[name]) is int and scan[name]>0
 hostile=data['hostile_output'];assert hostile['written']==hostile['maximum']==16384
 assert hostile['writer_finished'] is True and hostile['owned_tty_verified'] is True

def validate(root,record):
 assert record['schema']==1 and record['issue']==230 and record['result']=='pass'
 assert record['profile']==PROFILE and set(record['claims'])==CLAIMS and all(v is False for v in record['claims'].values())
 assert record['sources'] and record['prior_records'] and REQUIRED<=record['records'].keys()
 for source in record['sources']+record['prior_records']:raw(root,source)
 assert set(record['routes'])=={'host','netbsd'}
 for platform in ('host','netbsd'):
  assert set(record['routes'][platform])=={'classic','unicode'}
  for name,unicode in [('classic',False),('unicode',True)]:route(json.loads(raw(root,record['routes'][platform][name])),unicode)
 records={name:json.loads(raw(root,ref)) for name,ref in record['records'].items()}
 assert all(value['result']=='pass' for value in records.values())
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
 assert records['tdd']['red_exit']!=0 and records['tdd']['green_exit']==0 and records['tdd']['captured_before_edit'] is True
 return records

def check():
 data=(ROOT/'dev-docs/evidence/main-menu-mail/results.json').read_bytes()
 assert RESULT_SHA256 and hashlib.sha256(data).hexdigest()==RESULT_SHA256,'immutable main-menu mail result changed'
 validate(ROOT,json.loads(data))
 print('Main-menu mail evidence passed: source-bound host/NetBSD classic/Unicode, Files separation, private board-I/O exclusion and cleanup; no production claim.')
if __name__=='__main__':check()
