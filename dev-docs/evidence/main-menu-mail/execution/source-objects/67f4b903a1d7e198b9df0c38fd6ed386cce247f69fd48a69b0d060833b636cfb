"""Synthetic evidence contract cases; never substitute for native execution."""
import copy
import hashlib
import json
import unittest
from unittest.mock import patch
import check_archive_mail_evidence as guard


def fixture():
    digest=hashlib.sha256(b'executed').hexdigest()
    routes={}
    for route,platform in [('host','Darwin'),('netbsd','NetBSD')]:
        positive=dict(result='pass',platform=platform,checks=sorted(guard.POSITIVE),count=7,native_product_client=True,configured_homes=2,server_plaintext_audit='requires-separate-remote-proof',marker_sha256=('a' if route=='host' else 'b')*64,live_provider_contact=False,no_complete_archive_or_read_receipt_claim=True)
        death=dict(result='pass',platform=platform,checks=sorted(guard.PROCESS_DEATH),count=4,signal='SIGKILL',publication_observed=True,recovered_new_count=3,canonical_history_count=9,exact_fsync_boundary_claim=False,server_plaintext_audit='requires-separate-proof')
        ui=dict(result='pass',platform=platform,checks=sorted(guard.UI),count=6,private_fixture_removed=True,provider_contact=False)
        routes[route]=dict(positive=positive,process_death=death,ui=ui)
    return dict(schema=1,issue=211,result='pass-manual-archive',default_feature_off=True,profile=copy.deepcopy(guard.PROFILE),runs=routes,
        rust=dict(host_passed=139,netbsd_passed=139,intentional_ignored_helpers=2,helpers_invoked_by_parent=True),
        default_off={route:dict(result='pass',unsupported_command_rejected=True,store_not_created=True) for route in routes},
        compatibility={route+'_'+kind:dict(result='pass',count=count) for route in routes for kind,count in [('checkpoint',23),('vault',13),('picker',8)]},
        server_plaintext_audits={route:dict(result='pass',marker_sha256=routes[route]['positive']['marker_sha256'],server_homes=2,files_scanned=12,script_sha256=digest) for route in routes},
        build=dict(complete_builder_mib=2048,fresh_1g_compile_claim=False),runtime=dict(configured_mib=1024,physmem_bytes=1054990336,ncpu=2),
        execution_sources={'sample.rs':digest},frozen_sources={'audit.py':digest},
        source_stages=[dict(label='synthetic executed stage',files=[dict(name='sample.rs',sha256=digest,resolved_path='frozen/sample.rs')])],
        teardown={key:True for key in guard.TEARDOWN},claims={key:False for key in guard.FALSE_CLAIMS})


