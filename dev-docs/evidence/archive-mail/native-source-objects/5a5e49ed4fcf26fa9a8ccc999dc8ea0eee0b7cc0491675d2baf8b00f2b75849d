#!/usr/bin/env python3
"""Verify saved real-emulator artwork positions and immutable sources; no GUI."""
import hashlib
import json
from evidence_sources import ROOT, source_bytes


def check():
    d = json.loads((ROOT / 'dev-docs/evidence/nightwire-terminals/results.json').read_text())
    assert d['schema'] == 1 and d['result'] == 'pass'
    assert len(d['prior_nightwire_inputs_unchanged']) == 167
    for group in ['prior_nightwire_inputs_unchanged', 'harness_sources']:
        for name, digest in d[group].items():
            assert hashlib.sha256(source_bytes(name, digest)).hexdigest() == digest, name
    assert hashlib.sha256((ROOT / 'dev-docs/evidence/nightwire-netbsd/results.json').read_bytes()).hexdigest() == d['prior_nightwire_record_sha256']
    assert len(d['profiles']) == 15
    checkpoints = glyphs = frames = 0
    expected_names = set()
    for label in ['ghostty', 'kitty', 'alacritty']:
        expected_names.update(label + '-' + suffix for suffix in ['direct', 'direct-missing-font', 'tmux', 'screen', 'direct-compact'])
    assert {p['profile'] for p in d['profiles']} == expected_names
    wide_frames = None
    compact_frames = None
    for p in d['profiles']:
        compact = p['geometry'][0] < 24 or p['geometry'][1] < 80
        assert p['result'] == 'pass' and p['unclassified_input_bytes'] == 0
        assert p['geometry'][:2] == ([20, 80] if p['profile'] == 'alacritty-direct-missing-font' else
                                          [18, 40] if p['profile'].endswith('-compact') else [24, 80])
        assert p['viewport_requested'] == ([18, 40] if p['profile'].endswith('-compact') else [24, 80])
        assert p['tmux_status_disabled'] == (p['route'] == 'tmux')
        assert p['raw_tty_onlcr_emulation'] and not p['visual_pixels_captured']
        assert not p['sgr_palette_and_glyph_shape_certified']
        assert len(p['frames']) == (5 if compact else 20) and len(p['glyphs']) == (0 if compact else 17)
        signatures = {f['name']: f['sha256'] for f in p['frames']}
        if compact:
            if compact_frames is None:
                compact_frames = signatures
            assert compact_frames == signatures
        else:
            if wide_frames is None:
                wide_frames = signatures
            assert wide_frames == signatures
        for f in p['frames']:
            assert f['bytes'] <= (4096 if f['mode'] == 'menu' else 17408)
            assert f['checkpoints']
            for observed in f['checkpoints']:
                assert len(observed) == 4 and observed[:2] == observed[2:]
                assert 1 <= observed[0] <= p['geometry'][0] and 1 <= observed[1] <= p['geometry'][1]
                checkpoints += 1
            frames += 1
        for g in p['glyphs']:
            assert g['actual'] == g['expected'] == [1, 33] and g['pass_']
            glyphs += 1
    assert compact_frames and all(wide_frames[k] == v for k, v in compact_frames.items())
    assert (frames, checkpoints, glyphs) == (240, 2476, 187)
    assert d['summary'] == dict(profiles=15, frames=240, cursor_checkpoints=2476, repeated_glyph_observations=187, product_rendering_defects_reproduced=0)
    for label in ['ghostty', 'kitty', 'alacritty']:
        ref = d['references'][label]
        assert ref['text_identical'] and ref['no_js_selected_files'] and ref['packaging_modified']
        assert not ref['product_runtime_dependency']
    assert d['references']['tmux']['no_js_source_scan']
    assert all(d['teardown'].values()) and not any(d['claims'].values())
    assert d['initial_fixture_findings']['compact_newline_mismatch_profiles'] == 2
    assert d['initial_fixture_findings']['tmux_23_row_no_result']
    print('Nightwire real-terminal evidence passed: 15 profiles, 240 frames, 2476 cursor checkpoints; no pixel/capacity claim.')


if __name__ == '__main__':
    check()
