#!/usr/bin/env python3
"""Verify immutable selected-record reply observations without starting a runtime."""
import hashlib
import json
from pathlib import Path
import re

ROOT=Path(__file__).resolve().parents[1]
RESULT_SHA256="dd21e50f2e3e6be743d610b5ab058fa404d22115e6dd40958686d72dbd6fb0c1"
PROFILE=dict(normal_sessions=8,launch_ms=5000,credential_permits=2,kernel_slots=2,owner_helpers=3,drain_seconds=5,parked_prompt_seconds=120,history_records=64,outbox_records=8,subject_bytes=128,body_bytes=1024)
CLAIMS={'automatic_send','automatic_read_ack','background_polling','credential_lease','server_plaintext_or_keys','new_crypto','schema_change','new_dependency','physical_hardware','capacity','production_deployment'}
REQUIRED={'host','native','tdd','cleanup','compatibility','associations'}
SCOPES={'local':'device-local-inbox-reply','main':'same-ssh-main-menu-device-local-reply'}
CONTROLS={'local': ['fresh_queue_keeps_or_replaces_subject_exact_canonical_reply_and_pending_once', 'source_read_history_cursor_and_unrelated_pending_preserved_after_reopen', 'encrypted_checkpoint_has_no_known_plaintext_or_sidecar', 'tracking_disabled_received_reply_preserves_unknown_read_state', 'existing_C_compose_still_queues_exact_fresh_intent', 'existing_Re_scalar_boundary_shortening_and_hostile_subject_rendering', 'cancel_and_nonexact_queue_confirmation_create_no_reply_or_acknowledgement', 'queued_queue_typeahead_discarded_before_fresh_confirmation', 'expired_revoked_history_only_full_history_and_full_outbox_refuse_without_mutation', 'offline_inbox_remains_readonly_and_has_no_reply_offer', 'CtrlC_CtrlD_at_commit_lock_and_leave_no_partial_reply'], 'main': ['main_menu_reply_cancel_preserves_selected_source_and_all_canonical_state', 'actual_home_outage_then_local_reply_queue_without_home_password_prompt', 'canonical_exact_new_reply_and_pending_once_original_read_cursor_unrelated_pending_preserved', 'fresh_hidden_unlock_lock_return_same_SSH_and_board_logout_work', 'generated_reply_private_markers_absent_corrected_GIO_and_board_files', 'mail_reply_never_acquires_FACTION1_file_protocol_authority']}

FILES_CONTROLS={'home_outage_history_pending_cursor_exact_same_ssh', 'explicit_encrypted_tracking_upgrade_and_skip_preserve_unread', 'wrong_unlock_and_canceled_unlock_lock_return_same_ssh', 'explicit_post_body_read_acknowledgement_persists_locally', 'same_ssh_approved_files_restricted_sftp_review_download', 'independent_registration_pending_and_explicit_approval', 'local_confirmation_cancel_and_dropped_typeahead_same_ssh', 'opt_in_generic_locked_stale_notices_only_at_files', 'private_markers_absent_remote_transcript_database_store_logs', 'private_view_ssh_disconnect_fails_closed_without_redial', 'actual_home_tcp_refusal_preserves_encrypted_state', 'normal_return_to_bbs_and_same_ssh_files_reentry', 'bounded_hostile_remote_output_quiescent_during_hidden_prompt', 'fresh_hidden_unlock_and_locked_return_same_ssh'}
LEGACY_CONTROLS={'current-default-native-v1_files_rpc_same_ssh_return', 'older-native-v1_files_rpc_same_ssh_return'}
INBOX_CONTROLS={'last_unread_ack_returns_empty_filter_and_repeated_ack_is_idempotent', 'fresh_ack_marks_exact_original_received_record_and_refreshes_filter_selection_durably', 'disabled_tracking_keeps_unknown_and_rejects_zero_unread_filter_without_upgrade', 'CtrlC_and_CtrlD_lock_from_inbox_and_restore_terminal_without_mutation', 'classic_and_explicit_unicode_subjects_are_inert_and_fit_40_columns', 'all_64_retained_records_are_eleven_bounded_pages_with_partial_final_page', 'expired_revoked_history_only_inbox_inspection_and_explicit_local_ack_stay_available', 'seven_record_status_order_arrows_JK_pages_selected_only_and_unread_filter_readonly', 'configured_online_I_uses_same_local_inbox_without_a_home_cycle', 'read_ack_write_failure_locks_poisoned_handle_and_preserves_canonical_checkpoint', 'explicit_offline_I_inbox_entry_and_back_preserve_checkpoint', 'empty_and_single_real_checkpoint_bounds_never_wrap_or_acknowledge', 'actual_pty_process_death_before_ack_preserves_unread_and_canonical_reopen', 'queued_open_read_typeahead_and_cancel_after_body_never_acknowledge'}

