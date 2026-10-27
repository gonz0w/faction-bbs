"""Archive acceptance harness boundaries, without services or saved acceptance."""
import json
import subprocess
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch
import probe_archive_mail as fixture


class ArchiveFixture(unittest.TestCase):
    def test_bounded_page_counters_and_boolean_completion(self):
        valid=dict(entries=4,accepted=3,duplicates=1,complete=False)
        self.assertEqual(fixture.page_result(json.dumps(valid).encode()),valid)
        for changed in [dict(entries=5,accepted=4,duplicates=1,complete=True),dict(valid,accepted=4),dict(valid,entries=True),dict(valid,complete=1),dict(valid,extra='private'),dict(valid,duplicates=-1)]:
            with self.assertRaises(fixture.FixtureFailure):fixture.page_result(json.dumps(changed).encode())
        with self.assertRaises(fixture.FixtureFailure):fixture.page_result(b'private non-json diagnostic')

    def test_status_has_exact_scope_and_does_not_bypass_inactive_state(self):
        token=dict(home='alice@home.test',local_device=101,epoch='ab'*16,after='opaque-1')
        valid=dict(enabled=True,cursor_present=True,ready=True,checkpoint=token)
        self.assertEqual(fixture.archive_status(json.dumps(valid).encode()),valid)
        blocked=dict(enabled=True,cursor_present=True,ready=False,checkpoint=None)
        self.assertEqual(fixture.archive_status(json.dumps(blocked).encode()),blocked)
        invalid=[dict(valid,enabled=False),dict(valid,cursor_present=False),dict(valid,ready=False),dict(valid,checkpoint=dict(token,epoch='0'*32)),dict(valid,checkpoint=dict(token,after='bad id')),dict(valid,checkpoint=dict(token,local_device=True)),dict(blocked,checkpoint=token)]
        for value in invalid:
            with self.assertRaises(fixture.FixtureFailure):fixture.archive_status(json.dumps(value).encode())

    def test_catchup_uses_stdin_credentials_and_preserves_run_as(self):
        home=dict(store='/private/store',config='/private/config',local_password='local-private',password='home-private',run_as='memberfixture')
        answer=dict(entries=0,accepted=0,duplicates=0,complete=True)
        with patch.object(fixture.base.subprocess,'run',return_value=subprocess.CompletedProcess([],0,json.dumps(answer).encode(),b'')) as run:
            self.assertEqual(fixture.catchup(Path('/binary'),home),answer)
        args,kwargs=run.call_args
        self.assertEqual(args[0],['/binary','catchup','/private/store','/private/config'])
        self.assertNotIn('private',str(args[0][0]))
        self.assertEqual(kwargs['input'],b'local-private\nhome-private\n')
        self.assertEqual(kwargs['user'],'memberfixture');self.assertEqual(kwargs['group'],'memberfixture');self.assertEqual(kwargs['extra_groups'],[])
        with patch.object(fixture.base.subprocess,'run',return_value=subprocess.CompletedProcess([],1,b'private payload',b'private error')):
            with self.assertRaisesRegex(fixture.FixtureFailure,'^native action rejected$'):fixture.catchup(Path('/binary'),home)

    def test_public_report_does_not_claim_remote_scan_or_include_private_metadata(self):
        result=fixture.public_result(list(fixture.CHECKS),'requires-separate-remote-proof','a'*64)
        self.assertNotIn(fixture.PLAINTEXT_CHECK,result['checks'])
        self.assertEqual(result['server_plaintext_audit'],'requires-separate-remote-proof')
        self.assertFalse({'cursor','epoch','password','subject','body','marker'} & result.keys())
        local=fixture.public_result(list(fixture.CHECKS)+[fixture.PLAINTEXT_CHECK],'pass-local-scan','a'*64)
        self.assertIn(fixture.PLAINTEXT_CHECK,local['checks'])
        for checks,audit in [(list(fixture.CHECKS),'pass-local-scan'),(list(fixture.CHECKS)+[fixture.PLAINTEXT_CHECK],'requires-separate-remote-proof'),(list(fixture.CHECKS)[:-1],'requires-separate-remote-proof')]:
            with self.assertRaises(fixture.FixtureFailure):fixture.public_result(checks,audit,'a'*64)

    def test_private_fixture_requires_owned_marker_and_bounded_private_profile(self):
        with tempfile.TemporaryDirectory(prefix='faction211-') as temporary:
            root=Path(temporary);root.chmod(0o700)
            marker=root/'OWNED';marker.write_text('faction211 disposable synthetic fixture\n');marker.chmod(0o600)
            path=root/'profile.json'
            value=dict(marker='ab'*16,fixture_marker=str(marker),alice={},bob={})
            path.write_text(json.dumps(value));path.chmod(0o600)
            self.assertEqual(fixture.load_profile(path),value)
            marker.write_text('unrelated fixture')
            with self.assertRaises(fixture.FixtureFailure):fixture.load_profile(path)
            marker.write_text('211');path.chmod(0o644)
            with self.assertRaises(fixture.FixtureFailure):fixture.load_profile(path)


if __name__=='__main__':unittest.main()
