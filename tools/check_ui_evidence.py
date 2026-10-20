#!/usr/bin/env python3
"""Validate saved full shared UI evidence; does not rerun a NetBSD VM."""
import hashlib,json,re
from pathlib import Path
from headroom_memory import sample_guard,RESOURCES,ROLES

ROOT=Path(__file__).resolve().parents[1]

def unpack(stream):
    columns=stream['sample_columns'];rows=stream['sample_rows']
    assert stream['sample_count']==len(rows)==150
    assert len(columns)==len(set(columns))
    samples=[]
    for row in rows:
        assert len(row)==len(columns),'incomplete sample row'
        sample={}
        for name,value in zip(columns,row):
            assert type(value) in (int,float) and value>=0
            target=sample;parts=name.split('.')
            for part in parts[:-1]:target=target.setdefault(part,{})
            target[parts[-1]]=value
        samples.append(sample)
    assert all(a['finished_ns']<=b['monotonic_ns'] for a,b in zip(samples,samples[1:]))
    assert all(s['finished_ns']>=s['monotonic_ns'] for s in samples)
    assert stream['markers']['start_ns']<=samples[0]['monotonic_ns']<samples[-1]['finished_ns']<=stream['markers']['stop_ns']
    return samples

def validate_trial(trial,binary):
    assert trial['binary_sha256']==binary
    t=trial['measurement'];assert t['outcome']=='completed' and t['failure'] is None
    assert t['completed_stage']==t['highest_observed_approved_sessions']==250
    assert t['model']=='shared' and t['fresh_login_after_load']=='pass'
    assert t['seed']=={'approved_accounts':1000,'rooms':50,'posts':100000,'post_body_bytes':1024,'independent_password_salts':True}
    limits=t['fixture_limits'];assert limits['worker_limit']==250 and limits['credential_slots']==2 and limits['launch_refill_ms']==100 and limits['launch_burst']==8
    assert limits['experimental_maxproc']==2048 and limits['experimental_maxfiles']==16384
    assert t['load_stop']['workers_and_frontends_reaped'] and t['load_stop']['seconds']<12
    assert all(x['passed'] for x in t['guard_decisions'])
    stages=t['stages'];assert [(s['sessions'],s['phase']) for s in stages]==[(0,'seeded_idle_baseline')]+[(n,phase) for n in [32,100,200,250] for phase in ['approved_menu_idle','local_chat_idle']]
    for s in stages:
        assert s['all_alive'] and s['distinct_accounts']==s['sessions']
        for sample in s['samples']:
            assert sample_guard(sample)['passed']
            if s['sessions']:
                roles=sample['roles'];assert roles['broker']['count']==1 and roles['frontend']['count']==s['sessions']
                assert roles.get('member',{}).get('count',0)==0
                assert roles['ssh']['count']>=2*s['sessions']+1
    assert [s['cohort_in_chat'] for s in t['high_load_credentials']]==[198,248]
    assert all(s['successful_credentials']==s['failed_credentials']==2 and s['failed_transports_reaped'] and s['hidden_input']=='pass' for s in t['high_load_credentials'])
    assert len(t['owner_credentials'])==1 and t['owner_credentials'][0]['cohort_in_chat']==250 and t['owner_credentials'][0]['successful_owner_credentials']==2
    assert len(t['full_ui_at_250'])==1
    ui=t['full_ui_at_250'][0];assert ui['connected_members']==250 and ui['peers_in_chat']==249
    assert all(ui[k]=='pass' for k in ['room_list','selected_room_page','post_submission','return_to_live_chat'])
    minimum=[]
    for kind in ['calibrations','high_load_credentials','owner_credentials','loaded_controls','full_ui_at_250']:
        for stream in t[kind]:
            samples=unpack(stream)
            guards=[sample_guard(s) for s in samples]
            assert all(g['passed'] for g in guards),'unchanged stress guard failed'
            minimum.extend(g['reclaimable_headroom_bytes'] for g in guards)
            if 'calibration' in stream:
                calibration=stream['calibration'];assert calibration['fully_inside_samples']>=1
                assert abs(calibration['observed_anon_delta_bytes']-calibration['known_touched_bytes'])<=1024*1024
    for kind in ['calibrations','loaded_controls']:
        assert [s['calibration']['known_touched_bytes'] for s in t[kind]]==[0,19*1024*1024,38*1024*1024]
    assert all(s['all_sampled_guards_passed'] and s['minimum']>=128*1024*1024 for s in t['stress_guard_results'])
    assert len(t['live_process_limits'])==9
    for entry in t['live_process_limits']:
        readback=entry['readback'];assert readback['resources']==RESOURCES
        assert readback['process_columns']==['role','rss_bytes','virtual_bytes','limit_profile','count']
        for role,rss,virtual,profile,count in readback['process_rows']:
            assert role in ROLES and rss>=0 and virtual>=0 and count>=1 and 0<=profile<len(readback['limit_profiles'])
        for profile in readback['limit_profiles']:
            assert len(profile)==12 and all(len(pair)==2 for pair in profile)
        assert sum(row[4] for row in readback['process_rows'] if row[0]=='frontend')==entry['sessions']
    allocator=t['allocator_readback'];assert (allocator['narenas'],allocator['tcache'],allocator['retain'],allocator['dirty_decay_ms'],allocator['muzzy_decay_ms'])==(1,False,False,0,0)
    for host in t['host_service_readbacks'].values():
        assert host['columns']==['service','rss_bytes','virtual_bytes','locked_mapping_virtual_bytes']
        rows=host['processes'];assert all(row[3]==0 for row in rows if row[0]=='ntpd')
        assert {'ntpd','syslogd','dhcpcd','cron','inetd','master','mdnsd'}<={row[0] for row in rows}
    return min(minimum)

