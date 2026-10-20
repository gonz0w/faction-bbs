#!/usr/bin/env python3
"""Check saved #58 native evidence and its frozen inputs; does not rerun a VM."""
import hashlib
import json
import re
import statistics
from pathlib import Path
from evidence_sources import source_bytes

ROOT=Path(__file__).resolve().parents[1]


def stream(run):
    columns=run['sample_columns'];rows=run['sample_rows']
    assert columns==sorted(set(columns)) and len(rows)==run['sample_count']==150
    assert all(re.fullmatch(r'(monotonic_ns|finished_ns|all_roles_rss_sum_bytes|vm_bytes\.[a-z_]+|limits\.[a-z_]+|roles\.(member|frontend|broker|ssh)\.[a-z_]+)',c) for c in columns)
    assert all(len(row)==len(columns) and all(isinstance(v,int) and v>=0 for v in row) for row in rows)
    values=[dict(zip(columns,row)) for row in rows]
    assert all(x['finished_ns']>=x['monotonic_ns'] for x in values)
    assert all(a['monotonic_ns']<b['monotonic_ns'] for a,b in zip(values,values[1:]))
    assert run['markers']['start_ns']<=values[0]['monotonic_ns']<values[-1]['finished_ns']<=run['markers']['stop_ns']
    gaps=[(b['monotonic_ns']-a['monotonic_ns'])/1e6 for a,b in zip(values,values[1:])]
    assert run['gap_ms']=={'min':min(gaps),'median':statistics.median(gaps),'max':max(gaps)}
    assert run['requested_interval_ms']==10
    assert all(x['vm_bytes.swap_used']==0 for x in values)
    return values


