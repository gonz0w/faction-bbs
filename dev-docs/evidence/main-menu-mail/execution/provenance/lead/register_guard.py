from pathlib import Path
import hashlib, subprocess, json
root=Path('/Users/cam/DEV/faction-bbs'); owned=Path('/private/tmp/faction230-lead-OWNED')
test=root/'tools/test_menu_mail_evidence.py'; workflow=root/'.github/workflows/documentation.yml'
for label,path in [('guard-test-before-registration',test),('workflow-before-registration',workflow)]:
 (owned/label).write_bytes(path.read_bytes())
s=test.read_text();s=s.replace('    def test_accepts_complete_bounded_observations(self):','    def test_ci_runs_the_immutable_record_guard(self):\n        workflow=Path(__file__).resolve().parents[1]/\'.github/workflows/documentation.yml\'\n        self.assertIn(\'python3 tools/check_menu_mail_evidence.py\',workflow.read_text())\n\n    def test_accepts_complete_bounded_observations(self):')
test.write_text(s)
(owned/'guard-registration-red-test.py').write_bytes(test.read_bytes())
p=subprocess.run(['python3','-m','unittest','discover','-s','tools','-p','test_menu_mail_evidence.py'],cwd=root,stdout=subprocess.PIPE,stderr=subprocess.STDOUT)
(owned/'guard-registration-red.log').write_bytes(p.stdout);assert p.returncode!=0
workflow.write_text(workflow.read_text().replace('          python3 tools/check_mail_inbox_evidence.py','          python3 tools/check_mail_inbox_evidence.py\n          python3 tools/check_menu_mail_evidence.py'))
p2=subprocess.run(['python3','-m','unittest','discover','-s','tools','-p','test_menu_mail_evidence.py'],cwd=root,stdout=subprocess.PIPE,stderr=subprocess.STDOUT)
(owned/'guard-registration-green.log').write_bytes(p2.stdout);assert p2.returncode==0
for label,path in [('guard-registration-green-test.py',test),('workflow-after-registration.yml',workflow)]: (owned/label).write_bytes(path.read_bytes())
(owned/'guard-registration.json').write_text(json.dumps(dict(red_exit=p.returncode,green_exit=p2.returncode,captured_before_edit=True),indent=2)+'\n')
print(p2.stdout.decode())
