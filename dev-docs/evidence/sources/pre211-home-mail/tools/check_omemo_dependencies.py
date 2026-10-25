#!/usr/bin/env python3
"""Review the optional native mail graph and exact qualified C subset; no VM."""
import argparse,hashlib,json,subprocess
from pathlib import Path
from check_rust_dependencies import SUFFIXES,MANIFESTS,TARGETS
ROOT=Path(__file__).resolve().parents[1]
INVENTORY=ROOT/'dev-docs/dependencies/native-mail.json'
SOURCES=['gen/omemo2.c','gen/omemo2.h','hacl.c','openssl.c','driver.h','omemo.h']
def scan_package(p,features):
 root=Path(p['manifest_path']).parent
 forbidden=[f.relative_to(root).as_posix() for f in root.rglob('*') if (f.is_file() and (f.suffix.lower() in SUFFIXES or f.name in MANIFESTS)) or (f.is_dir() and f.name=='node_modules')]
 if forbidden or not p.get('license'):raise SystemExit('Native dependency source/license policy failed: '+p['name'])
 return dict(name=p['name'],version=p['version'],license=p['license'],features=sorted(features))
def check_vendor(root=ROOT):
 d=json.loads((root/'dev-docs/evidence/omemo-interop/results.json').read_text())
 assert d['candidate']['revision']=='835d8277108b96c691fb6a992512f37315cb52d7'
 expected=d['candidate']['source_hashes']['corrected']
 base=root/'crates/faction-omemo/vendor'
 actual={}
 for line in (base/'SHA256SUMS').read_text().splitlines():
  digest,name=line.split();assert name in SOURCES and name not in actual
  assert hashlib.sha256((base/name).read_bytes()).hexdigest()==digest==expected[name],name
  actual[name]=digest
 assert set(actual)==set(SOURCES)
 for name in ['LICENSE-ISC','LICENSE-HACL-MIT','LICENSE-HACL-APACHE']:
  assert (base/name).is_file() and (base/name).stat().st_size>500
 for f in base.rglob('*'):
  assert not f.is_symlink() and f.suffix.lower() not in SUFFIXES
 return actual

def inventory():
 graphs={}
 for target in TARGETS:
  d=json.loads(subprocess.check_output(['cargo','metadata','--locked','--format-version','1','--filter-platform',target,'--features','faction-client/native-mail','--manifest-path',str(ROOT/'Cargo.toml')],text=True))
  selected={n['id']:n['features'] for n in d['resolve']['nodes']}
  rows=[scan_package(p,selected[p['id']]) for p in d['packages'] if p['id'] in selected and p['source'] is not None]
  graphs[target]=sorted(rows,key=lambda p:(p['name'],p['version']))
 return json.dumps(graphs,indent=2,sort_keys=True)+'\n'
if __name__=='__main__':
 parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--write',action='store_true');a=parser.parse_args()
 check_vendor();actual=inventory()
 if a.write:INVENTORY.write_text(actual);print('Optional native inventory written; review source/license/feature diff.')
 elif not INVENTORY.is_file() or actual!=INVENTORY.read_text():raise SystemExit('Optional native graph changed; review and update its inventory intentionally.')
 else:print('Optional Linux/NetBSD native graph and exact qualified C subset verified; no fresh VM/crypto execution or audit claim.')
