import hashlib,json,pathlib,shutil,sys
root=pathlib.Path('/Users/cam/DEV/faction-bbs'); dest=pathlib.Path('/private/tmp/faction230-client-OWNED')/sys.argv[1]
dest.mkdir(exist_ok=False)
files=[root/'Cargo.toml',root/'Cargo.lock']+[p for p in (root/'crates').rglob('*') if p.is_file() and not any(x in p.parts for x in ('.DS_Store','.git','target'))]
manifest={}
for p in files:
 rel=p.relative_to(root); q=dest/rel; q.parent.mkdir(parents=True,exist_ok=True); shutil.copyfile(p,q); manifest[str(rel)]=hashlib.sha256(q.read_bytes()).hexdigest()
(dest/'manifest.json').write_text(json.dumps(manifest,sort_keys=True,indent=2)+'\n')
(dest/'OWNED').write_text('Disposable issue 230 client exact executed sources; no private keys.\n')
print(str(dest),hashlib.sha256((dest/'manifest.json').read_bytes()).hexdigest())
