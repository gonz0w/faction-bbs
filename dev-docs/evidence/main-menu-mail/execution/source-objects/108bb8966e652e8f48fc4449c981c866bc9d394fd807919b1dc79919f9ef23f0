#!/usr/bin/env python3
"""Verify saved backup-check/Recovery software observations; never start a service/VM."""
import hashlib,json
from evidence_sources import ROOT,source_bytes,operator_host_baseline_manifest


def check():
    d=json.loads((ROOT/'dev-docs/evidence/operator-recovery-netbsd/results.json').read_text())
    assert d['schema']==1 and d['result']=='pass'
    assert d['host_rust_tests']==d['native_rust_tests']==184 and d['host_python_tests']==118
    assert d['selected_native_source_matches']==len(d['execution_sources'])==159
    assert len(d['harness_sources'])==14
    for group in ['execution_sources','harness_sources']:
        for name,digest in d[group].items():assert hashlib.sha256(source_bytes(name,digest)).hexdigest()==digest,name
    assert hashlib.sha256((ROOT/'dev-docs/evidence/operator-host-baseline-manifest.json').read_bytes()).hexdigest()==d['prior_operator_host_manifest_sha256']
    m=operator_host_baseline_manifest();assert len(m['files'])==162 and len(m['records'])==1
    for name,digest in m['records'].items():assert hashlib.sha256((ROOT/name).read_bytes()).hexdigest()==digest
    for key in ['host_operator_ui','native_operator_ui']:
        v=d[key];assert v['result']=='pass' and v['count']==len(v['checks'])==8 and not v['capacity_claim']
    v=d['shared_operator'];assert v['result']=='pass' and v['count']==len(v['checks'])==10 and v['normal_sessions']==8
    assert not v['capacity_claim'] and not v['physical_sbc_claim']
    samples=v['observations']['resource_samples'];assert len(samples)==12
    assert all(s['dashboard_lwps']==1 and s['owner_lwps']==4 for s in samples)
    assert min(s['dashboard_rss_kib'] for s in samples)==max(s['dashboard_rss_kib'] for s in samples)==2436
    assert min(s['owner_rss_kib'] for s in samples)==max(s['owner_rss_kib'] for s in samples)==5156
    assert min(s['vm_free_bytes'] for s in samples)>333*1048576
    host=v['observations']['host_summary'];assert 'CPU 2 busy: warming up' in host
    assert 'Net RX/TX: warming up' in host and 'Non-loopback interfaces: 1' in host
    assert any(line.startswith('RAM used: ') and '/1006 MiB ' in line for line in host)
    assert not any('unavailable' in line for line in host)
    serial=d['serial_console'];assert serial['result']=='pass' and serial['count']==len(serial['checks'])==4
    assert not any(serial[k] for k in ['autologin','login_profile_changed','physical_console_claim'])
    for key,native in [('host_backup_checks',False),('native_backup_checks',True)]:
        v=d[key];assert v['result']=='pass' and v['count']==len(v['checks'])==11
        assert v['native_distinct_uid_cli']==native and v['developer_api_fixture']!=native
        assert not v['capacity_claim'] and not v['hardening_claim']
    for key in ['shared_nightwire','native_nightwire_logins']:
        v=d[key];assert v['result']=='pass' and v['count']==len(v['checks'])==7 and not v['capacity_claim']
    v=d['native_nightwire_previews'];assert v['result']=='pass' and v['profile_count']==len(v['preview_profiles'])==18
    b=d['budgets'];assert b==dict(normal_sessions=8,credential_permits=2,owner_helpers=3,owner_lwps=4,shared_output_bytes=32768,drain_seconds=5,status_file_bytes=2048,publish_seconds=5,query_op_budget=50000,query_progress_milliseconds=5,history_samples=60,refresh_minimum_seconds=2,refresh_maximum_seconds=60,host_counter_slots=64,host_network_buffer_bytes=65536,host_interface_limit=16,host_refresh_helpers=0,new_registry_packages=0,backup_receipt_bytes=512,backup_objects=2000,backup_total_bytes=268435456)
    assert all(d['teardown'].values()) and not any(d['claims'].values())
    print('Recovery evidence passed: 184 Rust per OS, 11 backup checks per OS, 8 UI per OS, 10 shared/4 serial controls; no capacity/hardening claim.')


if __name__=='__main__':check()