def raw(root,ref):
    path=Path(ref['path']);assert not path.is_absolute() and '..' not in path.parts
    assert re.fullmatch('[0-9a-f]{64}',ref['sha256'])
    target=root/path;assert not target.is_symlink() and target.resolve().is_relative_to(root.resolve())
    data=target.read_bytes();assert hashlib.sha256(data).hexdigest()==ref['sha256'],str(path)
    return data

def route(root,ref,kind,platform,mode,available):
    value=json.loads(raw(root,ref));assert value['result']=='pass'
    assert value['platform']==('Darwin' if platform=='host' else 'NetBSD')
    assert value['scope']==SCOPES[kind] and value['unicode'] is (mode=='unicode')
    assert value['count']==len(value['checks'])==len(CONTROLS[kind]) and set(value['checks'])==set(CONTROLS[kind])
    if kind=='local':
        assert value['private_fixture_removed'] is True and value['configured_home_endpoint_was_closed'] is True
        assert value['home_password_prompt_observed'] is False
    else:
        assert value['files_disabled'] is True and value['actual_home_transport_error']=='Connect'
        assert value['exact_canonical_reply_invariants'] is True and value['same_ssh_pid_until_normal_close'] is True
        assert value['automatic_redial_observed'] is False
        assert value['marker_count']>0 and value['private_matches']==value['file_protocol_matches']==0
        assert re.fullmatch('[0-9a-f]{32}',value['capture_id']) and value['source_sha256'] in available
        inputs=value['private_input_hashes']
        assert set(inputs)=={'trace_sha256','marker_set_sha256','file_scan_input_sha256'}
        assert all(re.fullmatch('[0-9a-f]{64}',v) for v in inputs.values())
        scan=value['privacy_scan'];assert scan['matches']==scan['symlinks']==0
        assert all(scan[k]>0 for k in ('board_files','store_files','log_files'))
        stats=value['board_io_reconstruction']
        assert all(stats[k]>0 for k in ('records','streams','decoded_bytes','read_bytes','write_bytes'))
        assert stats['decoded_bytes']==stats['read_bytes']+stats['write_bytes']
    return value

def test_counts(root,ref,passed,ignored):
    text=raw(root,ref).decode();counts=re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored',text)
    assert counts and all(int(f)==0 for p,f,i in counts)
    assert sum(int(p) for p,f,i in counts)==passed and sum(int(i) for p,f,i in counts)==ignored

