import tempfile,unittest,tarfile,io,hashlib
from pathlib import Path
import check_xmpp_transport_dependencies as policy
ROOT=Path(__file__).resolve().parents[1]
class TransportPolicy(unittest.TestCase):
 def test_source_and_license_policy_refuses_javascript_and_missing_terms(self):
  with tempfile.TemporaryDirectory() as tmp:
   d=Path(tmp);(d/'Cargo.toml').write_text('');p=dict(name='synthetic',version='1',license='MIT',source='registry+https://github.com/rust-lang/crates.io-index',manifest_path=str(d/'Cargo.toml'))
   policy.scan_package(p,[], 'a'*64)
   (d/'hidden.mjs').write_text('')
   with self.assertRaises((AssertionError,SystemExit)):policy.scan_package(p,[],'a'*64)
   (d/'hidden.mjs').unlink();p['license']=None
   with self.assertRaises((AssertionError,SystemExit)):policy.scan_package(p,[],'a'*64)
 def test_compiled_source_must_match_the_locked_crate_archive(self):
  with tempfile.TemporaryDirectory() as tmp:
   d=Path(tmp);root=d/'source';root.mkdir();(root/'lib.rs').write_bytes(b'executed')
   archive=d/'crate.tar.gz'
   with tarfile.open(archive,'w:gz') as t:
    info=tarfile.TarInfo('synthetic-1/lib.rs');info.size=8;t.addfile(info,io.BytesIO(b'executed'))
   digest=hashlib.sha256(archive.read_bytes()).hexdigest()
   policy.verify_archive_source(root,archive,digest)
   (root/'lib.rs').write_bytes(b'tampered')
   with self.assertRaises(AssertionError):policy.verify_archive_source(root,archive,digest)
   (root/'lib.rs').write_bytes(b'executed')
   with self.assertRaises(AssertionError):policy.verify_archive_source(root,archive,'a'*64)
 def test_local_ci_runs_the_standalone_locked_probe_and_actual_fixture(self):
  s=(ROOT/'.github/workflows/documentation.yml').read_text()
  self.assertIn('python3 tools/check_xmpp_transport_dependencies.py',s)
  self.assertIn('cargo test --manifest-path experiments/208-xmpp-transport/Cargo.toml --all-targets --release --locked',s)
  self.assertIn('python3 tools/probe_xmpp_transport.py --binary "$CARGO_TARGET_DIR/release/faction-xmpp-probe"',s)
 def test_unselected_upstream_ci_is_never_imported(self):
  for p in (ROOT/'.github/workflows').glob('*.yml'):
   self.assertNotIn('uses:',p.read_text())
if __name__=='__main__':unittest.main()
