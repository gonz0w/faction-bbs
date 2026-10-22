#!/usr/bin/env python3
"""Verify all frozen source bytes and historical result identities."""
import hashlib
from evidence_sources import ROOT,manifest,source_bytes,key_manifest,global_manifest,file_baseline_manifest,native_client_baseline_manifest,mail_client_baseline_manifest,room_reader_baseline_manifest,post_metadata_baseline_manifest,authored_articles_baseline_manifest,broader_articles_baseline_manifest,unicode_subjects_baseline_manifest,modern_terminals_baseline_manifest,kitty_graphics_baseline_manifest,terminal_compatibility_baseline_manifest,menu_themes_baseline_manifest,seasonal_themes_baseline_manifest,custom_menus_baseline_manifest,nightwire_baseline_manifest
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
    # These records also referenced the old normal rc script, whose exact bytes
    # already exist in the fourth snapshot. Preserve results, never refresh hashes.
    retired = {'035-netbsd-confinement.json': '00e929f1100ec2cb66fa03449e3709bc83459e00c55fdb02c658c257061e9bc8', '036-netbsd-isolation.json': '69dac3fe8ecd5a95aa00d1e1ca32ff39af9c72e3e2dcc649864951ce32266f4c', '040-netbsd-global-admission.json': 'de02985415c8df4b9f1ecca405f7e72ea882f843f9a258c6b37cd2aa0c55bf2b'}
    import json
    for name,digest in retired.items():
        path=ROOT/'dev-docs/research/results'/name
        assert hashlib.sha256(path.read_bytes()).hexdigest()==digest,'historical record changed: '+name
        for source,expected in json.loads(path.read_text())['execution_sources'].items():
            assert hashlib.sha256(source_bytes(source,expected)).hexdigest()==expected
    extra=file_baseline_manifest()
    base=ROOT/'dev-docs/evidence'/extra['source_root']
    assert {str(p.relative_to(base)) for p in base.rglob('*') if p.is_file()}==set(extra['files'])
    for name,digest in extra['files'].items():
        assert hashlib.sha256((base/name).read_bytes()).hexdigest()==digest
    for name,digest in extra['records'].items():
        assert hashlib.sha256((ROOT/name).read_bytes()).hexdigest()==digest,'historical file baseline record changed: '+name
    extra=native_client_baseline_manifest()
    base=ROOT/'dev-docs/evidence'/extra['source_root']
    assert {str(p.relative_to(base)) for p in base.rglob('*') if p.is_file()}==set(extra['files'])
    for name,digest in extra['files'].items():
        assert hashlib.sha256((base/name).read_bytes()).hexdigest()==digest
    for name,digest in extra['records'].items():
        assert hashlib.sha256((ROOT/name).read_bytes()).hexdigest()==digest
    extra=mail_client_baseline_manifest()
    base=ROOT/'dev-docs/evidence'/extra['source_root']
    assert {str(p.relative_to(base)) for p in base.rglob('*') if p.is_file()}==set(extra['files'])
    for name,digest in extra['files'].items():
        assert hashlib.sha256((base/name).read_bytes()).hexdigest()==digest
        assert hashlib.sha256(source_bytes(name,digest)).hexdigest()==digest
    for name,digest in extra['records'].items():
        assert hashlib.sha256((ROOT/name).read_bytes()).hexdigest()==digest
    extra=room_reader_baseline_manifest()
    base=ROOT/'dev-docs/evidence'/extra['source_root']
    assert {str(p.relative_to(base)) for p in base.rglob('*') if p.is_file()}==set(extra['files'])
    for name,digest in extra['files'].items():
        assert hashlib.sha256((base/name).read_bytes()).hexdigest()==digest
        assert hashlib.sha256(source_bytes(name,digest)).hexdigest()==digest
    for name,digest in extra['records'].items():
        assert hashlib.sha256((ROOT/name).read_bytes()).hexdigest()==digest
    extra=post_metadata_baseline_manifest()
    base=ROOT/'dev-docs/evidence'/extra['source_root']
    assert len(extra['files'])==90 and len(extra['records'])==1
    assert {str(p.relative_to(base)) for p in base.rglob('*') if p.is_file()}==set(extra['files'])
    for name,digest in extra['files'].items():
        assert hashlib.sha256((base/name).read_bytes()).hexdigest()==digest
        assert hashlib.sha256(source_bytes(name,digest)).hexdigest()==digest
    for name,digest in extra['records'].items():
        assert hashlib.sha256((ROOT/name).read_bytes()).hexdigest()==digest
    extra=authored_articles_baseline_manifest()
    base=ROOT/'dev-docs/evidence'/extra['source_root']
    assert len(extra['files'])==95 and len(extra['records'])==1
    assert {str(p.relative_to(base)) for p in base.rglob('*') if p.is_file()}==set(extra['files'])
    for name,digest in extra['files'].items():
        assert hashlib.sha256((base/name).read_bytes()).hexdigest()==digest
        assert hashlib.sha256(source_bytes(name,digest)).hexdigest()==digest
    for name,digest in extra['records'].items():
        assert hashlib.sha256((ROOT/name).read_bytes()).hexdigest()==digest
    extra=broader_articles_baseline_manifest()
    base=ROOT/'dev-docs/evidence'/extra['source_root']
    assert len(extra['files'])==100 and len(extra['records'])==1
    assert {str(p.relative_to(base)) for p in base.rglob('*') if p.is_file()}==set(extra['files'])
    for name,digest in extra['files'].items():
        assert hashlib.sha256((base/name).read_bytes()).hexdigest()==digest
        assert hashlib.sha256(source_bytes(name,digest)).hexdigest()==digest
    for name,digest in extra['records'].items():
        assert hashlib.sha256((ROOT/name).read_bytes()).hexdigest()==digest
    extra=unicode_subjects_baseline_manifest()
    base=ROOT/'dev-docs/evidence'/extra['source_root']
    assert len(extra['files'])==101 and len(extra['records'])==1
    assert {str(p.relative_to(base)) for p in base.rglob('*') if p.is_file()}==set(extra['files'])
    for name,digest in extra['files'].items():
        assert hashlib.sha256((base/name).read_bytes()).hexdigest()==digest
        assert hashlib.sha256(source_bytes(name,digest)).hexdigest()==digest
    for name,digest in extra['records'].items():
        assert hashlib.sha256((ROOT/name).read_bytes()).hexdigest()==digest
    extra=modern_terminals_baseline_manifest()
    base=ROOT/'dev-docs/evidence'/extra['source_root']
    assert len(extra['files'])==106 and len(extra['records'])==1
    assert {str(p.relative_to(base)) for p in base.rglob('*') if p.is_file()}==set(extra['files'])
    for name,digest in extra['files'].items():
        assert hashlib.sha256((base/name).read_bytes()).hexdigest()==digest
        assert hashlib.sha256(source_bytes(name,digest)).hexdigest()==digest
    for name,digest in extra['records'].items():
        assert hashlib.sha256((ROOT/name).read_bytes()).hexdigest()==digest
    extra=kitty_graphics_baseline_manifest()
    base=ROOT/'dev-docs/evidence'/extra['source_root']
    assert len(extra['files'])==112 and len(extra['records'])==1
    assert {str(p.relative_to(base)) for p in base.rglob('*') if p.is_file()}==set(extra['files'])
    for name,digest in extra['files'].items():
        assert hashlib.sha256((base/name).read_bytes()).hexdigest()==digest
        assert hashlib.sha256(source_bytes(name,digest)).hexdigest()==digest
    for name,digest in extra['records'].items():
        assert hashlib.sha256((ROOT/name).read_bytes()).hexdigest()==digest
    extra=terminal_compatibility_baseline_manifest()
    base=ROOT/'dev-docs/evidence'/extra['source_root']
    assert len(extra['files'])==124 and len(extra['records'])==1
    assert {str(p.relative_to(base)) for p in base.rglob('*') if p.is_file()}==set(extra['files'])
    for name,digest in extra['files'].items():
        assert hashlib.sha256((base/name).read_bytes()).hexdigest()==digest
        assert hashlib.sha256(source_bytes(name,digest)).hexdigest()==digest
    for name,digest in extra['records'].items():
        assert hashlib.sha256((ROOT/name).read_bytes()).hexdigest()==digest
    extra=menu_themes_baseline_manifest()
    base=ROOT/'dev-docs/evidence'/extra['source_root']
    assert len(extra['files'])==122 and len(extra['records'])==1
    assert {str(p.relative_to(base)) for p in base.rglob('*') if p.is_file()}==set(extra['files'])
    for name,digest in extra['files'].items():
        assert hashlib.sha256((base/name).read_bytes()).hexdigest()==digest
        assert hashlib.sha256(source_bytes(name,digest)).hexdigest()==digest
    for name,digest in extra['records'].items():
        assert hashlib.sha256((ROOT/name).read_bytes()).hexdigest()==digest
    extra=seasonal_themes_baseline_manifest()
    base=ROOT/'dev-docs/evidence'/extra['source_root']
    assert len(extra['files'])==133 and len(extra['records'])==1
    assert {str(p.relative_to(base)) for p in base.rglob('*') if p.is_file()}==set(extra['files'])
    for name,digest in extra['files'].items():
        assert hashlib.sha256((base/name).read_bytes()).hexdigest()==digest
        assert hashlib.sha256(source_bytes(name,digest)).hexdigest()==digest
    for name,digest in extra['records'].items():
        assert hashlib.sha256((ROOT/name).read_bytes()).hexdigest()==digest
    extra=custom_menus_baseline_manifest()
    base=ROOT/'dev-docs/evidence'/extra['source_root']
    assert len(extra['files'])==137 and len(extra['records'])==1
    assert {str(p.relative_to(base)) for p in base.rglob('*') if p.is_file()}==set(extra['files'])
    for name,digest in extra['files'].items():
        assert hashlib.sha256((base/name).read_bytes()).hexdigest()==digest
        assert hashlib.sha256(source_bytes(name,digest)).hexdigest()==digest
    for name,digest in extra['records'].items():
        assert hashlib.sha256((ROOT/name).read_bytes()).hexdigest()==digest
    extra=nightwire_baseline_manifest()
    base=ROOT/'dev-docs/evidence'/extra['source_root']
    assert len(extra['files'])==176 and len(extra['records'])==2
    assert {str(p.relative_to(base)) for p in base.rglob('*') if p.is_file()}==set(extra['files'])
    for name,digest in extra['files'].items():
        assert hashlib.sha256((base/name).read_bytes()).hexdigest()==digest
        assert hashlib.sha256(source_bytes(name,digest)).hexdigest()==digest
    for name,digest in extra['records'].items():
        assert hashlib.sha256((ROOT/name).read_bytes()).hexdigest()==digest
    print('Nineteen source snapshots and thirty-two unchanged historical records passed.')
if __name__=='__main__':check()
