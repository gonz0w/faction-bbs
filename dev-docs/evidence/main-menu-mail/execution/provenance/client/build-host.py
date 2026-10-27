from pathlib import Path
import hashlib,json,os,shutil,subprocess,sys,time
root=Path('/Users/cam/DEV/faction-bbs'); owned=Path('/private/tmp/faction230-client-OWNED'); mode=sys.argv[1];dest=owned/('host-'+mode+('-'+sys.argv[2] if len(sys.argv)>2 else ''));dest.mkdir(exist_ok=False)
def source_map():
 paths=[root/'Cargo.toml',root/'Cargo.lock']+[p for p in (root/'crates').rglob('*') if p.is_file() and not any(s in p.parts for s in ('.DS_Store','.git','target'))]
 return {str(p.relative_to(root)):hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(paths)}
before=source_map();(dest/'source-before.json').write_text(json.dumps(before,sort_keys=True,indent=2)+'\n')
cmd=['cargo','build','--release','-p','faction-client']
if mode=='network': cmd+=['--features','network-mail','--example','checkpoint_ui_fixture','--example','home_mail_fixture','--bin','faction-client']
else:cmd+=['--bin','faction-client']
record={'command':cmd,'cwd':str(root),'CARGO_BUILD_JOBS':'2','start_utc':time.strftime('%Y-%m-%dT%H:%M:%SZ',time.gmtime())}
with (dest/'build.log').open('w') as log: record['exit_code']=subprocess.call(cmd,cwd=root,env={**os.environ,'CARGO_BUILD_JOBS':'2'},stdout=log,stderr=subprocess.STDOUT)
record['end_utc']=time.strftime('%Y-%m-%dT%H:%M:%SZ',time.gmtime());after=source_map();(dest/'source-after.json').write_text(json.dumps(after,sort_keys=True,indent=2)+'\n');record['sources_unchanged']=before==after
if record['exit_code']==0:
 paths=['target/release/faction-client']
 if mode=='network':paths+=['target/release/examples/checkpoint_ui_fixture','target/release/examples/home_mail_fixture']
 record['binaries']={}
 for rel in paths:
  p=root/rel;q=dest/p.name;shutil.copyfile(p,q);q.chmod(0o700);record['binaries'][str(q)]=hashlib.sha256(q.read_bytes()).hexdigest()
record['source_manifest_sha256']=hashlib.sha256((dest/'source-before.json').read_bytes()).hexdigest();(dest/'build.json').write_text(json.dumps(record,sort_keys=True,indent=2)+'\n');print(json.dumps(record,indent=2),flush=True)
if record['exit_code'] or not record['sources_unchanged']:sys.exit(1)
