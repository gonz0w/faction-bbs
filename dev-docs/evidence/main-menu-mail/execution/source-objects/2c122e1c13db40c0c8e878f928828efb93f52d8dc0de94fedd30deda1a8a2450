#!/usr/bin/env python3
"""Validate saved installed-service evidence; does not rerun the NetBSD VM."""
import hashlib,json,re
from pathlib import Path
from check_ui_evidence import validate_trial
from evidence_sources import source_bytes

ROOT=Path(__file__).resolve().parents[1]

def validate_sources(sources,root=ROOT,historical=False):
    for name,digest in sources.items():
        p=Path(name);assert not p.is_absolute() and '..' not in p.parts
        assert re.fullmatch('[a-f0-9]{64}',digest)
        raw=source_bytes(name,digest,root) if historical else (root/p).read_bytes()
        assert hashlib.sha256(raw).hexdigest()==digest,'current service execution drift: '+name


def check():
    d=json.loads((ROOT/'dev-docs/research/results/034-netbsd-shared-service.json').read_text())
    assert d['schema']==1 and d['target']=='aarch64-unknown-netbsd'
    assert d['reference_commit']=='09b506ea4f6da2830a0f2f480918ffb5048749aa'
    assert d['guest']['vcpus']==2 and d['guest']['configured_ram_bytes']==1073741824
    sources=d['execution_sources']
    validate_sources(sources,historical=True)
    for name in ['Cargo.toml','Cargo.lock','crates/faction-service/src/main.rs','crates/faction-service/src/ui.rs','crates/faction-service/src/config.rs','crates/faction-core/src/schema.sql','crates/faction-core/src/chat.sql','native/faction-tty/main.c','tools/build_terminal_frontend.py','deploy/netbsd/faction_broker','deploy/netbsd/sshd_config.in','deploy/netbsd/service.toml.in']:
        assert name in sources
    assert d['native_sources']==sources,'native transferred bytes differ'
    for digest in d['binaries'].values():assert re.fullmatch('[a-f0-9]{64}',digest)
    assert d['installed_build_identity']['service']==d['binaries']['service']
    assert d['installed_build_identity']['owner']==d['binaries']['owner']
    v=d['validation']
    assert v['native_workspace_tests']==62 and v['final_service_tests']==4
    for key,count in [('shared_ui',27),('service_lifecycle',11),('ssh',20),('worker_lifecycle',13),('frontend',20)]:
        rows=v[key]['results'];assert len(rows)==count and len({x['oracle'] for x in rows})==count and all(x['result']=='pass' for x in rows)
    ui=v['shared_ui'];assert ui['binary_sha256']==d['binaries']['service']
    assert ui['busy_peer_offered_bytes']>0 and ui['host_backpressure_events']>=0
    assert ui['fairness_seconds'] and max(ui['fairness_seconds'])<2
    assert 0<v['service_lifecycle']['default_eight_stop_seconds']<10
    assert v['frontend_hardening']=={'native_analyzer':'pass','pie':'pass','relro':'pass','immediate_binding':'pass','no_rwe_load':'pass'}
    assert d['service_contract']=={'backend':'shared','default_sessions':8,'default_launch_ms':5000,'experimental_sessions':250,'experimental_launch_ms':100,'host_tuning_installed_by_service':False,'worker_rollback':'pass'}
    assert len(d['trials'])==3 and len({str(t['boot']) for t in d['trials']})==3
    minima=[]
    for trial in d['trials']:
        assert trial['measurement']['scope']=='installed rc.d shared service; short workloads, not production capacity'
        minima.append(validate_trial(trial,d['binaries']['service']))
    settings=d['settings'];assert settings['restored']['ntp_config_sha256']==settings['original']['ntp_config_sha256']
    assert settings['restored']['allocator']['narenas']==8 and settings['restored']['allocator']['tcache'] is True
    assert any(row[0]=='ntpd' and row[3]>0 for row in settings['restored']['host']['processes'])
    assert d['final_fixture']=={'integrity':'ok','accounts':1001,'unique_hashes':1001,'rooms':50,'posts':100000}
    assert all(d['cleanup'][k] for k in ['guest_stopped','private_inputs_removed','forwards_closed'])
    print('Installed shared service evidence passed: preserved executed source pins, 250 members across three boots, lifecycle/rollback and unchanged guard; minimum %.2f MiB.'%(min(minima)/1024/1024))

if __name__=='__main__':check()
