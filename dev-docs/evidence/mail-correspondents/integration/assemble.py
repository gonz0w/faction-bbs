"""Assemble only frozen, scanned issue235 source and observation selections."""
from pathlib import Path
import hashlib,json,shutil,sys
R=Path('/Users/cam/DEV/faction-bbs');D=R/'dev-docs/evidence/mail-correspondents';D.mkdir(parents=True)
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def ref(p):return dict(path=str(p.relative_to(R)),sha256=sha(p))
def write(name,value):
 p=D/name;p.parent.mkdir(parents=True,exist_ok=True);p.write_text(json.dumps(value,indent=2,sort_keys=True)+'\n');return ref(p)
sources={};stages=[]
def add(p):
 v=ref(p);sources[v['path']]=v
for name,relative,manifest_name in [('native','faction235-native-OWNED/public-staging','manifest.json'),('client','faction235-client-OWNED/public-staging','package-manifest.json'),('lead','faction235-lead-OWNED/public-staging','manifest.json'),('lead-final','faction235-lead-OWNED/public-final','manifest.json')]:
 src=Path('/private/tmp')/relative;m=json.loads((src/manifest_name).read_text());assert len(m)+1==len([p for p in src.rglob('*') if p.is_file()])
 for p,h in m.items():assert sha(src/p)==h,p
 shutil.copytree(src,D/name)
 for p in (D/name).rglob('*'):
  if p.is_file():add(p)
 for h,v in json.loads((src/'reuse.json').read_text()).items():
  p=R/v['repository_path'];assert sha(p)==h==v['sha256'];add(p)
 if name=='client':
  for p in sorted((D/name/'source-maps').glob('*.json')):stages.append(dict(label='client-'+p.stem,manifest=ref(p)))
 else:
  for stage in json.loads((src/'source-stages.json').read_text()):
   p=D/name/stage['manifest']['relativepath'];assert sha(p)==stage['manifest']['sha256']
   stages.append(dict(label=stage['label'] if name=='native' else name+'-'+stage['label'],manifest=ref(p)))
# Explicit safe read-only review and post-stop/scanner observations.
for name,manifest in [('review',Path('/private/tmp/faction235-review-OWNED/public-candidates.json')),('integration',Path('/private/tmp/faction235-native-OWNED/public-extra-candidates.json'))]:
 folder=D/name;folder.mkdir()
 for item in json.loads(manifest.read_text())['files']:
  src=Path(item['path']);assert sha(src)==item['sha256'];target=folder/src.name;assert not target.exists();shutil.copyfile(src,target);add(target)
 shutil.copyfile(manifest,folder/'candidate-manifest.json');add(folder/'candidate-manifest.json')
# Freeze the exact assembly code, not a reconstructed earlier recipe.
shutil.copyfile('/private/tmp/faction235-lead-OWNED/assemble.py',D/'integration/assemble.py');add(D/'integration/assemble.py')
stages.append(dict(label='host-final',manifest=ref(D/'lead/maps/host-after.json')))
review_scripts={p.name:sha(p) for p in (D/'review').glob('*.py')};stages.append(dict(label='review-script-variants',manifest=write('records/review-sources.json',review_scripts)))
for name,h in review_scripts.items():assert h in {v['sha256'] for v in sources.values()}
n=D/'native/logs';c=D/'client';l=D/'lead'
routes={'local':{},'main':{}}
for kind in routes:
 for platform in ('host','netbsd'):
  for mode in ('classic','unicode'):
   path=(n/f'host-local-{mode}01.json') if platform=='host' and kind=='local' else (n/f'host-main/{mode}.json') if platform=='host' else (n/f'native-runtime/{kind}-{mode}.json')
   routes[kind][platform+'_'+mode]=ref(path)
