#!/usr/bin/env python3
"""Verify saved real-profile identities and qualified claims; does not run GUIs."""
import hashlib,json
from evidence_sources import ROOT,source_bytes

def check():
    d=json.loads((ROOT/'dev-docs/evidence/terminal-compatibility/results.json').read_text())
    assert d['schema']==1 and d['metadata_schema']==7 and d['result']=='pass_with_classic_only_profiles'
    assert d['host_rust_tests']==d['native_rust_tests']==151 and d['python_tests']==107
    assert d['summary']=={'profiles':12,'fully_matching_cell_profiles':9,'classic_only_profiles':3,'cell_and_cluster_observations':336,'bbs_routes':48,'actual_image_retention_deletion_pairs':12,'real_ssh_pty_resize_controls':2,'unclassified_input_bytes':0}
    profiles=d['profiles'];assert len({p['profile'] for p in profiles})==12
    expected={name+'-'+route for name in ['ghostty','kitty','alacritty'] for route in ['direct','tmux','screen','missing-font']};assert {p['profile'] for p in profiles}==expected
    images=0
    for p in profiles:
        assert p['unexpected_input_bytes']==0 and p['classic_cells_pass'] and len(p['cases'])==28
        assert p['result']==('classic-only' if p['route']=='screen' else 'pass')
        assert p['native_cells_pass']==(p['route']!='screen')
        assert p['gui_visual_pixels_captured'] is False and p['gui_max_sampled_rss_kib']>0
        assert p['route_environment_override']==({'TERM':'xterm-256color'} if p['route']=='screen' else {})
        for c in p['cases']:
            if c['mode']=='classic':assert c['pass'] and c['cursor']==[1,c['expected_cells']+1]
        assert {c['route'] for c in p['bbs_routes']}=={'local','ssh','native','native-kitty'}
        for c in p['bbs_routes']:
            assert c['result']=='pass' and c['classic_recovered'] and c['native_unicode_article'] and c['unclassified_input_bytes']==0
            if c['route'].startswith('native'):assert c['sync_reply_forwarded'] is False
            image=c['graphic_uploaded'];expected_image=p['profile'] in {'ghostty-direct','ghostty-missing-font','kitty-direct','kitty-missing-font'} and c['route']!='native'
            assert image==expected_image
            if image:
                images+=1;assert c['renderer_acknowledged_image'] and c['renderer_acknowledged_deletion']
                assert bytes.fromhex(c['actual_image_retention']['reply_hex']).endswith(b';OK\x1b\\')
                assert b';ENOENT' in bytes.fromhex(c['actual_image_after_classic']['reply_hex'])
            else:assert c['actual_image_retention'] is c['actual_image_after_classic'] is None
    assert images==12
    assert d['resize']['result']=='pass' and len(d['resize']['cases'])==2
    for c in d['resize']['cases']:assert c['result']=='pass' and c['unexpected_input_bytes']==0 and c['new_pty_geometry']==[18,40]
    assert d['resize']['cases'][0]['classic_recovered'] and d['resize']['cases'][1]['image_deleted']
    for name,version in [('ghostty','1.3.1'),('kitty','0.49.2'),('alacritty','0.17.0'),('tmux','3.8'),('screen','4.00.03')]:assert d['references'][name]['version']==version
    for name in ['ghostty','kitty','alacritty']:assert d['references'][name]['identity']['text_identical'] and d['references'][name]['identity']['packaging_modified']
    assert d['references']['tmux']['utf8proc'] is d['references']['tmux']['jemalloc'] is False
    assert d['budgets']=={'normal_sessions':8,'admission_refill_ms':5000,'new_owner_helpers':0,'new_product_dependencies':0,'shared_output_bytes':32768,'drain_seconds':5,'probe_milliseconds':300,'research_reply_bytes':128,'research_relay_pending_output_bytes':65536,'research_synthetic_transcript_bytes':524288,'graphics_frames_per_second':1}
    assert len(d['execution_sources'])==115 and len(d['harness_sources'])==7 and len(d['reproduction_sources'])==2
    for group in ['execution_sources','harness_sources','reproduction_sources']:
        for name,digest in d[group].items():assert hashlib.sha256(source_bytes(name,digest)).hexdigest()==digest
    assert all(d['teardown'].values()) and len(d['teardown'])==4
    assert all(v is False for v in d['claims'].values())
    print('Real terminal evidence passed: 12 qualified profiles, 336 observations, 48 BBS routes, 12 image retention/deletion pairs and 2 live SSH resize controls; no visual/GPU/hardware/capacity claim.')
if __name__=='__main__':check()
