"""Read-only publication scan of integrated issue235 evidence and exact finalizer."""
from pathlib import Path
import hashlib,json,os,re,stat
R=Path('/Users/cam/DEV/faction-bbs');D=R/'dev-docs/evidence/mail-correspondents'
I=Path('/private/tmp/faction235-native-OWNED/private/generated-private-values.private.json')
F=Path('/private/tmp/faction235_finalize.py');S=Path(__file__).resolve();O=Path('/private/tmp/faction235-integrated-scan.json')
def sha(data):return hashlib.sha256(data).hexdigest()
info=I.lstat();assert stat.S_ISREG(info.st_mode) and info.st_uid==os.getuid() and stat.S_IMODE(info.st_mode)==0o600 and info.st_nlink==1
raw=I.read_bytes();assert sha(raw)=='77f4a4ed0b49e3d1705809374e6d620b2f6268d0cf06c5fb03ee612a71292d3e'
values=json.loads(raw);assert len(values)==len(set(values))==467 and all(isinstance(v,str) and v for v in values);needles=[v.encode() for v in values]
key=re.compile(rb'(?m)^-----BEGIN (?:OPENSSH |RSA |EC |ENCRYPTED )?PRIVATE KEY-----')
def findings(data):
 return dict(generated_private_value=any(n in data for n in needles),raw_private_key=bool(key.search(data)),raw_vault=data.startswith((b'FACTVLT1',b'FACTVLT2')))
# Exercise each rejection category with actual inventory material where relevant.
poison=next(n for n in needles if b'\n' not in n and not n.startswith(b'-----'))
assert findings(poison)==dict(generated_private_value=True,raw_private_key=False,raw_vault=False)
private_key=next(n for n in needles if key.search(n));assert findings(private_key)['generated_private_value'] and findings(private_key)['raw_private_key']
for magic in (b'FACTVLT1',b'FACTVLT2'):assert findings(magic+b' public poison control')['raw_vault']
assert findings(b'public safe control; no member data')==dict(generated_private_value=False,raw_private_key=False,raw_vault=False)
assert D.is_dir() and not D.is_symlink() and F.is_file() and not F.is_symlink() and S.is_file() and not S.is_symlink()
files={}
for p in sorted(D.rglob('*')):
 assert not p.is_symlink()
 if p.is_file():
  assert stat.S_ISREG(p.lstat().st_mode) and p.resolve().is_relative_to(D.resolve())
  data=p.read_bytes();assert not any(findings(data).values()),'private material in '+str(p.relative_to(D));files[str(p.relative_to(R))]=sha(data)
assert files
extra={}
for p in (F,S):
 data=p.read_bytes();assert not any(findings(data).values()),'private material in explicitly selected source';extra[p.name]=sha(data)
# A concurrent edit or file-set change cannot inherit this verdict.
assert {str(p.relative_to(R)):sha(p.read_bytes()) for p in D.rglob('*') if p.is_file()}==files
assert all(sha(p.read_bytes())==extra[p.name] for p in (F,S))
assert I.read_bytes()==raw and not O.exists()
result=dict(result='pass',scope='entire-integrated-issue235-evidence-and-exact-finalizer',generated_private_values_scanned=len(values),private_inventory_sha256=sha(raw),private_matches=0,raw_private_key_or_vault_matches=0,explicit_manifests_verified=True,integrated_files=len(files),integrated_file_sha256=files,selected_source_sha256=extra,source_and_file_set_unchanged=True,actual_generated_value_poison_rejected=True,actual_private_key_poison_rejected=True,both_vault_magic_poisons_rejected=True,known_public_synthetic_fixture_literals_excluded=True,owned_roots_deleted_by_this_scan=False)
data=(json.dumps(result,indent=2,sort_keys=True)+'\n').encode();assert not any(findings(data).values());O.write_bytes(data);O.chmod(0o600)
print('Integrated evidence and exact finalizer passed467actual-private-value scan;',len(files),'files; zero private/key/vault matches; all roots retained.')
