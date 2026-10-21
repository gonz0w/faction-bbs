#!/usr/bin/env python3
"""Verify all frozen source bytes and historical result identities."""
import hashlib
from evidence_sources import ROOT,manifest,source_bytes,key_manifest,global_manifest
def check():
    data=manifest();snapshots=[data]+data.get('snapshots',[])
    assert data['schema']==1 and data['origin_commit']=='09b506ea4f6da2830a0f2f480918ffb5048749aa'
    assert len(snapshots)==2 and snapshots[1]['origin_commit']=='627a8241bf736db2371938641d8bef88785d374a'
    count=0
    for snapshot in snapshots:
        base=ROOT/'dev-docs/evidence'/snapshot['source_root']
        assert {str(p.relative_to(base)) for p in base.rglob('*') if p.is_file()}==set(snapshot['files'])
        for name,digest in snapshot['files'].items():
            assert hashlib.sha256((base/name).read_bytes()).hexdigest()==digest
            assert hashlib.sha256(source_bytes(name,digest)).hexdigest()==digest
        for name,digest in snapshot['records'].items():
            assert hashlib.sha256((ROOT/name).read_bytes()).hexdigest()==digest,'historical record changed: '+name
            count+=1
    assert count==9
    extra=key_manifest()
    assert extra['origin_commit']=='71491cc61bf4a6606ec65bb73ad72ed585f18927'
    assert len(extra['files'])==137 and len(extra['records'])==2
    base=ROOT/'dev-docs/evidence'/extra['source_root']
    assert {str(p.relative_to(base)) for p in base.rglob('*') if p.is_file()}==set(extra['files'])
    for name,digest in extra['files'].items():
        assert hashlib.sha256((base/name).read_bytes()).hexdigest()==digest
        assert hashlib.sha256(source_bytes(name,digest)).hexdigest()==digest
    for name,digest in extra['records'].items():
        assert hashlib.sha256((ROOT/name).read_bytes()).hexdigest()==digest,'historical record changed: '+name
    extra=global_manifest()
    assert extra['origin_commit']=='5b62d0a705c988e47ec2cc2ce4b74f87de600386' and len(extra['files'])==145 and len(extra['records'])==1
    base=ROOT/'dev-docs/evidence'/extra['source_root']
    assert {str(p.relative_to(base)) for p in base.rglob('*') if p.is_file()}==set(extra['files'])
    for name,digest in extra['files'].items():
        assert hashlib.sha256((base/name).read_bytes()).hexdigest()==digest
        assert hashlib.sha256(source_bytes(name,digest)).hexdigest()==digest
    for name,digest in extra['records'].items():assert hashlib.sha256((ROOT/name).read_bytes()).hexdigest()==digest
    print('Four executed source snapshots and twelve unchanged historical records passed.')
if __name__=='__main__':check()
