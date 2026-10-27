"""Final offline #230 review. No fixtures, runtime probes, builds or mutations."""
from pathlib import Path
import copy
import hashlib
import importlib.util
import json
import subprocess
import sys

root = Path('/Users/cam/DEV/faction-bbs')
base = root / 'dev-docs/evidence/main-menu-mail'
result_path = base / 'results.json'
expected_pin = 'a6de523049a995792a234ec142a4f9a9af0349b3a7a1fca4423d324dbc8f8536'

def digest(data):
    return hashlib.sha256(data).hexdigest()

raw = result_path.read_bytes()
assert digest(raw) == expected_pin
record = json.loads(raw)
relative_files = [str(path.relative_to(root)) for path in (root / 'crates').rglob('*') if path.is_file() and '.DS_Store' not in path.parts]
relative_files += ['.gitattributes', '.github/workflows/documentation.yml', 'tools/check_menu_mail_evidence.py', 'tools/test_menu_mail_evidence.py', 'tools/probe_menu_mail.py', 'tools/test_probe_menu_mail.py', 'docs/main-menu-mail.md', 'dev-docs/main-menu-mail.md', 'dev-docs/evidence/main-menu-mail/README.md', 'dev-docs/evidence/main-menu-mail/publication-layout.md', 'dev-docs/evidence/main-menu-mail/results.json']
before = {name: digest((root / name).read_bytes()) for name in sorted(relative_files)}
Path('/private/tmp/faction230-final-readonly-before.json').write_text(json.dumps(before, sort_keys=True, indent=2) + '\n')
spec = importlib.util.spec_from_file_location('review_guard', root / 'tools/check_menu_mail_evidence.py')
guard = importlib.util.module_from_spec(spec)
spec.loader.exec_module(guard)
assert guard.RESULT_SHA256 == expected_pin
records = guard.validate(root, record)

# Independently prove the publication repair changes only declared source paths.
layout = record['publication_layout']
old_bytes = (root / layout['original_record']['path']).read_bytes()
assert digest(old_bytes) == 'a2c16502e0a61c1bc2e09a70272b8c7f0c61ded9966625cadffe6d20969a90cd'
previous = json.loads(old_bytes)
current = copy.deepcopy(record)
current.pop('publication_layout')
assert len(layout['relocations']) == 10
for relocation in layout['relocations']:
    old = [entry for entry in previous['sources'] if entry['path'] == relocation['original_path']]
    assert len(old) == 1 and old[0]['sha256'] == relocation['sha256']
    assert digest((root / relocation['new_path']).read_bytes()) == relocation['sha256']
    old[0]['path'] = relocation['new_path']
assert current == previous

stages = record['source_stages']
native = next(stage for stage in stages if stage['label'] == 'native-final')
native_files = {entry['name']: entry['sha256'] for entry in native['files']}
crate_files = {name: sha for name, sha in before.items() if name.startswith('crates/')}
assert all(native_files.get(name) == sha for name, sha in crate_files.items())
assert records['native-source-match']['files'] == native_files
assert records['private-io-rescan']['checker_sha256'] == '63298f7aaf97b76f2d995202bdf7a2e5869b094cc4b74816e9636f85b529dccb'
rescan_source = next(entry for entry in record['sources'] if entry['sha256'] == records['private-io-rescan']['checker_sha256'])
executed_probe = (root / rescan_source['path']).read_bytes()
current_probe = (root / 'tools/probe_menu_mail.py').read_bytes()
assert current_probe == executed_probe.replace(b"''' \nCHECKS", b"'''\nCHECKS")
for platform in ['host', 'netbsd']:
    for mode in ['classic', 'unicode']:
        row = records['private-io-rescan']['routes'][platform + '_' + mode]
        assert row['original_route_result_sha256'] == record['routes'][platform][mode]['sha256']
        assert type(row['generated_marker_count']) is int and row['generated_marker_count'] == 11
        assert row['private_marker_matches'] == row['file_protocol_prefix_matches'] == 0

cleanup = records['cleanup']
assert cleanup['owned_roots_removed'] == 8 and cleanup['saved_files_checked'] == 6
assert cleanup['closed_loopback_forwards'] == 4 and cleanup['owned_ssh_children_remaining'] == 0
assert cleanup['saved_size_mtime_unchanged'] is True and cleanup['vm_controller_stopped'] is True
removed = [Path('/private/tmp') / ('faction230-' + name + '-OWNED') for name in ['native', 'client', 'server', 'lead', 'packaging', 'oracle', 'review', 'integration']]
assert all(not path.exists() for path in removed)
vm = json.loads((base / 'integration/metadata/vm-teardown.json').read_bytes())
assert vm['result'] == 'pass' and vm['pid_uid_birth_command_fingerprints_checked'] is True
assert vm['saved_files_checked'] == 6 and vm['owned_ssh_children_remaining'] == 0
assert vm['owned_vm_stopped'] is True and vm['owned_controller_stopped'] is True
markers = json.loads((base / 'integration/metadata/source-root-markers.json').read_bytes())
assert markers['exact_preserved_manifest_matches'] == 2 and markers['cleanup_script_unchanged'] is True
publication_scan = json.loads((base / 'integration/metadata/final-publication-scan.json').read_bytes())
assert publication_scan['result'] == 'pass' and publication_scan['actual_generated_value_scan'] is True
assert publication_scan['secret_matches'] == publication_scan['private_key_material_matches'] == 0

checks = {}
for script in ['tools/check_menu_mail_evidence.py', 'tools/check_evidence_sources.py', 'tools/check_repository.py']:
    run = subprocess.run([sys.executable, script], cwd=root, capture_output=True, text=True)
    assert run.returncode == 0, {'script': script, 'returncode': run.returncode, 'stderr': run.stderr}
    checks[script] = {'returncode': run.returncode, 'stdout': run.stdout.strip()}
after = {name: digest((root / name).read_bytes()) for name in before}
assert before == after
assert result_path.read_bytes() == raw
Path('/private/tmp/faction230-final-readonly-after.json').write_text(json.dumps(after, sort_keys=True, indent=2) + '\n')
verdict = {'schema': 1, 'issue': 230, 'result': 'pass', 'remaining_actionable_findings': [], 'root_sha256': expected_pin, 'original_root_sha256': digest(old_bytes), 'unchanged_markdown_relocations': 10, 'source_references': len(record['sources']), 'source_stages': len(stages), 'current_crate_files_matching_native_final': len(crate_files), 'native_final_source_inputs': len(native_files), 'source_drift': [], 'corrected_private_io_routes': 4, 'actual_generated_markers_per_route': 11, 'private_marker_matches': 0, 'file_protocol_prefix_matches': 0, 'owned_roots_currently_absent': 8, 'saved_owner_files_verified_by_preserved_cleanup_execution': 6, 'closed_forwards_verified_by_preserved_cleanup_execution': 4, 'publication_changes_only_declared_paths_and_metadata': True, 'whitespace_only_probe_delivery_change_preserved': True, 'offline_checks': checks, 'new_fixtures_runtime_builds_vms': False, 'private_source_or_capture_copies': False, 'runtime_model_verified': False}
Path('/private/tmp/faction230-final-readonly-review.json').write_text(json.dumps(verdict, sort_keys=True, indent=2) + '\n')
print(json.dumps(verdict, sort_keys=True, indent=2))
