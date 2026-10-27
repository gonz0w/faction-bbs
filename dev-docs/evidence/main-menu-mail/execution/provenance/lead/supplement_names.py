from pathlib import Path
import subprocess,json
r=Path('/Users/cam/DEV/faction-bbs');o=Path('/private/tmp/faction230-lead-OWNED');n=Path('/private/tmp/faction230-native-OWNED')
g=r/'tools/check_menu_mail_evidence.py';t=r/'tools/test_menu_mail_evidence.py'
for name,path in [('names-guard-before.py',g),('names-test-before.py',t)]: (o/name).write_bytes(path.read_bytes())
s=t.read_text().replace('    def test_accepts_complete_bounded_observations(self):', '''    def test_rejects_substituted_supplemental_control(self):
        record=copy.deepcopy(self.record);aggregate=json.loads((self.root/record['records']['protocol']['path']).read_bytes());ref=aggregate['routes']['host']['protocol'];path=self.root/ref['path'];data=json.loads(path.read_bytes());data['checks'][0]='unrelated';payload=json.dumps(data).encode();path.write_bytes(payload)
        newsha=hashlib.sha256(payload).hexdigest();ref['sha256']=newsha
        for item in record['sources']:
            if item['path']==str(path.relative_to(self.root)):item['sha256']=newsha
        newaggregate=self.save('mutated-protocol.json',aggregate);record['records']['protocol']=newaggregate
        with self.assertRaises(AssertionError):self.module.validate(self.root,record)

    def test_accepts_complete_bounded_observations(self):''')
t.write_text(s);(o/'names-red-test.py').write_bytes(t.read_bytes())
p=subprocess.run(['python3','-m','unittest','discover','-s','tools','-p','test_menu_mail_evidence.py'],cwd=r,stdout=subprocess.PIPE,stderr=subprocess.STDOUT);(o/'names-red.log').write_bytes(p.stdout);assert p.returncode!=0
paths={'protocol':'host-negative-final02/protocol.json','hostile':'host-negative-final02/hostile.json','compact':'host-main-final02/classic-compact.json','fallback':'host-standalone-final01/result.json','classic':'host-files-final01/classic.json','unicode':'host-files-final01/unicode.json','legacy':'host-files-final01/legacy.json'}
values={key:json.loads((n/path).read_bytes()) for key,path in paths.items()}
constant='EXPECTED_CONTROLS='+repr({key:set(value['checks']) for key,value in values.items()})+'\nEXPECTED_SCOPES='+repr({key:value['scope'] for key,value in values.items()})+'\n'
s=g.read_text().replace('def supplemental(root,records):',constant+'def supplemental(root,records):').replace("    if scope:assert data['scope']==scope", "    assert set(data['checks'])==EXPECTED_CONTROLS[name] and data['scope']==EXPECTED_SCOPES[name]")
g.write_text(s)
s=t.read_text().replace("checks=[str(x) for x in range(count)],scope=scope", "checks=sorted(self.module.EXPECTED_CONTROLS[name]),scope=self.module.EXPECTED_SCOPES[name]")
t.write_text(s)
p2=subprocess.run(['python3','-m','unittest','discover','-s','tools','-p','test_menu_mail_evidence.py'],cwd=r,stdout=subprocess.PIPE,stderr=subprocess.STDOUT);(o/'names-green.log').write_bytes(p2.stdout);print(p2.stdout.decode());assert p2.returncode==0
for name,path in [('names-green-test.py',t),('names-green-guard.py',g)]: (o/name).write_bytes(path.read_bytes())
(o/'names-binding.json').write_text(json.dumps(dict(red_exit=p.returncode,green_exit=p2.returncode,captured_before_edit=True),indent=2)+'\n')
