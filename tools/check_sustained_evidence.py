#!/usr/bin/env python3
"""Validate saved active/abuse evidence; never claims to rerun the NetBSD VM."""
import json
import re
from check_integrated_evidence import ROOT, validate_sources
from check_ui_evidence import validate_trial
from check_confinement_evidence import validate_limits
from headroom_memory import sample_guard


def expand(stream):
    if 'samples' in stream:return stream['samples']
    columns=stream['sample_columns'];rows=stream['sample_rows']
    assert len(columns)==len(set(columns)) and len(rows)==stream['sample_count']
    samples=[]
    for row in rows:
        assert len(row)==len(columns)
        sample={}
        for name,value in zip(columns,row):
            assert type(value) in (int,float) and value>=0
            target=sample;parts=name.split('.')
            for part in parts[:-1]:target=target.setdefault(part,{})
            target[parts[-1]]=value
        samples.append(sample)
    return samples


def active_facts(active):
    assert active['outcome']=='completed' and active['failure'] is None
    assert active['requested_seconds']==900 and active['active_seconds']>=900
    rounds=active['rounds'];assert len(rounds)>=600
    assert rounds[-1]['at_seconds']>=898
    assert all(a['at_seconds']<b['at_seconds'] for a,b in zip(rounds,rounds[1:]))
    assert all(r['recipients']==250 and 0<r['seconds']<10 for r in rounds)
    latencies=sorted(r['seconds'] for r in rounds)
    stats=active['delivery_latency_seconds']
    assert stats=={'p50':latencies[int((len(latencies)-1)*.5)],
                  'p95':latencies[int((len(latencies)-1)*.95)],'max':max(latencies)}
    ops=active['operations'];ui=[o for o in ops if o['kind']=='room_read_post_menu_input_burst']
    logins=[o for o in ops if o['kind']=='fresh_member_credential']
    assert len(ui)>=20 and len(logins)>=10 and len(ops)==len(ui)+len(logins)
    assert all(o['invalid_keys']==2048 and o['other_chat_recipients']==249 and 0<o['witness_seconds']<10 for o in ui)
    assert all(o['distinct_accounts']==250 and o['seconds']>0 for o in logins)
    assert active['final_distinct_members']==250
    abuse=active['abuse'];assert abuse['silent_accepted']==8 and abuse['excess_drops']>=16
    assert 720<=abuse['at_seconds']<800 and not abuse['listener_reset']
    # Witnesses must continue both during grace and after the silent cohort dies.
    assert sum(abuse['at_seconds']<=r['at_seconds']<=abuse['at_seconds']+30 for r in rounds)>=20
    assert sum(r['at_seconds']>=abuse['at_seconds']+40 for r in rounds)>=60
    stream=active['native_stream'];samples=expand(stream)
    assert stream['sample_count']==len(samples)==905 and stream['requested_interval_ms']==1000
    assert stream['guest_seconds']>=905 and 0<stream['gap_ms']['max']<5000
    assert all(a['finished_ns']<=b['monotonic_ns'] for a,b in zip(samples,samples[1:]))
    assert stream['markers']['start_ns']<=samples[0]['monotonic_ns']<samples[-1]['finished_ns']<=stream['markers']['stop_ns']
    for s in samples:
        roles=s['roles'];assert roles['broker']['count']==1 and roles.get('member',{}).get('count',0)==0
        assert 249<=roles['frontend']['count']<=250 and 499<=roles['ssh']['count']<=517
    assert max(s['roles']['ssh']['count'] for s in samples)>=515, 'loaded preauth costs were not sampled'
    cpu=active['whole_vm_interval']['whole_vm_cpu_percent_one_core'];assert 0<=cpu<=200
    guards=[sample_guard(s) for s in samples]
    return {'rounds':len(rounds),'console_observations':250*len(rounds),'peer_deliveries':249*len(rounds),'ui_cycles':len(ui),'fresh_logins':len(logins),
            'minimum_bytes':min(g['reclaimable_headroom_bytes'] for g in guards),
            'all_guards_passed':all(g['passed'] for g in guards),'cpu_percent_one_core':cpu}


