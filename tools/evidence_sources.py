"""Resolve historical executed bytes without rewriting their recorded digests."""
import hashlib,json,re
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
MANIFEST_SHA256="023fcfd2575b82251c42bb181746e798ec332ecac36d61c103b36d3634af05b6"

def manifest(root=ROOT):
    raw=(root/'dev-docs/evidence/manifest.json').read_bytes()
    assert hashlib.sha256(raw).hexdigest()==MANIFEST_SHA256,'frozen manifest changed'
    return json.loads(raw)

def source_bytes(name,expected,root=ROOT):
    assert re.fullmatch('[a-f0-9]{64}',expected)
    path=Path(name);assert not path.is_absolute() and '..' not in path.parts
    m=manifest(root)
    if m['files'].get(name)==expected:
        target=root/'dev-docs/evidence'/m['source_root']/name
    else:target=root/name
    assert target.resolve().is_relative_to(root.resolve()),'source path escaped repository'
    raw=target.read_bytes()
    assert hashlib.sha256(raw).hexdigest()==expected,'executed source bytes changed: '+name
    return raw
