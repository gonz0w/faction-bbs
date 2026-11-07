"""Issue #354: conserve the frozen validation inventory while moving research."""
import os
from pathlib import Path
import re
import subprocess
import unittest

import check_provider_dependencies as provider

ROOT = Path(__file__).resolve().parents[1]
RESEARCH_STEP = 'Validate the isolated transport and provider authority research'
SCANNER = 'python3 tools/check_xmpp_transport_dependencies.py'


def commands(job):
    return [line[10:] for block in re.findall(
        r'(?m)^        run: \|\n((?:          [^\n]*\n|\n)*)', job)
        for line in block.splitlines() if line.strip()]


MOVED_COMMANDS = [
    'cargo fmt --manifest-path experiments/208-xmpp-transport/Cargo.toml -- --check',
    'cargo clippy --manifest-path experiments/208-xmpp-transport/Cargo.toml --all-targets --release --locked -- -D warnings',
    'cargo test --manifest-path experiments/208-xmpp-transport/Cargo.toml --all-targets --release --locked',
    'cargo build --manifest-path experiments/208-xmpp-transport/Cargo.toml --release --locked',
    'python3 tools/probe_xmpp_transport.py --binary "$CARGO_TARGET_DIR/release/faction-xmpp-probe"',
    'python3 tools/check_provider_auth_dependencies.py',
    'cargo fmt --manifest-path experiments/270-provider-auth/Cargo.toml -- --check',
    'cargo clippy --manifest-path experiments/270-provider-auth/Cargo.toml --all-targets --release --locked -- -D warnings',
    'cargo test --manifest-path experiments/270-provider-auth/Cargo.toml --release --locked -- --test-threads=1',
]


class ResearchRoutingCI(unittest.TestCase):
    def setUp(self):
        self.source = (ROOT / '.github/workflows/documentation.yml').read_text()

    def test_exact_final_nine_commands_leave_native_for_existing_provider(self):
        native = commands(provider._job(self.source, 'native_mail'))
        receiving = commands(provider._job(self.source, 'provider'))
        self.assertEqual(receiving[-10:], [SCANNER] + MOVED_COMMANDS)
        for command in MOVED_COMMANDS:
            self.assertNotIn(command, native)
            self.assertEqual(self.source.splitlines().count('          ' + command), 1)

    def test_receiver_reuses_provider_setup_target_toolchain_and_two_build_jobs(self):
        job = provider._job(self.source, 'provider')
        self.assertEqual(re.findall(r'^      - name: (.*)$', job, re.M), [
            'Fetch the public repository revision',
            'Warm the selected native attachment source graphs',
            'Validate the opt-in attachment provider', RESEARCH_STEP])
        research = provider._step(job, RESEARCH_STEP)
        self.assertEqual(research, '        env:\n'
                         '          CARGO_TARGET_DIR: ${{ runner.temp }}/faction-provider\n'
                         '          CARGO_BUILD_JOBS: 2\n'
                         '          RUSTUP_TOOLCHAIN: 1.96.0\n'
                         '        run: |\n' + ''.join('          ' + line + '\n'
                             for line in [SCANNER] + MOVED_COMMANDS))
        # The original provider setup/qualification and validation finish first;
        # default sequential steps preserve failure propagation into research.
        before = job.split('      - name: ' + RESEARCH_STEP + '\n')[0]
        self.assertIn('      - name: Warm the selected native attachment source graphs', before)
        self.assertIn('      - name: Validate the opt-in attachment provider', before)
        self.assertIn('          ' + provider.TOOLCHAIN, before.splitlines())
        self.assertNotRegex(job, r'(?m)^    (?:if|continue-on-error|shell|strategy|needs|env):')

    def test_product_xmpp_and_native_source_scan_keep_their_original_prerequisites(self):
        native = commands(provider._job(self.source, 'native_mail'))
        scanner = native.index(SCANNER)
        product = next(i for i, line in enumerate(native)
                       if line.startswith('cargo fmt --manifest-path crates/faction-xmpp/'))
        self.assertLess(scanner, product)
        self.assertEqual(native[-1], 'python3 tools/probe_home_transport.py --binary '
                         '"$CARGO_TARGET_DIR/release/examples/transport_fixture"')
        research = commands(provider._step(provider._job(self.source, 'provider'), RESEARCH_STEP))
        self.assertEqual(research[0], SCANNER)
        self.assertLess(research.index('python3 tools/check_provider_auth_dependencies.py'),
                        next(i for i, line in enumerate(research)
                             if line.startswith('cargo fmt --manifest-path experiments/270-')))

    def test_existing_limits_edges_and_success_aggregation_remain_required(self):
        # This existing guard validates the provider inventory, job dependencies,
        # fixed 15-minute budgets and all required success-result bindings.
        provider.validate_ci(self.source)
        for name in ['native_mail', 'provider']:
            job = provider._job(self.source, name)
            self.assertIn('    timeout-minutes: 15', job.splitlines())
            self.assertIn('    runs-on: ubuntu-24.04', job.splitlines())

    def test_actual_aggregate_shell_rejects_failed_cancelled_or_skipped_receiver(self):
        doc = provider._job(self.source, 'documentation')
        self.assertIn('    if: ${{ always() }}', doc.splitlines())
        gate = provider._step(doc, 'Require optional native tests')
        self.assertIn('          PROVIDER_RESULT: ${{ needs.provider.result }}', gate.splitlines())
        script = '\n'.join(commands(gate))
        variables = re.findall(r'^          ([A-Z_]+_RESULT):', gate, re.M)
        for result in ['success', 'failure', 'cancelled', 'skipped']:
            env = dict(os.environ, **dict.fromkeys(variables, 'success'))
            env['PROVIDER_RESULT'] = result
            completed = subprocess.run(['/bin/sh', '-e', '-c', script], env=env,
                                       capture_output=True, text=True, timeout=5)
            with self.subTest(receiver=result):
                self.assertEqual(completed.returncode == 0, result == 'success')


if __name__ == '__main__':
    unittest.main()
