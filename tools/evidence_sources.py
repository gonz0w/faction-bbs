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

def file_baseline_manifest(root=ROOT):
    raw=(root/'dev-docs/evidence/file-areas-baseline-manifest.json').read_bytes()
    assert hashlib.sha256(raw).hexdigest()=='04e2da05bdb34f926bb7e85d181973bffde5bed1e0f8874f7492fabc7e57cfb5','frozen file baseline changed'
    return json.loads(raw)

def native_client_baseline_manifest(root=ROOT):
    raw=(root/'dev-docs/evidence/native-client-baseline-manifest.json').read_bytes()
    assert hashlib.sha256(raw).hexdigest()=='8139ae0981e9418ef9f10a6877aa051ecffad825fc9b9e3da56ee1caaebdc231','frozen native client baseline changed'
    return json.loads(raw)


def mail_client_baseline_manifest(root=ROOT):
    raw=(root/'dev-docs/evidence/mail-client-baseline-manifest.json').read_bytes()
    assert hashlib.sha256(raw).hexdigest()=='54b81f2454c64e4cb598bd55d8b8a535fd31f3bded266b15ed8c99346bd8c0b9','frozen mail client baseline changed'
    return json.loads(raw)


def room_reader_baseline_manifest(root=ROOT):
    raw=(root/'dev-docs/evidence/room-reader-baseline-manifest.json').read_bytes()
    assert hashlib.sha256(raw).hexdigest()=='ba5a596d9b5527ad171d6ea2f3d70f0ecda6d3490de9e04c902ef39489a80d65','frozen room reader baseline changed'
    return json.loads(raw)


def post_metadata_baseline_manifest(root=ROOT):
    raw=(root/'dev-docs/evidence/post-metadata-baseline-manifest.json').read_bytes()
    assert hashlib.sha256(raw).hexdigest()=='ed55e477c95b313004aebe7f7ab00aa16d9640bdc20f60a1490b6b011182cd57','frozen post metadata baseline changed'
    return json.loads(raw)


def authored_articles_baseline_manifest(root=ROOT):
    raw=(root/'dev-docs/evidence/authored-articles-baseline-manifest.json').read_bytes()
    assert hashlib.sha256(raw).hexdigest()=='4cc8ba0773e4675f875ae1f53c3043da8ee6a89ef11a140de99f646a2ad27c88','frozen authored article baseline changed'
    return json.loads(raw)


def broader_articles_baseline_manifest(root=ROOT):
    raw=(root/'dev-docs/evidence/broader-articles-baseline-manifest.json').read_bytes()
    assert hashlib.sha256(raw).hexdigest()=='7ad3539aa44efb95564aa342adeb7fce0e3dd2ed0f9ec573a14be740706fa6fa','frozen broader article baseline changed'
    return json.loads(raw)


def unicode_subjects_baseline_manifest(root=ROOT):
    raw=(root/'dev-docs/evidence/unicode-subjects-baseline-manifest.json').read_bytes()
    assert hashlib.sha256(raw).hexdigest()=='ef31461f4405196c3b1b24b2a7a4abe80353ab652ed14ef1426e043daa61798d','frozen Unicode subject baseline changed'
    return json.loads(raw)


def modern_terminals_baseline_manifest(root=ROOT):
    raw=(root/"dev-docs/evidence/modern-terminals-baseline-manifest.json").read_bytes()
    assert hashlib.sha256(raw).hexdigest()=="8eda37a79f543ac2c35f9bbe0159d14bcbbecec8f3377dabe04c81bc84ebb79b","frozen modern terminals baseline changed"
    return json.loads(raw)


def source_bytes(name,expected,root=ROOT):
    assert re.fullmatch('[a-f0-9]{64}',expected)
    path=Path(name);assert not path.is_absolute() and '..' not in path.parts
    m=manifest(root)
    candidates=[m]+m.get('snapshots',[])+[key_manifest(root),global_manifest(root),file_baseline_manifest(root),native_client_baseline_manifest(root),mail_client_baseline_manifest(root),room_reader_baseline_manifest(root),post_metadata_baseline_manifest(root),authored_articles_baseline_manifest(root),broader_articles_baseline_manifest(root),unicode_subjects_baseline_manifest(root),modern_terminals_baseline_manifest(root)]
    matches=[snapshot for snapshot in candidates if snapshot['files'].get(name)==expected]
    if matches:
        target=root/'dev-docs/evidence'/matches[0]['source_root']/name
    else:target=root/name
    assert target.resolve().is_relative_to(root.resolve()),'source path escaped repository'
    raw=target.read_bytes()
    assert hashlib.sha256(raw).hexdigest()==expected,'executed source bytes changed: '+name
    return raw
