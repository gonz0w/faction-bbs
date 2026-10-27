#!/usr/bin/env python3
"""Verify source-bound #216/#217 software observations; never starts services."""
import hashlib,json,re
from pathlib import PurePosixPath
from evidence_sources import ROOT,source_bytes,picker_mail_unread_baseline_manifest

RECORD_SHA256 = '219206a2d4643a3e3167d2984dfc6e571bf94ff271385baf15297a8b571c079e'
UNREAD = {
 'old_schema_count_and_history_inspection_do_not_enable_or_acknowledge',
 'declined_local_read_upgrade_preserves_exact_old_checkpoint',
 'confirmed_encrypted_upgrade_adds_local_counts_without_sidecar_or_protocol_changes',
 'count_only_unlock_and_lock_preserve_unread_and_canonical_bytes',
 'received_body_output_and_empty_enter_do_not_implicitly_mark_read',
 'cancel_after_received_body_before_ack_keeps_message_unread',
 'queued_record_navigation_input_cannot_acknowledge_after_body',
 'explicit_post_body_read_persists_only_local_received_acknowledgement',
 'reopen_and_repeated_ack_preserve_read_state_without_another_write',
 'classic_and_explicit_unicode_history_share_exact_local_read_semantics',
 'expired_revoked_and_history_only_retained_mail_remains_locally_acknowledgeable',
 'read_write_failure_poison_locks_then_canonical_reopen_keeps_pending_and_unread',
 'terminal_session_death_before_explicit_ack_never_marks_read',
}
VISIT = {
 'actual_home_tcp_refusal_preserves_encrypted_state',
 'bounded_hostile_remote_output_quiescent_during_hidden_prompt',
 'explicit_encrypted_tracking_upgrade_and_skip_preserve_unread',
 'explicit_post_body_read_acknowledgement_persists_locally',
 'fresh_hidden_unlock_and_locked_return_same_ssh',
 'home_outage_history_pending_cursor_exact_same_ssh',
 'independent_registration_pending_and_explicit_approval',
 'local_confirmation_cancel_and_dropped_typeahead_same_ssh',
 'normal_return_to_bbs_and_same_ssh_files_reentry',
 'opt_in_generic_locked_stale_notices_only_at_files',
 'private_markers_absent_remote_transcript_database_store_logs',
 'private_view_ssh_disconnect_fails_closed_without_redial',
 'same_ssh_approved_files_restricted_sftp_review_download',
 'wrong_unlock_and_canceled_unlock_lock_return_same_ssh',
}

TEARDOWN = {'owned_vm_removed','owned_private_inputs_removed','ports_closed','saved_playground_stopped','saved_playground_unchanged'}
NO_CLAIMS = {'background_polling','arbitrary_session_overlay','live_private_chat','roaming','attachments','security_audit','physical_hardware','capacity','production_deployment','indefinite_pause'}
PRIOR_SHA256 = '3f0d85a11e16038da86560293f26a52b5dbb0b6188255fffc38e539c9adf6456'

def load_record(raw):
    assert hashlib.sha256(raw).hexdigest()==RECORD_SHA256,'saved unread/visit record changed or not pinned'
    return json.loads(raw)

def named(row,names):
    assert row['result']=='pass'
    assert type(row['count']) is int and row['count']==len(names)
    assert len(row['checks'])==len(set(row['checks']))==len(names) and set(row['checks'])==names

def verify(name,digest,resolve):
    path=PurePosixPath(name)
    assert name and not path.is_absolute() and '..' not in path.parts and str(path)==name and '\\' not in name
    assert re.fullmatch('[a-f0-9]{64}',digest)
    assert hashlib.sha256(resolve(name,digest)).hexdigest()==digest,name

def validate_visit(v,unicode):
    assert v['unicode'] is unicode
    named(v,VISIT)
    for key in ['same_client_pid','same_ssh_pid_until_explicit_disconnect','pending_ids_unchanged','archive_token_unchanged','history_unchanged','local_read_upgrade_and_acknowledgement','home_outage_canonical_unchanged','nonempty_cursor','approved_member','restricted_sftp','remote_disconnect_failed_closed','no_automatic_redial','terminal_restored']:assert v[key] is True,key
    assert v['unread_before_ack']==1 and v['unread_after_ack']==0
    assert v['payload_bytes']==131328 and re.fullmatch('[a-f0-9]{64}',v['payload_sha256'])
    assert v['home_transport_error']=='Connect'
    scan=v['privacy_scan'];assert set(scan)=={'board_files','store_files','log_files','matches','symlinks'}
    assert all(type(n) is int for n in scan.values())
    assert all(scan[k]>0 for k in ['board_files','store_files','log_files'])
    assert scan['matches']==scan['symlinks']==0
    hostile=v['hostile_output']
    assert hostile['maximum']==1048576 and hostile['chunk']==32768
    assert type(hostile['written']) is int and 0<hostile['written']<=hostile['maximum']
    assert hostile['writer_finished'] is True and hostile['owned_tty_verified'] is True

