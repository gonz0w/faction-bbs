"""Synthetic contract tests; no saved acceptance record or native run is created."""
import copy
import hashlib
import json
import unittest
from unittest.mock import patch
import check_home_mail_evidence as guard


def fixture():
    digest = hashlib.sha256(b'executed').hexdigest()
    ui = dict(result='pass', count=9, checks=sorted(guard.UI_CHECKS), private_fixture_removed=True, provider_contact=False)
    matrix = dict(result='pass', count=20, private_fixture_removed=True, product_transport_exercised=True, encrypted_mail_proof=False, live_federation=False,
                  outcomes=[dict(check=name, status=status, result='pass', attempts=1, tls_version='TLSv1.2' if name == 'verified_tls12_profile' else 'TLSv1.3') for name,status in guard.TLS_OUTCOMES.items()])
    return dict(schema=1, issue=209, result='pass-native-home-mail', default_feature_off=True,
        runs={route:dict(ui,platform=platform) for route,platform in [('host','Darwin'),('netbsd','NetBSD')]},
        rust=dict(host_passed=104,netbsd_passed=104,intentional_ignored_helper=1,helper_invoked_by_parent=True),
        transport={route:dict(rust_passed=25,matrix=copy.deepcopy(matrix)) for route in ['host','netbsd']},
        native_interop=dict(result='pass',platform='NetBSD',scope='complete',checks=sorted(guard.INTEROP_CHECKS),independent_unprivileged_homes=2,native_product_client=True,no_offline_archive_reliability_claim=True,live_provider_contact=False),
        default_off={route:dict(result='pass',unsupported_command_rejected=True,store_not_created=True) for route in ['host','netbsd']},
        compatibility={route+'_'+kind:dict(result='pass',count=count) for route in ['host','netbsd'] for kind,count in [('checkpoint',23),('vault',13),('picker',8)]},
        cost=dict(result='pass',usage=dict(wall_seconds=1.2,user_seconds=.3,system_seconds=.1,max_rss_kib=8192,scope=guard.COST_SCOPE),whole_client=True,includes_unlock=True,includes_network_cycle=True,compiler_included=False,server_fixture_included=False,capacity_claim=False,terminal_restored=True),
        build=dict(complete_builder_mib=2048,fresh_1g_compile_claim=False),
        runtime=dict(configured_mib=1024,physmem_bytes=1054990336,ncpu=2),
        execution_sources={'sample.rs':digest},frozen_sources={'before.rs':digest},
        teardown={key:True for key in guard.TEARDOWN},claims={key:False for key in guard.FALSE_CLAIMS})


class HomeMailEvidence(unittest.TestCase):
    def validate(self, value):
        guard.validate(value, lambda name,digest:b'executed')

    def reject(self, change):
        value=fixture();change(value)
        with self.assertRaises((AssertionError,KeyError)):
            self.validate(value)

    def test_complete_synthetic_record(self):
        self.validate(fixture())

    def test_missing_route_or_acceptance_or_wrong_count(self):
        for change in [lambda d:d['runs'].pop('netbsd'),lambda d:d['transport'].pop('host'),lambda d:d['default_off'].pop('host'),lambda d:d['runs']['host']['checks'].pop(),lambda d:d['native_interop']['checks'].pop(),lambda d:d['native_interop'].update(scope='negative-controls-only'),lambda d:d['rust'].update(netbsd_passed=103),lambda d:d['transport']['host'].update(rust_passed=24),lambda d:d['transport']['netbsd']['matrix']['outcomes'].pop()]:
            self.reject(change)

    def test_duplicate_or_wrong_tls_outcome_is_not_a_control(self):
        for change in [lambda d:d['native_interop']['checks'].__setitem__(0,d['native_interop']['checks'][1]),lambda d:d['transport']['host']['matrix']['outcomes'][0].update(status='tls'),lambda d:d['transport']['host']['matrix']['outcomes'][0].update(attempts=2),lambda d:d['transport']['host']['matrix']['outcomes'][1].update(tls_version='TLSv1.3')]:
            self.reject(change)

    def test_default_off_legacy_routes_and_helper_are_required(self):
        for route in ['host','netbsd']:
            self.reject(lambda d:d['default_off'][route].update(store_not_created=False))
            self.reject(lambda d:d['default_off'][route].update(unsupported_command_rejected=False))
            for kind in ['checkpoint','vault','picker']:
                self.reject(lambda d:d['compatibility'][route+'_'+kind].update(count=0))
        self.reject(lambda d:d['rust'].update(helper_invoked_by_parent=False))
        self.reject(lambda d:d.update(default_feature_off=False))

    def test_cost_must_be_positive_finite_and_whole_ui_scope(self):
        for key in ['wall_seconds','user_seconds','system_seconds','max_rss_kib']:
            for invalid in [0,-1,True,float('inf'),float('nan'),'1']:
                self.reject(lambda d:d['cost']['usage'].update({key:invalid}))
        self.reject(lambda d:d['cost']['usage'].update(scope='ciphertext only'))
        for key in ['whole_client','includes_unlock','includes_network_cycle','terminal_restored']:
            self.reject(lambda d:d['cost'].update({key:False}))
        for key in ['compiler_included','server_fixture_included','capacity_claim']:
            self.reject(lambda d:d['cost'].update({key:True}))

    def test_compiler_runtime_and_no_overclaims(self):
        for change in [lambda d:d['runtime'].update(physmem_bytes=2147483648),lambda d:d['runtime'].update(ncpu=4),lambda d:d['build'].update(complete_builder_mib=1024),lambda d:d['build'].update(fresh_1g_compile_claim=True)]:
            self.reject(change)
        for key in guard.FALSE_CLAIMS:
            self.reject(lambda d:d['claims'].update({key:True}))
        for key in guard.TEARDOWN:
            self.reject(lambda d:d['teardown'].update({key:False}))

    def test_source_tampering_empty_sources_and_path_escape(self):
        with self.assertRaises(AssertionError):
            guard.validate(fixture(),lambda name,digest:b'tampered')
        for group in ['execution_sources','frozen_sources']:
            self.reject(lambda d:d.update({group:{}}))
            self.reject(lambda d:d.update({group:{'../escape':hashlib.sha256(b'executed').hexdigest()}}))

    def test_raw_record_is_pinned_before_json_parsing(self):
        raw=json.dumps(fixture()).encode()
        with patch.object(guard,'RECORD_SHA256',hashlib.sha256(raw).hexdigest()):
            self.assertEqual(guard.load_record(raw)['issue'],209)
            for altered in [raw+b' ',b'not-json']:
                with self.assertRaises(AssertionError):guard.load_record(altered)


if __name__=='__main__':unittest.main()
