#!/usr/bin/env python3
"""Check saved graphics results/source identities, without rerunning a VM/GUI."""
import hashlib,json
from evidence_sources import ROOT,source_bytes

def check():
    d=json.loads((ROOT/'dev-docs/evidence/kitty-graphics-netbsd/results.json').read_text())
    assert d['schema']==1 and d['metadata_schema']==7 and d['result']=='pass'
    assert d['host_rust_tests']==d['native_rust_tests']==151 and d['host_python_tests']==104
    assert d['graphics_pty_controls_per_host']==7 and d['native_local_gate_pty_controls_per_host']==3
    assert d['modern_pty_controls_per_host']==d['classic_unicode_pty_controls_per_host']==5
    shared=d['shared_reader'];assert shared['result']=='pass' and shared['count']==len(set(shared['checks']))==19
    assert {'shared_kitty_bounded_rgb_stream_classic_cleanup','shared_kitty_unknown_and_refused_classic_fallback','shared_kitty_malformed_incomplete_late_probe_isolation','shared_kitty_interrupted_transfer_cleanup'}<=set(shared['checks'])
    assert shared['capacity_claim'] is shared['dedicated_ssh_admission_tested'] is False
    assert d['screen']['exit_code']==0 and d['screen']['result']=={'reply_hex':'','only_inline_rgb_query':True,'graphics_enabled':False}
    assert d['measured_buffers']=={'rgb_static_bytes':38400,'encoded_bytes':51200,'wire_bytes':51410,'chunks':13,'max_command_bytes':4054,'chunk_capacity_bytes':4096,'probe_struct_bytes':40,'gate_struct_bytes':64,'query_bytes':38,'cleanup_bytes':27}
    assert d['budgets']=={'normal_sessions':8,'new_owner_helpers':0,'new_dependencies':0,'shared_output_bytes':32768,'drain_seconds':5,'upload_seconds':5,'frames_per_second':1,'images_per_session':1,'probe_response_bytes':128,'probe_milliseconds':300,'native_apc_capacity_bytes':4096,'native_input_read_bytes':4096,'native_graphics_events_bytes_max':8192,'per_member_image_cache_bytes':0}
    assert d['fixture']=={'vcpus':2,'ram_mib':1024,'admin_loopback_only':True,'synthetic_board':True}
    assert d['teardown']=={'vm_stopped':True,'private_clone_removed':True}
    assert all(v is False for v in d['claims'].values())
    assert len(d['execution_sources'])==112
    assert {'crates/faction-terminal/src/graphics.rs','crates/faction-terminal/tests/graphics.rs','crates/faction-terminal/examples/graphics_budget.rs','tools/smoke_kitty_graphics.py','tools/smoke_native_graphics.py','tools/probe_screen_kitty.py'}<=d['execution_sources'].keys()
    for name,digest in d['execution_sources'].items():assert hashlib.sha256(source_bytes(name,digest)).hexdigest()==digest
    print('Kitty evidence passed: 151 host/native Rust tests, seven graphics and three local-gate PTY controls per host, nineteen shared controls and bounded buffers; no GUI/capacity/hardening claim.')

if __name__=='__main__':check()
