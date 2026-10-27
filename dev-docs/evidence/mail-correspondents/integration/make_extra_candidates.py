"""Explicit safe supplement selection after the immutable native bundle froze."""
from pathlib import Path
import hashlib,json,re
P=Path('/private/tmp/faction235-native-OWNED');scan=json.loads((P/'publication-scan-final.json').read_text());assert scan['result']=='pass';needles=[v.encode() for v in json.loads((P/'private/generated-private-values.private.json').read_text())]
files=list(scan['scanned_extra_sha256'])+['publication-scan-final.json','publication-scan-final.log',Path(__file__).name];rows=[]
def safe(data):
 assert not any(v in data for v in needles)
 assert not re.search(rb'(?m)^-----BEGIN (?:OPENSSH |RSA |EC |ENCRYPTED )?PRIVATE KEY-----',data)
 assert not data.startswith((b'FACTVLT1',b'FACTVLT2'))
for name in files:
 f=P/name;assert f.is_file() and not f.is_symlink();data=f.read_bytes();safe(data);h=hashlib.sha256(data).hexdigest()
 if name in scan['scanned_extra_sha256']:assert h==scan['scanned_extra_sha256'][name]
 if f.suffix in {'.json','.py'}:assert data.endswith(b'\n')
 rows.append(dict(path=str(f),sha256=h))
output=P/'public-extra-candidates.json';assert not output.exists();data=(json.dumps(dict(result='pass',files=rows,generated_private_values_scanned=len(needles),private_matches=0,raw_private_key_or_vault_matches=0,frozen_native_manifest_sha256=hashlib.sha256((P/'public-staging/manifest.json').read_bytes()).hexdigest(),native_root_retained=True),indent=2)+'\n').encode();safe(data);output.write_bytes(data)
print('Explicit16safe supplement files selected and scanned; frozen native bundle unchanged.')
