#!/usr/bin/env python3
"""Check saved custom-menu observations; no listeners, clocks or VM invocation."""
import hashlib
import json
from evidence_sources import ROOT, source_bytes

def check():
    d=json.loads((ROOT/'dev-docs/evidence/custom-menus-netbsd/results.json').read_text())
    assert d['schema']==1 and d['result']=='pass'
    assert d['host_rust_tests']==d['native_rust_tests']==167 and d['host_python_tests']==111
    assert d['selected_native_source_tree_matches']==len(d['execution_sources'])==130
    assert len(d['harness_sources'])==7 and len(d['preview_sha256'])==2
    for group in ['execution_sources','harness_sources','preview_sha256']:
        for name,digest in d[group].items():assert hashlib.sha256(source_bytes(name,digest)).hexdigest()==digest
    for group in ['host_custom_menus','native_custom_menus','shared_custom_menus']:
        c=d[group];assert c['result']=='pass' and c['count']==len(c['checks'])==4 and not c['capacity_claim']
    assert 'restricted_ssh_grid_spatial_focus_across_reads_and_enter' in d['shared_custom_menus']['checks']
    for group in ['host_seasonal','native_seasonal']:
        c=d[group];assert c['result']=='pass' and c['count']==len(c['checks'])==9 and not c['host_clock_changed']
    for group,count in [('shared_seasonal',8),('shared_regressions',22)]:
        c=d[group];assert c['result']=='pass' and c['count']==len(c['checks'])==count
    assert d['limits']=={'grid_rows':5,'grid_columns':2,'full_menu_bytes_below':4096,'focus_bytes_at_most':512,'art_columns':80,'art_rows':16,'art_input_bytes':16384,'art_output_bytes':16384,'normal_sessions':8,'admission_refill_ms':5000,'shared_output_bytes':32768,'drain_seconds':5,'new_owner_helpers':0,'new_external_packages':0}
    assert d['login_preview_bytes']=={'40-ansi':181,'40-plain':142,'80-ansi':1447,'80-plain':142}
    assert len(d['installed_native_binaries_sha256'])==3
    assert all(v is False for v in d['claims'].values())
    assert len(d['teardown'])==3 and all(d['teardown'].values())
    print('Custom menus evidence passed: 167 Rust per OS, 111 host Python, four PTY controls per OS, four actual shared SSH/native controls, eight calendar and 22 shared regressions; no capacity/art/font claim.')

if __name__=='__main__':check()
