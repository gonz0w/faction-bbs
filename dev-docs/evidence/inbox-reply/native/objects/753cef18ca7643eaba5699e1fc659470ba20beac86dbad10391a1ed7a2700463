#!/usr/bin/env python3
"""Validate the saved menu software observations and exact executed source bytes."""
import hashlib
import json
from evidence_sources import ROOT, source_bytes

def check():
    d = json.loads((ROOT/'dev-docs/evidence/menu-themes-netbsd/results.json').read_text())
    assert d['schema'] == 1 and d['result'] == 'pass'
    assert d['host_rust_tests'] == d['native_rust_tests'] == 154 and d['python_tests'] == 109
    assert d['selected_native_source_tree_matches'] == len(d['execution_sources']) == 116
    assert len(d['harness_sources']) == 3
    for group in ['execution_sources', 'harness_sources']:
        for name, digest in d[group].items():
            assert hashlib.sha256(source_bytes(name, digest)).hexdigest() == digest
    assert len(d['preview_sha256']) == 3
    for name, digest in d['preview_sha256'].items():
        assert hashlib.sha256(source_bytes(name, digest)).hexdigest() == digest
    assert d['menu_budgets'] == [
        {'columns': 40, 'full_bytes': 1311, 'focus_bytes': 170},
        {'columns': 79, 'full_bytes': 1328, 'focus_bytes': 172},
        {'columns': 80, 'full_bytes': 1957, 'focus_bytes': 246},
        {'columns': 132, 'full_bytes': 1974, 'focus_bytes': 248},
        {'columns': 240, 'full_bytes': 1974, 'focus_bytes': 248}]
    assert d['shared']['count'] == len(d['shared']['checks']) == 22
    assert d['shared']['result'] == d['restricted_member']['result'] == 'pass'
    assert all(c in d['shared']['checks'] for c in ['shared_compact_sysop_theme_and_season_at_40x18', 'shared_wide_descriptions_lightbar_and_arrow_enter', 'shared_resized_theme_uses_plain_recovery'])
    assert [(c['route'], c['columns']) for c in d['restricted_member']['routes']] == [('ssh',80),('ssh',40),('native',80)]
    for c in d['restricted_member']['routes']:
        assert c['result'] == 'pass' and c['arrows_and_enter'] and c['direct_hotkey'] and c['logout']
    assert d['limits'] == {'menu_full_bytes':4096, 'two_row_redraw_bytes':512,
        'shared_output_bytes':32768, 'drain_seconds':5, 'normal_sessions':8,
        'admission_refill_ms':5000, 'new_owner_helpers':0,
        'new_product_dependencies':0, 'new_resident_client_fields':0}
    assert len(d['teardown']) == 3 and all(d['teardown'].values())
    assert all(v is False for v in d['claims'].values())
    print('Menu evidence passed: 154 host/native Rust tests, 109 Python tests, 22 shared controls, 3 restricted member routes and bounded cell redraws; no visual/capacity claim.')

if __name__ == '__main__': check()
