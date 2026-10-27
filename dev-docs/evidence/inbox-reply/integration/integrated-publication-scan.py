"""Final actual-private-value audit of the assembled issue233 public evidence."""
from pathlib import Path
import hashlib,json,re,stat
root=Path('/Users/cam/DEV/faction-bbs/dev-docs/evidence/inbox-reply');inventory=Path('/private/tmp/faction233-native-OWNED/private/generated-private-values.private.json');needles=[x.encode() for x in json.loads(inventory.read_text())];assert len(needles)==81 and root.is_dir() and not root.is_symlink()
files=sorted(path for path in root.rglob('*') if path.is_file());mapping={};matches=[]
def inspect(name,data):
 if any(value in data for value in needles) or re.search(rb'(?m)^-----BEGIN (?:OPENSSH |RSA |EC |ENCRYPTED )?PRIVATE KEY-----',data) or data.startswith((b'FACTVLT1',b'FACTVLT2')):matches.append(name)
for path in files:
 assert stat.S_ISREG(path.lstat().st_mode) and not path.is_symlink() and path.resolve().is_relative_to(root.resolve());data=path.read_bytes();relative=str(path.relative_to(root));inspect(relative,data);mapping[relative]=hashlib.sha256(data).hexdigest()
assert {str(p.relative_to(root)) for p in root.rglob('*') if p.is_file()}==set(mapping)
assert all(hashlib.sha256((root/name).read_bytes()).hexdigest()==digest for name,digest in mapping.items()),'assembled tree changed during audit'
extras={}
for path in [Path('/private/tmp/faction233_finalize.py'),Path(__file__)]:
 assert path.is_file() and not path.is_symlink();data=path.read_bytes();inspect(path.name,data);extras[path.name]=hashlib.sha256(data).hexdigest()
assert not matches,'generated private values or raw private artifact in assembled evidence/finalizer'
verdict={'result':'pass','generated_private_values_scanned':len(needles),'private_inventory_sha256':hashlib.sha256(inventory.read_bytes()).hexdigest(),'scanned_tree_files':len(mapping),'tree_filemap_sha256':hashlib.sha256(json.dumps(mapping,sort_keys=True,separators=(',',':')).encode()).hexdigest(),'tree_stable_during_scan':True,'private_matches':0,'raw_private_key_or_vault_matches':0,'extra_source_sha256':extras,'known_public_synthetic_fixture_literals_excluded':True}
raw=(json.dumps(verdict,indent=2)+'\n').encode();inspect('safe-verdict',raw);assert not matches;output=Path('/private/tmp/faction233-integrated-scan.json');assert not output.exists();output.write_bytes(raw);output.chmod(0o600)
print('Final assembled evidence and exact four-root finalizer passed81-private-value audit:',len(mapping),'stable files; zero matches.')
