"""Stop only exact owned VM/controller processes; preserve scratch for lead copying."""
from lab import *
import hashlib,signal,time,errno
records=json.loads((P/'runtime-process-baseline.private.json').read_text())
def same(record):
 r=subprocess.run(['ps','-p',str(record['pid']),'-o','uid=,lstart=,command='],capture_output=True,text=True)
 return r.returncode==0 and r.stdout.strip()==record['start_command'] and hashlib.sha256(r.stdout.strip().encode()).hexdigest()==record['fingerprint'] and int(r.stdout.split()[0])==record['uid']==os.getuid()
assert all(same(v) for v in records.values())
assert json.loads((P/'publication-scan-initial.json').read_text())['result']=='pass'
remote('test -f /etc/faction-disposable-test; /sbin/shutdown -p now',check=False,timeout=10)
for _ in range(80):
 if not same(records['qemu']):break
 time.sleep(.25)
else:raise RuntimeError('Owned VM failed clean shutdown; no other process will be signalled')
if same(records['controller']):os.kill(records['controller']['pid'],signal.SIGTERM)
for _ in range(40):
 if not same(records['controller']):break
 time.sleep(.1)
assert not any(same(v) for v in records.values())
ports=json.loads((V/'ports.json').read_text())
for port in ports.values():
 with __import__('socket').socket() as s:
  s.settimeout(2);assert s.connect_ex(('127.0.0.1',port))==errno.ECONNREFUSED
D=R/'local/demo';B=D/'saved/20261022T005925Z';before=json.loads((P/'saved-before.private.json').read_text());after={}
for name in before:
 path=(B/name) if name in {'playground.qcow2','kernel.img'} else (D/name);h=hashlib.sha256()
 with path.open('rb') as f:
  for data in iter(lambda:f.read(1048576),b''):h.update(data)
 st=path.stat();after[name]=dict(sha256=h.hexdigest(),size=st.st_size,mtime_ns=st.st_mtime_ns)
assert before==after and len(before)==6
commands=subprocess.check_output(['ps','-axo','command='],text=True).splitlines();assert not any(str(D)+'/' in line and ('qemu-system' in line or 'serial-controller.py' in line) for line in commands)
(P/'saved-identity-proof.json').write_text(json.dumps(dict(result='pass',before=before,after=after,saved_stopped_before=True,saved_stopped_after=True),indent=2)+'\n')
result=dict(result='pass',owned_vm_stopped=True,owned_controller_stopped=True,identity_checked_before_stop=True,uid_start_command_fingerprints_matched=True,forwards_closed=True,forward_count=len(ports),checked_saved_files=6,saved_stopped=True,saved_hashes_unchanged=True,saved_sizes_mtimes_unchanged=True,native_root_retained=True,owned_removed=False)
(P/'owned-stop-result.json').write_text(json.dumps(result,indent=2)+'\n');print('Exact owned VM/controller stopped; all4forwards refused; saved6hash/size/mtime identities unchanged; private root retained for leadcopy.')
