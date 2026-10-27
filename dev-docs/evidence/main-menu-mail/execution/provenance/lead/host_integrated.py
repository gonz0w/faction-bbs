from pathlib import Path
import hashlib,json,os,subprocess,time
R=Path('/Users/cam/DEV/faction-bbs');P=Path(__file__).resolve().parent;S=P/'host-source-objects';S.mkdir(exist_ok=True)
def snapshot(label):
 paths=subprocess.check_output(['git','ls-files','-co','--exclude-standard'],cwd=R,text=True).splitlines();files={}
 for name in sorted(set(paths)):
  path=R/name
  if path.is_file() and not path.is_symlink() and (name in {'Cargo.toml','Cargo.lock'} or name.startswith(('crates/','native/'))):
   if path.name=='.DS_Store':continue
   raw=path.read_bytes();digest=hashlib.sha256(raw).hexdigest();(S/digest).write_bytes(raw);files[name]=digest
 (P/label).write_text(json.dumps(files,indent=2,sort_keys=True)+'\n')
 return files
before=snapshot('host-integrated-before.json');env=os.environ.copy();env['CARGO_BUILD_JOBS']='2';checks=[]
for index,argv in enumerate([['cargo','fmt','--all','--','--check'],['cargo','clippy','--workspace','--all-targets','--release','--locked','--','-D','warnings'],['cargo','test','--workspace','--release','--locked','--','--test-threads=1']]):
 started=time.monotonic()
 with (P/f'host-integrated-{index}.log').open('wb') as log:result=subprocess.run(argv,cwd=R,env=env,stdout=log,stderr=log)
 checks.append(dict(command=argv,returncode=result.returncode,result='pass' if result.returncode==0 else 'fail',seconds=round(time.monotonic()-started,3)))
 (P/'host-integrated.json').write_text(json.dumps(dict(result='pass' if all(x['returncode']==0 for x in checks) else 'fail',checks=checks),indent=2)+'\n')
 if result.returncode:raise SystemExit(result.returncode)
after=snapshot('host-integrated-after.json');assert before==after,'compiled source context drifted';print('Host integrated default workspace fmt/clippy/tests passed; source context stable.')
