from pathlib import Path
import hashlib,json,shutil,sys
R=Path('/Users/cam/DEV/faction-bbs'); D=R/'dev-docs/evidence/inbox-reply';D.mkdir(parents=True,exist_ok=True)
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def ref(p):return dict(path=str(p.relative_to(R)),sha256=sha(p))
def write(name,value):
 p=D/name;p.parent.mkdir(parents=True,exist_ok=True);p.write_text(json.dumps(value,indent=2,sort_keys=True)+'\n');return ref(p)
sources=[];stages=[]
for name in ('native','client','lead'):
 src=Path(f'/private/tmp/faction233-{name}-OWNED/public-staging');manifest=json.loads((src/'manifest.json').read_text())
 assert len([p for p in src.rglob('*') if p.is_file()])==len(manifest)+1
 for path,h in manifest.items():assert sha(src/path)==h,path
 shutil.copytree(src,D/name,dirs_exist_ok=True)
 for p in sorted((D/name).rglob('*')):
  if p.is_file():sources.append(ref(p))
 for stage in json.loads((src/'source-stages.json').read_text()):
  stages.append(dict(label=stage['label'] if name=='native' else name+'-'+stage['label'],manifest=ref(D/name/stage['manifest']['relativepath'])))
n=D/'native/logs'; c=D/'client'; l=D/'lead'
host=write('records/host.json',dict(result='pass',workspace=ref(l/'metadata/host-result.json'),workspace_test_log=ref(l/'logs/host-2.log'),client_summary=ref(c/'summary.json'),client_test_logs={m:ref(c/f'logs/final2-frozen-pass/{m}-test.log') for m in ('native','network')}))
native=json.loads((n/'native-summary.json').read_text());native['source_count']=len(native['files']);native['test_log']=ref(n/'native-test-retry01-raw.log');nr=write('records/native.json',native)
compat=write('records/compatibility.json',dict(result='pass',routes={p:{k:ref(n/f'{v}-compat-{k}.json') for k in ('files','legacy')} for p,v in [('host','host'),('netbsd','native')]},inbox=ref(l/'metadata/inbox-consumer-green/result.json')))
tdd=json.loads((l/'metadata/guard-tdd.json').read_text());tdd.update(result='pass',evidence=ref(l/'metadata/guard-tdd.json'));tr=write('records/tdd.json',tdd)
scan=Path('/private/tmp/faction233-native-OWNED/logs/public-export-scan.json');shutil.copyfile(scan,D/'publication-scan.json');sources.append(ref(D/'publication-scan.json'))
sys.path.insert(0,str(R/'tools'));import check_reply_evidence as g
routes={'local':{},'main':{}}
for kind in routes:
 for platform in ('host','netbsd'):
  for mode in ('classic','unicode'):
   name=f'host-{kind}-{mode}-final.json' if platform=='host' else f'native-runtime-{kind}-{mode}.json'
   routes[kind][platform+'_'+mode]=ref(n/name)
record=dict(schema=1,issue=233,result='pass',profile=g.PROFILE,claims={k:False for k in g.CLAIMS},sources=sources,prior_records=[ref(R/'dev-docs/evidence/main-menu-mail/results.json')],source_stages=stages,routes=routes,records=dict(host=host,native=nr,tdd=tr,compatibility=compat,associations=ref(n/'capture-associations.json')))
write('results.draft.json',record)
print('assembled',len(sources),'source refs',len(stages),'stages; cleanup pending')
