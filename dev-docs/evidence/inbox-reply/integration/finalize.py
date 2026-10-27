from pathlib import Path
import hashlib,json,os,shutil,subprocess,sys
R=Path('/Users/cam/DEV/faction-bbs');D=R/'dev-docs/evidence/inbox-reply'
roots=[Path('/private/tmp')/f'faction233-{role}-OWNED' for role in ('native','client','lead','review')]
# No wildcard removal or shared playground access. Refuse a live owned process.
rows=subprocess.check_output(['ps','-axo','pid,ppid,uid,command'],text=True).splitlines()
for p in roots:
 assert p.is_dir() and not p.is_symlink() and p.stat().st_uid==os.getuid() and (p/'OWNED').is_file()
 assert not [row for row in rows if str(p) in row],f'live process references {p.name}'
for p in roots:shutil.rmtree(p);assert not p.exists()
private=Path('/private/tmp/faction233-native-teardown-processes.private.json')
if private.exists():assert private.is_file() and not private.is_symlink() and private.stat().st_uid==os.getuid();private.unlink()
stop=json.loads((D/'integration/faction233-native-teardown-stop-result.json').read_text());assert stop['result']=='pass' and stop['owned_vm_stopped'] and stop['owned_ssh_children_absent']
cleanup=dict(result='pass',owned_removed=all(not p.exists() for p in roots),removed_roots=[p.name for p in roots],saved_stopped=stop['saved_stopped'],saved_hashes_unchanged=stop['saved_hashes_unchanged'],saved_sizes_mtimes_unchanged=stop['saved_sizes_mtimes_unchanged'],forwards_closed=stop['forwards_closed'],closed_forward_count=stop['closed_forward_count'],checked_saved_files=stop['checked_saved_files'],private_teardown_process_inventory_removed=not private.exists())
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def ref(p):return dict(path=str(p.relative_to(R)),sha256=sha(p))
cp=D/'records/cleanup.json';cp.write_text(json.dumps(cleanup,indent=2,sort_keys=True)+'\n')
shutil.copyfile('/private/tmp/faction233_finalize.py',D/'integration/finalize.py')
shutil.copyfile('/private/tmp/faction233-integrated-scan.json',D/'integration/integrated-publication-scan.json')
shutil.copyfile(R/'tools/check_reply_evidence.py',D/'integration/evidence-guard-before-pin.py')
p=D/'results.draft.json';record=json.loads(p.read_text());record['records']['cleanup']=ref(cp)
# Bind safe integration and independent review observations; preserve candidate refs exactly.
known={r['path'] for r in record['sources']}
for folder in ('review','integration'):
 for source in sorted((D/folder).rglob('*')):
  if source.is_file() and str(source.relative_to(R)) not in known:record['sources'].append(ref(source))
record['sources'].append(ref(D/'records/integration-checks.json'))
final=D/'results.json';final.write_text(json.dumps(record,indent=2,sort_keys=True)+'\n');p.unlink()
guard=R/'tools/check_reply_evidence.py';text=guard.read_text();assert 'RESULT_SHA256=None' in text;guard.write_text(text.replace('RESULT_SHA256=None',f'RESULT_SHA256="{sha(final)}"',1))
sys.path.insert(0,str(R/'tools'));import check_reply_evidence as g;g.check()
print(json.dumps(dict(result='pass',owned_roots_removed=4,root_sha256=sha(final),source_refs=len(record['sources']),source_stages=len(record['source_stages']))))
