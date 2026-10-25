#!/usr/bin/env python3
"""Check saved modern-terminal evidence; does not rerun a VM or GUI emulator."""
import hashlib,json
from evidence_sources import ROOT,source_bytes

def check():
    d=json.loads((ROOT/'dev-docs/evidence/modern-terminals-netbsd/results.json').read_text())
    assert d['schema']==1 and d['metadata_schema']==7 and d['result']=='pass'
    assert d['host_rust_tests']==d['native_rust_tests']==145 and d['host_python_tests']==104
    assert d['modern_pty']['host_controls']==d['modern_pty']['native_controls']==5
    assert len(set(d['modern_pty']['cases']))==5 and d['classic_unicode_pty_controls_per_host']==5
    shared=d['shared_reader'];assert shared['result']=='pass' and shared['count']==len(set(shared['checks']))==15
    assert {'shared_modern_unicode_cells_rgb_and_synchronized_pages','shared_classic_override_recovers_unknown_width_profile','shared_unknown_malformed_incomplete_late_probe_isolation'} <= set(shared['checks'])
    assert shared['capacity_claim'] is shared['dedicated_ssh_admission_tested'] is False
    assert d['screen']['exit_code']==0 and d['screen']['result']=={'reply_hex':'','only_sync_query':True,'graphics_enabled':False}
    review=json.loads((ROOT/'dev-docs/dependencies/unicode-width.json').read_text());assert d['dependency_review']==review
    assert review['name']=='unicode-width' and review['version']=='0.2.2' and review['selected_features']==[] and review['active_transitive_dependencies']==[]
    assert review['build_script'] is False and review['source_no_js_scan'] and review['source_forbids_unsafe']
    assert review['license']=='MIT OR Apache-2.0' and len(review['license_hashes'])>=2
    assert d['budgets']=={'normal_sessions':8,'new_owner_helpers':0,'shared_output_bytes':32768,'drain_seconds':5,'probe_output_bytes':9,'probe_response_bytes':32,'probe_milliseconds':300,'settings_panel_bytes_max':2048,'sync_frame_overhead_bytes':16,'active_image_pixels':0,'active_image_bytes':0,'active_image_cache_bytes':0}
    assert d['fixture']=={'vcpus':2,'ram_mib':1024,'admin_loopback_only':True,'synthetic_board':True}
    assert d['teardown']=={'vm_stopped':True,'private_clone_removed':True}
    assert all(v is False for v in d['claims'].values())
    assert len(d['execution_sources'])==106
    assert {'crates/faction-terminal/src/text.rs','crates/faction-terminal/src/capabilities.rs','crates/faction-terminal/tests/capabilities.rs','tools/smoke_modern_terminals.py','tools/probe_screen_sync.py'} <= d['execution_sources'].keys()
    for name,digest in d['execution_sources'].items():assert hashlib.sha256(source_bytes(name,digest)).hexdigest()==digest
    print('Modern terminal evidence passed: 145 host/native Rust tests, five modern PTY controls per host, fifteen shared controls and actual screen fallback; no GUI/bitmap/capacity/hardening claim.')

if __name__=='__main__':check()
