import hashlib,json,shutil,tempfile,unittest,re,subprocess
from pathlib import Path
from check_omemo_dependencies import scan_package,check_vendor
ROOT=Path(__file__).resolve().parents[1]
class NativePolicy(unittest.TestCase):
 def test_native_ci_is_a_dependency_of_required_documentation_gate(self):
  text=(ROOT/'.github/workflows/documentation.yml').read_text()
  self.assertIn('needs: native_mail',text)
  self.assertIn('cargo test -p faction-client -p faction-omemo --features faction-client/native-mail',text)
  self.assertIn('python3 tools/check_omemo_dependencies.py',text)
 def test_native_failure_cancel_or_skip_cannot_skip_a_required_gate(self):
  text=(ROOT/'.github/workflows/documentation.yml').read_text()
  doc=text.split('  documentation:\n',1)[1]
  self.assertIn('if: ${{ always() }}',doc[:250])
  guard=re.search(r'(?ms)^      - name: Require optional native tests\n(.*?)(?=^      - name:)',doc)
  self.assertIsNotNone(guard)
  self.assertIn('NATIVE_MAIL_RESULT: ${{ needs.native_mail.result }}',guard.group(1))
  command=re.search(r'(?m)^        run: (.+)$',guard.group(1)).group(1)
  for result in ['success','failure','cancelled','skipped','']:
   code=subprocess.run(['/bin/sh','-c',command],env={'NATIVE_MAIL_RESULT':result},capture_output=True).returncode
   self.assertEqual(code==0,result=='success',result)
 def test_source_and_license_refuse_javascript_and_missing_terms(self):
  with tempfile.TemporaryDirectory(prefix='faction203-policy-') as tmp:
   path=Path(tmp);(path/'Cargo.toml').write_text('')
   p=dict(name='sample',version='1',license='MIT',manifest_path=str(path/'Cargo.toml'))
   self.assertEqual(scan_package(p,[])['license'],'MIT')
   (path/'hidden.js').write_text('')
   with self.assertRaises(SystemExit):scan_package(p,[])
   (path/'hidden.js').unlink();p['license']=None
   with self.assertRaises(SystemExit):scan_package(p,[])
 def test_vendored_source_substitution_rejects_independent_qualification_pins(self):
  check_vendor(ROOT)
  with tempfile.TemporaryDirectory(prefix='faction203-vendor-') as tmp:
   path=Path(tmp);dst=path/'crates/faction-omemo/vendor';dst.parent.mkdir(parents=True)
   shutil.copytree(ROOT/'crates/faction-omemo/vendor',dst)
   source=path/'dev-docs/evidence/omemo-interop';source.mkdir(parents=True)
   shutil.copy2(ROOT/'dev-docs/evidence/omemo-interop/results.json',source/'results.json')
   (dst/'gen/omemo2.c').write_bytes(b'substitution')
   with self.assertRaises(AssertionError):check_vendor(path)
if __name__=='__main__':unittest.main()
