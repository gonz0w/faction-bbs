#!/usr/bin/env python3
"""Check saved SSH admission denials, live accounting and unchanged memory guards."""
import hashlib,json,re
from pathlib import Path
from check_integrated_evidence import ROOT,validate_sources
from check_ui_evidence import validate_trial,unpack
from check_confinement_evidence import validate_limits
from headroom_memory import sample_guard

def validate(d,root=ROOT,allow_pending_cleanup=False):
    assert d['schema']==1 and d['target']=='aarch64-unknown-netbsd'
    assert d['reference_commit']=='0c668ecd27bc9f25093a1b71cd04a98aae53bbb9'
    assert d['guest']['vcpus']==2 and d['guest']['configured_ram_bytes']==1073741824
    sources=d['execution_sources'];validate_sources(sources,root=root)
    for n in ['native/faction-key-admission/main.c','tools/build_key_admission.py',
              'deploy/netbsd/faction_key_admission','deploy/netbsd/key_admission_sshd_config.in',
              'experiments/055-key-admission/netbsd_metrics.c','experiments/055-key-admission/live_limits.c',
              'tools/check_key_admission_scaling.py','tools/check_key_admission_host.py',
              'tools/check_key_admission_oracles.py','tools/check_key_admission_channels.py',
              'tools/check_key_admission_config.py','tools/check_key_preauth.py']:
        assert n in sources
    assert d['native_sources']==sources
    for digest in d['binaries'].values():assert re.fullmatch('[a-f0-9]{64}',digest)
    assert d['installed_build_identity']['key_helper']==d['binaries']['key_helper']
    assert d['installed_sources']['key_admission_rc']==sources['deploy/netbsd/faction_key_admission']
    assert d['installed_sources']['key_preflight']==sources['tools/check_key_admission_config.py']
    assert d['contract']=={'default_reservations':8,'default_refill_ms':5000,'maximum_reservations':250,
        'maximum_burst':8,'experiment_refill_ms':250,'application_refill_ms':100,'base_preauthentication_maximum':8,
        'helper_per_ip_tracking':False,'resident_admission_daemon':False,'ssh_parser':False,'host_tuning_installed':False,
        'source_penalties_retained':True,'missing_state_requires_quiescence':True}
    v=d['validation'];assert v['native_workspace_tests']==63
    for name,count in [('admission',14),('channels',6),('shared_ui',29),('frontend',31),('normal_frontend',31),('ssh',20),('worker_lifecycle',13)]:
        rows=v[name]['results'];assert len(rows)==len({x['oracle'] for x in rows})==count
        assert all(x['result']=='pass' for x in rows)
    assert v['admission']['binary_sha256']==d['binaries']['key_helper']
    assert v['shared_ui']['binary_sha256']==d['binaries']['service']
    assert v['hardening']==dict(native_analyzer='pass',pie='pass',relro='pass',immediate_binding='pass',no_rwe_load='pass')
    pressure=v['preauthentication_pressure']
    assert pressure['accepted_silent']==8 and pressure['excess_drops']>=16 and pressure['seconds']>=36
    assert len(pressure['samples'])>=25 and all(sample_guard(s)['passed'] for s in pressure['samples'])
    assert all(s['roles']['ssh']['count']<=17 for s in pressure['samples'])
    assert pressure['after_deadlines']['roles']['ssh']['count']==1
    assert len(d['trials'])==3 and len({str(t['boot']) for t in d['trials']})==3
    minima=[]
    for trial in d['trials']:
        t=trial['measurement'];minima.append(validate_trial(trial,d['binaries']['service']));validate_limits(t)
        assert t['scope']=='installed key admission + isolated rc.d shared service; short workloads, not production capacity'
        assert t['admission_before']['capacity']==250 and t['admission_before']['refill_ms']==250
        assert t['admission_after']['peak']==250 and t['admission_after']['full']==2
        assert t['admission_after']['rate']==0
        probe=t['at_capacity_probe'];assert len(probe)==1 and probe[0]['extra_transports_denied']==2
        assert probe[0]['before']['launches']==probe[0]['after']['launches']
        assert probe[0]['after']['full']==probe[0]['before']['full']+2
        for stage in t['stages']:
            for sample in stage['samples']:
                assert sample['roles']['ssh']['count']==2*stage['sessions']+1
        for entry in t['live_process_limits']:
            ledger=entry['readback'];assert ledger['ssh_zombie_processes']==0
            rows=ledger['process_rows'];assert sum(r[4] for r in rows if r[0]=='ssh_listener')==1
            assert sum(r[4] for r in rows if r[0] in ['ssh_privileged','ssh_session'])==2*entry['sessions']
    rejected=d['rejected_inetd_candidate'];assert len(rejected['trials'])==3
    assert len({str(t['boot']) for t in rejected['trials']})==3
    assert re.fullmatch('[a-f0-9]{64}',rejected['listener_binary_sha256'])
    for name,digest in rejected['inputs']['execution_sources'].items():
        name=rejected['inputs']['source_overrides'].get(name,name)
        path=Path(name);assert not path.is_absolute() and '..' not in path.parts
        assert hashlib.sha256((root/path).read_bytes()).hexdigest()==digest,'rejected input drift: '+name
    failed=0
    for trial in rejected['trials']:
        assert trial['measurement']['completed_stage']==250
        values=[]
        for kind in ['calibrations','high_load_credentials','owner_credentials','loaded_controls','full_ui_at_250']:
            for stream in trial['measurement'][kind]:
                values.extend(sample_guard(s)['reclaimable_headroom_bytes'] for s in unpack(stream))
        assert min(values)==trial['minimum_bytes']
        failed+=min(values)<128*1048576
    assert failed==2,'rejected repeatability failures must remain visible'
    diagnostic=d['ledger_diagnostic']
    assert diagnostic['measurement']['outcome']=='stopped_at_functional_failure'
    assert diagnostic['measurement']['failure']['offered_stage']==32
    assert diagnostic['measurement']['failure']['detail']=='profiling fixture operation failed'
    for name,digest in diagnostic['sources'].items():
        assert hashlib.sha256((root/name).read_bytes()).hexdigest()==digest,'diagnostic bytes drift'
    settings=d['settings'];assert settings['original']['ntp_config_sha256']==settings['restored']['ntp_config_sha256']
    assert settings['restored']['allocator']['narenas']==8 and settings['restored']['allocator']['tcache'] is True
    for k in ['max_files','max_processes']:
        assert settings['original']['snapshot']['limits'][k]==settings['restored']['snapshot']['limits'][k]
    assert any(row[0]=='ntpd' and row[3]>0 for row in settings['restored']['host']['processes'])
    assert d['restored_isolation']==dict(profile_disabled=True,npf_inactive_empty=True,experimental_npf_boot_removed=True,
        global_allocator_removed=True,sealed_allocator_removed=True,frontend_setuid_removed=True,profile_mounts_absent=True,usermount=0)
    assert d['final_fixture']==dict(integrity='ok',accounts=1001,unique_hashes=1001,rooms=50,posts=100000)
    if not allow_pending_cleanup:assert all(d['cleanup'][k] for k in ['guest_stopped','private_inputs_removed','forwards_closed'])
    assert d['restored_admission']==dict(disabled=True,no_live_member_transports=True,normal_path_passed=True)
    return minima

def check():
    d=json.loads((ROOT/'dev-docs/research/results/037-netbsd-admission.json').read_text())
    minima=validate(d)
    print('SSH admission evidence passed: key-admission bounds, SSH denials, rejected failures preserved, three 250-member boots; minimum %.2f MiB.'%(min(minima)/1048576))

if __name__=='__main__':check()
