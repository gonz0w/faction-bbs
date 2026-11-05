"""Keep the real foreground-exchange PTY consumer in the existing mail job."""
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[1]


class InboxExchangePolicy(unittest.TestCase):
    def test_network_mail_job_executes_the_production_exchange_probe(self):
        workflow = (ROOT / '.github/workflows/documentation.yml').read_text()
        job = workflow.split('  native_mail:\n', 1)[1].split('  provider:\n', 1)[0]
        command = ('python3 tools/probe_inbox_exchange.py '
                   '--client "$CARGO_TARGET_DIR/release/faction-client" '
                   '--fixture "$CARGO_TARGET_DIR/release/examples/checkpoint_ui_fixture" '
                   '--owned-root "$RUNNER_TEMP/inbox-owned" --name exchange-network')
        self.assertIn(command, job)
        self.assertLess(job.index('--features network-mail --bin faction-client'),
                        job.index(command))
        self.assertLess(job.index('"$RUNNER_TEMP/inbox-owned/OWNERSHIP"'),
                        job.index(command))


if __name__ == '__main__':
    unittest.main()
