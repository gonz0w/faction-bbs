"""Stop only the reviewed issue233 owned VM; retain its root for proof copying."""
from pathlib import Path
import os,json,hashlib,subprocess,shlex,signal,socket,errno,time,sys,stat
P=Path('/private/tmp/faction233-native-OWNED');V=P/'vm';R=Path('/Users/cam/DEV/faction-bbs');D=R/'local/demo';B=D/'saved/20261022T005925Z'
assert sys.argv[1:]==['--execute-after-review'];os.umask(0o077)
assert P.resolve()==P and stat.S_ISDIR(P.lstat().st_mode) and P.lstat().st_uid==os.getuid() and stat.S_IMODE(P.lstat().st_mode)==0o700 and (P/'OWNED').is_file()
sys.path.insert(0,str(P));sys.path.insert(0,str(P/'source-final01/tools'))
from playground import process,same_process
from lab import remote
output=Path('/private/tmp/faction233-native-teardown-stop-result.json');assert not output.exists()
before=json.loads((P/'saved-before.private.json').read_text());ports=json.loads((V/'ports.json').read_text());assert len(set(ports.values()))==4
records=[process(int((V/'runtime-qemu.pid').read_text())),process(int((V/'runtime-controller.pid').read_text()))]
assert all(row is not None and str(V) in row['command'] for row in records)
assert Path(shlex.split(records[0]['command'])[0]).name.startswith('qemu-system') and str(V/'disk.qcow2') in records[0]['command']
assert str(V/'serial-controller.py') in records[1]['command']
private=Path('/private/tmp/faction233-native-teardown-processes.private.json');assert not private.exists();private.write_text(json.dumps(records,indent=2)+'\n');private.chmod(0o600)
def owner_stopped():
 for line in subprocess.check_output(['ps','-axo','command='],text=True).splitlines():
  try:args=shlex.split(line)
  except ValueError:continue
  if not args:continue
  if Path(args[0]).name.startswith('qemu-system') and any(x.startswith(str(D)+'/') or 'file='+str(D)+'/' in x for x in args):raise AssertionError('saved owner VM active')
  if Path(args[0]).name.lower().startswith('python') and any(x.startswith(str(D)+'/') and x.endswith('serial-controller.py') for x in args):raise AssertionError('saved owner controller active')
def digest(path):
 h=hashlib.sha256()
 with path.open('rb') as source:
  for block in iter(lambda:source.read(1048576),b''):h.update(block)
 return h.hexdigest()
owner_stopped()
shutdown=remote('/sbin/shutdown -p now',check=False,timeout=10)
Path('/private/tmp/faction233-native-shutdown.private.log').write_bytes(shutdown.stdout+shutdown.stderr)
for _ in range(80):
 actual=process(records[0]['pid'])
 if actual is None:break
 assert actual==records[0],'owned VM identity changed; refusing further action'
 time.sleep(.25)
else:raise RuntimeError('owned VM did not cleanly power off; root retained')
if same_process(records[1]):os.kill(records[1]['pid'],signal.SIGTERM)
for _ in range(50):
 if process(records[1]['pid']) is None:break
 assert same_process(records[1]);time.sleep(.1)
else:raise RuntimeError('owned controller remains; root retained')
closed=[]
for port in ports.values():
 with socket.socket() as connection:
  connection.settimeout(1);code=connection.connect_ex(('127.0.0.1',port));assert code==errno.ECONNREFUSED,'owned forward did not close';closed.append(True)
checks={}
for name,expected in before.items():
 path=B/name if name in {'playground.qcow2','kernel.img'} else D/name;info=path.stat();actual={'sha256':digest(path),'size':info.st_size,'mtime_ns':info.st_mtime_ns};assert actual==expected,name;checks[name]=True
owner_stopped()
for line in subprocess.check_output(['ps','-axo','command='],text=True).splitlines():
 try:args=shlex.split(line)
 except ValueError:continue
 if not args:continue
 if Path(args[0]).name=='ssh' and any(str(P/'member-key')==x or str(V/'known_hosts') in x for x in args):raise AssertionError('owned SSH child remains')
result={'result':'pass','phase':'owned VM stopped; native root retained for proof copying','owned_vm_stopped':True,'owned_controller_stopped':True,'forwards_closed':all(closed),'closed_forward_count':len(closed),'saved_stopped':True,'saved_hashes_unchanged':True,'saved_sizes_mtimes_unchanged':True,'checked_saved_files':len(checks),'saved_disk_sha256':before['playground.qcow2']['sha256'],'saved_kernel_sha256':before['kernel.img']['sha256'],'owned_ssh_children_absent':True,'native_root_retained':P.is_dir(),'owned_removed':False,'source_sha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest()}
with output.open('x') as out:json.dump(result,out,indent=2);out.write('\n')
print('Reviewed owned VM/controller stopped; four loopback forwards closed; six saved file identities unchanged; native root retained for proof copying.')
