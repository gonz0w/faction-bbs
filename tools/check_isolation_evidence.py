#!/usr/bin/env python3
"""Validate current sealed-profile evidence and its rejected allocation candidate."""
import hashlib,json,re
from pathlib import Path
from check_integrated_evidence import ROOT,validate_sources
from check_confinement_evidence import validate_limits
from check_ui_evidence import unpack,validate_trial
from headroom_memory import sample_guard

def validate_rejected(candidate,root=ROOT):
    replacements=candidate['source_overrides']
    assert set(replacements)=={'native/faction-isolation/startup.c','tools/prepare_isolation.py',
        'deploy/netbsd/faction_isolated','tools/check_isolation_lifecycle.py','experiments/055-isolation/boundary_probe.py'}
    for name,digest in candidate['execution_sources'].items():
        relative=Path(replacements.get(name,name))
        assert not relative.is_absolute() and '..' not in relative.parts
        assert hashlib.sha256((root/relative).read_bytes()).hexdigest()==digest,'rejected source drift: '+name
    assert len(candidate['trials'])==2
    minima=[]
    for trial in candidate['trials']:
        assert trial['functional_completed_stage']==250
        assert re.fullmatch('[a-f0-9]{64}',trial['service_binary_sha256'])
        control=trial['failed_control'];assert control['cohort_in_chat']==250
        assert control['calibration']['known_touched_bytes']==38*1024*1024
        samples=unpack(control)
        minimum=min(sample_guard(s)['reclaimable_headroom_bytes'] for s in samples)
        assert minimum==trial['minimum_bytes']<128*1024*1024,'rejected guard failure disappeared'
        assert all(s['roles']['frontend']['count']==250 and s['roles']['ssh']['count']>=501 for s in samples)
        minima.append(minimum)
    return minima

def validate(d,root=ROOT,allow_pending_cleanup=False):
    assert d['schema']==1 and d['target']=='aarch64-unknown-netbsd'
    assert d['reference_commit']=='473feb479c7544b10e91515f9d7b22e4ab9b494c'
    assert d['guest']['vcpus']==2 and d['guest']['configured_ram_bytes']==1073741824
    sources=d['execution_sources'];validate_sources(sources,root=root)
    for name in ['native/faction-isolation/startup.c','tools/prepare_isolation.py','tools/build_isolation.py',
                 'deploy/netbsd/faction_isolated','tools/check_isolation_lifecycle.py',
                 'tools/check_isolation_scaling.py','tools/check_isolation_host.py',
                 'experiments/055-isolation/boundary_probe.py','experiments/055-isolation/isolation_probe.c',
                 'experiments/055-isolation/frontend_probe.py','experiments/055-isolation/slot_probe.py']:
        assert name in sources
    assert d['native_sources']==sources
    for digest in d['binaries'].values():assert re.fullmatch('[a-f0-9]{64}',digest)
    for name in ['service','frontend']:assert d['installed_build_identity'][name]==d['binaries'][name]
    assert d['installed_sources']['preflight']==sources['tools/prepare_isolation.py']
    assert d['installed_sources']['rc']==sources['deploy/netbsd/faction_isolated']
    assert re.fullmatch('[a-f0-9]{64}',d['installed_build_identity']['native_object'])
    assert d['policy']['profile_modes']=={'service':0o700,'frontend':0o4755}
    v=d['validation'];assert v['native_workspace_tests']==63
    for name,count in [('shared_ui',29),('frontend',31),('service_lifecycle',16),('normal_frontend',31),('ssh',20),('worker_lifecycle',13)]:
        rows=v[name]['results'];assert len(rows)==len({x['oracle'] for x in rows})==count
        assert all(x['result']=='pass' for x in rows)
    assert v['shared_ui']['binary_sha256']==d['binaries']['service']
    assert v['shared_ui']['fairness_seconds'] and max(v['shared_ui']['fairness_seconds'])<2
    assert 0<v['service_lifecycle']['default_eight_stop_seconds']<10
    boundary=v['boundary'];assert boundary['positive_root_controls']=='pass' and boundary['blocked_packets_received']==0 and boundary['root_control_tcp_accepts']==2
    assert {x['role'] for x in boundary['results']}=={'owner','terminal'} and len(boundary['results'])==2
    for role in boundary['results']:
        assert role==dict(role=role['role'],privilege_drop=True,filesystem=True,exec=True,tcp_udp_v4_v6=True,raw_socket_denied=True)
    assert set(boundary['sealed_allocator_readbacks'])=={'owner','terminal'}
    for profile in boundary['sealed_allocator_readbacks'].values():
        assert (profile['narenas'],profile['tcache'],profile['retain'],profile['dirty_decay_ms'],profile['muzzy_decay_ms'],profile['issetugid'])==(1,False,False,0,0,1)
    assert v['slot_aliases']=={'schema':1,'bidirectional_credentials_and_pid_locks':'pass','lock_inodes_unlinked':False}
    assert v['hardening']==dict(native_analyzer='pass',pie='pass',relro='pass',immediate_binding='pass',no_rwe_load='pass')
    contract=d['service_contract'];assert contract=={'default_sessions':8,'default_launch_ms':5000,
        'experimental_sessions':250,'experimental_launch_ms':100,'host_tuning_installed_by_service':False,
        'firewall_applied_by_service':False,'normal_binaries_preserved':True,'frontend_setuid':True}
    assert d['policy']['active_stateless_gid_uid_verified'] is True and d['policy']['usermount']==0
    assert d['policy']['mount_count']==3
    assert len(d['trials'])==3 and len({str(t['boot']) for t in d['trials']})==3
    minima=[]
    for trial in d['trials']:
        assert trial['measurement']['scope']=='installed isolated rc.d shared service; short workloads, not production capacity'
        minima.append(validate_trial(trial,d['binaries']['service']));validate_limits(trial['measurement'])
    validate_rejected(d['rejected_named_sysctl_candidate'],root)
    settings=d['settings'];assert settings['restored']['ntp_config_sha256']==settings['original']['ntp_config_sha256']
    assert settings['restored']['allocator']['narenas']==8 and settings['restored']['allocator']['tcache'] is True
    for name in ['max_files','max_processes']:
        assert settings['restored']['snapshot']['limits'][name]==settings['original']['snapshot']['limits'][name]
    assert any(row[0]=='ntpd' and row[3]>0 for row in settings['restored']['host']['processes'])
    assert d['restored_isolation']=={'profile_disabled':True,'npf_inactive_empty':True,
        'experimental_npf_boot_removed':True,'global_allocator_removed':True,'sealed_allocator_removed':True,'frontend_setuid_removed':True,
        'profile_mounts_absent':True,'usermount':0}
    assert d['final_fixture']==dict(integrity='ok',accounts=1001,unique_hashes=1001,rooms=50,posts=100000)
    if not allow_pending_cleanup:assert all(d['cleanup'][k] for k in ['guest_stopped','private_inputs_removed','forwards_closed'])
    return minima

def check():
    d=json.loads((ROOT/'dev-docs/research/results/036-netbsd-isolation.json').read_text())
    minima=validate(d)
    print('Native isolation evidence passed: sealed controls/UI/lifecycle, rejected candidate retained, three 250-member boots; minimum %.2f MiB.'%(min(minima)/1048576))

if __name__=='__main__':check()
