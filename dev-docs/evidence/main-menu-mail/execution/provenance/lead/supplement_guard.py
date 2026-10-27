from pathlib import Path
import subprocess,json
root=Path('/Users/cam/DEV/faction-bbs');own=Path('/private/tmp/faction230-lead-OWNED')
g=root/'tools/check_menu_mail_evidence.py';t=root/'tools/test_menu_mail_evidence.py'
for name,path in [('supplement-guard-before.py',g),('supplement-test-before.py',t)]: (own/name).write_bytes(path.read_bytes())
s=t.read_text().replace("    def test_accepts_complete_bounded_observations(self):", "    def test_rejects_empty_supplemental_pass_metadata(self):\n        for name in ('protocol','compatibility','legacy'):\n            record=copy.deepcopy(self.record);record['records'][name]=self.save('empty-'+name+'.json',dict(result='pass'))\n            with self.assertRaises((AssertionError,KeyError)):self.module.validate(self.root,record)\n\n    def test_accepts_complete_bounded_observations(self):")
# Positive records are extended only after actual empty-metadata RED.
t.write_text(s);(own/'supplement-red-test.py').write_bytes(t.read_bytes())
p=subprocess.run(['python3','-m','unittest','discover','-s','tools','-p','test_menu_mail_evidence.py'],cwd=root,stdout=subprocess.PIPE,stderr=subprocess.STDOUT);(own/'supplement-red.log').write_bytes(p.stdout);assert p.returncode!=0
helper='''def supplemental(root,records):
 specs={
  'protocol':{'protocol':(12,'actual-ssh-mail-protocol-and-compatibility-files-disabled'),'hostile':(3,None)},
  'compatibility':{'compact':(3,None),'fallback':(4,None)},
  'legacy':{'classic':(14,'same-ssh-parked-files-local-mail-restricted-sftp'),'unicode':(14,'same-ssh-parked-files-local-mail-restricted-sftp'),'legacy':(2,'actual-old-and-default-native-v1-files-rpc')},
 }
 for category,kinds in specs.items():
  routes=records[category]['routes'];assert set(routes)=={'host','netbsd'}
  for platform,entries in routes.items():
   assert set(entries)==set(kinds)
   for name,(count,scope) in kinds.items():
    data=json.loads(raw(root,entries[name]));assert data['result']=='pass'
    assert data['count']==len(data['checks'])==len(set(data['checks']))==count
    assert data['capacity_claim'] is False and data['physical_hardware'] is False
    if scope:assert data['scope']==scope
    if 'platform' in data:assert data['platform']==('Darwin' if platform=='host' else 'NetBSD')
    if name in ('classic','unicode'):assert data['unicode'] is (name=='unicode') and data['restricted_sftp'] is True
    if name=='protocol':assert data['files_disabled'] is True and data['no_private_unlock'] is True and data['normal_limits_unchanged'] is True
    if name=='hostile':assert data['private_store_unchanged'] is True and data['normal_output_cap']==32768
    if name=='compact':assert data['dumb_output_no_escape'] is True and data['compact_offer_preserved'] is True
    if name=='fallback':assert data['no_shared_handoff_claim'] is True
    if name=='legacy':assert data['private_mail_not_opened'] is True

'''
# Exact actual compact spelling will be checked before GREEN below.
import json as j
compact=j.loads(Path('/private/tmp/faction230-native-OWNED/host-main-final02/classic-compact.json').read_text());print('compact-fields',list(compact))
helper=helper.replace("data['dumb_output_no_escape']", "data['dumb_output_no_escape']" if 'dumb_output_no_escape' in compact else "data['dumb_output_no_escapes']" if 'dumb_output_no_escapes' in compact else "data['dumb_output_no_esc']")
s=g.read_text().replace('def validate(root,record):',helper+'def validate(root,record):').replace(" assert all(value['result']=='pass' for value in records.values())", " assert all(value['result']=='pass' for value in records.values())\n supplemental(root,records)")
g.write_text(s)
s=t.read_text();needle="        for name,value in values.items():self.record['records'][name]=save(name+'.json',value)"
addition='''        specs={'protocol':{'protocol':(12,'actual-ssh-mail-protocol-and-compatibility-files-disabled'),'hostile':(3,None)},'compatibility':{'compact':(3,None),'fallback':(4,None)},'legacy':{'classic':(14,'same-ssh-parked-files-local-mail-restricted-sftp'),'unicode':(14,'same-ssh-parked-files-local-mail-restricted-sftp'),'legacy':(2,'actual-old-and-default-native-v1-files-rpc')}}
        for category,kinds in specs.items():
            values[category]['routes']={}
            for platform in ('host','netbsd'):
                entries={};values[category]['routes'][platform]=entries
                for name,(count,scope) in kinds.items():
                    data=dict(result='pass',count=count,checks=[str(x) for x in range(count)],scope=scope,platform='Darwin' if platform=='host' else 'NetBSD',capacity_claim=False,physical_hardware=False,unicode=name=='unicode',restricted_sftp=True,files_disabled=True,no_private_unlock=True,normal_limits_unchanged=True,private_store_unchanged=True,normal_output_cap=32768,dumb_output_no_esc=True,compact_offer_preserved=True,no_shared_handoff_claim=True,private_mail_not_opened=True)
                    entries[name]=save(category+'-'+platform+'-'+name+'.json',data)
'''
s=s.replace(needle,addition+needle);t.write_text(s)
p2=subprocess.run(['python3','-m','unittest','discover','-s','tools','-p','test_menu_mail_evidence.py'],cwd=root,stdout=subprocess.PIPE,stderr=subprocess.STDOUT);(own/'supplement-green.log').write_bytes(p2.stdout);print(p2.stdout.decode());assert p2.returncode==0
for name,path in [('supplement-green-guard.py',g),('supplement-green-test.py',t)]: (own/name).write_bytes(path.read_bytes())
(own/'supplement-binding.json').write_text(json.dumps(dict(red_exit=p.returncode,green_exit=p2.returncode,captured_before_edit=True),indent=2)+'\n')
