"""Immutable signer evidence must prove its gates, source identities and limits."""
import copy
import hashlib
import json
from pathlib import Path
import tempfile
import unittest
import check_signer_qualification_evidence as evidence

ERRORS = (AssertionError, KeyError, ValueError, OSError, TypeError, StopIteration)
COMPATIBILITY = ('base_binary_hashes_unchanged', 'baseline_signer_hash_matches_preserved_rejected',
                 'admin_ssh_before_after', 'admin_ssh_tty_after', 'sshd_config_valid',
                 'baseline_cli_before_after')
CLAIMS = ('installed', 'production_issuer', 'pkgsrc_tested', 'physical_hardware',
          'current_member_ssh_trial', 'capacity')


def operation(status='success', code=0):
    return dict(status=status, exit=code, elapsed_ms=1)


def qualified_probe(probe_hash, candidate, verifier):
    controls = []
    for case in range(1, 6):
        ops = {name: operation() for name in ('issue', 'parse', 'ca_fingerprint', 'leaf_fingerprint',
                 'possession', 'verify', 'possession-reuse', 'reuse_verify')}
        ops.update({name: operation('normal_error', 1) for name in
                    ('wrong_ca', 'wrong_principal', 'wrong_private_key', 'tamper')})
        controls.append(dict(case=case, artifact_produced=True, accepted=True,
            independently_parsed=True, signature_verified=True, reuse_verified=True,
            wrong_ca_denied=True, wrong_principal_denied=True, wrong_private_key_denied=True,
            tamper_denied=True, operations=ops))
    temporal = []
    for kind in ('expired', 'future'):
        ops = {name: operation() for name in ('issue', 'parse', 'ca_fingerprint', 'leaf_fingerprint',
                                            'possession', 'in_window_verify')}
        ops['current_verify'] = operation('normal_error', 1)
        temporal.append(dict(case=kind, independently_parsed=True, validity_direction_confirmed=True,
                             in_window_verified=True, current_time_denied=True, operations=ops))
    return dict(result='qualified', overall_qualification=True, fixture_setup_complete=True,
        failure_layer=None, valid_issuance_count=5, rounds=5, validity_seconds=300,
        command_timeout_seconds=10, command_output_limit_bytes=65536, core_dumps_disabled=True,
        keys_removed=True, expired_denied=True, future_denied=True, missing_ca_denied=True,
        candidate_sha256=candidate, verifier_sha256=verifier, probe_sha256=probe_hash,
        controls=controls, temporal_controls=temporal, missing_ca_operation=operation('normal_error', 255),
        setup_operations=[dict(**operation(), artifacts_produced=True) for _ in range(4)])


class SignerEvidenceTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name)
        names = ['tools/probe_roaming_signer.py', 'tools/test_roaming_signer.py',
                 'tools/probe_signing_exit.py', 'tools/test_signing_exit.py', 'tools/smoke_local_console.py']
        sources = [self.ref('source-'+str(i), ('executed synthetic source '+name).encode())
                   for i, name in enumerate(names)]
        files = [dict(name=name, **ref) for name, ref in zip(names, sources)]
        pins = {f['name']: f['sha256'] for f in files}
        probe_hash = pins['tools/probe_roaming_signer.py']
        baseline_hash, candidate_hash = 'b1'*32, 'c2'*32
        host = qualified_probe(probe_hash, 'd3'*32, 'd3'*32)
        native = qualified_probe(probe_hash, candidate_hash, baseline_hash)
        baseline = copy.deepcopy(native)
        baseline.update(result='rejected', overall_qualification=False, valid_issuance_count=0,
                        candidate_sha256=baseline_hash, verifier_sha256=baseline_hash,
                        failure_layer='qualification', expired_denied=False, future_denied=False)
        for control in baseline['controls']:
            for field in ('accepted', 'independently_parsed', 'signature_verified', 'reuse_verified',
                          'wrong_ca_denied', 'wrong_principal_denied', 'wrong_private_key_denied', 'tamper_denied'):
                control[field] = False
            control['operations'] = dict(issue=operation('signal', -11))
        for control in baseline['temporal_controls']:
            for field in ('independently_parsed', 'validity_direction_confirmed', 'in_window_verified', 'current_time_denied'):
                control[field] = False
            control['operations'] = dict(issue=operation('signal', -11))
        unit = dict(result='pass', tests=16, failures=0, errors=0)
        self.records = {
            'host-probe': host, 'native-probe': native, 'baseline': baseline,
            'host-unit': unit, 'native-unit': copy.deepcopy(unit),
            'build': dict(result='pass', steps=[dict(command=['./configure'], returncode=0),
                                               dict(command=['make', '-j1', 'ssh-keygen'], returncode=0)]),
            'source-match': dict(result='pass', all_match=True, files=pins),
            'compatibility': dict(result='pass', **{key: True for key in COMPATIBILITY},
                                  secret_echoed=False, keys_or_state_retained=False),
            'source-provenance': dict(result='pass', archive_sha256='a9dc9565dffe8640f64d863cd29a32bc4a3dbdec0566a7fc44c5d6ee767d5f39',
                                     release_signature_verified=True, archive_paths_safe=True,
                                     no_js_ts_archive_members=True, selected_candidates=1),
            'cleanup': dict(owned_removed=True, saved_stopped=True, saved_hashes_unchanged=True, forwards_closed=True),
        }
        self.result = dict(schema=1, issue=226, result='qualified',
            records={name: self.ref(name+'.json', value) for name, value in self.records.items()},
            sources=sources, source_stages=[dict(label='native-executed', files=files)],
            claims={name: False for name in CLAIMS})

    def ref(self, name, value):
        raw = value if isinstance(value, bytes) else json.dumps(value).encode()
        (self.root/name).write_bytes(raw)
        return dict(path=name, sha256=hashlib.sha256(raw).hexdigest())

    def changed_record(self, name, path, value):
        changed = copy.deepcopy(self.records[name])
        target = changed
        for part in path[:-1]:
            target = target[part]
        target[path[-1]] = value
        result = copy.deepcopy(self.result)
        result['records'][name] = self.ref('changed.json', changed)
        return result

    def test_complete_source_bound_result(self):
        records = evidence.validate(self.root, self.result)
        self.assertEqual(records['native-probe']['valid_issuance_count'], 5)

    def test_false_qualification_crash_and_missing_gates_fail(self):
        changes = [
            ('host-probe', ['overall_qualification'], False),
            ('native-probe', ['result'], 'rejected'),
            ('native-probe', ['valid_issuance_count'], 4),
            ('native-probe', ['controls', 0, 'operations', 'issue'], operation('signal', -11)),
            ('native-probe', ['controls', 0, 'operations', 'wrong_ca'], operation('signal', -11)),
            ('native-probe', ['controls', 0, 'operations', 'wrong_private_key'], operation('timeout', -9)),
            ('native-probe', ['controls', 0, 'operations', 'parse'], operation('unavailable', None)),
            ('native-probe', ['controls', 0, 'reuse_verified'], False),
            ('native-probe', ['controls', 0, 'operations', 'reuse_verify'], operation('normal_error', 1)),
            ('native-probe', ['temporal_controls', 1, 'validity_direction_confirmed'], False),
            ('native-probe', ['temporal_controls', 1, 'operations', 'current_verify'], operation('output_limit', -9)),
            ('native-probe', ['missing_ca_operation'], operation('signal', -11)),
            ('native-probe', ['keys_removed'], False),
            ('native-probe', ['setup_operations', 3, 'artifacts_produced'], False),
            ('baseline', ['overall_qualification'], True),
            ('baseline', ['controls', 0, 'accepted'], True),
            ('baseline', ['controls', 0, 'operations', 'parse'], operation()),
        ]
        for name, path, value in changes:
            with self.subTest(record=name, path=path), self.assertRaises(ERRORS):
                evidence.validate(self.root, self.changed_record(name, path, value))
        missing = copy.deepcopy(self.result)
        value = copy.deepcopy(self.records['native-probe'])
        del value['controls'][0]['operations']['verify']
        missing['records']['native-probe'] = self.ref('missing-gate.json', value)
        with self.assertRaises(ERRORS):
            evidence.validate(self.root, missing)

    def test_missing_records_claims_and_supporting_failures_fail(self):
        for name in self.records:
            missing = copy.deepcopy(self.result)
            del missing['records'][name]
            with self.subTest(missing=name), self.assertRaises(ERRORS):
                evidence.validate(self.root, missing)
        for claim in CLAIMS:
            unsafe = copy.deepcopy(self.result)
            unsafe['claims'][claim] = True
            with self.subTest(claim=claim), self.assertRaises(ERRORS):
                evidence.validate(self.root, unsafe)
        changes = [
            ('host-unit', ['tests'], 15), ('native-unit', ['failures'], 1),
            ('native-unit', ['errors'], 1), ('build', ['steps', 0, 'returncode'], 1),
            ('source-match', ['all_match'], False),
            ('source-provenance', ['release_signature_verified'], False),
            ('source-provenance', ['archive_paths_safe'], False),
            ('source-provenance', ['no_js_ts_archive_members'], False),
            ('source-provenance', ['selected_candidates'], 2),
            ('source-provenance', ['archive_sha256'], '0'*64),
            ('compatibility', ['secret_echoed'], True),
            ('compatibility', ['keys_or_state_retained'], True),
        ]
        changes += [('compatibility', [key], False) for key in COMPATIBILITY]
        changes += [('cleanup', [key], False) for key in self.records['cleanup']]
        for name, path, value in changes:
            with self.subTest(record=name, path=path), self.assertRaises(ERRORS):
                evidence.validate(self.root, self.changed_record(name, path, value))

    def test_path_digest_source_and_binary_mismatches_fail(self):
        for path in ('../escape', '/absolute', 'source-0/../source-0'):
            bad = copy.deepcopy(self.result)
            bad['sources'][0]['path'] = path
            with self.subTest(path=path), self.assertRaises(ERRORS):
                evidence.validate(self.root, bad)
        link = self.root/'linked-source'
        link.symlink_to(self.root/'source-0')
        bad = copy.deepcopy(self.result)
        bad['sources'][0]['path'] = link.name
        with self.assertRaises(ERRORS):
            evidence.validate(self.root, bad)
        for location in ('sources', 'source_stages'):
            bad = copy.deepcopy(self.result)
            target = bad['sources'][0] if location == 'sources' else bad['source_stages'][0]['files'][0]
            target['sha256'] = '0'*64
            with self.subTest(location=location), self.assertRaises(ERRORS):
                evidence.validate(self.root, bad)
        bad = copy.deepcopy(self.result)
        bad['source_stages'][0]['files'].append(copy.deepcopy(bad['source_stages'][0]['files'][0]))
        with self.assertRaises(ERRORS):
            evidence.validate(self.root, bad)
        changes = [
            ('native-probe', ['candidate_sha256'], self.records['baseline']['candidate_sha256']),
            ('native-probe', ['verifier_sha256'], '0'*64),
            ('host-probe', ['probe_sha256'], '0'*64),
            ('source-match', ['files'], {'other.py': '0'*64}),
        ]
        for name, path, value in changes:
            with self.subTest(record=name, path=path), self.assertRaises(ERRORS):
                evidence.validate(self.root, self.changed_record(name, path, value))
        (self.root/'source-0').write_bytes(b'changed executed source')
        with self.assertRaises(ERRORS):
            evidence.validate(self.root, self.result)


class CommittedSignerEvidenceTests(unittest.TestCase):
    def test_committed_record_and_all_bound_inputs(self):
        evidence.check()
