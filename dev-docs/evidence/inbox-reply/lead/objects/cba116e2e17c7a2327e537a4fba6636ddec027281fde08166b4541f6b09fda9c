#!/usr/bin/env python3
"""Verify original-art software observations and prior immutable inputs; no VM."""
import hashlib
import json
from evidence_sources import ROOT,source_bytes

def check():
    d=json.loads((ROOT/'dev-docs/evidence/nightwire-netbsd/results.json').read_text())
    assert d['schema']==1 and d['result']=='pass'
    assert d['host_rust_tests']==d['native_rust_tests']==171 and d['host_python_tests']==112
    assert d['selected_native_source_tree_matches']==len(d['execution_sources'])==157
    assert len(d['harness_sources'])==5 and len(d['art_files'])==12 and len(d['preview_sha256'])==7
    for group in ['execution_sources','harness_sources','art_files','preview_sha256','prior_paging_inputs_unchanged']:
        for name,digest in d[group].items():assert hashlib.sha256(source_bytes(name,digest)).hexdigest()==digest
    assert len(d['prior_paging_inputs_unchanged'])==144
    assert hashlib.sha256((ROOT/'dev-docs/evidence/login-paging-netbsd/results.json').read_bytes()).hexdigest()==d['prior_paging_record_sha256']
    for group in ['host_previews','native_previews']:
        p=d[group];assert p['result']=='pass' and p['profile_count']==len(p['preview_profiles'])==18
        assert p['short_cover_matches'] and p['seasonal_geometry_matches'] and p['menu_map_unchanged'] and not p['capacity_claim']
    assert d['host_previews']['preview_profiles']==d['native_previews']['preview_profiles']
    for group in ['host_logins','native_logins','shared_logins']:
        c=d[group];assert c['result']=='pass' and c['count']==len(c['checks'])==7 and not c['capacity_claim']
    assert d['shared_logins']['normal_sessions']==8 and d['shared_logins']['admission_refill_ms']==5000
    assert len(d['installed_native_binaries_sha256'])==3
    assert d['limits']=={'packs':4,'asset_files':12,'generated_svgs':7,'preview_profiles_per_os':18,'short_art_rows':16,'long_art_rows':48,'page_rows':16,'normal_sessions':8,'admission_refill_ms':5000,'shared_output_bytes':32768,'drain_seconds':5,'new_runtime_dependencies':0,'runtime_source_edits':0,'default_pack_edits':0}
    assert all(v is False for v in d['claims'].values())
    assert len(d['teardown'])==3 and all(d['teardown'].values())
    print('Nightwire evidence passed: 18 render profiles/seven local routes per OS, seven actual SSH/native routes, 157 native input matches and all 144 prior inputs unchanged; no final-art/font/capacity claim.')

if __name__=='__main__':check()
