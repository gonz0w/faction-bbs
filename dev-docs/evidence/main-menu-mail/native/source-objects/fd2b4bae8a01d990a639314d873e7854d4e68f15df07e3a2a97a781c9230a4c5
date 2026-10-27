#!/usr/bin/env python3
"""Validate saved bounded-key native evidence, current sources and full guards."""
import json,re
from check_integrated_evidence import ROOT,validate_sources
from check_ui_evidence import validate_trial
from check_confinement_evidence import validate_limits
from check_sustained_evidence import expand
from headroom_memory import sample_guard

def activity(a):
    assert a['outcome']=='completed' and a['failure'] is None and a['requested_seconds']==180 and a['seconds']>=180
    assert a['same_listener_and_inode'] and not a['listener_reset'] and not a['registry_reset'] and a['effective_source_penalties']!='no'
    assert a['before']['active']==a['after']['active']==a['final_distinct_members']==250
    assert len(a['rounds'])>=100
    rounds=a['rounds'];assert rounds[-1]['at_seconds']>=178 and all(x['at_seconds']<y['at_seconds'] for x,y in zip(rounds,rounds[1:]))
    assert all(r['members'] in [249,250] and r['confirmed_peer_receipts']==r['members']-1 and 0<r['seconds']<10 for r in rounds)
    assert any(r['members']==249 for r in rounds) and rounds[0]['members']==rounds[-1]['members']==250
    attack=a['abuse'];assert attack['mode']=='full' and attack['cohort']==8 and not attack['authenticated']
    assert attack['before']['active']==attack['after']['active']==250 and attack['charged_lookups']==0
    assert attack['after']['full']-attack['before']['full']==24
    assert len(attack['rows'])==8 and all([q['response'] for q in row]==['denied','denied','disconnected'] for row in attack['rows'])
    attempts=a['same_source_attempts'];assert attempts and not attempts[0]['accepted'] and attempts[-1]['accepted']
    assert len(attempts)<=36 and all(0<t['at_seconds']<180 for t in attempts)
    stream=a['native_stream'];samples=expand(stream)
    assert stream['sample_count']==len(samples)==185 and stream['requested_interval_ms']==1000
    assert stream['guest_seconds']>=185 and 0<stream['gap_ms']['min']<=stream['gap_ms']['max']<5000
    assert stream['markers']['start_ns']<=samples[0]['monotonic_ns']<samples[-1]['finished_ns']<=stream['markers']['stop_ns']
    assert all(s['finished_ns']>=s['monotonic_ns'] for s in samples)
    assert all(x['finished_ns']<=y['monotonic_ns'] for x,y in zip(samples,samples[1:]))
    assert all(249<=s['roles']['frontend']['count']<=250 and s['roles']['broker']['count']==1 and s['roles'].get('member',{}).get('count',0)==0 and 499<=s['roles']['ssh']['count']<=525 for s in samples)
    guards=[sample_guard(s) for s in samples];assert all(g['passed'] for g in guards),'lookup activity guard failed'
    return dict(minimum_bytes=min(g['reclaimable_headroom_bytes'] for g in guards),peer_receipts=sum(r['confirmed_peer_receipts'] for r in rounds),rounds=len(rounds),same_source_recovery_seconds=attempts[-1]['at_seconds'])

