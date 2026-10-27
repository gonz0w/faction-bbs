"""Copy only explicit non-private separate-review artifacts for publication."""
import hashlib
import json
from pathlib import Path
import shutil

root = Path('/private/tmp/faction233-review-OWNED')
target = root / 'public-staging'
assert (root / 'OWNED').is_file()
assert not target.exists()
target.mkdir(mode=0o700)
names = [
    'audit_publication.py', 'audit-publication-01.log',
    'audit_captures.py', 'audit-captures-01.log',
    'audit_draft_root.py', 'audit-draft-root-01.log',
    'package_review.py',
]
for name in names:
    source = root / name
    assert source.is_file() and not source.is_symlink()
    shutil.copyfile(source, target / name)
verdict = dict(
    issue=233,
    result='pass',
    phase='separate pre-teardown source/publication review; final cleanup/root-pin/archive/CI review pending',
    product_source_sha256=dict(
        checkpoint_ui='e19243bd6d661942c43e3d9165aa1dd6e5595edf9f16d537944c0d591ce6c74e',
        checkpoint_ui_reply='0c3660f7ea1e10af56db91b4c45c2309a4658af1318c9269b66b23b49958f7ea'),
    candidate_manifests=dict(
        native='dda3f0c0f87b71c9bffa05a332750383587748f3db7a002a23695acfede46520',
        client='5271a082be5a389490961ea0165c18ef0ced96355d8a12ce88adfd3bf4c19436',
        lead='570d9df54ea7fe326622fed29b3b27b6bea82b8abd800a9abe64bfaf3c4a76d1'),
    observed=dict(
        source_stages=58, native_build_inputs=699, runtime_helper_overrides=9,
        local_controls_per_mode_per_os=11, main_controls_per_mode_per_os=6,
        independently_revalidated_original_main_captures=4,
        markers_per_main_capture=15, generated_private_values_scanned=81,
        draft_publication_files_scanned=1686, generated_private_matches=0,
        raw_test_counts=dict(host_workspace=473, client_network=139, client_native=90, native_selected=272),
        files_controls_per_os=14, older_default_v1_controls_per_os=2, original_inbox_consumer_controls=14),
    review_scripts_saved_before_execution=True,
    review_script_hashes_unchanged_before_after=True,
    product_edits_or_builds_performed_by_reviewer=False,
    private_captures_or_values_exported=False,
    original_native_capture_archive_read_in_memory_without_extraction=True,
    scratch_log_limitation_reviewed=True,
    blockers=[],
    required_before_completion=['actual owned-fixture teardown and stopped saved-playground integrity record', 'immutable final root digest and successful guard', 'clean committed archive validation', 'required exact-head CI'],
    role=dict(requested='temper_reviewer', requested_model='gpt-6.1-sol', requested_effort='high', observed_runtime_model=None, observed_runtime_effort=None,
              runtime_metadata_status='unavailable; requested settings are not observed metadata'),
    assurance='separate agent review of work not authored by reviewer; not independent human assurance',
)
(target / 'verdict.json').write_text(json.dumps(verdict, indent=2, sort_keys=True) + '\n')
manifest = {p.name: hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(target.iterdir())}
(target / 'manifest.json').write_text(json.dumps(manifest, indent=2, sort_keys=True) + '\n')
print('Review publication created:', len(manifest) + 1, 'explicit safe files including manifest')
print('Manifest SHA-256:', hashlib.sha256((target / 'manifest.json').read_bytes()).hexdigest())
