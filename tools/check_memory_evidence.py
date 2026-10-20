#!/usr/bin/env python3
"""Verify saved native scaling evidence; never reruns NetBSD or claims capacity."""
import hashlib
import json
from pathlib import Path

ROOT=Path(__file__).resolve().parents[1]


def check():
    data=json.loads((ROOT/'dev-docs/research/results/027-netbsd-memory-scaling.json').read_text())
    assert data['schema']==1 and data['target']=='aarch64-unknown-netbsd'
    assert data['native_rust_tests']==58 and data['native_pty_workflows']==5
    assert data['guest']['vcpus']==2 and 900*1024*1024<data['guest']['physical_ram_bytes']<=1024**3
    run=data['scaling']
    assert run['outcome'] in ['completed','stopped_at_headroom_guard']
    assert run['completed_stage']>=100 and run['highest_observed_approved_sessions']<=250
    assert run['seed']=={'approved_accounts':1000,'rooms':50,'posts':100000,'post_body_bytes':1024,'independent_password_salts':True}
    assert run['fixture_limits']['credential_slots']==2 and run['fixture_limits']['worker_limit']==250
    assert run['fixture_limits']['experimental_maxproc']==2048 and run['fixture_limits']['experimental_maxfiles']==16384
    for row in run['stages']:
        assert row['sessions']==row['distinct_accounts'] and row['all_alive']
        assert row['end']['vm_bytes']['swap_used']==0
        if row['sessions']:
            assert row['end']['roles']['member']['count']==row['sessions']
            assert row['end']['roles']['frontend']['count']==row['sessions']
            assert row['end']['roles']['ssh']['count']>=2*row['sessions']+1
    assert max(r['sessions'] for r in run['deliveries'])==run['completed_stage']
    assert all(r['sessions']==r['distinct_accounts'] and r['delivery_seconds']>0 for r in run['deliveries'])
    burst=run['authentication_burst']
    assert burst['offered']==8 and burst['successful']+burst['budget_rejected']==8
    assert burst['successful']>=1 and burst['budget_rejected']>=1 and burst['samples']
    assert run['load_stop']['workers_and_frontends_reaped'] and run['load_stop']['seconds']<11
    assert run['fresh_login_after_load']=='pass'
    assert data['fixture_integrity']=={'integrity':'ok','accounts':1001,'unique_hashes':1001,'rooms':50,'posts':100000}
    assert data['final_run_kernel_oom_events']==0
    assert data['experimental_limits_restored']
    assert data['cleanup']['guest_stopped'] and data['cleanup']['private_inputs_removed']
    assert len(data['execution_sources'])>=40
    for name,expected in data['execution_sources'].items():
        assert hashlib.sha256((ROOT/name).read_bytes()).hexdigest()==expected,'Executed profiling source changed: '+name
    assert data['execution_sources']['experiments/027-memory-scaling/netbsd_metrics.c']
    print('Saved native memory/scaling evidence passed: distinct-account SSH, guarded admission, budget/recovery, integrity, cleanup and executed source hashes. No VM rerun or production-capacity claim.')


if __name__=='__main__':check()
