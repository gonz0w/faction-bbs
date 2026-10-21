#!/usr/bin/env python3
"""Validate saved current confinement evidence; never reruns a VM."""
import json
import re
from check_integrated_evidence import ROOT,validate_sources
from check_ui_evidence import validate_trial
from headroom_memory import RESOURCES


def validate_limits(measurement):
    for entry in measurement['live_process_limits']:
        ledger=entry['readback'];assert ledger['resources']==RESOURCES
        for row in ledger['process_rows']:
            role,_,_,index,_=row
            if role not in ['broker','frontend']:continue
            profile=dict(zip(RESOURCES,ledger['limit_profiles'][index]))
            expected={'core_bytes':0,'locked_bytes':0,'processes':0,
                      'files':548 if role=='broker' else 8,
                      'threads':3 if role=='broker' else 0}
            for name,value in expected.items():
                assert profile[name]==[value,value],f'{role} {name} ceiling drift'


def check():
    d=json.loads((ROOT/'dev-docs/research/results/035-netbsd-confinement.json').read_text())
    assert d['schema']==1 and d['target']=='aarch64-unknown-netbsd'
    assert d['reference_commit']=='627a8241bf736db2371938641d8bef88785d374a'
    assert d['guest']['vcpus']==2 and d['guest']['configured_ram_bytes']==1073741824
    sources=d['execution_sources'];validate_sources(sources,historical=True)
    for name in ['Cargo.toml','Cargo.lock','crates/faction-service/src/main.rs',
                 'crates/faction-service/src/ui.rs','crates/faction-unix/src/process_security.c',
                 'crates/faction-unix/src/lib.rs','crates/faction-unix/build.rs',
                 'native/faction-tty/main.c','deploy/netbsd/faction_broker',
                 'experiments/055-confinement/process_probe.c',
                 'experiments/055-confinement/frontend_probe.py',
                 'experiments/055-confinement/startup_probe.py',
                 'experiments/055-confinement/terminal_probe.py']:
        assert name in sources
    assert d['native_sources']==sources
    for digest in d['binaries'].values():assert re.fullmatch('[a-f0-9]{64}',digest)
    for name in ['owner','service']:assert d['installed_build_identity'][name]==d['binaries'][name]
    v=d['validation'];assert v['native_workspace_tests']==63
    for name,count in [('shared_ui',29),('service_lifecycle',11),('ssh',20),
                       ('worker_lifecycle',13),('frontend',31),('startup',5)]:
        rows=v[name]['results']
        assert len(rows)==len({x['oracle'] for x in rows})==count
        assert all(x['result']=='pass' for x in rows)
    assert v['process_envelope']=={'frontend':'pass','service':'pass','fork_denied':True,
        'extra_threads_denied':True,'mlock_denied':True,'descriptor_cap_enforced':True,
        'hard_limit_raise_denied':True,'crash_without_core_file':True}
    assert v['frontend_hardening']=={'native_analyzer':'pass','pie':'pass','relro':'pass','immediate_binding':'pass','no_rwe_load':'pass'}
    ui=v['shared_ui'];assert ui['binary_sha256']==d['binaries']['service']
    assert ui['fairness_seconds'] and max(ui['fairness_seconds'])<2
    assert 0<v['service_lifecycle']['default_eight_stop_seconds']<10
    assert d['service_contract']=={'backend':'shared','default_sessions':8,'default_launch_ms':5000,
        'experimental_sessions':250,'experimental_launch_ms':100,'host_tuning_installed_by_service':False,'worker_rollback':'pass'}
    assert len(d['trials'])==3 and len({str(t['boot']) for t in d['trials']})==3
    minima=[]
    for trial in d['trials']:
        minima.append(validate_trial(trial,d['binaries']['service']))
        validate_limits(trial['measurement'])
    settings=d['settings'];assert settings['restored']['ntp_config_sha256']==settings['original']['ntp_config_sha256']
    assert settings['restored']['allocator']['narenas']==8 and settings['restored']['allocator']['tcache'] is True
    for name in ['max_files','max_processes']:
        assert settings['restored']['snapshot']['limits'][name]==settings['original']['snapshot']['limits'][name]
    assert any(row[0]=='ntpd' and row[3]>0 for row in settings['restored']['host']['processes'])
    assert d['final_fixture']=={'integrity':'ok','accounts':1001,'unique_hashes':1001,'rooms':50,'posts':100000}
    assert all(d['cleanup'][k] for k in ['guest_stopped','private_inputs_removed','forwards_closed'])
    print('Native confinement evidence passed: preserved executed source pins, kernel ceilings/peer denials, three 250-member boots; minimum %.2f MiB.'%(min(minima)/1024/1024))


if __name__=='__main__':check()
