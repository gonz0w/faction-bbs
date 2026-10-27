"""Export only this reviewer's saved public-safe evidence; execute no tests."""
from pathlib import Path
import hashlib
import json
import shutil

owned = Path('/private/tmp/faction230-review-OWNED')
public = owned / 'public'
public.mkdir(mode=0o700, exist_ok=False)
directories = {
    'static-review-sources',
    'final-review-sources',
    'classic-review-sources',
    'evidence-guard-audit',
}
root_files = {
    'OWNED',
    'audit_classic_review.py',
    'audit_final_review.py',
    'classic-review-after.json',
    'classic-review-audit.json',
    'classic-review-before.json',
    'final-review-after-manifest.json',
    'final-review-audit.json',
    'final-review-before-manifest.json',
    'static-review-after-manifest.json',
    'static-review-manifest.json',
    'package_public.py',
}
for source in sorted(owned.rglob('*')):
    relative = source.relative_to(owned)
    if not source.is_file() or '__pycache__' in relative.parts or '.DS_Store' in relative.parts:
        continue
    if relative.parts[0] not in directories and str(relative) not in root_files:
        continue
    assert not source.is_symlink()
    target = public / relative
    target.parent.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(source, target)
    assert source.read_bytes() == target.read_bytes()

observations = {
    'schema': 1,
    'issue': 230,
    'role': 'separate product reviewer and bounded evidence artifact reviewer',
    'requested_model': 'gpt-6.1-sol',
    'requested_effort': 'high',
    'runtime_model_verified': False,
    'authored_product_changes': False,
    'product_review': {
        'result': 'no remaining actionable product findings after classic fix',
        'scope': ['native client', 'shared service', 'terminal codec and themes', 'standalone and inherited CLI adapters', 'member and technical documentation'],
        'findings': [
            {'priority': 2, 'cause': 'Global @MAIL prefix rejection also rejected passwords, subjects, drafts and chat', 'resolution_observed': 'Exact @MAIL1 protocol prefix refusal limited to Menu/Files/NativeFiles; actual four-case RED and GREEN logs inspected'},
            {'priority': 2, 'cause': 'Matching parked ACK reset an already-started partial trailing control before validation', 'resolution_observed': 'Queued ACK and offer tails finish started controls within bounded deadlines; actual split-tail RED and GREEN logs inspected; valid fragmented ANSI and UTF-8 preserved'},
            {'priority': 2, 'cause': 'Unconditional mail OSC offer violated existing escape-free dumb-terminal contract; plain dispatch also replaced Menu before mail state check', 'resolution_observed': 'Plain-menu action restores Menu before dispatch; initial offer gated on ANSI or explicit native text hint; actual dispatch RED, ESC-byte RED and package GREEN logs inspected'},
        ],
        'final_classic_reviewed_sources': {
            'crates/faction-service/src/ui.rs': 'a54a324c04b2df708e37d9b7fff2940f9157938470024e297e64c920e42925e9',
            'crates/faction-service/src/native_mail.rs': '3693cbcf0495c8a5a3a6e342c8c361fa693afb5366f2da94577b6d3e39406540',
            'crates/faction-service/src/native_mail_tests.rs': 'cb770e01fbb5019d2c7badbd9a716def3c95369e3b3df331e5205a99a97a6fba',
        },
    },
    'evidence_guard_review': {
        'initial_offline_repro': 'evidence-guard-audit/repro.py',
        'initial_observations': 'evidence-guard-audit/observed.json',
        'initial_sources_preserved_exactly': True,
        'findings': [
            {'cause': 'Main-route validator allowed host references to satisfy NetBSD and omitted route scope', 'resolution_observed': 'Updated checker binds Darwin/NetBSD platform and exact main-menu scope'},
            {'cause': 'Supplemental categories accepted result-only placeholders and then count-only arbitrary control names', 'resolution_observed': 'Updated checker binds required subrecords, exact observed EXPECTED_CONTROLS sets and EXPECTED_SCOPES'},
            {'cause': 'Raw kdump presentation scan missed a private marker split across GIO records', 'resolution_observed': 'Saved synthetic split-record reproduction demonstrated false absence; native agent implementing strict observed hex decoding and separate retrospective capture re-evaluation', 'final_corrected_oracle_audited': False},
        ],
    },
    'execution_limits': {
        'offline_guard_reproduction_executed': True,
        'reproduction_saved_before_execution': True,
        'source_hashes_stable_during_reproduction': True,
        'reviewer_runtime_tests': False,
        'reviewer_builds': False,
        'reviewer_vms': False,
        'private_runtime_fixtures_created': False,
        'runtime_capture_files_included': False,
    },
    'provenance_caveat': 'One earlier server package GREEN whole-workspace capture includes unrelated concurrent client transport source drift. Its server/terminal/CLI dependency sources match. The later classic package RED/GREEN/Clippy/format snapshots are drift-free; retained audits distinguish them.',
    'pending': ['Frozen root evidence record and final source bindings', 'Corrected oracle and all retained capture retrospective verdicts', 'Native acceptance and teardown artifact audit'],
    'publication': 'This newly written observation summary records the reviewer conclusions already reported to the lead; it does not reconstruct unrecorded execution scripts or claim fresh runtime verification.',
}
(public / 'review-observations.json').write_text(json.dumps(observations, sort_keys=True, indent=2) + '\n')
manifest = {}
for path in sorted(public.rglob('*')):
    if path.is_file():
        manifest[str(path.relative_to(public))] = hashlib.sha256(path.read_bytes()).hexdigest()
(public / 'manifest.json').write_text(json.dumps(manifest, sort_keys=True, indent=2) + '\n')
print(json.dumps({'files': len(manifest), 'manifest_sha256': hashlib.sha256((public / 'manifest.json').read_bytes()).hexdigest(), 'private_runtime_material_included': False}, indent=2))
