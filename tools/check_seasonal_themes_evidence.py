#!/usr/bin/env python3
"""Verify saved calendar/login software observations; never changes a clock or VM."""
import hashlib
import json
from evidence_sources import ROOT, source_bytes

def check():
    d=json.loads((ROOT/'dev-docs/evidence/seasonal-themes-netbsd/results.json').read_text())
    assert d['schema']==1 and d['result']=='pass'
    assert d['host_rust_tests']==d['native_rust_tests']==162 and d['host_python_tests']==110
    assert d['selected_native_source_tree_matches']==len(d['execution_sources'])==125 and len(d['harness_sources'])==5
    for group in ['execution_sources','harness_sources','native_seasonal_harness_matches','preview_sha256']:
        for name,digest in d[group].items():assert hashlib.sha256(source_bytes(name,digest)).hexdigest()==digest
    assert len(d['native_seasonal_harness_matches'])==1 and len(d['preview_sha256'])==3
    for group in ['host_seasonal','native_seasonal']:
        c=d[group];assert c['result']=='pass' and c['count']==len(c['checks'])==9 and not c['host_clock_changed'] and not c['capacity_claim']
        assert 'actual_os_utc_local_year_boundary_and_dst' in c['checks']
    shared=d['shared_seasonal'];assert shared['result']=='pass' and shared['count']==len(shared['checks'])==8
    assert 'system_timezone_shared_startup_and_member_login' in shared['checks']
    assert 'restricted_pending_member_gets_no_season_identity' in shared['checks']
    assert 'manual_override_recovers_broken_calendar' in shared['checks']
    assert not shared['host_clock_changed'] and shared['normal_sessions']==8 and shared['admission_refill_ms']==5000
    assert d['shared_regressions']['result']=='pass' and d['shared_regressions']['count']==len(d['shared_regressions']['checks'])==22
    assert d['login_preview_bytes']=={'autumn-40':{'ansi_preview_bytes':183,'plain_preview_bytes':144},'autumn-80':{'ansi_preview_bytes':604,'plain_preview_bytes':144},'winter-40':{'ansi_preview_bytes':187,'plain_preview_bytes':148},'winter-80':{'ansi_preview_bytes':657,'plain_preview_bytes':148}}
    assert d['limits']=={'seasonal_overrides':8,'shared_catalog_themes':9,'per_shared_client_theme_indexes':1,'normal_sessions':8,'admission_refill_ms':5000,'shared_output_bytes':32768,'drain_seconds':5,'new_owner_helpers':0,'new_external_packages':0}
    assert len(d['installed_native_binaries_sha256'])==3
    assert all(v is False for v in d['claims'].values())
    assert len(d['teardown'])==3 and all(d['teardown'].values())
    print('Seasonal evidence passed: 162 host/native Rust, 110 host Python, nine calendar PTY controls per OS, eight actual shared SSH/native controls and 22 shared regressions; no clock/capacity/art-certification claim.')

if __name__=='__main__':check()
