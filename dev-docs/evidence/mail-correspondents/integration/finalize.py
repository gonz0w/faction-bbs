"""Remove only reviewed, stopped issue235 fixtures and bind actual final cleanup."""
from pathlib import Path
import hashlib,json,os,shutil,subprocess,sys
R=Path('/Users/cam/DEV/faction-bbs');D=R/'dev-docs/evidence/mail-correspondents'
roots=[Path('/private/tmp')/f'faction235-{role}-OWNED' for role in ('native','client','lead','review')]
stop=json.loads((D/'integration/owned-stop-result.json').read_text())
assert stop['result']=='pass' and stop['owned_vm_stopped'] and stop['owned_controller_stopped'] and stop['uid_start_command_fingerprints_matched']
rows=subprocess.check_output(['ps','-axo','pid,ppid,uid,command'],text=True).splitlines()
for p in roots:
 assert p.is_dir() and not p.is_symlink() and p.stat().st_uid==os.getuid() and (p/'OWNED').is_file()
 assert not [row for row in rows if str(p) in row],f'live owned process references {p.name}'
for p in roots:shutil.rmtree(p);assert not p.exists()
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def ref(p):return dict(path=str(p.relative_to(R)),sha256=sha(p))
cleanup=dict(result='pass',owned_removed=all(not p.exists() for p in roots),removed_roots=[p.name for p in roots],saved_stopped=stop['saved_stopped'],saved_hashes_unchanged=stop['saved_hashes_unchanged'],saved_sizes_mtimes_unchanged=stop['saved_sizes_mtimes_unchanged'],checked_saved_files=stop['checked_saved_files'],forwards_closed=stop['forwards_closed'],forward_count=stop['forward_count'])
p=D/'records/cleanup.json';p.write_text(json.dumps(cleanup,indent=2,sort_keys=True)+'\n')
shutil.copyfile('/private/tmp/faction235_finalize.py',D/'integration/finalize.py')
shutil.copyfile('/private/tmp/faction235-integrated-scan.json',D/'integration/integrated-publication-scan.json')
shutil.copyfile('/private/tmp/faction235-integrated-scan.py',D/'integration/integrated-publication-scan.py')
shutil.copyfile(R/'tools/check_correspondents_evidence.py',D/'integration/evidence-guard-before-pin.py')
p=D/'results.draft.json';record=json.loads(p.read_text());record['records']['cleanup']=ref(D/'records/cleanup.json')
known={v['path'] for v in record['sources']}
for p in (D/'integration').iterdir():
 if p.is_file() and str(p.relative_to(R)) not in known:record['sources'].append(ref(p))
record['sources'].sort(key=lambda v:v['path'])
final=D/'results.json';final.write_text(json.dumps(record,indent=2,sort_keys=True)+'\n');(D/'results.draft.json').unlink()
p=R/'tools/check_correspondents_evidence.py';s=p.read_text();assert 'RESULT_SHA256 = None' in s;p.write_text(s.replace('RESULT_SHA256 = None',f'RESULT_SHA256 = "{sha(final)}"',1))
sys.path.insert(0,str(R/'tools'));import check_correspondents_evidence as guard;guard.check()
print(json.dumps(dict(result='pass',owned_roots_removed=4,result_sha256=sha(final),source_refs=len(record['sources']),source_stages=len(record['source_stages']))))
