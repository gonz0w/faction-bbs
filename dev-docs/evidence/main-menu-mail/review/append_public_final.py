"""Append explicitly selected safe review artifacts without changing old files."""
from pathlib import Path
import hashlib
import json
import shutil

owned = Path('/private/tmp/faction230-review-OWNED')
public = owned / 'public'

def digest(data):
    return hashlib.sha256(data).hexdigest()

original_manifest_bytes = (public / 'manifest.json').read_bytes()
assert digest(original_manifest_bytes) == '4015e75961f6aad6d78238cc0134de0583709f86624855660688eabbab6c3586'
original = json.loads(original_manifest_bytes)
assert all(digest((public / name).read_bytes()) == sha for name, sha in original.items())
allowlist = [
    'audit_staged_layout.py',
    'audit_staged_layout02.py',
    'staged-layout-audit/execution-original-manifest.json',
    'staged-layout-audit/review-original-manifest.json',
    'staged-layout-audit/failed01-observation.json',
    'staged-layout-audit02/execution-original-manifest.json',
    'staged-layout-audit02/review-original-manifest.json',
    'staged-layout-audit02/result.json',
    'diagnose_hex_layout.py',
    'diagnose_hex_repeats.py',
    'diagnose_hex_fields.py',
    'audit_corrected_rescan.py',
    'corrected-rescan-audit/source-before.json',
    'corrected-rescan-audit/source-after.json',
    'corrected-rescan-audit/result.json',
    'corrected-rescan-audit/tools/probe_menu_mail.py',
    'corrected-rescan-audit/tools/test_probe_menu_mail.py',
    'append_public_final.py',
]
for name in allowlist:
    source, target = owned / name, public / name
    assert source.is_file() and not source.is_symlink() and not target.exists()
    assert 'private.log' not in name and 'associations.private' not in name
    target.parent.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(source, target)
    assert source.read_bytes() == target.read_bytes()

diagnosis = {
    'schema': 1,
    'issue': 230,
    'result': 'structural diagnosis completed; final corrected decoder separately reviewed',
    'source': 'https://raw.githubusercontent.com/NetBSD/src/trunk/usr.bin/kdump/kdump.c',
    'source_function': 'hexdump_buf',
    'observations': [
        'First retained hex capture had 12 exact tab-plus-three-asterisk rows; each observed next explicit offset verified omitted complete repeated rows.',
        'After repeat-marker support, 20 rows had a blank 48-character hex field and a 16-character ASCII field; NetBSD source identifies its exactly-one-duplicate branch.',
        'Final decoder reconstructs only verified repeat extents and verifies a single blank repeat against the previous complete row and its ASCII rendering.',
    ],
    'diagnostic_output_boundary': 'Only structure, counts and offsets were reported. No captured private payload or marker values were displayed or copied to this export.',
    'execution_provenance': 'This observation document is newly written from actual diagnostic outputs reported to the lead; the saved diagnostic scripts are preserved, with no reconstructed run scripts or fabricated captured logs.',
}
(public / 'repeat-diagnosis-observations.json').write_text(json.dumps(diagnosis, sort_keys=True, indent=2) + '\n')
verdict = {
    'schema': 1,
    'issue': 230,
    'result': 'pass',
    'scope': 'separate final product review, staged immutable exports, strict decoder and four linked retrospective trace rescans',
    'authored_product_changes': False,
    'runtime_model_verified': False,
    'requested_model': 'gpt-6.1-sol',
    'requested_effort': 'high',
    'remaining_actionable_findings_in_reviewed_scope': [],
    'resolved_product_findings': ['ordinary content and password input preserved', 'already-started trailing controls validated before local handoff', 'plain dispatch and escape-free dumb-terminal fallback preserved'],
    'resolved_evidence_findings': ['host and NetBSD main-route identity bound', 'supplemental control names and scopes bound', 'private markers checked in reconstructed GIO byte streams across record boundaries'],
    'reviewed_final_product_sources': {
        'crates/faction-service/src/ui.rs': 'a54a324c04b2df708e37d9b7fff2940f9157938470024e297e64c920e42925e9',
        'crates/faction-service/src/native_mail.rs': '3693cbcf0495c8a5a3a6e342c8c361fa693afb5366f2da94577b6d3e39406540',
        'crates/faction-service/src/native_mail_tests.rs': 'cb770e01fbb5019d2c7badbd9a716def3c95369e3b3df331e5205a99a97a6fba',
        'crates/faction-client/src/main.rs': 'be98cd35c3cfed6cff7ce84f5442260330c6557da913ad5ba47c7f5a0215d624',
        'crates/faction-client/src/stream.rs': '667e4ea2c90f8979038d3a234bd72eeb4f801e66e499678a7f109341fa44488b',
        'crates/faction-client/src/transport.rs': '0e94270b1db6831774a7106b16bc18160798736b01d982d57da56670b693c9cd',
    },
    'reviewed_final_oracle_sources': json.loads((public / 'corrected-rescan-audit/source-after.json').read_bytes()),
    'staged_layout_audit': {'path': 'staged-layout-audit02/result.json', 'sha256': digest((public / 'staged-layout-audit02/result.json').read_bytes())},
    'independent_corrected_rescan_audit': {'path': 'corrected-rescan-audit/result.json', 'sha256': digest((public / 'corrected-rescan-audit/result.json').read_bytes())},
    'corrected_rescan_result_sha256': 'a290b9ee52071052c2520d0b74fbc18f6227ad169447abb96668569168f7d27e',
    'four_route_marker_counts': {'host_classic': 11, 'host_unicode': 11, 'netbsd_classic': 11, 'netbsd_unicode': 11},
    'private_marker_matches': 0,
    'file_protocol_prefix_matches': 0,
    'association_limit': 'Unique successful private-inputs timestamp within one second before each result on the same filesystem plus serialized driver order; this is not an original embedded route identifier.',
    'review_execution_boundary': 'Saved offline guard reproduction and independent retained-capture rescan only; no UI/runtime tests, builds, VM operations or private fixture creation by reviewer.',
    'private_material_in_export': False,
    'pending_final_acceptance': ['frozen root evidence audit', 'exact owned teardown and saved-playground integrity binding'],
    'prior_export_preserved': True,
}
(public / 'final-product-and-artifact-review.json').write_text(json.dumps(verdict, sort_keys=True, indent=2) + '\n')
assert all(digest((public / name).read_bytes()) == sha for name, sha in original.items())
assert (public / 'manifest.json').read_bytes() == original_manifest_bytes
manifest = {}
for path in sorted(public.rglob('*')):
    if path.is_file():
        name = str(path.relative_to(public))
        assert name != 'manifest-final.json'
        assert name in original or name in allowlist or name in ['manifest.json', 'repeat-diagnosis-observations.json', 'final-product-and-artifact-review.json']
        manifest[name] = digest(path.read_bytes())
(public / 'manifest-final.json').write_text(json.dumps(manifest, sort_keys=True, indent=2) + '\n')
print(json.dumps({'files': len(manifest), 'manifest_sha256': digest((public / 'manifest-final.json').read_bytes()), 'original_export_unchanged': True, 'private_material_included': False}, indent=2))
