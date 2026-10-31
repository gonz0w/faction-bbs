"""Keep the inbox acceptance in both optional client build graphs."""
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


class InboxCiContract(unittest.TestCase):
    def test_native_only_and_network_inbox_controls_are_required(self):
        workflow = (ROOT / '.github/workflows/documentation.yml').read_text()
        native = workflow.split('  native_mail:', 1)[1].split('  documentation:', 1)[0]
        commands = [line.strip() for line in native.splitlines()
                    if line.strip().startswith('python3 tools/probe_mail_inbox.py ')]
        self.assertEqual(len(commands), 2, 'native-only and network inbox PTYs must both execute')
        self.assertNotIn('--online', commands[0])
        self.assertIn('--online', commands[1])
        for command in commands:
            self.assertIn('--client "$CARGO_TARGET_DIR/release/faction-client"', command)
            self.assertIn('--fixture "$CARGO_TARGET_DIR/release/examples/checkpoint_ui_fixture"', command)
            self.assertIn('--owned-root "$RUNNER_TEMP/inbox-owned"', command)
        self.assertLess(native.index(commands[0]), native.index('--features faction-client/network-mail'))
        self.assertGreater(native.index(commands[1]), native.index('--features network-mail --bin faction-client'))

    def test_saved_inbox_evidence_guard_is_required(self):
        workflow = (ROOT / '.github/workflows/documentation.yml').read_text()
        documentation = workflow.split('  documentation:', 1)[1]
        self.assertIn('python3 tools/check_mail_inbox_evidence.py', documentation)

    def test_inbox_verification_keeps_required_native_prerequisite_and_job_bounds(self):
        workflow = (ROOT / '.github/workflows/documentation.yml').read_text()
        documentation = workflow.split('  documentation:', 1)[1]
        self.assertIn('needs: [native_mail, provider, provider_operator, provider_client]', documentation)
        for variable in ['NATIVE_MAIL_RESULT','PROVIDER_RESULT','PROVIDER_OPERATOR_RESULT','PROVIDER_CLIENT_RESULT']:
            self.assertIn('test "$'+variable+'" = success',documentation)
        self.assertIn('test "$NATIVE_MAIL_RESULT" = success', documentation)
        self.assertEqual(workflow.count('timeout-minutes: 15'), 5)
        for job in ['native_mail','provider','provider_operator','provider_client','documentation']:
            header=workflow.split('  '+job+':',1)[1].split('    steps:',1)[0]
            self.assertIn('timeout-minutes: 15',header)
        self.assertNotIn('continue-on-error:', workflow)


if __name__ == '__main__':
    unittest.main()
