import pathlib,subprocess,hashlib,json,sys,time,os
root=pathlib.Path('/Users/cam/DEV/faction-bbs'); out=pathlib.Path('/private/tmp/faction230-server-OWNED')
label=sys.argv[1]; dest=out/(time.strftime('%Y%m%dT%H%M%S')+'-'+label); dest.mkdir()
paths=subprocess.check_output(['git','ls-files','--cached','--others','--exclude-standard','-z'],cwd=root).decode().split('\0')
manifest={}
for s in paths:
 p=root/s
 if not s or any(x in p.parts for x in ('.git','target','.DS_Store')) or not p.is_file():continue
 if s.startswith(('crates/','Cargo.','rust-toolchain','tools/','.github/')):
  b=p.read_bytes(); manifest[s]=hashlib.sha256(b).hexdigest(); q=dest/'sources'/s; q.parent.mkdir(parents=True,exist_ok=True);q.write_bytes(b)
(dest/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
cmd=sys.argv[2:]; (dest/'command.json').write_text(json.dumps(cmd)+'\n')
print(dest,flush=True)
if cmd:
 with (dest/'output.log').open('wb') as f:
  r=subprocess.run(cmd,cwd=root,stdout=f,stderr=subprocess.STDOUT,env={**os.environ,'CARGO_BUILD_JOBS':'2'})
 after={}
 for s in subprocess.check_output(['git','ls-files','--cached','--others','--exclude-standard','-z'],cwd=root).decode().split('\0'):
  p=root/s
  if s and p.is_file() and not any(x in p.parts for x in ('.git','target','.DS_Store')) and s.startswith(('crates/','Cargo.','rust-toolchain','tools/','.github/')):
   b=p.read_bytes();after[s]=hashlib.sha256(b).hexdigest();q=dest/'sources-after'/s;q.parent.mkdir(parents=True,exist_ok=True);q.write_bytes(b)
 (dest/'manifest-after.json').write_text(json.dumps(after,indent=2)+'\n')
 (dest/'context-drift.json').write_text(json.dumps([s for s in manifest.keys() | after.keys() if manifest.get(s)!=after.get(s)],indent=2)+'\n')
 (dest/'status').write_text(str(r.returncode)+'\n');print((dest/'output.log').read_text(),end='');sys.exit(r.returncode)
