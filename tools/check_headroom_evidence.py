#!/usr/bin/env python3
"""Validate saved native headroom evidence; never starts or reruns a VM."""
import hashlib
import json
import re
from collections import Counter
from pathlib import Path
from check_transport_evidence import stream
from headroom_memory import live_limits,sample_guard,RESOURCES

ROOT=Path(__file__).resolve().parents[1]


def summarized(row):
    return {prefix:{k.split('.',1)[1]:v for k,v in row.items() if k.startswith(prefix+'.')}
            for prefix in ['vm_bytes','limits']}


def calibration(observed):
    rows=stream(observed);a=observed['calibration'];m=a['allocation_markers']
    inside=[x['vm_bytes.anon'] for x in rows if m['touched_ns']<=x['monotonic_ns'] and x['finished_ns']<=m['released_ns']]
    assert len(inside)==a['fully_inside_samples']>=1 and max(inside)==a['max_anon_bytes']
    assert a['observed_anon_delta_bytes']==max(inside)-a['baseline_anon_bytes']
    assert abs(a['observed_anon_delta_bytes']-a['known_touched_bytes'])<1024*1024


def check():
    d=json.loads((ROOT/'dev-docs/research/results/032-netbsd-headroom.json').read_text())
    assert d['schema']==1 and d['target']=='aarch64-unknown-netbsd'
    assert d['guest']['vcpus']==2 and d['guest']['configured_ram_bytes']==1024**3
    assert 900*1024**2<d['guest']['physical_ram_bytes']<=1024**3
    assert len(d['execution_sources'])>=60 and d['native_verified_source_count']>=45
    for name,sha in d['execution_sources'].items():
        assert hashlib.sha256((ROOT/name).read_bytes()).hexdigest()==sha,'Executed input changed: '+name
    for binary in d['native_binaries'].values():
        assert re.fullmatch('[0-9a-f]{64}',binary['sha256']) and binary['bytes']>0
    assert d['native_binaries']['faction-ssh']['sha256']==d['native_binaries']['faction-ssh-minimal']['sha256']
    boots=[]
    for profile,count in [('compact',1),('time',3)]:
        assert len(d['trials'][profile])==count
        for index,run in enumerate(d['trials'][profile]):
            boots.append(run['boot_metadata'][0]);assert run['model']=='shared' and run['allocator_profile']==('compact' if profile=='time' else profile)
            assert run['execution_source_set']=='dispatch'
            assert run['failure'] is None and run['fresh_login_after_load']=='pass'
            assert run['backend_binary_sha256']==d['native_binaries']['shared-bbs']['sha256']
            assert run['frontend_binary_sha256']==d['native_binaries']['faction-ssh']['sha256']
            a=run['allocator_readback'];assert a['version'].startswith('5.3.0') and a['config_stats']
            assert a['narenas']==1 and a['tcache'] is False and a['dirty_decay_ms']==a['muzzy_decay_ms']==0
            assert a['retain']==(profile=='retained')
            assert run['fixture_limits']=={'original_maxproc':1044,'original_maxfiles':3404,'experimental_maxproc':2048,'experimental_maxfiles':16384,'worker_limit':250,'launch_refill_ms':100,'launch_burst':8,'credential_slots':2}
            assert run['seed']=={'approved_accounts':1000,'rooms':50,'posts':100000,'post_body_bytes':1024,'independent_password_salts':True}
            for stage in run['stages']:
                n=stage['sessions'];assert stage['all_alive'] and stage['distinct_accounts']==n
                for s in stage['samples']:
                    assert s['vm_bytes']['swap_used']==0
                    if n:
                        assert s['roles']['frontend']['count']==n and s['roles']['ssh']['count']>=2*n+1
                        assert s['roles']['broker']['count']==1 and s['roles'].get('member',{}).get('count',0)==0
            for decision in run['guard_decisions']:
                result=sample_guard(decision['snapshot'])
                assert all(decision[k]==result[k] for k in result)
                assert decision['shortfall_bytes']==max(0,128*1024**2-result['reclaimable_headroom_bytes'])
            if run['outcome']=='stopped_at_headroom_guard':assert any(not x['passed'] for x in run['guard_decisions'])
            else:assert run['outcome']=='completed' and run['completed_stage']==250
            assert run['highest_observed_approved_sessions']<=250
            assert run['load_stop']['workers_and_frontends_reaped'] and run['load_stop']['seconds']<12
            assert run['load_stop']['admitted_sessions']==run['highest_observed_approved_sessions']
            for batch in run['admission_batches']:
                assert 0<batch['admitted_in_batch']<=8 and batch['admitted_after']-batch['admitted_before']==batch['admitted_in_batch']
            assert len(run['calibrations'])==3
            for c,amount in zip(run['calibrations'],[0,19,38]):
                assert c['calibration']['known_touched_bytes']==amount*1024**2;calibration(c)
            for readback in run['live_process_limits']:
                n=readback['sessions'];p=dict(readback['readback']);assert p['resources']==RESOURCES
                assert p['process_columns']==['role','rss_bytes','virtual_bytes','limit_profile','count']
                assert all(len(x)==5 and type(x[4]) is int and 1<=x[4]<=4096 and type(x[3]) is int and 0<=x[3]<len(p['limit_profiles']) for x in p['process_rows'])
                assert len({tuple(x[:4]) for x in p['process_rows']})==len(p['process_rows'])
                p['process_rows']=[x[:4] for x in p['process_rows'] for _ in range(x[4])]
                p['process_columns']=p['process_columns'][:4]
                raw={'resources':RESOURCES,'processes':[{'role':x[0],'rss_bytes':x[1],'virtual_bytes':x[2],'limits':p['limit_profiles'][x[3]]} for x in p['process_rows']]}
                assert live_limits(raw)==p
                roles=Counter(x[0] for x in p['process_rows'])
                assert roles['frontend']==n and roles['broker']==roles['ssh_listener']==1
                assert roles['ssh_privileged']==roles['ssh_session']==n
                assert roles['owner_tool']==(2 if readback['phase']=='two_owner_password_prompts' else 0)
            for c in run['high_load_credentials']:
                stream(c);n=c['cohort_in_chat']+2
                assert n>=200 and c['successful_credentials']==c['failed_credentials']==2
                assert c['hidden_input']=='pass' and c['failed_transports_reaped']
                assert c['after_failure_snapshot']['roles']['frontend']['count']==n-2
                assert any(x['sessions']==n and x.get('phase')=='after_two_credentials' for x in run['deliveries'])
            if run['completed_stage']==250:
                assert len(run['owner_credentials'])==1 and len(run['loaded_controls'])==3
                owner=run['owner_credentials'][0];rows=stream(owner)
                assert owner['cohort_in_chat']==250 and owner['successful_owner_credentials']==2 and owner['hidden_input']=='pass'
                assert all(x['roles.frontend.count']==250 and x['roles.ssh.count']>=501 for x in rows)
                assert any(x.get('phase')=='after_two_owner_credentials' for x in run['deliveries'])
                for c,amount in zip(run['loaded_controls'],[0,19,38]):
                    assert c['cohort_in_chat']==250 and c['calibration']['known_touched_bytes']==amount*1024**2;calibration(c)
            expected_stress=[]
            for kind,runs in [('member_credentials',run['high_load_credentials']),('owner_credentials',run['owner_credentials']),('loaded_controls',run['loaded_controls'])]:
                for i,observed in enumerate(runs):
                    rows=stream(observed);guards=[sample_guard(summarized(x)) for x in rows]
                    expected_stress.append({'kind':kind,'run':i,'minimum':min(g['reclaimable_headroom_bytes'] for g in guards),'all_sampled_guards_passed':all(g['passed'] for g in guards)})
            assert run['stress_guard_results']==expected_stress
            if profile=='time':
                assert run['outcome']=='completed' and run['completed_stage']==250
                assert all(x['passed'] for x in run['guard_decisions'])
                assert all(x['all_sampled_guards_passed'] for x in run['stress_guard_results'])
                assert all(sample_guard(s)['passed'] for stage in run['stages'] for s in stage['samples'])
            if 'slab_geometry' in run:
                g=run['slab_geometry'];assert g['page_bytes']==4096 and len(g['bins'])==36
                for size,regions,slab,shards in g['bins']:
                    assert regions==slab//size and slab%4096==0 and shards==1
                    if profile=='packed':assert slab==((size+4095)//4096)*4096
            if profile=='dss':assert run['dss_readback']=={'dss':'primary'}
            if profile=='time':
                assert run['host_profile']=='pageable-time; all base services retained'
                for host in run['host_service_readbacks'].values():
                    times=[row for row in host['processes'] if row[0]=='ntpd']
                    assert times and all(x[3]==0 for x in times)
                    assert {'syslogd','dhcpcd','cron','inetd','master','mdnsd'}<=set(x[0] for x in host['processes'])
            if index==0:
                assert [x['state'] for x in run['transport_states']]==['authenticated_transport_no_channel','handed_off_password_prompt']
    assert len(boots)==len(set(boots))==4
    sets=d['source_sets']
    assert set(sets)=={'base','slab','dss','host','dispatch'}
    assert set(sets['base'])<set(sets['slab'])<set(sets['dss'])<set(sets['host'])<set(sets['dispatch'])==set(d['execution_sources'])
    audit=d['time_daemon_comparison']
    locked=next(x for x in audit['locked']['host']['processes'] if x[0]=='ntpd' and x[3]>0)
    assert locked[3]>=18*1024**2
    assert all(x[3]==0 for x in audit['unlocked']['host']['processes'] if x[0]=='ntpd')
    assert audit['locked']['ntp_config_sha256']==audit['restored']['ntp_config_sha256']!=audit['unlocked']['ntp_config_sha256']
    assert audit['unlocked']['snapshot']['vm_bytes']['wired']<1024**2
    restored=audit['restored']['allocator'];assert restored['narenas']==8 and restored['tcache'] and restored['dirty_decay_ms']==10000 and restored['retain'] is False
    regression=d['regression_profile'];assert regression['allocator']['narenas']==1 and regression['allocator']['tcache'] is False
    assert regression['ntp_config_sha256']==audit['unlocked']['ntp_config_sha256']
    for name in ['host_before','host_after']:
        host=regression[name]['processes'];assert any(x[0]=='ntpd' for x in host) and all(x[3]==0 for x in host if x[0]=='ntpd')
        assert {'syslogd','dhcpcd','cron','inetd','master','mdnsd'}<=set(x[0] for x in host)
    for label,cycles in [('cached',1),('isolated',3)]:
        p=d['reentry_regression'][label];assert p['completed_cycles']==cycles and p['requested_cycles']==3 and p['fresh_unique_markers']
    b=d['build_isolation_verification'];assert b['same_source_after_forced_rebuild_matches_isolated']
    assert b['cached_sha256']!=b['forced_rebuild_sha256']==b['isolated_rebuild_sha256']==d['native_binaries']['shared-bbs-pre-dispatch']['sha256']
    earlier=d['exploratory_trials'];assert set(earlier)=={'compact','retained','packed','dss','time'}
    earlier_boots=[]
    for profile,count in [('compact',2),('retained',1),('packed',1),('dss',1),('time',3)]:
        assert len(earlier[profile])==count
        for i,run in enumerate(earlier[profile]):
            earlier_boots.append(run['boot_metadata'][0])
            assert run['execution_source_set'] in sets and run['execution_source_set']!='dispatch'
            a=run['allocator_readback'];assert a['narenas']==1 and a['tcache'] is False
            assert a['dirty_decay_ms']==a['muzzy_decay_ms']==0 and a['retain']==(profile=='retained')
            if 'slab_geometry' in run:
                g=run['slab_geometry'];assert g['page_bytes']==4096 and len(g['bins'])==36
                for size,regions,slab,shards in g['bins']:
                    assert regions==slab//size and slab%4096==0 and shards==1
                    if profile=='packed':assert slab==((size+4095)//4096)*4096
            if profile=='dss':assert run['dss_readback']=={'dss':'primary'}
            stress=[]
            for kind,runs in [('member_credentials',run['high_load_credentials']),('owner_credentials',run['owner_credentials']),('loaded_controls',run['loaded_controls'])]:
                for j,observed in enumerate(runs):
                    guards=[sample_guard(summarized(row)) for row in stream(observed)]
                    stress.append({'kind':kind,'run':j,'minimum':min(g['reclaimable_headroom_bytes'] for g in guards),'all_sampled_guards_passed':all(g['passed'] for g in guards)})
            assert stress==run['stress_guard_results']
            assert run['backend_binary_sha256']==d['native_binaries']['shared-bbs-pre-dispatch']['sha256']
            assert run['load_stop']['workers_and_frontends_reaped'] and run['fresh_login_after_load']=='pass'
            for decision in run['guard_decisions']:
                assert all(decision[k]==v for k,v in sample_guard(decision['snapshot']).items())
            for kind in ['calibrations','high_load_credentials','owner_credentials','loaded_controls']:
                for observed in run[kind]:
                    stream(observed)
                    if 'calibration' in observed:calibration(observed)
            if profile=='time' and i==2:
                assert run['outcome']=='stopped_at_functional_failure' and run['completed_stage']==200
                assert run['failure']['observed_response']=='credential work busy; reconnect later'
                assert 'detail' not in run['failure']
            else:assert run['failure'] is None
    assert len(earlier_boots)==len(set(earlier_boots))==8 and len(set(boots+earlier_boots))==12
    fixed=d['dispatcher_reentry'];assert fixed['completed_cycles']==fixed['requested_cycles']==3 and fixed['result']=='pass'
    assert fixed['binary_sha256']==d['native_binaries']['shared-bbs']['sha256']
    assert fixed['binary_sha256']!=d['native_binaries']['shared-bbs-pre-dispatch']['sha256']
    pilot=d['excluded_cached_build_pilot'];assert pilot['completed_stage']==32 and pilot['highest_observed_approved_sessions']==100
    assert pilot['failure_phase']=='chat_delivery' and pilot['reaped'] and pilot['fresh_login']=='pass'
    assert len(d['minimal_frontend_oracles']['results'])==20 and all(x['result']=='pass' for x in d['minimal_frontend_oracles']['results'])
    checks=d['regressions'];assert checks['rust_tests']==58 and checks['dispatcher_tests']==1 and checks['pty_workflows']==5
    for name,count in [('ssh',20),('lifecycle',13),('shared',16)]:
        assert len(checks[name]['results'])==count and all(x['result']=='pass' for x in checks[name]['results'])
    assert d['restored_kernel_limits']==['1044','3404'] and d['kernel_oom_events']==0
    assert d['final_backend_and_frontends_reaped'] and d['final_fixture_integrity']=={'integrity':'ok','accounts':1001,'unique_hashes':1001,'rooms':50,'posts':100000}
    assert d['final_metrics']['roles']['ssh']['count']==1
    assert all(d['final_metrics']['roles'].get(k,{}).get('count',0)==0 for k in ['member','frontend','broker'])
    assert all(d['cleanup'][k] for k in ['guest_stopped','private_inputs_removed','forwards_closed'])
    print('Saved native headroom evidence passed: exact unchanged guards, repeat boots, complete SSH costs, live limits, calibrated member/owner workload, build isolation, failures/recovery and cleanup. No VM rerun or supported-capacity claim.')


if __name__=='__main__':check()