class ArchiveEvidence(unittest.TestCase):
    def validate(self,value):guard.validate(value,lambda name,digest:b'executed')
    def reject(self,change):
        value=fixture();change(value)
        with self.assertRaises((AssertionError,KeyError)):self.validate(value)

    def test_complete_contract_and_native_optional_local_scan(self):
        self.validate(fixture())
        value=fixture();row=value['runs']['netbsd']['positive'];row['checks'].append(guard.LOCAL_SCAN);row.update(count=8,server_plaintext_audit='pass-local-scan');self.validate(value)

    def test_missing_route_controls_or_real_kill_refused(self):
        for change in [lambda d:d['runs'].pop('netbsd'),lambda d:d['runs']['host'].pop('process_death'),lambda d:d['runs']['host']['positive']['checks'].pop(),lambda d:d['runs']['netbsd']['ui'].update(count=5),lambda d:d['runs']['host']['process_death'].update(signal='normal-exit'),lambda d:d['runs']['host']['process_death'].update(publication_observed=False),lambda d:d['runs']['netbsd']['process_death'].update(recovered_new_count=2),lambda d:d['runs']['netbsd']['process_death'].update(canonical_history_count=8),lambda d:d['runs']['host']['process_death'].update(exact_fsync_boundary_claim=True)]:self.reject(change)

    def test_home_ids_metadata_authority_and_four_result_cap_cannot_be_relaxed(self):
        for change in [dict(own_home_only=False),dict(query_ids_matched=False),dict(cursor_scope=['home']),dict(archive_page_max=5),dict(server_metadata_grants_trust=True),dict(server_metadata_retires_outbox=True),dict(authenticated_time='server-delay'),dict(explicit_migration=False),dict(archive_mode_presence=True),dict(archive_mode_live_receive=True),dict(outgoing_store_hint=False)]:
            self.reject(lambda d:d['profile'].update(change))

    def test_audits_require_both_routes_matching_markers_and_executed_script(self):
        for change in [lambda d:d['server_plaintext_audits'].pop('host'),lambda d:d['server_plaintext_audits']['host'].update(result='unverified'),lambda d:d['server_plaintext_audits']['host'].update(marker_sha256=d['runs']['netbsd']['positive']['marker_sha256']),lambda d:d['server_plaintext_audits']['netbsd'].update(server_homes=1),lambda d:d['server_plaintext_audits']['host'].update(files_scanned=0),lambda d:d['server_plaintext_audits']['host'].update(script_sha256='0'*64),lambda d:d['runs']['host']['positive'].update(server_plaintext_audit='pass-local-scan')]:self.reject(change)

    def test_defaults_compatibility_native_counts_and_runtime_scope_required(self):
        for change in [lambda d:d['rust'].update(netbsd_passed=138),lambda d:d['rust'].update(helpers_invoked_by_parent=False),lambda d:d['default_off'].pop('host'),lambda d:d['default_off']['host'].update(store_not_created=False),lambda d:d['compatibility']['netbsd_checkpoint'].update(count=22),lambda d:d['compatibility']['host_vault'].update(count=0),lambda d:d['compatibility']['host_picker'].update(count=0),lambda d:d['runtime'].update(configured_mib=2048),lambda d:d['runtime'].update(physmem_bytes=2147483648),lambda d:d['runtime'].update(ncpu=4),lambda d:d['build'].update(fresh_1g_compile_claim=True)]:self.reject(change)

    def test_no_overclaims_and_complete_teardown(self):
        for key in guard.FALSE_CLAIMS:self.reject(lambda d:d['claims'].update({key:True}))
        for key in guard.TEARDOWN:self.reject(lambda d:d['teardown'].update({key:False}))
        self.reject(lambda d:d['teardown'].pop('ports_closed'))

    def test_source_and_stage_hashes_are_mandatory_and_tamper_evident(self):
        with self.assertRaises(AssertionError):guard.validate(fixture(),lambda name,digest:b'tampered')
        for change in [lambda d:d.update(execution_sources={}),lambda d:d.update(frozen_sources={}),lambda d:d.update(source_stages=[]),lambda d:d['source_stages'][0]['files'][0].update(sha256='0'*64),lambda d:d['source_stages'][0]['files'][0].update(resolved_path='/private/fixture'),lambda d:d['source_stages'][0]['files'][0].pop('sha256')]:self.reject(change)

    def test_optional_cost_requires_actual_whole_ui_scope_and_finite_native_units(self):
        # This exact scope is emitted by the preserved native collector. Platform
        # comes from the separately captured native runtime, not a rewritten cost.
        scope='one complete faction-client manual UI process, including checkpoint unlock and one explicit empty MAM2 archive page over verified TLS/account; no pending writes'
        value=fixture();value['native_archive_cost']=dict(result='pass',platform='NetBSD',usage=dict(scope=scope,wall_seconds=1.0,user_seconds=0,system_seconds=0,max_rss_kib=4096),compiler_included=False,server_fixture_included=False,capacity_claim=False,empty_archive_page=True,canonical_unchanged=True,terminal_restored=True,collector_sha256=hashlib.sha256(b'executed').hexdigest())
        self.validate(value)
        for key,invalid in [('scope','decode only'),('wall_seconds',0),('max_rss_kib',0),('user_seconds',-1),('system_seconds',float('nan'))]:
            changed=copy.deepcopy(value);changed['native_archive_cost']['usage'][key]=invalid
            with self.assertRaises(AssertionError):self.validate(changed)
        for key,invalid in [('empty_archive_page',False),('canonical_unchanged',False),('terminal_restored',False),('collector_sha256','0'*64)]:
            changed=copy.deepcopy(value);changed['native_archive_cost'][key]=invalid
            with self.assertRaises(AssertionError):self.validate(changed)

    def test_ci_runs_archive_ui_and_saved_record_guard_with_existing_gates(self):
        workflow=(guard.ROOT/'.github/workflows/documentation.yml').read_text()
        native,documentation=workflow.split('  documentation:',1)
        self.assertIn('python3 tools/probe_home_archive_ui.py --client "$CARGO_TARGET_DIR/release/faction-client" --fixture "$CARGO_TARGET_DIR/release/examples/checkpoint_ui_fixture"',native)
        self.assertIn('python3 tools/check_archive_mail_evidence.py',documentation)
        self.assertIn('needs: native_mail',documentation)
        self.assertEqual(workflow.count('timeout-minutes: 15'),2)
        self.assertIn("test_*.py",documentation)

    def test_raw_record_pin_precedes_parsing_and_rejects_reformatting(self):
        raw=json.dumps(fixture()).encode()
        with patch.object(guard,'RECORD_SHA256',hashlib.sha256(raw).hexdigest()):
            self.assertEqual(guard.load_record(raw)['issue'],211)
            for changed in [raw+b' ',b'not json']:
                with self.assertRaises(AssertionError):guard.load_record(changed)


if __name__=='__main__':unittest.main()