# Check actual unchanged legacy/current compatibility observations before binding them.
sys.path.insert(0,str(R/'tools'));import check_reply_evidence as old
compat={}
for platform,prefix,reg in [('host','host','host-regressions02'),('netbsd','native','native-runtime')]:
 refs={key:ref(n/f'{prefix}-compatibility/{key}.json') for key in ('files','legacy')}
 refs['inbox']=ref(n/reg/'inbox.json')
 refs.update({f'reply_{mode}':ref(n/reg/f'reply-{mode}.json') for mode in ('classic','unicode')})
 for key,expected in [('files',old.FILES_CONTROLS),('legacy',old.LEGACY_CONTROLS),('inbox',old.INBOX_CONTROLS),('reply_classic',set(old.CONTROLS['local'])),('reply_unicode',set(old.CONTROLS['local']))]:
  value=json.loads((R/refs[key]['path']).read_text());assert value['result']=='pass' and value['count']==len(value['checks'])==len(expected) and set(value['checks'])==expected
  if key=='files':assert value['privacy_scan']['matches']==value['privacy_scan']['symlinks']==0 and value['same_ssh_pid_until_explicit_disconnect'] is True
  if key=='legacy':assert value['private_mail_not_opened'] is True
  if key=='inbox':assert value['provider_contact'] is False and value['read_receipt_sent'] is False and value['private_fixture_removed'] is True
  if key.startswith('reply_'):assert value['configured_home_endpoint_was_closed'] is True and value['home_password_prompt_observed'] is False and value['private_fixture_removed'] is True
 compat[platform]=refs
comp=write('records/compatibility.json',dict(result='pass',routes=compat))
rust=write('records/rust.json',dict(result='pass',runs=dict(workspace=dict(passed=473,ignored=0,log=ref(l/'logs/host-2.log')),network=dict(passed=144,ignored=5,log=ref(c/'logs/network-tests.log')),native=dict(passed=95,ignored=5,log=ref(c/'logs/native-tests.log')),netbsd=dict(passed=277,ignored=5,log=ref(n/'native-build/tests-build.log')))))
tdd=write('records/tdd.json',dict(result='pass',captured_before_edit=True,red_exit=101,green_exit=0,actual_ui_red=ref(c/'logs/actual-interface-red.log'),actual_ui_red_helper=ref(c/'logs/actual-interface-red-helper.log'),missing_model_red=ref(c/'logs/group-red.log'),corrected_guard_red=ref(D/'lead-final/guard-stages/guard-ack-red/test.log'),corrected_guard_green=ref(D/'lead-final/guard-stages/guard-ack-green/test.log')))
import check_correspondents_evidence as guard
production={name:sha(R/name) for name in ('crates/faction-client/src/checkpoint_ui.rs','crates/faction-client/src/checkpoint_ui_inbox.rs')}
record=dict(schema=1,issue=235,result='pass',profile=guard.PROFILE,claims={name:False for name in guard.CLAIMS},production=production,sources=sorted(sources.values(),key=lambda v:v['path']),prior_records=[ref(R/'dev-docs/evidence/inbox-reply/results.json')],source_stages=stages,routes=routes,records=dict(associations=ref(n/'capture-associations.json'),native=ref(n/'native-summary.json'),tdd=tdd,publication=ref(D/'integration/publication-scan-final.json'),rust=rust,compatibility=comp))
write('results.draft.json',record)
# Validate raw bindings and substantive routes while cleanup remains intentionally pending.
available={v['sha256'] for v in record['sources']}
for v in record['sources']+record['prior_records']:guard.raw(R,v)
for stage in stages:assert all(h in available for h in guard.value(R,stage['manifest']).values()),stage['label']
for kind,refs in routes.items():
 for key,r in refs.items():guard.route(R,r,kind,key.split('_')[0],key.endswith('_unicode'),available)
print(json.dumps(dict(result='pass',new_files=len([p for p in D.rglob('*') if p.is_file()]),source_refs=len(sources),source_stages=len(stages),cleanup_pending=True)))
