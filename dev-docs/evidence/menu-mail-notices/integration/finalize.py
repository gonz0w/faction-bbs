"""Finalize reviewed issue239 observations after actual four-root fixture removal.

Invoke this saved file as a standalone command only after the lead has accepted
its source, the copied selection, private original-capture review and stop proof.
Importing this module performs no filesystem mutation or process inspection.
"""
from pathlib import Path
import hashlib
import importlib.util
import json
import os
import re
import shutil
import subprocess

NAMES={'faction239-native-OWNED','faction239-client-OWNED0700',
       'faction239-evidence-OWNED','faction239-review-OWNED'}


def require_records(stop,review,scan,capture_ids,selection_sha):
    assert re.fullmatch('[0-9a-f]{64}',selection_sha)
    assert len(capture_ids)==6 and all(re.fullmatch('[0-9a-f]{32}',v) for v in capture_ids)
    assert stop['result']==review['result']==scan['result']=='pass'
    for field in ('owned_vm_stopped','owned_controller_stopped','uid_start_command_fingerprints_matched',
                  'saved_stopped','saved_hashes_unchanged','saved_sizes_mtimes_unchanged','forwards_closed'):
        assert stop[field] is True,field
    for field in ('original_private_captures_reviewed','private_input_hashes_verified',
                  'source_binary_result_bindings_verified','corrected_stream_reconstruction_verified',
                  'canonical_state_invariants_verified','generated_private_inventory_reviewed'):
        assert review[field] is True,field
    assert len(review['capture_ids'])==6 and set(review['capture_ids'])==capture_ids
    assert review['publication_selection_sha256']==scan['publication_selection_sha256']==selection_sha
    assert type(scan['generated_private_values_scanned']) is int and scan['generated_private_values_scanned']>0
    assert scan['private_matches']==scan['raw_private_key_or_vault_matches']==0
    assert scan['explicit_manifests_verified'] is True


def require_roots(roots,process_rows):
    assert len(roots)==4 and {p.name for p in roots}==NAMES
    for path in roots:
        assert path.is_dir() and not path.is_symlink() and path.stat().st_uid==os.getuid()
        assert path.stat().st_mode&0o777==0o700
        marker=path/'OWNED'
        assert marker.is_file() and not marker.is_symlink() and marker.stat().st_uid==os.getuid()
        assert not [row for row in process_rows if str(path) in row],path.name


def digest(path):return hashlib.sha256(path.read_bytes()).hexdigest()


def finalize():
    directory=Path(__file__).resolve().parent.parent
    root=directory.parents[2]
    guard_path=root/'tools/check_menu_mail_notices_evidence.py'
    spec=importlib.util.spec_from_file_location('notice_final_guard',guard_path)
    guard=importlib.util.module_from_spec(spec);spec.loader.exec_module(guard)
    assert guard.RESULT_SHA256=='' and not (directory/'results.json').exists()
    before_pin=directory/'integration/evidence-guard-before-pin.py'
    assert before_pin.read_bytes()==guard_path.read_bytes()
    draft=directory/'results.draft.json';record=json.loads(draft.read_bytes())
    assert record['schema']==1 and record['issue']==239 and record['result']=='pending'
    selection=directory/'integration/candidate-manifest.json'
    selected=json.loads(selection.read_bytes());assert selected
    for name,sha in selected.items():guard.raw(root,dict(path=name,sha256=sha))
    stop_path=directory/'integration/owned-stop-result.json'
    review_path=directory/'integration/private-capture-review.json'
    scan_path=directory/'integration/publication-scan-final.json'
    stop=json.loads(stop_path.read_bytes());review=json.loads(review_path.read_bytes())
    scan=json.loads(scan_path.read_bytes())
    assert review['reviewed_draft_sha256']==digest(draft)
    ids={guard.value(root,ref)['capture_id'] for routes in record['routes'].values() for ref in routes.values()}
    require_records(stop,review,scan,ids,digest(selection))
    # Verify copied source/observation bytes before removing any private root.
    for ref in record['sources']+record['prior_records']:guard.raw(root,ref)
    sources={ref['sha256'] for ref in record['sources']}
    for kind,routes in record['routes'].items():
        for key,ref in routes.items():guard.route(root,ref,kind,key,sources)
    roots=[Path('/private/tmp')/name for name in sorted(NAMES)]
    rows=subprocess.check_output(['ps','-axo','pid,ppid,uid,command'],text=True).splitlines()
    require_roots(roots,rows)
    for path in roots:
        shutil.rmtree(path)
        assert not path.exists()
    cleanup=dict(result='pass',owned_removed=all(not p.exists() for p in roots),
        removed_roots=[p.name for p in roots],saved_stopped=stop['saved_stopped'],
        saved_hashes_unchanged=stop['saved_hashes_unchanged'],
        saved_sizes_mtimes_unchanged=stop['saved_sizes_mtimes_unchanged'],forwards_closed=stop['forwards_closed'])
    cleanup_path=directory/'records/cleanup.json';cleanup_path.parent.mkdir(parents=True,exist_ok=True)
    cleanup_path.write_text(json.dumps(cleanup,indent=2,sort_keys=True)+'\n')
    def ref(path):return dict(path=str(path.relative_to(root)),sha256=digest(path))
    record['records'].update(cleanup=ref(cleanup_path),publication=ref(scan_path),private_capture_review=ref(review_path),stop=ref(stop_path))
    known={v['path'] for v in record['sources']}
    for path in (cleanup_path,before_pin,Path(__file__).resolve(),selection,review_path,stop_path,scan_path):
        if str(path.relative_to(root)) not in known:record['sources'].append(ref(path))
    record['sources'].sort(key=lambda v:v['path']);record['result']='pass'
    # Full validation is first permitted after observed removal, never on a draft.
    guard.validate(root,record)
    final=directory/'results.json';final.write_text(json.dumps(record,indent=2,sort_keys=True)+'\n')
    source=guard_path.read_text();assert source.count('RESULT_SHA256=""')==1
    guard_path.write_text(source.replace('RESULT_SHA256=""',f'RESULT_SHA256="{digest(final)}"',1))
    spec.loader.exec_module(guard);guard.check()
    draft.unlink()
    print(json.dumps(dict(result='pass',owned_roots_removed=4,result_sha256=digest(final))))


if __name__=='__main__':finalize()
