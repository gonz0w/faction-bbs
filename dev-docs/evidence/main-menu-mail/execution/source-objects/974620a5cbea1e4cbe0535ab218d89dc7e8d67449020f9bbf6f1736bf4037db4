#!/usr/bin/env python3
"""Verify saved local-operator software observations; never start a service/VM."""
import hashlib,json
from evidence_sources import ROOT,source_bytes,nightwire_baseline_manifest


def check():
    d=json.loads((ROOT/'dev-docs/evidence/operator-netbsd/results.json').read_text())
    assert d['schema']==1 and d['result']=='pass'
    assert d['host_rust_tests']==d['native_rust_tests']==179 and d['host_python_tests']==115
    assert d['selected_native_source_matches']==len(d['execution_sources'])==152
    assert len(d['harness_sources'])==8
    for group in ['execution_sources','harness_sources']:
        for name,digest in d[group].items():assert hashlib.sha256(source_bytes(name,digest)).hexdigest()==digest,name
    assert hashlib.sha256((ROOT/'dev-docs/evidence/nightwire-baseline-manifest.json').read_bytes()).hexdigest()==d['prior_nightwire_manifest_sha256']
    m=nightwire_baseline_manifest();assert len(m['files'])==176 and len(m['records'])==2
    for name,digest in m['records'].items():assert hashlib.sha256((ROOT/name).read_bytes()).hexdigest()==digest
    for key in ['host_operator_ui','native_operator_ui']:
        v=d[key];assert v['result']=='pass' and v['count']==len(v['checks'])==6 and not v['capacity_claim']
    v=d['shared_operator'];assert v['result']=='pass' and v['count']==len(v['checks'])==10 and v['normal_sessions']==8
    assert not v['capacity_claim'] and not v['physical_sbc_claim']
    samples=v['observations']['resource_samples'];assert len(samples)==12
    assert all(s['dashboard_lwps']==1 and s['owner_lwps']==4 for s in samples)
    assert min(s['dashboard_rss_kib'] for s in samples)==max(s['dashboard_rss_kib'] for s in samples)==2396
    assert min(s['owner_rss_kib'] for s in samples)==max(s['owner_rss_kib'] for s in samples)==5144
    assert min(s['vm_free_bytes'] for s in samples)>334*1048576
    host=v['observations']['host_summary'];assert 'Host CPUs: 2' in host and 'Physical RAM: 1006 MiB' in host
    assert 'RAM used/free: unavailable' in host and 'Network/peers/backup: unavailable' in host
    for key in ['shared_nightwire','native_nightwire_logins']:
        v=d[key];assert v['result']=='pass' and v['count']==len(v['checks'])==7 and not v['capacity_claim']
    v=d['native_nightwire_previews'];assert v['result']=='pass' and v['profile_count']==len(v['preview_profiles'])==18
    b=d['budgets'];assert b==dict(normal_sessions=8,credential_permits=2,owner_helpers=3,owner_lwps=4,shared_output_bytes=32768,drain_seconds=5,status_file_bytes=2048,publish_seconds=5,query_op_budget=50000,query_progress_milliseconds=5,history_samples=60,refresh_minimum_seconds=2,refresh_maximum_seconds=60,host_helper_deadline_ms=250,host_helper_bytes=1024,new_registry_packages=0)
    assert all(d['teardown'].values()) and not any(d['claims'].values())
    print('Operator evidence passed: 179 Rust per OS, 6 UI per OS, 10 native shared controls; no capacity/hardening claim.')


if __name__=='__main__':check()
