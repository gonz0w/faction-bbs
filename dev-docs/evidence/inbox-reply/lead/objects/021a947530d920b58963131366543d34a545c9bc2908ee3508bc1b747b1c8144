#!/usr/bin/env python3
"""Review the standalone research graph; never adopt upstream CI or product deps."""
import argparse,hashlib,json,subprocess,tomllib,tarfile
from pathlib import Path
from check_rust_dependencies import SUFFIXES,MANIFESTS
ROOT=Path(__file__).resolve().parents[1]
MANIFEST=ROOT/'experiments/208-xmpp-transport/Cargo.toml'
INVENTORY=ROOT/'dev-docs/dependencies/xmpp-transport-research.json'
TARGETS=['x86_64-unknown-linux-gnu','aarch64-unknown-netbsd','aarch64-apple-darwin']
def scan_package(p,features,checksum):
 root=Path(p['manifest_path']).parent
 assert p.get('license') and p.get('source')=='registry+https://github.com/rust-lang/crates.io-index','unreviewed source/license'
 for f in root.rglob('*'):
  assert not f.is_symlink(),'dependency symlink'
  assert not (f.is_file() and (f.suffix.lower() in SUFFIXES or f.name in MANIFESTS)) and f.name!='node_modules','dependency violates no-JS source policy'
 return dict(name=p['name'],version=p['version'],license=p['license'],checksum=checksum,features=sorted(features))
def verify_archive_source(root,archive,checksum):
 assert hashlib.sha256(archive.read_bytes()).hexdigest()==checksum,'locked archive checksum changed'
 expected=set()
 with tarfile.open(archive) as t:
  for member in t.getmembers():
   if member.isdir():continue
   assert member.isfile(),'non-regular registry source'
   parts=Path(member.name).parts
   assert len(parts)>1 and '..' not in parts and not Path(member.name).is_absolute()
   rel=Path(*parts[1:]);expected.add(rel.as_posix())
   assert (root/rel).is_file() and not (root/rel).is_symlink()
   assert hashlib.sha256((root/rel).read_bytes()).digest()==hashlib.sha256(t.extractfile(member).read()).digest(),'extracted registry source changed'
 actual={p.relative_to(root).as_posix() for p in root.rglob('*') if p.is_file() and p.name not in {'.cargo-ok','.cargo-checksum.json'}}
 assert actual==expected,'unreviewed registry source files'

def inventory():
 source=tomllib.loads(MANIFEST.read_text());dep=source['dependencies']
 assert source['package']['publish'] is False and 'workspace' in source and 'patch' not in source
 assert dep['tokio-xmpp']==dict(version='=6.0.0',**{'default-features':False,'features':['native-tls']})
 assert dep['native-tls']=='=0.2.18' and dep['rxml']['version']=='=0.14.0'
 assert {'max_level_off','release_max_level_off'}<=set(dep['log']['features'])
 lock=tomllib.loads(MANIFEST.with_name('Cargo.lock').read_text())
 checks={(p['name'],p['version']):p['checksum'] for p in lock['package'] if p.get('source')}
 graphs={};verified=set()
 for target in TARGETS:
  d=json.loads(subprocess.check_output(['cargo','metadata','--manifest-path',str(MANIFEST),'--locked','--format-version','1','--filter-platform',target],text=True))
  nodes={n['id']:n for n in d['resolve']['nodes']}
  packages=[p for p in d['packages'] if p['id'] in nodes and p['source']]
  for p in packages:
   if p['id'] not in verified:
    root=Path(p['manifest_path']).parent
    archive=root.parents[2]/'cache'/root.parent.name/(p['name']+'-'+p['version']+'.crate')
    verify_archive_source(root,archive,checks[(p['name'],p['version'])]);verified.add(p['id'])
  rows=[scan_package(p,nodes[p['id']]['features'],checks[(p['name'],p['version'])]) for p in packages]
  graphs[target]=sorted(rows,key=lambda p:(p['name'],p['version']))
 return json.dumps(dict(schema=1,scope='standalone-research-not-product',upstream_ci_selected=False,graphs=graphs),indent=2,sort_keys=True)+'\n'
if __name__=='__main__':
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--write',action='store_true');a=p.parse_args();actual=inventory()
 if a.write:INVENTORY.write_text(actual);print('Standalone transport inventory written; review exact lock, licenses and selected source/build graph.')
 else:
  assert INVENTORY.read_text()==actual,'standalone XMPP research graph changed'
  print('Pinned Linux/NetBSD/macOS research graph passes selected source/license/no-JS policy; upstream CI and product adoption excluded.')
