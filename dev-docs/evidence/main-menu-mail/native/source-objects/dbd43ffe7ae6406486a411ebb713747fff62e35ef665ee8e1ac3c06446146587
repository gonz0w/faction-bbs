#!/usr/bin/env python3
"""Verify saved paging observations and executed bytes; never starts a VM/listener."""
import hashlib
import json
from evidence_sources import ROOT,source_bytes

def check():
    d=json.loads((ROOT/'dev-docs/evidence/login-paging-netbsd/results.json').read_text())
    assert d['schema']==1 and d['result']=='pass'
    assert d['host_rust_tests']==d['native_rust_tests']==171 and d['host_python_tests']==112
    assert d['selected_native_source_tree_matches']==len(d['execution_sources'])==135
    assert len(d['harness_sources'])==9 and len(d['preview_sha256'])==3
    for group in ['execution_sources','harness_sources','preview_sha256']:
        for name,digest in d[group].items():assert hashlib.sha256(source_bytes(name,digest)).hexdigest()==digest
    for group in ['host_paging','native_paging','shared_paging']:
        c=d[group];assert c['result']=='pass' and c['count']==len(c['checks'])==8 and not c['capacity_claim']
    assert 'current_revocation_denies_next_art_page' in d['shared_paging']['checks']
    assert 'ssh_resize_cancels_art_at_drained_boundary' in d['shared_paging']['checks']
    assert 'bad_later_page_fails_preflight_before_bind' in d['shared_paging']['checks']
    for group,count in [('host_custom_menus',4),('native_custom_menus',4),('host_seasonal',9),('native_seasonal',9),('shared_seasonal',8),('shared_regressions',22)]:
        c=d[group];assert c['result']=='pass' and c['count']==len(c['checks'])==count
    assert d['limits']=={'art_columns':80,'long_art_rows':256,'long_art_input_bytes':65536,'page_rows':16,'art_page_bytes':16384,'max_pages':16,'short_art_rows':16,'short_art_input_bytes':16384,'preview_bytes':17408,'normal_sessions':8,'admission_refill_ms':5000,'shared_output_bytes':32768,'drain_seconds':5,'per_member_full_art_copies':0,'new_owner_helpers':0,'new_external_packages':0}
    assert set(d['preview_page_bytes'])=={'1','2','3'} and all(0<n<=17408 for n in d['preview_page_bytes'].values())
    assert len(d['installed_native_binaries_sha256'])==3
    assert all(v is False for v in d['claims'].values())
    assert len(d['teardown'])==3 and all(d['teardown'].values())
    print('Login paging evidence passed: 171 Rust per OS, 112 host Python, eight paging PTY controls per OS, eight actual shared paging controls, calendar/reader regressions; no capacity/font/final-art claim.')

if __name__=='__main__':check()
