"""Audit frozen public candidates plus actual scoped teardown outputs."""
from pathlib import Path
import hashlib,json,re
P=Path('/private/tmp/faction233-native-OWNED');inventory=P/'private/generated-private-values.private.json';needles=[x.encode() for x in json.loads(inventory.read_text())];assert len(needles)==81
roots={'native':P/'public-staging','client':Path('/private/tmp/faction233-client-OWNED/public-staging'),'lead':Path('/private/tmp/faction233-lead-OWNED/public-staging'),'review':Path('/private/tmp/faction233-review-OWNED/public-staging')}
result={};matches=[]
def inspect(label,data):
 if any(value in data for value in needles) or re.search(rb'(?m)^-----BEGIN (?:OPENSSH |RSA |EC |ENCRYPTED )?PRIVATE KEY-----',data) or data.startswith((b'FACTVLT1',b'FACTVLT2')):matches.append(label)
for label,root in roots.items():
 assert root.is_dir() and not root.is_symlink();raw=(root/'manifest.json').read_bytes();manifest=json.loads(raw);actual={str(f.relative_to(root)) for f in root.rglob('*') if f.is_file()};assert actual==set(manifest)|{'manifest.json'}
 for relative in sorted(actual):
  path=root/relative;assert not path.is_symlink() and path.resolve().is_relative_to(root.resolve());data=path.read_bytes()
  if relative!='manifest.json':assert hashlib.sha256(data).hexdigest()==manifest[relative]
  inspect(label+'/'+relative,data)
 assert (root/'manifest.json').read_bytes()==raw
 result[label]={'files':len(actual),'manifest_sha256':hashlib.sha256(raw).hexdigest()}
assert result['review']['manifest_sha256']=='2f732605c609bd26107aa929e6645c01800f15e9e8fcf0ff7f22b19067949938'
extras={}
for path in [Path('/private/tmp/faction233-native-teardown-stop.py'),Path('/private/tmp/faction233-native-teardown-stop-source.json'),Path('/private/tmp/faction233-native-teardown-stop-result.json'),Path('/private/tmp/faction233-native-teardown-stop.log'),P/'logs/public-export-scan.json',Path(__file__)]:
 assert path.is_file() and not path.is_symlink();data=path.read_bytes();inspect(path.name,data);extras[path.name]=hashlib.sha256(data).hexdigest()
assert not matches,'actual private values or private artifact in selected publication'
stop=json.loads(Path('/private/tmp/faction233-native-teardown-stop-result.json').read_text());assert stop['result']=='pass' and stop['owned_vm_stopped'] and stop['owned_controller_stopped'] and stop['forwards_closed'] and stop['checked_saved_files']==6 and stop['native_root_retained'] and not stop['owned_removed']
verdict={'result':'pass','generated_private_values_scanned':len(needles),'private_inventory_sha256':hashlib.sha256(inventory.read_bytes()).hexdigest(),'private_matches':0,'raw_private_key_or_vault_matches':0,'explicit_manifests_verified':True,'bundles':result,'scanned_extra_sha256':extras,'native_root_and_inventory_retained_for_lead_copy':True,'known_public_synthetic_fixture_literals_excluded':True}
raw=(json.dumps(verdict,indent=2)+'\n').encode();inspect('scan-verdict',raw);assert not matches
output=Path('/private/tmp/faction233-native-final-publication-scan.json');assert not output.exists();output.write_bytes(raw);output.chmod(0o600)
print('Four frozen candidates plus actual teardown source/result/log passed81-generated-value audit; native root and inventory retained for lead copying.')
