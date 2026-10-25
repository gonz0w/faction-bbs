#!/usr/bin/env python3
"""Verify saved #205 member-local console observations; never start a VM."""
import hashlib,json,re
from evidence_sources import ROOT,source_bytes,checkpoint_ui_baseline_manifest
RECORD_SHA256='71c7bd9a553ee8c69f60b4aca4e55fa3ec4ec3a6aef5c23a84671042a95565ec'
REQUIRED={
 'explicit_local_create_hidden_secret_and_new_recovery',
 'public_device_inspection_and_exclusive_lock',
 'typed_ed25519_peer_enrollment_survives_reopen',
 'changed_or_server_added_peer_is_never_silently_accepted',
 'authentication_failure_never_renders_private_state',
 'cancel_relocks_and_restores_terminal',
 'oversize_hidden_secret_rejected_and_queued_secret_flushed_before_shell',
 'typeahead_before_hidden_passphrase_is_discarded',
 'bounded_authenticated_history_ids_and_classic_inert_unicode',
 'reopened_outbox_persisted_ids_pending_and_fresh_device_guard',
 'explicit_retirement_preserves_history_without_fake_delivery_claim',
 'explicit_unicode_override_keeps_control_and_bidi_text_inert',
 'official_backup_is_history_only_and_fresh_device_needs_reverification',
 'expired_state_is_visible_without_silent_restart',
 'revoked_state_is_visible_without_silent_restart',
 'full_state_is_visible_without_silent_restart',
 'poisoned_write_exits_and_canonical_reopen_resolves_handle',
 'recovery_and_password_reset_cannot_unquarantine_old_ratchets',
 'deliberate_upgrade_preserves_legacy_records_and_rotates_recovery',
 'legacy_curve_or_partial_ed_identity_cannot_enroll',
 'confirmation_typeahead_and_control_splicing_cannot_change_trust',
 'hostile_remote_filter_pty_has_no_unlock_upgrade_or_store_authority',
 'profile_metadata_cannot_select_or_open_checkpoint_store'}
TEARDOWN={'owned_vm_removed','owned_private_inputs_removed','ports_closed','saved_playground_stopped','saved_playground_unchanged'}
FALSE_CLAIMS={'live_xmpp','network_inbox_send','roaming','attachments','physical_hardware','capacity','security_audit','full_conformance'}
def load_record(raw):
 assert hashlib.sha256(raw).hexdigest()==RECORD_SHA256,'saved checkpoint console record changed'
 return json.loads(raw)
def validate(d,resolve):
 assert d['schema']==1 and d['issue']==205 and d['result']=='pass-local-checkpoint-console'
 assert d['default_feature_off'] is True
 assert set(d['runs'])=={'host','netbsd'} and d['runs']['netbsd']['platform']=='NetBSD'
 for row in d['runs'].values():
  assert row['result']=='pass' and row['count']==len(row['checks'])==len(set(row['checks']))
  assert REQUIRED<=set(row['checks'])
  assert row['fixture_children_removed'] is True and row['network_mail'] is False and row['hardware_claim'] is False
 assert set(d['teardown'])==TEARDOWN and all(v is True for v in d['teardown'].values())
 assert set(d['claims'])==FALSE_CLAIMS and all(v is False for v in d['claims'].values())
 assert d['rust']['host_passed']==d['rust']['netbsd_passed']==72
 assert d['rust']['intentional_ignored_helper']==1 and d['rust']['helper_invoked_by_parent'] is True
 for route in ['host','netbsd']:
  disabled=d['default_off'][route]
  assert disabled['result']=='pass' and disabled['unsupported_command_rejected'] is True and disabled['store_not_created'] is True
  for kind,count in [('vault',13),('picker',8)]:
   old=d['compatibility'][route+'_'+kind]
   assert old['result']=='pass' and old['count']==count
 for group in ['execution_sources','frozen_sources']:
  assert d[group]
  for name,digest in d[group].items():
   assert re.fullmatch('[a-f0-9]{64}',digest)
   assert hashlib.sha256(resolve(name,digest)).hexdigest()==digest,name

def check():
 d=load_record((ROOT/'dev-docs/evidence/checkpoint-console/results.json').read_bytes())
 validate(d,source_bytes)
 prior=checkpoint_ui_baseline_manifest()
 for name,digest in prior['files'].items():assert hashlib.sha256(source_bytes(name,digest)).hexdigest()==digest
 for name,digest in prior['records'].items():assert hashlib.sha256((ROOT/name).read_bytes()).hexdigest()==digest,'prior #203 record changed'
 print('Saved member-local checkpoint console evidence verified: source-matched host/NetBSD PTYs, private-fixture teardown and unchanged prior results; no live mail/audit/capacity claim.')
if __name__=='__main__':check()
