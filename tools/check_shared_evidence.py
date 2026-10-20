#!/usr/bin/env python3
"""Validate saved shared-board/credential evidence; never runs a VM or claims capacity."""
import hashlib
import json
import re
import statistics
from pathlib import Path

ROOT=Path(__file__).resolve().parents[1]
RESULTS=ROOT/'dev-docs/research/results'


def samples(run):
    columns=run['sample_columns'];rows=run['sample_rows']
    assert len(columns)==len(set(columns)) and columns==sorted(columns)
    assert {'monotonic_ns','finished_ns','vm_bytes.anon','vm_bytes.kernel_pool','vm_bytes.file','vm_bytes.free'}<=set(columns)
    assert len(rows)==run['sample_count']==300
    assert all(len(row)==len(columns) and all(isinstance(n,int) and n>=0 for n in row) for row in rows)
    assert all(re.fullmatch(r'(monotonic_ns|finished_ns|all_roles_rss_sum_bytes|vm_bytes\.[a-z_]+|limits\.[a-z_]+|roles\.(member|frontend|broker|ssh)\.[a-z_]+)',c) for c in columns)
    data=[dict(zip(columns,row)) for row in rows]
    assert all(x['finished_ns']>=x['monotonic_ns'] for x in data)
    assert all(a['monotonic_ns']<b['monotonic_ns'] for a,b in zip(data,data[1:]))
    assert run['markers']['start_ns']<=data[0]['monotonic_ns']<data[-1]['finished_ns']<=run['markers']['stop_ns']
    gaps=[(b['monotonic_ns']-a['monotonic_ns'])/1e6 for a,b in zip(data,data[1:])]
    assert min(gaps)==run['gap_ms']['min'] and max(gaps)==run['gap_ms']['max'] and statistics.median(gaps)==run['gap_ms']['median']
    assert run['requested_interval_ms']==10 and run['collector_query_ms']['max']<100
    assert all(x['vm_bytes.swap_used']==0 for x in data)
    return data


def check():
    comparison=RESULTS/'029-netbsd-shared-board.json';d=json.loads(comparison.read_text())
    assert d['schema']==1 and d['target']=='aarch64-unknown-netbsd'
    assert d['guest']['vcpus']==2 and d['guest']['configured_ram_bytes']==1024**3
    assert 900*1024*1024<d['guest']['physical_ram_bytes']<=1024**3
    assert d['native_rust_tests']==58 and d['native_pty_workflows']==5
    for name,count in [('normal_gateway_ssh_oracles',20),('normal_gateway_lifecycle_oracles',13),('shared_oracles',16)]:
        assert len(d[name]['results'])==count and all(x['result']=='pass' for x in d[name]['results'])
    assert d['native_verified_source_count']>=40 and len(d['execution_sources'])>=50
    for name,sha in d['execution_sources'].items():
        assert hashlib.sha256((ROOT/name).read_bytes()).hexdigest()==sha,'Executed source changed: '+name
    for binary in d['native_binaries'].values():assert re.fullmatch('[0-9a-f]{64}',binary['sha256']) and binary['bytes']>0
    for model in ['worker_model','shared_model']:
        run=d[model];assert run['model']=={'worker_model':'workers','shared_model':'shared'}[model]
        assert run['outcome']=='stopped_at_headroom_guard' and run['completed_stage']==100
        assert 100<=run['highest_observed_approved_sessions']<250 and run['failure'] is None
        assert run['fixture_limits']['credential_slots']==2 and run['fixture_limits']['experimental_maxproc']==2048
        assert run['fixture_limits']['experimental_maxfiles']==16384
        for stage in run['stages']:
            assert stage['all_alive'] and stage['sessions']==stage['distinct_accounts']
            assert stage['end']['vm_bytes']['swap_used']==0
            if stage['sessions']:
                n=stage['sessions'];roles=stage['end']['roles']
                assert roles['frontend']['count']==n and roles['ssh']['count']>=2*n+1 and roles['broker']['count']==1
                assert roles.get('member',{}).get('count',0)==(n if model=='worker_model' else 0)
        stop=run['load_stop'];assert stop['admitted_sessions']==run['highest_observed_approved_sessions']
        assert stop['workers_and_frontends_reaped'] and stop['seconds']<12
        assert stop['snapshot_before_stop']['roles']['frontend']['count']==stop['admitted_sessions']
        assert stop['snapshot_before_stop']['vm_bytes']['swap_used']==0
        assert run['fresh_login_after_load']=='pass'
    assert d['shared_model']['highest_observed_approved_sessions']>d['worker_model']['highest_observed_approved_sessions']
    assert d['kernel_oom_events']==0 and d['restored_kernel_limits']==['1044','3404']
    assert d['final_backend_and_frontends_reaped'] and d['final_fixture_integrity']['integrity']=='ok'
    assert d['prepared_fixture']=={'integrity':'ok','accounts':1001,'unique_hashes':1001,'rooms':50,'posts':100000}
    assert all(d['cleanup'][key] for key in ['guest_stopped','private_inputs_removed','forwards_closed'])
    p=json.loads((RESULTS/'030-netbsd-credential-peaks.json').read_text())
    assert p['schema']==1 and p['credential_slots']==2
    assert p['provenance']['sha256']==hashlib.sha256(comparison.read_bytes()).hexdigest()
    assert p['argon2']=={'algorithm':'argon2id','version':19,'memory_kib':19456,'iterations':2,'lanes':1}
    assert p['fresh_login_after_release']=='pass' and p['hidden_input']=='pass'
    assert len(p['calibrations'])==3
    for run,amount in zip(p['calibrations'],[0,19,38]):
        rows=samples(run);c=run['calibration'];assert c['known_touched_bytes']==amount*1024*1024
        m=c['allocation_markers'];inside=[x['vm_bytes.anon'] for x in rows if m['touched_ns']<=x['monotonic_ns'] and x['finished_ns']<=m['released_ns']]
        assert len(inside)==c['fully_inside_samples']>=1 and max(inside)==c['max_anon_bytes']
        assert c['observed_anon_delta_bytes']==max(inside)-c['baseline_anon_bytes']
        assert abs(c['observed_anon_delta_bytes']-c['known_touched_bytes'])<1024*1024
    assert len(p['bursts'])==4 and {x['case'] for x in p['bursts']}=={'both_slots_held','wrong_password','valid_password','valid_password_repeat'}
    for run in p['bursts']:
        rows=samples(run);assert run['offered']==8
        assert run['successful']+run['budget_rejected']+run['credentials_rejected']==8
        assert rows[0]['roles.member.count']==rows[0]['roles.frontend.count']==8
        assert rows[0]['roles.ssh.count']>=17
        assert max(x['vm_bytes.anon'] for x in rows)==run['observed_max_anon_bytes']
        if run['case']=='both_slots_held':assert run['budget_rejected']==8
        elif run['case']=='wrong_password':assert run['successful']==0 and run['credentials_rejected']>=1
        else:assert run['successful']>=1
        # Cumulative successes are deliberately NOT capped at two.
    print('Saved native shared/credential evidence passed: matched SSH counts, guard/cleanup/recovery, 16 boundaries, complete calibrated sample matrices and frozen source/binary provenance. No VM rerun or 250-user support claim.')


if __name__=='__main__':check()
