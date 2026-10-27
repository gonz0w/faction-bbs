"""Audit explicit immutable public candidates against actual generated private values."""
from pathlib import Path
import hashlib,json,re,sys
P=Path('/private/tmp/faction235-native-OWNED');R=Path('/Users/cam/DEV/faction-bbs');inventory=P/'private/generated-private-values.private.json';needles=[v.encode() for v in json.loads(inventory.read_text())];assert needles
roots={'native':P/'public-staging','client':Path('/private/tmp/faction235-client-OWNED/public-staging'),'lead':Path('/private/tmp/faction235-lead-OWNED/public-staging'),'lead_final':Path('/private/tmp/faction235-lead-OWNED/public-final')};bundles={};matches=[];seen=set();reused={}
def digest(data):return hashlib.sha256(data).hexdigest()
def inspect(label,data):
 h=digest(data)
 if h in seen:return
 seen.add(h)
 if any(value in data for value in needles):matches.append((label,'generated-private-value'))
 if re.search(rb'(?m)^-----BEGIN (?:OPENSSH |RSA |EC |ENCRYPTED )?PRIVATE KEY-----',data) or data.startswith((b'FACTVLT1',b'FACTVLT2')):matches.append((label,'private-key-or-vault'))
# Deliberate poison control of the actual active inventory, without publishing it.
poison=next(value for value in needles if b'\n' not in value and not value.startswith(b'-----'));inspect('private-poison-control',poison);assert matches==[('private-poison-control','generated-private-value')];matches.clear();seen.clear()
for label,root in roots.items():
 manifest_name='package-manifest.json' if label=='client' else 'manifest.json'
 assert root.is_dir() and not root.is_symlink();raw=(root/manifest_name).read_bytes();manifest=json.loads(raw);actual={str(f.relative_to(root)) for f in root.rglob('*') if f.is_file()};assert actual==set(manifest)|{manifest_name}
 for name in sorted(actual):
  f=root/name;assert not f.is_symlink() and f.resolve().is_relative_to(root.resolve());data=f.read_bytes()
  if name!=manifest_name:assert digest(data)==manifest[name]
  inspect(label+'/'+name,data)
 if (root/'reuse.json').exists():
  for h,ref in json.loads((root/'reuse.json').read_text()).items():
   assert ref['sha256']==h;f=R/ref['repository_path'];assert f.is_file() and not f.is_symlink() and f.resolve().is_relative_to(R.resolve());data=f.read_bytes();assert digest(data)==h;inspect('reused/'+ref['repository_path'],data);reused[h]=ref['repository_path']
 assert (root/manifest_name).read_bytes()==raw
 bundles[label]=dict(files=len(actual),manifest_sha256=digest(raw))
review=Path('/private/tmp/faction235-review-OWNED/public-candidates.json');rows=json.loads(review.read_text());assert rows['result']=='pass' and len(rows['files'])==9
for ref in rows['files']:
 f=Path(ref['path']);assert f.is_file() and not f.is_symlink();data=f.read_bytes();assert digest(data)==ref['sha256'];inspect('review/'+f.name,data)
bundles['review']=dict(files=9,manifest_sha256=digest(review.read_bytes()))
extras={}
for f in [Path(__file__)]+([P/'owned_stop.py',P/'owned-stop-result.json',P/'owned-stop.log',P/'saved-identity-proof.json'] if '--with-cleanup' in sys.argv else []):
 data=f.read_bytes();inspect('extra/'+f.name,data);extras[f.name]=digest(data)
assert not matches, 'private fixture material found in explicit publication candidate: '+repr(matches)
if '--with-cleanup' in sys.argv:
 stop=json.loads((P/'owned-stop-result.json').read_text());assert stop['result']=='pass' and stop['owned_vm_stopped'] and stop['owned_controller_stopped'] and stop['forwards_closed'] and stop['checked_saved_files']==6 and stop['native_root_retained'] and not stop['owned_removed']
result=dict(result='pass',generated_private_values_scanned=len(needles),private_inventory_sha256=digest(inventory.read_bytes()),private_matches=0,raw_private_key_or_vault_matches=0,explicit_manifests_verified=True,bundles=bundles,reused_committed_source_objects_scanned=len(reused),scanned_extra_sha256=extras,known_public_synthetic_fixture_literals_excluded=True,private_poison_control_rejected=True)
output=P/('publication-scan-final.json' if '--with-cleanup' in sys.argv else 'publication-scan-initial.json');assert not output.exists();data=(json.dumps(result,indent=2)+'\n').encode();inspect('scan-verdict',data);assert not matches;output.write_bytes(data);output.chmod(0o600)
print('Explicit frozen candidates passed',len(needles),'actual generated-private-value scan; no private values printed.')
