#!/usr/bin/env python3
"""Verify all frozen source bytes and historical result identities."""
import hashlib
from evidence_sources import ROOT,manifest,source_bytes
def check():
    data=manifest();base=ROOT/'dev-docs/evidence'/data['source_root']
    assert data['schema']==1 and data['origin_commit']=='09b506ea4f6da2830a0f2f480918ffb5048749aa'
    assert {str(p.relative_to(base)) for p in base.rglob('*') if p.is_file()}==set(data['files'])
    for name,digest in data['files'].items():assert hashlib.sha256(source_bytes(name,digest)).hexdigest()==digest
    for name,digest in data['records'].items():assert hashlib.sha256((ROOT/name).read_bytes()).hexdigest()==digest,'historical record changed: '+name
    print('Frozen sources and eight historical records passed; no recorded hashes refreshed.')
if __name__=='__main__':check()
