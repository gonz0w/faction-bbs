from pathlib import Path
import subprocess,json
r=Path('/Users/cam/DEV/faction-bbs');o=Path('/private/tmp/faction230-lead-OWNED')
g=r/'tools/check_menu_mail_evidence.py';t=r/'tools/test_menu_mail_evidence.py'
for n,p in [('platform-guard-before.py',g),('platform-test-before.py',t)]: (o/n).write_bytes(p.read_bytes())
s=t.read_text().replace("data=dict(result='pass',unicode=unicode,", "data=dict(result='pass',platform='Darwin' if route=='host' else 'NetBSD',scope='main-menu-local-mail-files-disabled-same-ssh',unicode=unicode,")
s=s.replace("    def test_accepts_complete_bounded_observations(self):", "    def test_rejects_host_results_reused_as_native(self):\n        record=copy.deepcopy(self.record);record['routes']['netbsd']=copy.deepcopy(record['routes']['host'])\n        with self.assertRaises(AssertionError):self.module.validate(self.root,record)\n\n    def test_accepts_complete_bounded_observations(self):")
t.write_text(s);(o/'platform-red-test.py').write_bytes(t.read_bytes())
p=subprocess.run(['python3','-m','unittest','discover','-s','tools','-p','test_menu_mail_evidence.py'],cwd=r,stdout=subprocess.PIPE,stderr=subprocess.STDOUT);(o/'platform-red.log').write_bytes(p.stdout);assert p.returncode!=0
s=g.read_text().replace('def route(data,unicode):','def route(data,unicode,platform):\n assert data[\'platform\']==platform and data[\'scope\']==\'main-menu-local-mail-files-disabled-same-ssh\'')
s=s.replace("route(json.loads(raw(root,record['routes'][platform][name])),unicode)","route(json.loads(raw(root,record['routes'][platform][name])),unicode,'Darwin' if platform=='host' else 'NetBSD')")
g.write_text(s)
p2=subprocess.run(['python3','-m','unittest','discover','-s','tools','-p','test_menu_mail_evidence.py'],cwd=r,stdout=subprocess.PIPE,stderr=subprocess.STDOUT);(o/'platform-green.log').write_bytes(p2.stdout);assert p2.returncode==0
for n,p in [('platform-green-test.py',t),('platform-green-guard.py',g)]: (o/n).write_bytes(p.read_bytes())
(o/'platform-binding.json').write_text(json.dumps(dict(red_exit=p.returncode,green_exit=p2.returncode,captured_before_edit=True),indent=2)+'\n');print(p2.stdout.decode())