def check():
    d=json.loads((ROOT/'dev-docs/research/results/031-netbsd-transport-memory.json').read_text())
    old=ROOT/'experiments/055-shared-board';new=ROOT/'experiments/058-transport-memory'
    for name in ['Cargo.toml','Cargo.lock']:assert (old/name).read_bytes()==(new/name).read_bytes()
    assert (new/'main.rs').read_text()==(old/'main.rs').read_text().replace('self.lease = Some(lease);','self.lease = Some(lease);\n                    self.request = 1; // New lease starts a new core request sequence.')
    assert d['schema']==1 and d['target']=='aarch64-unknown-netbsd'
    assert d['guest']['vcpus']==2 and d['guest']['configured_ram_bytes']==1024**3
    assert 900*1024*1024<d['guest']['physical_ram_bytes']<=1024**3
    assert d['native_verified_source_count']>=40 and len(d['execution_sources'])>=52
    for name,sha in d['execution_sources'].items():
        assert hashlib.sha256(source_bytes(name,sha)).hexdigest()==sha,'Executed source changed: '+name
    for binary in d['native_binaries'].values():
        assert re.fullmatch('[0-9a-f]{64}',binary['sha256']) and binary['bytes']>0
    for profile in ['default','compact','minimal']:
        assert len(d['trials'][profile])==2
        for index,run in enumerate(d['trials'][profile]):
            assert run['allocator_profile']==('default' if profile=='default' else 'compact') and run['model']=='shared'
            assert run['failure'] is None and run['fresh_login_after_load']=='pass'
            assert run['seed']=={'approved_accounts':1000,'rooms':50,'posts':100000,'post_body_bytes':1024,'independent_password_salts':True}
            a=run['allocator_readback']
            assert a['version'].startswith('5.3.0') and a['retain'] is False
            assert (a['narenas'],a['tcache'],a['dirty_decay_ms'])==((8,True,10000) if profile=='default' else (1,False,0))
            assert run['fixture_limits']['original_maxproc']==1044 and run['fixture_limits']['original_maxfiles']==3404
            assert run['fixture_limits']['credential_slots']==2
            assert run['frontend_binary']==('faction-ssh-minimal' if profile=='minimal' else 'faction-ssh-rust')
            assert run['completed_stage']==(250 if profile=='minimal' else 200)
            assert run['highest_observed_approved_sessions']==({'default':224,'compact':248,'minimal':250}[profile])
            assert run['fixture_limits']['experimental_maxproc']==2048 and run['fixture_limits']['experimental_maxfiles']==16384
            for stage in run['stages']:
                n=stage['sessions'];assert stage['all_alive'] and stage['distinct_accounts']==n
                assert stage['end']['vm_bytes']['swap_used']==0
                if n:
                    roles=stage['end']['roles']
                    assert roles['frontend']['count']==n and roles['ssh']['count']>=2*n+1
                    assert roles['broker']['count']==1 and roles.get('member',{}).get('count',0)==0
            decisions=run['guard_decisions'];assert decisions
            for row in decisions:
                vm=row['snapshot']['vm_bytes'];limits=row['snapshot']['limits']
                available=vm['free']+max(0,vm['file_clean']-vm['wired'])
                assert row['minimum_headroom_bytes']==128*1024*1024
                assert row['reclaimable_headroom_bytes']==available
                assert row['shortfall_bytes']==max(0,128*1024*1024-available)
                assert row['passed']==(available>=128*1024*1024 and vm['swap_used']==0 and limits['processes']+64<limits['max_processes'] and limits['open_files']+128<limits['max_files'])
            if run['outcome']=='stopped_at_headroom_guard':assert any(not x['passed'] for x in decisions)
            else:assert run['outcome']=='completed' and run['completed_stage']==250
            for row in run['admission_batches']:
                assert 0<row['admitted_in_batch']<=8
                assert row['admitted_after']-row['admitted_before']==row['admitted_in_batch']
            stop=run['load_stop'];assert stop['workers_and_frontends_reaped'] and stop['seconds']<12
            assert stop['admitted_sessions']==run['highest_observed_approved_sessions']<=250
            assert stop['snapshot_before_stop']['vm_bytes']['swap_used']==0
            assert len(run['calibrations'])==3
            for c,amount in zip(run['calibrations'],[0,19,38]):
                rows=stream(c);a=c['calibration'];m=a['allocation_markers']
                inside=[x['vm_bytes.anon'] for x in rows if m['touched_ns']<=x['monotonic_ns'] and x['finished_ns']<=m['released_ns']]
                assert a['known_touched_bytes']==amount*1024*1024
                assert len(inside)==a['fully_inside_samples']>=1 and max(inside)==a['max_anon_bytes']
                assert a['observed_anon_delta_bytes']==max(inside)-a['baseline_anon_bytes']
                assert abs(a['observed_anon_delta_bytes']-amount*1024*1024)<1024*1024
            assert sorted(c['cohort_in_chat'] for c in run['high_load_credentials'])==([198,248] if profile=='minimal' else [198])
            for c in run['high_load_credentials']:
                rows=stream(c);n=c['cohort_in_chat']+2
                assert n>=200 and c['successful_credentials']==c['failed_credentials']==2
                assert c['hidden_input']=='pass' and c['failed_transports_reaped']
                assert c['after_failure_snapshot']['roles']['frontend']['count']==n-2
                assert rows[0]['roles.frontend.count']==n and rows[0]['roles.ssh.count']>=2*n+1
                assert any(x['sessions']==n and x.get('phase')=='after_two_credentials' for x in run['deliveries'])
            if index==0:
                assert [x['state'] for x in run['transport_states']]==['authenticated_transport_no_channel','handed_off_password_prompt']
                assert all(x['transports']==32 for x in run['transport_states'])
    boots=[x['boot_metadata'][0] for rows in d['trials'].values() for x in rows]
    assert len(boots)==len(set(boots))==6
    assert len(d['minimal_frontend_oracles']['results'])==20 and all(x['result']=='pass' for x in d['minimal_frontend_oracles']['results'])
    assert all(d['minimal_build'][k] for k in ['pie','relro','immediate_binding','separate_rx_rw_loads'])
    assert d['minimal_build']['gcc_analyzer']=='pass' and d['minimal_build']['runtime_libraries']==['libc.so.12']
    assert d['native_binaries']['faction-ssh']['sha256']==d['native_binaries']['faction-ssh-minimal']['sha256']
    assert d['kernel_oom_events']==0 and d['restored_kernel_limits']==['1044','3404']
    for profile in ['default','compact']:
        checks=d['regressions'][profile]
        assert checks['rust_tests']==58 and checks['pty_workflows']==5
        for name,count in [('ssh',20),('lifecycle',13),('shared',16)]:
            assert len(checks[name]['results'])==count and all(x['result']=='pass' for x in checks[name]['results'])
    assert set(d['source_sets']['minimal'])==set(d['execution_sources'])
    assert set(d['source_sets']['minimal'])-set(d['source_sets']['baseline'])=={'experiments/058-transport-memory/minimal-frontend.c','experiments/058-transport-memory/frontend_probe.py'}
    assert d['trials']['minimal'][0]['outcome']=='completed'
    repeat=d['trials']['minimal'][1]
    assert repeat['outcome']=='stopped_at_headroom_guard' and repeat['guard_decisions'][-1]['reason']=='after_completed_stage'
    assert repeat['guard_decisions'][-1]['shortfall_bytes']>0
    assert d['final_metrics']['roles']['ssh']['count']==1
    assert all(d['final_metrics']['roles'].get(k,{}).get('count',0)==0 for k in ['member','frontend','broker'])
    assert d['final_backend_and_frontends_reaped'] and d['final_fixture_integrity']['integrity']=='ok'
    assert all(d['cleanup'][key] for key in ['guest_stopped','private_inputs_removed','forwards_closed'])
    print('Saved native transport evidence passed: frozen inputs, allocator readbacks, full SSH accounting, exact guards/batch effects, repeated trials, calibrated high-load credentials, denial/failure/recovery and cleanup. No VM rerun or supported-capacity claim.')


if __name__=='__main__':check()
