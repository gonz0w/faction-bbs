"""Offline regression checks for source-bound owner CLI observations."""
import copy
import hashlib
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest

SERVICE = {'actual-owner-cli-local-member-id-lookup', 'actual-owner-cli-invite-enable-numeric-member-link-consent-binding-enable', 'actual-owner-cli-state-through-pty', 'owner-inspection-preview-readonly-private-new-export-root-install', 'actual-faction-member-entry-denies-owner-command', 'exported-root-manifest-valid-service-startup', 'kernel-peer-uid-owned-tty-enters-approved-session', 'actual-owner-cli-disable-invalidates-live-service-session', 'disabled-binding-old-manifest-rejected-before-listening', 'stale-binding-generation-old-manifest-rejected-before-listening', 'explicit-reexport-current-generation-service-restarts', 'revoked-member-export-and-service-startup-fail-closed', 'real-export-restore-disables-authority-rotates-board-old-manifest-rejected', 'default-off-service-classic-owned-tty-login-menu'}

SPEC = importlib.util.spec_from_file_location('owner_evidence', Path(__file__).with_name('check_roaming_owner_evidence.py'))

class EvidenceTests(unittest.TestCase):
    def setUp(self):
        self.module = importlib.util.module_from_spec(SPEC)
        SPEC.loader.exec_module(self.module)
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.record = dict(schema=1, issue=228, result='pass', profile=self.module.PROFILE.copy(),
                           claims={name: False for name in self.module.CLAIMS}, sources=[], records={},
                           source_stages=[], prior_records=[])
        def save(name, data):
            path = self.root / name
            path.write_bytes(data)
            return dict(path=name, sha256=hashlib.sha256(data).hexdigest())
        source = save('source.rs', b'actual measured source')
        self.record['sources'] = [source]
        self.record['source_stages'] = [dict(label='native-final', files=[dict(name='crates/example.rs', sha256=source['sha256'])])]
        values = {name: dict(result='pass') for name in self.module.REQUIRED}
        values['host-cli']['passed'] = values['native-cli']['passed'] = 18
        values['native-service']['checks'] = [dict(name=name, result='pass') for name in sorted(SERVICE)]
        values['native-classic']['checks'] = [dict(name=name, result='pass', returncode=0) for name in ('classic-console-pty','export-restore-pty','complete-bundle-recovery')]
        values['recovery'].update(count=21, checks=[str(n) for n in range(21)], native_distinct_uid_cli=True, developer_api_fixture=False, capacity_claim=False, hardening_claim=False)
        values['native-source-match'].update(all_match=True, files={'crates/example.rs': source['sha256']})
        values['runtime'].update(physmem=1054990336, **self.module.PROFILE)
        values['cleanup'].update(owned_removed=True, saved_stopped=True, saved_hashes_unchanged=True, forwards_closed=True)
        values['tdd'].update(red_exit=101, green_exit=0, captured_before_edit=True)
        for name, value in values.items():
            self.record['records'][name] = save(name+'.json', json.dumps(value).encode())
        self.record['prior_records'] = [save('prior.json', b'unchanged historical observation')]

    def test_accepts_source_bound_bounded_observations(self):
        self.module.validate(self.root, self.record)

    def test_rejects_missing_modified_and_escaping_evidence(self):
        for mutate in (
            lambda r: r['sources'][0].update(sha256='0'*64),
            lambda r: r['sources'][0].update(path='../escape'),
            lambda r: r['records'].pop('native-service'),
            lambda r: r['prior_records'][0].update(sha256='0'*64),
        ):
            with self.subTest(mutate=mutate):
                r = copy.deepcopy(self.record)
                mutate(r)
                with self.assertRaises((AssertionError, KeyError, FileNotFoundError)):
                    self.module.validate(self.root, r)

    def test_rejects_changed_native_sources_and_enlarged_claims(self):
        for mutate in (
            lambda r: r['source_stages'][0]['files'][0].update(sha256='1'*64),
            lambda r: r['profile'].update(normal_sessions=250),
            lambda r: r['claims'].update(production=True),
            lambda r: r['claims'].update(extra=True),
        ):
            with self.subTest(mutate=mutate):
                r = copy.deepcopy(self.record)
                mutate(r)
                with self.assertRaises((AssertionError, KeyError)):
                    self.module.validate(self.root, r)

    def test_rejects_missing_or_failed_authority_controls_and_recovery(self):
        for name, change in [
            ('native-service', lambda d: d['checks'].pop()),
            ('native-service', lambda d: d['checks'][0].update(result='fail')),
            ('native-classic', lambda d: d['checks'].pop()),
            ('recovery', lambda d: d.update(native_distinct_uid_cli=False)),
            ('recovery', lambda d: d.update(count=0)),
        ]:
            with self.subTest(name=name, change=change):
                record = copy.deepcopy(self.record)
                ref = record['records'][name]
                path = self.root / ref['path']
                original = path.read_bytes()
                data = json.loads(original)
                change(data)
                payload = json.dumps(data).encode()
                path.write_bytes(payload)
                ref['sha256'] = hashlib.sha256(payload).hexdigest()
                try:
                    with self.assertRaises(AssertionError):
                        self.module.validate(self.root, record)
                finally:
                    path.write_bytes(original)

    def test_rejects_failed_records_and_missing_cleanup_or_red(self):
        for name, key, value in [('native-cli','result','fail'), ('host-cli','passed',0),
                                 ('cleanup','owned_removed',False), ('tdd','red_exit',0),
                                 ('tdd','captured_before_edit',False), ('runtime','physmem',2*1024**3)]:
            with self.subTest(name=name, key=key):
                r = copy.deepcopy(self.record)
                ref = r['records'][name]
                path = self.root / ref['path']
                original = path.read_bytes()
                data = json.loads(original)
                data[key] = value
                raw = json.dumps(data).encode()
                path.write_bytes(raw)
                ref['sha256'] = hashlib.sha256(raw).hexdigest()
                try:
                    with self.assertRaises(AssertionError):
                        self.module.validate(self.root, r)
                finally:
                    path.write_bytes(original)

if __name__ == '__main__':
    unittest.main()
