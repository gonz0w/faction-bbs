#!/usr/bin/env python3
"""Validate immutable saved native checkpoint evidence; never rerun a VM/crypto."""
import hashlib,json,re
from pathlib import Path
from evidence_sources import ROOT,source_bytes
BUILD_FOLLOWUP_SHA256='947bc169e9feecbcd12cfcc7bf4c23c72a446f5406629a867b943c4cbb01fd3e'
RECORD_SHA256='d93af1e39174b1382b1b261acd105ca25075267f1dcbb7b357c8a410f94acd79'
FALSE_CLAIMS={'network_transport','terminal_network_mail','roaming','attachments','physical_hardware','capacity','full_conformance','security_audit','whole_client_cost'}
TEARDOWN={'owned_vm_removed','owned_private_inputs_removed','ports_closed','saved_playground_stopped','saved_playground_unchanged'}
REQUIRED_CHECKS={
 'backup_is_history_only_until_explicit_fresh_device',
 'checkpoint_review_parent_sync_failure_prevents_usable_initialization',
 'checkpoint_review_recovery_quarantine_survives_password_reopen',
 'checkpoint_review_restart_outbox_recovers_acknowledgement_id',
 'delayed_keys_and_reply_ratchets_commit_with_history',
 'encrypted_send_receive_retry_and_ack_survive_reopen',
 'explicit_upgrade_preserves_v1_drafts_pins_and_original_backup',
 'locks_wrong_secrets_malformed_bytes_and_caps_fail_closed',
 'process_death_after_delivery_before_ack_retries_identical_ciphertext',
 'process_death_before_and_after_publication_keeps_canonical_state',
 'reset_write_ambiguity_never_rolls_back_into_a_live_handle',
 'validation_tamper_trust_and_duplicates_do_not_change_canonical_state',
 'write_errors_poison_until_durable_reopen'}
def load_record(raw):
 assert hashlib.sha256(raw).hexdigest()==RECORD_SHA256,'saved checkpoint record changed'
 return json.loads(raw)
def validate(d,resolve):
 assert d['schema']==1 and d['issue']==203 and d['result']=='pass-experimental-native-checkpoints'
 assert d['default_feature_off'] is True
 assert d['profile']==dict(namespace='urn:xmpp:omemo:2',sce='urn:xmpp:sce:1',timestamp_subset='UTC-whole-seconds-1970-9999')
 assert d['limits']==dict(plain=524288,history=64,outbox=8,peers=1,snapshot=32768,skipped_keys=32,session_seconds=2592000)
 for route in ['host','netbsd']:
  r=d['runs'][route]
  assert (r['checkpoint_passed'],r['checkpoint_failed'],r['checkpoint_ignored_helper'])==(17,0,1)
  assert r['helper_invoked_by_parent_tests'] is True
  assert (r['client_passed'],r['native_passed'],r['disabled_passed'],r['vault_count'],r['picker_count'])==(21,27,1,13,8)
  assert REQUIRED_CHECKS<=set(r['checkpoint_checks'])
  m=d['interop'][route]
  assert m['result']=='pass' and m['independent_two_way'] is True
  assert m['count']==len(m['checks'])==len(set(m['checks']))==92
  for side in ['native','reference']:assert side+'_initiates_first_and_reciprocal_reply_sce' in m['checks']
 assert set(d['claims'])==FALSE_CLAIMS and all(v is False for v in d['claims'].values())
 assert set(d['teardown'])==TEARDOWN and all(v is True for v in d['teardown'].values())
 for name in ['native_input_capture','frozen_sources']:
  assert d[name]
  for path,digest in d[name].items():
   assert re.fullmatch('[a-f0-9]{64}',digest)
   assert hashlib.sha256(resolve(path,digest)).hexdigest()==digest,path

def verify_capture_correction(current,original_raw):
 c=current['capture_correction']
 assert hashlib.sha256(original_raw).hexdigest()==c['original_record_sha256']
 original=json.loads(original_raw)
 excluded=c['excluded_non_build_metadata']
 assert excluded and len(set(excluded))==len(excluded)
 assert all(Path(n).name=='.DS_Store' and not Path(n).is_absolute() and '..' not in Path(n).parts for n in excluded)
 expected=dict(original['native_input_capture'])
 for name in excluded:del expected[name]
 assert current['native_input_capture']==expected,'runtime capture changed during metadata correction'
 assert {k:v for k,v in current.items() if k not in {'capture_correction','native_input_capture'}}=={k:v for k,v in original.items() if k!='native_input_capture'},'measured outcome changed during metadata correction'


def validate_build_followup(follow,resolve):
 assert follow['platform_flags']=={'linux':['-D_DEFAULT_SOURCE'],'netbsd':['-D_NETBSD_SOURCE'],'macos':[]}
 for name in ['runtime_code_changed','netbsd_flags_changed','vm_replayed_after_change','observations_refreshed']:
  assert follow[name] is False,name
 assert follow['old_inputs'] and set(follow['old_inputs'])==set(follow['preserved_inputs'])==set(follow['current_inputs'])
 for name,digest in follow['old_inputs'].items():
  assert hashlib.sha256(resolve(follow['preserved_inputs'][name],digest)).hexdigest()==digest
 for name,digest in follow['current_inputs'].items():
  assert hashlib.sha256(resolve(name,digest)).hexdigest()==digest
 for name,digest in follow.get('frozen_tdd_inputs',{}).items():
  assert hashlib.sha256(resolve(name,digest)).hexdigest()==digest

def resolve_followup_source(follow,name,digest,resolve):
 if name in follow['preserved_inputs']:
  assert follow['old_inputs'][name]==digest
  return resolve(follow['preserved_inputs'][name],digest)
 return resolve(name,digest)


def check():
 d=load_record((ROOT/'dev-docs/evidence/mail-checkpoints/results.json').read_bytes())
 verify_capture_correction(d,(ROOT/'dev-docs/evidence/mail-checkpoints/initial-capture-record.json').read_bytes())
 raw=(ROOT/'dev-docs/evidence/mail-checkpoints/linux-build-followup.json').read_bytes()
 assert hashlib.sha256(raw).hexdigest()==BUILD_FOLLOWUP_SHA256,'post-proof build follow-up changed'
 follow=json.loads(raw);validate_build_followup(follow,source_bytes)
 assert all(d['native_input_capture'][name]==digest for name,digest in follow['old_inputs'].items())
 validate(d,lambda name,digest:resolve_followup_source(follow,name,digest,source_bytes))
 print('Saved checkpoint evidence verified: host/NetBSD atomic recovery, exact source capture, independent reciprocal controls and owned cleanup; no network/UI/audit/capacity claim.')
if __name__=='__main__':check()
