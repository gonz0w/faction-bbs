import unittest
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
class CheckpointUiPolicy(unittest.TestCase):
 def test_required_native_job_builds_and_executes_real_ui_pty_controls(self):
  text=(ROOT/'.github/workflows/documentation.yml').read_text()
  native=text.split('  native_mail:\n',1)[1].split('  documentation:\n',1)[0]
  self.assertIn('cargo build -p faction-client --features native-mail --bin faction-client --example checkpoint_ui_fixture --release --locked',native)
  self.assertIn('python3 tools/probe_checkpoint_ui.py --client "$CARGO_TARGET_DIR/release/faction-client" --fixture "$CARGO_TARGET_DIR/release/examples/checkpoint_ui_fixture"',native)
if __name__=='__main__':unittest.main()
