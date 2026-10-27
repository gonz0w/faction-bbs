#!/usr/bin/env python3
"""Validate immutable #214 software observations; never starts a client or VM."""
import hashlib,json,re
from pathlib import PurePosixPath
from evidence_sources import ROOT,source_bytes,archive_mail_picker_baseline_manifest

RECORD_SHA256 = '3f0d85a11e16038da86560293f26a52b5dbb0b6188255fffc38e539c9adf6456'
UI = {'ambiguous_local_write_poison_drops_handle_then_canonical_reopen_succeeds', 'hidden_local_prompt_cancel_locks_and_restores_picker', 'explicit_unicode_override_reaches_existing_authenticated_history', 'archive_catchup_and_declined_upgrade_are_explicit_and_nonmutating', 'busy_checkpoint_is_not_unlocked_or_replaced', 'mail_return_preserves_exact_board_selection_and_canonical_checkpoint', 'nonempty_archive_cursor_epoch_history_and_pending_survive_outage_and_wrong_unlock', 'missing_loose_or_symlink_local_references_refuse_without_creation_or_state_change', 'compact_40x18_local_entry_history_lock_reopen_and_navigation_without_children', 'unavailable_home_keeps_history_pending_and_manual_locked_return', 'queued_picker_or_menu_input_cannot_supply_hidden_home_credentials', 'unconfigured_E_has_no_unlock_or_connection_authority', 'explicit_compose_reuses_durable_queue_and_preserves_existing_pending_ids', 'oversized_hidden_home_input_locks_flushes_and_returns', 'wrong_local_secret_locks_and_returns_without_private_history', 'hidden_home_prompt_cancel_locks_and_restores_picker', 'explicit_G_then_A_reuses_existing_archive_path_and_outage_preserves_cursor_and_pending'}
COMBINED = {'same_picker_send_and_archive_outage_preserve_exact_state', 'same_picker_approved_board_restricted_sftp_upload_review_download', 'private_mail_absent_from_visited_board_transcript_database_store_and_logs', 'locked_picker_releases_store_and_requires_hidden_unlock', 'real_encrypted_nonempty_cursor_history_and_pending_seed', 'independent_board_registration_pending_and_explicit_approval', 'actual_closed_home_tcp_connection_failure', 'same_picker_board_return_and_terminal_restoration'}
PROFILE=dict(catalog_version=1,config_key='home_mail',config_fields=['store','config'],menu_key='E',v1_key='M',local_catalog_only=True,implicit_upgrade=False,background=False,remote_unlock=False,new_network_format=False,classic_fallback=True,visited_board_receives_private_mail=False)
TEARDOWN={'owned_vm_removed','owned_private_inputs_removed','ports_closed','saved_playground_stopped','saved_playground_unchanged'}
NO_CLAIMS={'background_notifications','in_session_overlay','roaming','attachments','full_conformance','security_audit','physical_hardware','capacity','release','production_deployment'}

def load_record(raw):
    assert hashlib.sha256(raw).hexdigest()==RECORD_SHA256,'saved picker-mail record changed or not yet pinned'
    return json.loads(raw)

def named(row,expected):
    assert row['result']=='pass' and type(row['count']) is int and row['count']==len(expected)
    assert len(row['checks'])==len(set(row['checks']))==len(expected) and set(row['checks'])==expected

def verify(name,digest,resolve):
    path=PurePosixPath(name)
    assert isinstance(name,str) and name and not path.is_absolute() and '..' not in path.parts and str(path)==name and '\\' not in name
    assert re.fullmatch('[a-f0-9]{64}',digest)
    assert hashlib.sha256(resolve(name,digest)).hexdigest()==digest,name

def validate(d,resolve):
    assert d['schema']==1 and d['issue']==214 and d['result']=='pass-native-picker-home-mail'
    assert d['profile']==PROFILE
    assert set(d['routes'])=={'host','netbsd'}
    for route,platform in [('host','Darwin'),('netbsd','NetBSD')]:
        row=d['routes'][route];ui=row['ui'];named(ui,UI)
        assert ui['platform']==platform and ui['private_fixture_removed'] is True and ui['provider_contact'] is False and ui['listener_opened'] is False
        actual=row['combined'];named(actual,COMBINED)
        assert actual['platform']==platform and actual['scope']=='same-picker-unavailable-home-approved-board-restricted-sftp'
        for key in ['same_picker_pid','canonical_unchanged','history_unchanged','pending_ids_unchanged','archive_token_unchanged','nonempty_cursor','approved_member','restricted_sftp','private_fixture_retained_for_lead']:assert actual[key] is True,key
        assert actual['home_transport_error']=='Connect'
        assert actual['payload_bytes']==131328 and re.fullmatch('[a-f0-9]{64}',actual['payload_sha256'])
        scan=actual['privacy_scan'];assert set(scan)=={'board_files','store_files','log_files','matches','symlinks'}
        assert all(type(v) is int for v in scan.values())
        assert all(scan[key]>0 for key in ['board_files','store_files','log_files']) and scan['matches']==scan['symlinks']==0
        assert actual['physical_hardware'] is False and actual['capacity_claim'] is False
        assert set(row['unsupported'])=={'default','native_only'}
        for off in row['unsupported'].values():
            named(off,{'unsupported_home_mail_catalog_rejects_before_terminal_store_or_network'})
            assert off['private_fixture_removed'] is True
        tests=d['rust'][route]
        assert type(tests['network_passed']) is int and tests['network_passed']>=139
        assert tests['failed']==0 and tests['ignored']==2 and tests['helpers_invoked'] is True
    assert d['rust']['host']['network_passed']==d['rust']['netbsd']['network_passed']
    assert d['build']['configured_mib']==2048 and d['build']['compiler_runtime_distinct'] is True and d['build']['fresh_1g_build_claim'] is False
    assert d['runtime']['configured_mib']==1024 and d['runtime']['physmem_bytes']==1054990336 and d['runtime']['ncpu']==2
    assert set(d['teardown'])==TEARDOWN and all(v is True for v in d['teardown'].values())
    assert set(d['claims'])==NO_CLAIMS and all(v is False for v in d['claims'].values())
    for key in ['execution_sources','frozen_sources']:
        assert isinstance(d[key],dict) and d[key]
        for name,digest in d[key].items():verify(name,digest,resolve)
    stages=d['source_stages'];assert stages and len({s['label'] for s in stages})==len(stages)
    for stage in stages:
        assert stage['label'] and stage['files']
        for row in stage['files']:verify(row['resolved_path'],row['sha256'],resolve)

def check():
    d=load_record((ROOT/'dev-docs/evidence/picker-mail/results.json').read_bytes());validate(d,source_bytes)
    baseline=archive_mail_picker_baseline_manifest()
    for name,digest in baseline['files'].items():verify(name,digest,source_bytes)
    for name,digest in baseline['records'].items():assert hashlib.sha256((ROOT/name).read_bytes()).hexdigest()==digest,'prior archive evidence changed'
    print('Saved picker-mail evidence verified: exact host/NetBSD outage-to-board/files journey, locked private state, default/classic compatibility and owned teardown; no background/roaming/capacity claim.')

if __name__=='__main__':check()