def validate(d,root=ROOT,allow_pending_cleanup=False):
    assert d['schema']==1 and d['target']=='aarch64-unknown-netbsd'
    assert d['reference_commit']=='b672abf54c77d13dae784de55f837a7905f2b5b4'
    assert d['guest']['vcpus']==2 and d['guest']['configured_ram_bytes']==1073741824
    validate_sources(d['execution_sources'],root=root,historical=True);assert d['native_sources']==d['execution_sources']
    for name in ['check_sustained_host','check_sustained_scaling','sustained_workload','sustained_stream','check_unsigned_keys','check_source_fairness']:
        assert 'tools/'+name+'.py' in d['execution_sources']
    assert all(re.fullmatch('[a-f0-9]{64}',v) for v in d['binaries'].values())
    for key,path in [('isolation_preflight','tools/prepare_isolation.py'),('isolation_rc','deploy/netbsd/faction_isolated'),('key_preflight','tools/check_key_admission_config.py'),('key_rc','deploy/netbsd/faction_key_admission')]:
        assert d['installed_sources'][key]==d['execution_sources'][path]
    assert d['contract']==dict(default_reservations=8,default_refill_ms=5000,experimental_reservations=250,key_refill_ms=250,application_refill_ms=100,credential_slots=2,headroom_minimum_bytes=134217728,swap_allowed=False,active_interval_seconds=900)
    assert d['validation']['native_workspace_tests']==63
    for name,count in [('admission',14),('channels',6),('shared_ui',29),('ssh',20),('worker_lifecycle',13)]:
        rows=d['validation'][name]['results'];assert len(rows)==len({r['oracle'] for r in rows})==count
        assert all(r['result']=='pass' for r in rows)
    assert len(d['validation']['normal_frontend']['results'])==31
    assert all(r['result']=='pass' for r in d['validation']['normal_frontend']['results'])
    client=d['research_client'];assert set(client)=={'paramiko','bcrypt','cryptography','invoke','pynacl','cffi','pycparser'}
    assert client['paramiko']['version']=='4.0.0'
    for package in client.values():
        assert package['installed_files_sha256']
        assert all(re.fullmatch('[a-f0-9]{64}',h) for h in package['installed_files_sha256'].values())
    assert 'paramiko/transport.py' in client['paramiko']['installed_files_sha256']
    assert 'paramiko/auth_handler.py' in client['paramiko']['installed_files_sha256']
    query=d['unsigned_key_probe'];assert query['maximum_queries']==64 and query['maximum_seconds']==20
    assert len(query['queries'])==64 and all(q['response']=='key_ok' for q in query['queries'])
    assert not query['authenticated'] and query['signed_requests']==0 and query['client_version']=='4.0.0'
    before,during,after=(query[k] for k in ['before','during','after'])
    assert before['active']==after['active']==0 and during['active']==1
    assert during['launches']==before['launches']+1 and during['reuses']==before['reuses']+63
    assert after['launches']==during['launches'] and 0<query['seconds']<25
    cpu=d['cpu_accounting_crosscheck'];assert cpu['cpu_count']==2 and 0<=cpu['idle_index']<5
    assert [p['mode'] for p in cpu['phases']]==['sleep','busy','periodic_30ms_busy_20ms_sleep']
    for phase in cpu['phases']:
        assert 5<=phase['wall_seconds']<6 and phase['process_seconds']>=0 and phase['thread_seconds']>=0
        assert len(phase['aggregate_before'])==len(phase['aggregate_after'])==5
        assert all(b>=a for a,b in zip(phase['aggregate_before'],phase['aggregate_after']))
        assert len(phase['per_cpu_before'])==len(phase['per_cpu_after'])==2
    fairness=d['source_fairness'];assert fairness['silent_accepted']==8 and fairness['excess_drops']>=16
    assert not any(fairness[k] for k in ['listener_reset','registry_reset','source_penalties_disabled'])
    assert fairness['same_listener'] and fairness['same_registry_inode'] and fairness['effective_source_penalties']!='no'
    attempts=fairness['same_source_attempts'];assert not attempts[0]['accepted'] and attempts[-1]['accepted']
    assert all(x['new_reservations']==int(x['accepted']) for x in attempts)
    assert fairness['after']['active']==0 and all(sample_guard(s)['passed'] for s in fairness['samples'])
    assert len(d['trials'])==2 and len({str(t['boot']) for t in d['trials']})==2
    facts=[]
    for trial in d['trials']:
        assert trial['binary_sha256']==d['binaries']['service']
        t=trial['measurement'];assert len(t['sustained_activity'])==1
        fact=active_facts(t['sustained_activity'][0]);facts.append(fact)
        # Reuse the complete matched matrix checker, including every stress
        # sample and all original credential/limit/loaded-stop/recovery gates.
        validate_trial(trial,d['binaries']['service']);validate_limits(t)
        assert fact['all_guards_passed']
        assert t['admission_after']['peak']==250 and t['admission_after']['full']==2 and t['admission_after']['rate']==0
    assert d['delivery_counting']==dict(console_marker_observations=425500,non_sender_marker_receipts=423798,sender_observations=1702,sender_echo_distinguished=False,scope='native recipients counts matching consoles; exclude the sender from confirmed peer deliveries')
    assert sum(f['console_observations'] for f in facts)==425500 and sum(f['peer_deliveries'] for f in facts)==423798
    context=d['fixture_context'];assert context['first_non_load_counts']==dict(accounts=1002,rooms=52,non_load_posts=100004)
    a,b=(context[k] for k in ['stopped_after_first','stopped_after_second'])
    assert a['integrity']==b['integrity']=='ok' and context['second_boot_retains_first_posts']
    for row in [a,b]:
        assert row['accounts']==row['unique_hashes']==1002 and row['rooms']==52
        assert row['posts']==100004+row['sustained_posts']+row['load_posts']
    assert a['sustained_posts']>=20 and b['sustained_posts']>=a['sustained_posts']+20
    settings=d['settings'];assert settings['original']['ntp_config_sha256']==settings['restored']['ntp_config_sha256']
    assert settings['restored']['allocator']['narenas']==8 and settings['restored']['allocator']['tcache']
    for k in ['max_files','max_processes']:assert settings['original']['snapshot']['limits'][k]==settings['restored']['snapshot']['limits'][k]
    assert d['restored_isolation']==dict(profile_disabled=True,npf_inactive_empty=True,experimental_npf_boot_removed=True,global_allocator_removed=True,sealed_allocator_removed=True,frontend_setuid_removed=True,profile_mounts_absent=True,usermount=0)
    assert d['restored_admission']==dict(disabled=True,no_live_member_transports=True,normal_path_passed=True)
    assert d['final_fixture']==dict(integrity='ok',accounts=1001,unique_hashes=1001,rooms=50,posts=100000)
    if not allow_pending_cleanup:assert all(d['cleanup'][k] for k in ['guest_stopped','private_inputs_removed','forwards_closed'])
    return facts


def check():
    d=json.loads((ROOT/'dev-docs/research/results/038-netbsd-sustained-abuse.json').read_text())
    facts=validate(d)
    print('Sustained evidence: two fresh 250-member active intervals; %d verified non-sender receipts; unsigned lookup and source-fairness limits retained.' % sum(f['peer_deliveries'] for f in facts))


if __name__=='__main__':check()
