"""Resolve historical executed bytes without rewriting their recorded digests."""
import hashlib,json,re
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
MANIFEST_SHA256="cfbb0a840f3db6dc79954dfc49a80eb5f3a637560494e2eb4cfda7cb4ff717ac"

def manifest(root=ROOT):
    raw=(root/'dev-docs/evidence/manifest.json').read_bytes()
    assert hashlib.sha256(raw).hexdigest()==MANIFEST_SHA256,'frozen manifest changed'
    return json.loads(raw)

def key_manifest(root=ROOT):
    raw=(root/'dev-docs/evidence/key-lookup-manifest.json').read_bytes()
    assert hashlib.sha256(raw).hexdigest()=='db2914814764384ffd16d04dfce5de16a8a769c67fa494bb085b9a113bc81176','frozen key manifest changed'
    return json.loads(raw)

def global_manifest(root=ROOT):
    raw=(root/'dev-docs/evidence/global-admission-manifest.json').read_bytes()
    assert hashlib.sha256(raw).hexdigest()=='c578e074722b97c2fbbbeb543b0572199da52933190884f871ef4cbc22939e51','frozen global manifest changed'
    return json.loads(raw)

def source_bytes(name,expected,root=ROOT):
    assert re.fullmatch('[a-f0-9]{64}',expected)
    path=Path(name);assert not path.is_absolute() and '..' not in path.parts
    m=manifest(root)
    candidates=[m]+m.get('snapshots',[])+[key_manifest(root),global_manifest(root)]
    matches=[snapshot for snapshot in candidates if snapshot['files'].get(name)==expected]
    if matches:
        target=root/'dev-docs/evidence'/matches[0]['source_root']/name
    else:target=root/name
    assert target.resolve().is_relative_to(root.resolve()),'source path escaped repository'
    raw=target.read_bytes()
    assert hashlib.sha256(raw).hexdigest()==expected,'executed source bytes changed: '+name
    return raw