def check():
    d=json.loads((ROOT/'dev-docs/research/results/033-netbsd-shared-ui.json').read_text())
    assert d['schema']==1 and d['target']=='aarch64-unknown-netbsd'
    assert d['guest']['vcpus']==2 and d['guest']['configured_ram_bytes']==1073741824
    assert d['reference_commit']=='76abb2f326698e1c75c7c7b366f36dc4612c3bf8'
    for name,digest in d['execution_sources'].items():
        assert re.fullmatch('[a-f0-9]{64}',digest)
        assert hashlib.sha256((ROOT/name).read_bytes()).hexdigest()==digest,'executed source drift: '+name
    for name,digest in d['native_sources'].items():assert digest==d['execution_sources'][name]
    for folder in ['crates/faction-core','crates/faction-terminal','crates/faction-access','experiments/055-shared-ui']:
        assert any(name.startswith(folder+'/') for name in d['native_sources'])
    for digest in d['binaries'].values():assert re.fullmatch('[a-f0-9]{64}',digest)
    binary=d['binaries']['shared_ui'];oracles=d['validation']['shared_ui']
    assert oracles['binary_sha256']==binary and len(oracles['results'])==27 and all(x['result']=='pass' for x in oracles['results'])
    assert oracles['fairness_seconds'] and max(oracles['fairness_seconds'])<2
    for name,count in [('ssh',20),('lifecycle',13),('frontend',20)]:
        result=d['validation'][name];rows=result['results'];assert len(rows)==count and all(x['result']=='pass' for x in rows)
    for profile in d['validation']['native_runtime_profile'].values():
        assert profile['frontend_sha256']==d['binaries']['frontend_minimal']
        assert profile['allocator']['narenas']==1 and profile['allocator']['tcache'] is False
        assert any(row[0]=='ntpd' for row in profile['host']['processes'])
        assert all(row[3]==0 for row in profile['host']['processes'] if row[0]=='ntpd')
    assert d['validation']['native_workspace_tests']==58 and d['validation']['native_dispatch_tests']==1
    assert len(d['trials'])==3 and len({str(t['boot']) for t in d['trials']})==3
    minima=[validate_trial(t,binary) for t in d['trials']]
    settings=d['settings'];assert settings['restored']['ntp_config_sha256']==settings['original']['ntp_config_sha256']
    assert settings['restored']['allocator']['narenas']==8 and settings['restored']['allocator']['tcache'] is True
    assert any(row[0]=='ntpd' and row[3]>0 for row in settings['restored']['host']['processes'])
    assert d['final_fixture']=={'integrity':'ok','accounts':1001,'unique_hashes':1001,'rooms':50,'posts':100000}
    assert all(d['cleanup'][k] for k in ['guest_stopped','private_inputs_removed','forwards_closed'])
    print('Saved shared UI evidence passed: 27 native UI oracles, full SSH costs, three 250-member boots and unchanged sampled headroom guard; minimum %.2f MiB.'%(min(minima)/1024/1024))

if __name__=='__main__':check()