def supplemental(root,values,native_files):
    host=values['host'];workspace=json.loads(raw(root,host['workspace']))
    assert workspace['result']=='pass' and workspace['source_stable'] is True
    assert [row['command'][1] for row in workspace['checks']]==['fmt','clippy','test'] and all(row['returncode']==0 for row in workspace['checks'])
    test_counts(root,host['workspace_test_log'],473,0)
    client=json.loads(raw(root,host['client_summary']))['final2']
    for mode,count in [('network',139),('native',90)]:
        assert client[mode]['total_passed']==count and client[mode]['total_ignored']==4
        test_counts(root,host['client_test_logs'][mode],count,4)
    assert client['sources'] and all(native_files[name]==sha for name,sha in client['sources'].items())
    test_counts(root,values['native']['test_log'],272,4)
    compat=values['compatibility'];assert set(compat['routes'])=={'host','netbsd'}
    for platform,refs in compat['routes'].items():
        assert set(refs)=={'files','legacy'}
        for key,expected in [('files',FILES_CONTROLS),('legacy',LEGACY_CONTROLS)]:
            data=json.loads(raw(root,refs[key]));assert data['result']=='pass'
            assert data['count']==len(data['checks'])==len(expected) and set(data['checks'])==expected
            assert data['physical_hardware'] is False and data['capacity_claim'] is False
            if key=='files':
                assert data['scope']=='same-ssh-parked-files-local-mail-restricted-sftp'
                assert data['platform']==('Darwin' if platform=='host' else 'NetBSD') and data['unicode'] is False
                assert data['approved_member'] is True and data['restricted_sftp'] is True and data['same_ssh_pid_until_explicit_disconnect'] is True
                assert data['privacy_scan']['matches']==data['privacy_scan']['symlinks']==0
            else:assert data['scope']=='actual-old-and-default-native-v1-files-rpc' and data['private_mail_not_opened'] is True
    inbox=json.loads(raw(root,compat['inbox']));assert inbox['result']=='pass' and inbox['count']==len(inbox['checks'])==14 and set(inbox['checks'])==INBOX_CONTROLS
    assert inbox['private_fixture_removed'] is True and inbox['provider_contact'] is False and inbox['read_receipt_sent'] is False

def validate(root,record):
    assert record['schema']==1 and record['issue']==233 and record['result']=='pass'
    assert record['profile']==PROFILE and set(record['claims'])==CLAIMS and all(v is False for v in record['claims'].values())
    assert record['sources'] and record['prior_records'] and set(record['records'])>=REQUIRED
    for ref in record['sources']+record['prior_records']:raw(root,ref)
    available={ref['sha256'] for ref in record['sources']}
    assert set(record['routes'])=={'local','main'}
    for kind,routes in record['routes'].items():
        assert set(routes)=={p+'_'+m for p in ('host','netbsd') for m in ('classic','unicode')}
        for platform in ('host','netbsd'):
            for mode in ('classic','unicode'):route(root,routes[platform+'_'+mode],kind,platform,mode,available)
    values={key:json.loads(raw(root,ref)) for key,ref in record['records'].items()}
    assert all(v['result']=='pass' for v in values.values())
    associations=values['associations']['routes'];assert set(associations)==set(record['routes']['main'])
    capture_ids=set()
    for key,ref in record['routes']['main'].items():
        data=json.loads(raw(root,ref));association=associations[key]
        assert data['capture_id'] not in capture_ids;capture_ids.add(data['capture_id'])
        assert association['capture_id']==data['capture_id'] and association['source_sha256']==data['source_sha256']
        assert association['result_sha256']==ref['sha256'] and association['private_input_hashes']==data['private_input_hashes']
        assert association['generated_before_capture'] is True and association['input_trace_unchanged'] is True
    final=[]
    for stage in record['source_stages']:
        files=json.loads(raw(root,stage['manifest']));assert files and all(sha in available for sha in files.values())
        if stage['label']=='native-final':final.append(files)
    assert len(final)==1 and values['native']['files']==final[0] and values['native']['all_source_match'] is True
    assert values['native']['source_count']==len(final[0]) and values['native']['source_count']>0
    supplemental(root,values,final[0])
    assert values['native']['compile_memory_mib']==2048 and 1000*1024**2<values['native']['runtime_physmem']<1024**3
    assert all(values['cleanup'][k] is True for k in ('owned_removed','saved_stopped','saved_hashes_unchanged','forwards_closed'))
    assert values['tdd']['captured_before_edit'] is True and values['tdd']['red_exit']!=0 and values['tdd']['green_exit']==0
    return values

def check():
    data=(ROOT/'dev-docs/evidence/inbox-reply/results.json').read_bytes()
    assert RESULT_SHA256 and hashlib.sha256(data).hexdigest()==RESULT_SHA256,'immutable reply evidence changed'
    validate(ROOT,json.loads(data))
    print('Reply evidence passed: source-bound host/NetBSD local and same-SSH classic/Unicode, canonical queue/read preservation and private I/O exclusion; no deployment claim.')

if __name__=='__main__':check()