def validate(d,root=ROOT,allow_pending_cleanup=False):
    assert d['schema']==1 and d['target']=='aarch64-unknown-netbsd' and d['reference_commit']=='71491cc61bf4a6606ec65bb73ad72ed585f18927'
    assert d['guest']['vcpus']==2 and d['guest']['configured_ram_bytes']==1073741824
    validate_sources(d['execution_sources'],root=root,historical=True);assert d['native_sources']==d['execution_sources']
    for name in ['native/faction-key-admission/main.c','tools/check_key_admission_config.py']+['tools/'+n+'.py' for n in ['lookup_probe','lookup_workload','lookup_trace','check_lookup_state','check_lookup_options','check_lookup_restart','check_lookup_scaling','check_lookup_host']]:assert name in d['execution_sources']
    assert d['contract']==dict(lookup_allowance=4,max_auth_tries=3,default_capacity=8,default_refill_ms=5000,experimental_capacity=250,key_refill_ms=250,application_refill_ms=100,credential_slots=2,headroom_minimum_bytes=134217728,swap_allowed=False,state_version=2,state_bytes=8104,active_interval_seconds=180)
    assert all(re.fullmatch('[a-f0-9]{64}',h) for h in d['binaries'].values())
    for n,path in [('key_preflight','tools/check_key_admission_config.py'),('key_rc','deploy/netbsd/faction_key_admission'),('isolation_preflight','tools/prepare_isolation.py'),('isolation_rc','deploy/netbsd/faction_isolated')]:assert d['installed_sources'][n]==d['execution_sources'][path]
    v=d['validation'];assert v['native_workspace_tests']==63
    assert v['hardening']==dict(native_analyzer='pass',pie='pass',relro='pass',immediate_binding='pass',no_rwe_load='pass')
    for n,count in [('admission',14),('channels',6),('shared_ui',29),('state',6),('options',2),('restart',1),('normal_frontend',31),('ssh',20),('worker_lifecycle',13)]:
        rows=v[n]['results'];assert len(rows)==len({r['oracle'] for r in rows})==count and all(r['result']=='pass' for r in rows)
    assert v['admission']['binary_sha256']==d['binaries']['key_helper'] and v['shared_ui']['binary_sha256']==d['binaries']['service']
    assert v['state']['migration']==dict(old_bytes=6104,new_bytes=8104,same_inode=True,quiescent=True,implicit=False)
    assert v['options']['key_file_restored_exactly'] and v['options']['transport_only'] and 1<=v['options']['signed_client_charged_lookups']<=4
    assert [q['response'] for q in v['options']['options_rows']]==['denied','denied','disconnected']
    restart=v['restart'];assert restart['same_inode'] and restart['before']['active']==1 and restart['after']['active']==0
    assert restart['before']['reuses']==restart['after']['reuses']==3 and restart['before']['launches']==restart['after']['launches']==1
    assert [q['response'] for q in restart['rows']]==['key_ok']*4+['denied','denied','disconnected']
    for mode in ['known','unknown','bad_signature']:
        c=d['cohorts'][mode];p=c['probe'];t=c['trace'];assert p['mode']==mode and p['cohort']==8 and not p['authenticated'] and p['after']['active']==p['before']['active']==0
        responses=['key_ok']*4+['denied','denied','disconnected'] if mode=='known' else ['denied','denied','disconnected']
        assert len(p['rows'])==8 and all([q['response'] for q in row]==responses for row in p['rows'])
        returns=32 if mode=='known' else 24;executions=56 if mode=='known' else 24
        assert p['charged_lookups']==returns and t['helper_executions']==executions
        assert sum(r['helpers'] for r in t['classes'])==executions
        for k in ['fsync','pwrite','ftruncate','key_path_lookups','write']:assert t['totals'][k]==returns
        assert t['totals']['sysctl']==(476 if mode=='known' else 220)
        for k,total in t['totals'].items():assert total==sum(r[k]*r['helpers'] for r in t['classes'])
        if mode=='known':assert sum(r['helpers'] for r in t['classes'] if r['fsync']==r['pwrite']==r['key_path_lookups']==0)==24
    assert len(d['trials'])==2 and len({str(t['boot']) for t in d['trials']})==2
    facts=[]
    for trial in d['trials']:
        stress=validate_trial(trial,d['binaries']['service']);m=trial['measurement'];validate_limits(m)
        assert m['admission_after']['full']==26 and m['admission_after']['peak']==250 and m['admission_after']['rate']==0
        assert m['admission_before']['lookup_limit']==4 and m['admission_before']['version']==2
        assert len(m['lookup_activity'])==1;f=activity(m['lookup_activity'][0]);f['stress_minimum_bytes']=stress;facts.append(f)
        for entry in m['live_process_limits']:assert entry['readback']['ssh_zombie_processes']==0
    client=d['research_client'];assert client['paramiko']['version']=='4.0.0'
    assert all(p['installed_files_sha256'] and all(re.fullmatch('[a-f0-9]{64}',h) for h in p['installed_files_sha256'].values()) for p in client.values())
    context=d['fixture_context'];assert context['second_boot_retains_first_posts'] and context['invitation_keys']==1 and not context['maximum_key_file_workload_tested']
    for i,k in enumerate(['stopped_after_first','stopped_after_second'],1):
        row=context[k];assert row==dict(integrity='ok',accounts=1002,unique_hashes=1002,rooms=52,posts=100004+i,sustained_posts=0,load_posts=i)
    assert d['fixture_corrections']==[dict(kind='invitation-key fixture restoration mode',initial_mode='0600',outcome='helper failed closed; signed client denied',correction='preserve readable original mode and exact bytes; assert helper readability before option probe')]
    settings=d['settings'];assert settings['original']['ntp_config_sha256']==settings['restored']['ntp_config_sha256']
    for k in ['max_files','max_processes']:assert settings['original']['snapshot']['limits'][k]==settings['restored']['snapshot']['limits'][k]
    assert settings['restored']['allocator']['narenas']==8 and settings['restored']['allocator']['tcache'] is True
    assert d['restored_isolation']==dict(profile_disabled=True,npf_inactive_empty=True,experimental_npf_boot_removed=True,global_allocator_removed=True,sealed_allocator_removed=True,frontend_setuid_removed=True,profile_mounts_absent=True,usermount=0)
    assert d['restored_admission']==dict(disabled=True,no_live_member_transports=True,normal_path_passed=True)
    assert d['final_fixture']==dict(integrity='ok',accounts=1001,unique_hashes=1001,rooms=50,posts=100000)
    if not allow_pending_cleanup:assert all(d['cleanup'][k] for k in ['guest_stopped','private_inputs_removed','forwards_closed'])
    return facts

def check():
    d=json.loads((ROOT/'dev-docs/research/results/039-netbsd-bounded-lookups.json').read_text());facts=validate(d)
    print('Bounded-key evidence passed: 56 helpers/32 reads+commits in eight known-key probes, two fresh 250-member matrices; minimum stress %.2f MiB.'%(min(f['stress_minimum_bytes'] for f in facts)/1048576))
if __name__=='__main__':check()
