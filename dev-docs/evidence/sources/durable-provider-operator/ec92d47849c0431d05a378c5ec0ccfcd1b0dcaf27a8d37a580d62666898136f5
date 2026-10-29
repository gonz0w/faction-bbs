"""Keep the optional network path and its evidence inside required CI."""
from pathlib import Path
import unittest
ROOT=Path(__file__).resolve().parents[1]
class NetworkCI(unittest.TestCase):
    def test_cold_cache_downloads_only_locked_selected_graphs_before_verification(self):
        source=(ROOT/'.github/workflows/documentation.yml').read_text().split('  documentation:')[0]
        verify=source.index('python3 tools/check_network_mail_dependencies.py')
        for target in ['x86_64-unknown-linux-gnu','aarch64-unknown-netbsd','aarch64-apple-darwin']:
            command=f'cargo metadata --locked --format-version 1 --filter-platform {target} --features faction-client/network-mail > /dev/null'
            self.assertTrue(command in source,'selected locked graph must populate a cold cache')
            self.assertLess(source.index(command),verify)
        qualified=source.index('python3 tools/check_xmpp_transport_dependencies.py')
        compile=source.index('cargo clippy -p faction-client -p faction-omemo --features faction-client/network-mail')
        standalone=source.index('cargo clippy --manifest-path crates/faction-xmpp/Cargo.toml')
        self.assertLess(verify,compile);self.assertLess(qualified,standalone)
        self.assertNotIn('cargo fetch',source)
    def test_native_job_runs_product_network_tests_and_real_local_ui(self):
        source=(ROOT/'.github/workflows/documentation.yml').read_text().split('  documentation:')[0]
        self.assertIn('python3 tools/check_network_mail_dependencies.py',source)
        self.assertIn('--features faction-client/network-mail --all-targets --release --locked',source)
        self.assertIn('python3 tools/probe_home_ui.py',source)
        self.assertIn('--mail-fixture "$CARGO_TARGET_DIR/release/examples/home_mail_fixture"',source)
    def test_actual_transport_is_exercised_in_same_required_prerequisite(self):
        source=(ROOT/'.github/workflows/documentation.yml').read_text().split('  documentation:')[0]
        self.assertIn('cargo test --manifest-path crates/faction-xmpp/Cargo.toml --all-targets --release --locked',source)
        self.assertIn('python3 tools/probe_home_transport.py',source)
        self.assertIn('--binary "$CARGO_TARGET_DIR/release/examples/transport_fixture"',source)
    def test_saved_native_evidence_and_existing_failure_gate_are_required(self):
        source=(ROOT/'.github/workflows/documentation.yml').read_text()
        self.assertIn('python3 tools/check_home_mail_evidence.py',source)
        self.assertIn('needs: [native_mail, provider, provider_operator]',source)
        for variable in ['NATIVE_MAIL_RESULT','PROVIDER_RESULT','PROVIDER_OPERATOR_RESULT']:
            self.assertIn('test "$'+variable+'" = success',source)
        self.assertIn('if: ${{ always() }}',source)
        self.assertIn('test "$NATIVE_MAIL_RESULT" = success',source)
        self.assertIn('timeout-minutes: 15',source)
