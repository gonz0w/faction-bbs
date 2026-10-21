#!/usr/bin/env python3
"""Check saved native global admission evidence, preserving original guards."""
import hashlib,json,re
from pathlib import Path
from check_integrated_evidence import ROOT,validate_sources
from check_ui_evidence import validate_trial
from check_sustained_evidence import expand
from check_confinement_evidence import validate_limits
from headroom_memory import sample_guard

def validate(d,root=ROOT,allow_pending_cleanup=False):
    assert d['schema']==1 and d['target']=='aarch64-unknown-netbsd'
    assert d['reference_commit']=='5b62d0a705c988e47ec2cc2ce4b74f87de600386'
    assert d['guest']['vcpus']==2 and d['guest']['configured_ram_bytes']==1073741824
    assert d['guest']['versions']['ssh'].startswith('OpenSSH_10.3 ')
    validate_sources(d['execution_sources'],root=root);assert d['native_sources']==d['execution_sources']
    for name in ['native/faction-ssh-gate/gate.c','tools/prepare_ssh_gate.py','deploy/netbsd/faction_ssh_gate','experiments/055-global-admission/maps.c','experiments/055-global-admission/exec_environment.c']+['tools/'+n+'.py' for n in ['check_global_admission','check_global_scaling','check_global_host','global_workload','check_private_peers','check_global_channels','check_global_ui','check_gate_exec']]:assert name in d['execution_sources']
    assert d['contract']==dict(default_burst=8,default_global_refill_ms=5000,default_source_burst=2,default_source_refill_ms=10000,global_state_bytes=120,source_slots=32,experimental_global_refill_ms=250,experimental_source_refill_ms=500,default_key_capacity=8,experimental_key_capacity=250,key_lookup_limit=4,credential_slots=2,headroom_minimum_bytes=134217728,swap_allowed=False,pressure_seconds=60)
    assert all(re.fullmatch('[a-f0-9]{64}',h) for h in d['binaries'].values())
    for name,path in [('gate_preflight','tools/prepare_ssh_gate.py'),('gate_rc','deploy/netbsd/faction_ssh_gate')]:assert d['installed_sources'][name]==d['execution_sources'][path]
    v=d['validation'];assert v['native_workspace_tests']==63
    for name,count in [('global',13),('shared_ui',29),('channels',6),('normal_frontend',31),('ssh',20),('worker_lifecycle',13)]:
        rows=v[name]['results'];assert len(rows)==len({r['oracle'] for r in rows})==count and all(r['result']=='pass' for r in rows)
    assert v['hardening']==dict(native_analyzer='pass',pie='pass',relro='pass',immediate_binding='pass',no_rwe_load='pass')
    audit=v['exec'];assert audit['registry_open_descriptions']==1 and audit['only_listener_holds_registry']
    assert audit['environment']['checked_member_descendants']>=3 and audit['environment']['loader_environment_leaks']==0
    assert audit['mapping']['listener_mappings']>0 and audit['mapping']['leaked_processes']==0
    observations=v['global']['observations'];burst=observations['global_burst']
    assert burst['offered']==11 and burst['admitted']==burst['listener_forks']==8 and burst['seconds']<5
    assert burst['state']['passed']==8 and 0<=burst['state']['global_drops']<=3 # rapid drop diagnostics may not have flushed yet
    assert observations['source_burst']['banners']==[True,True,False,True]
    assert observations['mapping']['listener_mappings']>0 and observations['mapping']['leaked_processes']==0
    reboot=v['reboot'];assert reboot['same_inode'] and reboot['before']['passed']==reboot['after']['passed']>0
    assert reboot['after']['budget_ns']==0 and reboot['after']['starts']==reboot['before']['starts']+1
    assert reboot['securelevel']==1 and reboot['persistent_state_present_before_start']
    peer=v['private_peers']
    for key in ['two_authenticated_peers','same_outer_address','distinct_inner_addresses','allowed_ips_single_host','native_experimental','native_penalty_observed','fresh_approved_bbs_login_during_attack','useful_chat_during_attack','attack_continued_during_b_login','source_spoof_denied','other_peer_positive_after_spoof']:assert peer[key]
    for key in ['authenticated_attackers','listener_reset_during_attack','registry_reset_during_attack','penalties_weakened']:assert peer[key] is False
    assert peer['datagram_source_controls']==dict(a=1,b=2,forged=0) and peer['forged_datagram_offered_to_tunnel']
    assert peer['server_securelevel']==1 and 3<=peer['attempts']<=80
    assert d['fixture_context']['invitation_keys']==1 and not d['fixture_context']['maximum_key_file_workload_tested'] and not d['fixture_context']['250_private_peers_tested']
    assert len(d['trials'])==2 and len({str(t['boot']) for t in d['trials']})==2
    facts=[]
    for trial in d['trials']:
        minimum=validate_trial(trial,d['binaries']['service']);m=trial['measurement'];validate_limits(m)
        assert m['admission_after']['peak']==250 and m['admission_after']['full']==2 and m['admission_after']['rate']==0
        assert m['global_before']['refill_ms']==250 and m['global_before']['source_refill_ms']==500
        activity=m['global_activity'];assert activity['active_members']==250 and not activity['registry_reset'] and not activity['listener_reset']
        pressure=activity['pressure'];assert pressure['seconds']>=60 and pressure['offers']>100 and pressure['banners']>0
        assert activity['after']['global_drops']>activity['before']['global_drops'] and activity['after']['table_drops']==0
        assert 0<activity['after']['passed']-activity['before']['passed']<=8+int(pressure['seconds']/.25)+4
        mapping=activity['mapping'];assert mapping['listener_mappings']>0 and mapping['leaked_processes']==0 and mapping['checked_session_auth_frontend_processes']>=750
        stream=activity['native_stream'];samples=expand(stream);guards=[sample_guard(s) for s in samples]
        assert stream['sample_count']==len(samples)==65 and stream['requested_interval_ms']==1000 and all(g['passed'] for g in guards)
        assert stream['guest_seconds']>=65 and 0<stream['gap_ms']['min']<=stream['gap_ms']['max']<5000
        assert stream['markers']['start_ns']<=samples[0]['monotonic_ns']<samples[-1]['finished_ns']<=stream['markers']['stop_ns']
        assert all(s['finished_ns']>=s['monotonic_ns'] for s in samples)
        assert all(x['finished_ns']<=y['monotonic_ns'] for x,y in zip(samples,samples[1:]))
        assert all(s['roles']['frontend']['count']==250 and s['roles']['broker']['count']==1 and s['roles']['ssh']['count']>=501 for s in samples)
        rounds=activity['rounds'];assert len(rounds)>=20 and all(r['confirmed_peer_receipts']==249 and r['seconds']<10 for r in rounds)
        assert rounds[-1]['at_seconds']>=60
        assert all(x['at_seconds']<y['at_seconds'] for x,y in zip(rounds,rounds[1:]))
        facts.append(dict(stress_minimum_bytes=minimum,active_minimum_bytes=min(g['reclaimable_headroom_bytes'] for g in guards),peer_receipts=sum(r['confirmed_peer_receipts'] for r in rounds),offers=pressure['offers']))
    assert {'initial-gate.c','preflight-mode.py','rc-pid-check','runtime-state-gate.c','runtime-state-preflight.py','icmp-peer-probe.py'}=={Path(name).name for name in d['diagnostic_sources']}
    assert d['diagnostics']['postload']['policy_changed'] is False
    for name,digest in d['diagnostic_sources'].items():assert hashlib.sha256((root/name).read_bytes()).hexdigest()==digest
    settings=d['settings'];assert settings['original']['ntp_config_sha256']==settings['restored']['ntp_config_sha256']
    assert settings['restored']['allocator']['narenas']==8 and settings['restored']['allocator']['tcache'] is True
    for key in ['max_files','max_processes']:assert settings['original']['snapshot']['limits'][key]==settings['restored']['snapshot']['limits'][key]
    assert all(d['restored_global'].values())
    assert d['restored_isolation']['profile_disabled'] and d['restored_isolation']['npf_inactive_empty'] and d['restored_isolation']['frontend_setuid_removed']
    assert d['restored_admission']==dict(disabled=True,no_live_member_transports=True,normal_path_passed=True)
    assert d['final_fixture']==dict(integrity='ok',accounts=1001,unique_hashes=1001,rooms=50,posts=100000)
    if not allow_pending_cleanup:assert all(d['cleanup'][k] for k in ['guest_stopped','peer_guest_stopped','private_inputs_removed','forwards_closed'])
    return facts

def check():
    d=json.loads((ROOT/'dev-docs/research/results/040-netbsd-global-admission.json').read_text());facts=validate(d)
    print('Global gate evidence passed: native pre-fork/state controls, unchanged-penalty private peer separation, two guarded 250-member matrices; minimum stress %.2f MiB.'%(min(f['stress_minimum_bytes'] for f in facts)/1048576))
if __name__=='__main__':check()