def validate(d,resolve):
    assert d['schema']==1 and d['issues']==[216,217]
    assert set(d['routes'])=={'host','netbsd'}
    for route in d['routes'].values():
        named(route['unread'],UNREAD)
        assert {'unread','visit','visit_unicode','unconfigured'}.issubset(route)
        validate_visit(route['visit'],False)
        validate_visit(route['visit_unicode'],True)
        off=route['unconfigured']
        assert off['result']=='pass' and off['count']==3 and off['graphs']==['default','native_only','network']
    assert d['runtime']['netbsd_mib']==1024 and d['runtime']['ncpu']==2
    assert d['build']['compiler_mib']==2048 and d['build']['fresh_1g_build'] is False
    assert set(d['teardown'])==TEARDOWN and all(x is True for x in d['teardown'].values())
    assert set(d['claims'])==NO_CLAIMS and all(x is False for x in d['claims'].values())
    assert d['prior_picker_sha256']==PRIOR_SHA256
    assert d['execution_sources'] and d['source_stages']
    for name,digest in d['execution_sources'].items():verify(name,digest,resolve)
    labels=set()
    for stage in d['source_stages']:
        assert stage['label'] and stage['label'] not in labels and stage['files']
        labels.add(stage['label'])
        for row in stage['files']:verify(row['path'],row['sha256'],resolve)

def publication_resolver(mapping,resolve):
    """Validate newline-only views and resolve every original digest unchanged."""
    assert mapping['schema']==1 and mapping['files']
    prior=mapping['prior_record']
    verify(prior['path'],prior['sha256'],resolve)
    previous=resolve(prior['path'],prior['sha256'])
    for name,row in mapping['files'].items():
        verify(name,row['view_sha256'],resolve)
        verify(row['raw_path'],row['raw_sha256'],resolve)
        raw=resolve(row['raw_path'],row['raw_sha256'])
        view=resolve(name,row['view_sha256'])
        assert raw and not raw.endswith(b'\n')
        assert view==raw+b'\n','publication view changes more than a final newline'
        assert json.loads(view)==json.loads(raw)
    def historical(name,expected):
        row=mapping['files'].get(name)
        if row is not None and row['raw_sha256']==expected:
            return resolve(row['raw_path'],expected)
        return resolve(name,expected)
    return previous,historical

def check():
    d=load_record((ROOT/'dev-docs/evidence/local-unread-mail/results.json').read_bytes())
    validate(d,source_bytes)
    layout=d['publication_layout']
    assert layout['kind']=='newline-only-json-mirrors' and layout['no_measurements_refreshed'] is True
    verify(layout['byte_map_path'],layout['byte_map_sha256'],source_bytes)
    mapping=json.loads(source_bytes(layout['byte_map_path'],layout['byte_map_sha256']))
    previous,historical=publication_resolver(mapping,source_bytes)
    assert hashlib.sha256(previous).hexdigest()==layout['previous_record_sha256']=='332bf000f982b81a4fc4b4add952fe0b95d085a4e03caeec65d06d15b8383086'
    original=json.loads(previous);validate(original,historical)
    # Original manifest fingerprints refer to archived exact bytes, not views.
    prefix='dev-docs/evidence/local-unread-mail/qualification/'
    for manifest in ['manifest.json','manifest-source-extended.json','manifest-final.json']:
        path=prefix+manifest;digest=original['execution_sources'][path]
        for name,expected in json.loads(historical(path,digest)).items():
            verify(prefix+name,expected,historical)
    baseline=picker_mail_unread_baseline_manifest()
    for name,digest in baseline['files'].items():verify(name,digest,source_bytes)
    for name,digest in baseline['records'].items():verify(name,digest,lambda n,h:(ROOT/n).read_bytes())
    print('Saved unread/visit software evidence verified: encrypted local decisions, same-SSH parked Files/mail, bounded remote handling, source identity and owned teardown; no background/roaming/capacity claim.')
if __name__=='__main__':check()
