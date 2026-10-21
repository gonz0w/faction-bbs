#!/usr/bin/env python3
"""Private software-VM oracle only, not a supported public game server.
Run unprivileged in a separately owned marked disposable game VM. Original LORD
and DOSBox are external, separately licensed artifacts; never redistributed here.
The fixed two synthetic game identities use random 256-bit fixture tokens, never
BBS passwords. One original game process globally; DOSBox record locks unproven.
"""
import sys,argparse,fcntl,hashlib,hmac,json,os,re,resource,select,signal,socket,stat,subprocess,threading,time
from pathlib import Path

class Wire:
 def __init__(self,s):self.s=s;self.mode=0;self.command=0;self.count=0;self.lf=False
 def put(self,d):self.s.sendall(d.replace(b'\xff',b'\xff\xff'))
 def data(self,d):
  out=bytearray()
  for b in d:
   if self.mode==1:
    if b==255:out.append(b);self.mode=0
    elif b in (251,252,253,254):self.command=b;self.mode=2
    elif b==250:self.mode=3;self.count=0
    else:self.mode=0
   elif self.mode==2:
    if self.command in (251,253):
     # Only our announced ECHO/SGA; never reflect repeated option loops.
     if not (self.command==253 and b in (1,3)):self.s.sendall(bytes([255,254 if self.command==251 else 252,b]))
    self.mode=0
   elif self.mode in (3,4):
    self.count+=1
    if self.count>128:raise ValueError('subnegotiation limit')
    if self.mode==4 and b==240:self.mode=0
    else:self.mode=4 if b==255 else 3
   elif b==255:self.mode=1
   else:
    if self.lf:
     self.lf=False
     if b in (0,10):continue
    if b==13:self.lf=True
    out.append(b)
  return bytes(out)
 def line(self,prompt,echo):
  self.put(prompt);result=bytearray();deadline=time.monotonic()+30
  while time.monotonic()<deadline:
   self.s.settimeout(max(.1,deadline-time.monotonic()));d=self.s.recv(1)
   if not d:raise EOFError
   for b in self.data(d):
    if b in (10,13):self.put(b'\r\n');return result.decode('ascii')
    if b in (8,127):
     if result:result.pop();self.put(b'\b \b' if echo else b'')
    elif 32<=b<127:
     if len(result)>=96:raise ValueError('line limit')
     result.append(b)
     if echo:self.put(bytes([b]))
    else:raise ValueError('invalid key')
  raise TimeoutError

def dropfile(name,node):
 fields=['COM1:','38400','8',str(node),'38400','N','N','N','N',name,'NONE','NONE','NONE','UNUSED','10','1','10/21/26','900','15','GR','25','Y','NONE','0','12/31/99',str(node),'N','0','0','0','0','01/01/90',f'C:\\NODE{node}\\',f'C:\\NODE{node}\\','Game Sysop',name,'00:00','Y','N','Y','7','0','10/21/26','00:00','00:00','0','0','0','0','NONE','0','0']
 assert len(fields)==52
 return ('\r\n'.join(fields)+'\r\n').encode('ascii')

def limits():
 for r,v in [(resource.RLIMIT_CORE,0),(resource.RLIMIT_NOFILE,48),(resource.RLIMIT_NPROC,16),(resource.RLIMIT_DATA,192*1024*1024),(resource.RLIMIT_STACK,4*1024*1024),(resource.RLIMIT_FSIZE,16*1024*1024),(resource.RLIMIT_CPU,600)]:resource.setrlimit(r,(v,v))

def main():
 a=argparse.ArgumentParser();a.add_argument('--config',type=Path,required=True);a.add_argument('--disposable-fixture',action='store_true');a=a.parse_args()
 assert a.disposable_fixture and os.getuid()!=0
 marker=Path('/etc/faction-disposable-test').lstat();assert stat.S_ISREG(marker.st_mode) and marker.st_uid==0 and marker.st_mode&0o022==0
 st=a.config.lstat();assert stat.S_ISREG(st.st_mode) and st.st_uid==os.getuid() and st.st_mode&0o077==0
 cfg=json.loads(a.config.read_text());home=a.config.resolve().parent;c=home/'c';runtime=home/'runtime';runtime.mkdir(mode=0o700,exist_ok=True)
 for filename,digest in {'LORD.EXE': '22a4f815b6929d57c8d67ce45d0ab7145e13c72bd8322f747628fe76daae0f64', 'LORD.OVR': '09b2f8dd54149c2a19c4dd4fbf1c71e269ac986cb10772a17033e564621e52be'}.items():
  artifact=c/'lord'/filename
  assert artifact.is_file() and not artifact.is_symlink()
  assert hashlib.sha256(artifact.read_bytes()).hexdigest()==digest,'wrong original game artifact'
 leader=(runtime/'host.lock').open('a+b');fcntl.flock(leader,fcntl.LOCK_EX|fcntl.LOCK_NB)
 orphan=subprocess.run(['pgrep','-u',str(os.getuid()),'-f','^'+re.escape(cfg['dosbox'])+'( |$)'],capture_output=True,timeout=2)
 assert orphan.returncode==1,'existing game process; drain before startup'
 users=cfg['users'];assert len(users)==2
 for login,u in users.items():
  assert re.fullmatch('[a-z][a-z0-9]{2,15}',login) and u['node'] in (1,2)
  assert re.fullmatch('[A-Za-z][A-Za-z0-9 ]{2,23}',u['game_name']) and re.fullmatch('[0-9a-f]{64}',u['token_sha256'])
 assert len({u['node'] for u in users.values()})==2
 counts=dict(accepted=0,denied=0,runs=0,reaped=0,busy=0);(runtime/'counts.json').write_text(json.dumps(counts));counter_lock=threading.Lock();workers=threading.BoundedSemaphore(2)
 def count(k):
  with counter_lock:
   counts[k]+=1;tmp=runtime/'counts.tmp';tmp.write_text(json.dumps(counts));tmp.replace(runtime/'counts.json')
 def client(s):
  proc=None;com=None;other=None;drop=None;lock=None
  try:
   s.settimeout(3);s.sendall(bytes([255,251,1,255,251,3]));w=Wire(s)
   w.put(b'Private original LORD software fixture\r\nUse only your separate game access token.\r\n')
   login=w.line(b'Game login: ',True);token=w.line(b'Game access token (hidden): ',False)
   u=users.get(login);digest=hashlib.sha256(token.encode()).hexdigest();token=''
   if not u or not hmac.compare_digest(digest,u['token_sha256']):count('denied');w.put(b'Game login denied.\r\n');return
   lock=(runtime/'game.lock').open('a+b')
   try:fcntl.flock(lock,fcntl.LOCK_EX|fcntl.LOCK_NB)
   except BlockingIOError:count('busy');w.put(b'Original LORD is busy. One game node is enabled.\r\n');return
   node=u['node'];directory=c/f'NODE{node}';directory.mkdir(mode=0o700,exist_ok=True);drop=directory/'DOOR.SYS'
   with drop.open('wb') as f:f.write(dropfile(u['game_name'],node))
   drop.chmod(0o600)
   # A connected private TCP pair exists before DOS starts. Only its game-side
   # FD is inherited. No BBS, member socket, token, log, listener or lock FD.
   with socket.socket() as listener:
    listener.bind(('127.0.0.1',0));listener.listen(1);com=socket.create_connection(listener.getsockname(),timeout=2);other,_=listener.accept()
   conf=runtime/f'node-{node}.conf'
   conf.write_text(f'''[sdl]
fullscreen=false
output=surface
[dosbox]
machine=svga_s3
memsize=16
[cpu]
core=normal
cycles=1000
[mixer]
nosound=true
[midi]
mpu401=none
[serial]
serial1=nullmodem inhsocket:1
serial2=disabled
serial3=disabled
serial4=disabled
[autoexec]
mount c {c}
c:
cd \\lord
config -securemode
call start.bat {node} NORIP
exit
''')
   env={'HOME':str(home),'PATH':'/usr/bin:/bin:/usr/pkg/bin','SDL_VIDEODRIVER':'dummy','SDL_AUDIODRIVER':'dummy','LANG':'C','TERM':'ansi'}
   w.put(b'Starting original LORD; Ctrl-] returns to Faction.\r\n')
   with (runtime/f'node-{node}.log').open('wb') as log:
    proc=subprocess.Popen([sys.executable,str(Path(__file__).resolve()),'--child',cfg['dosbox'],str(conf),str(other.fileno())],stdin=subprocess.DEVNULL,stdout=log,stderr=log,env=env,cwd=home,close_fds=True,pass_fds=(other.fileno(),),start_new_session=True)
   other.close();other=None;count('runs');started=last=time.monotonic();total=0;s.settimeout(3);com.settimeout(3)
   while proc.poll() is None and time.monotonic()-started<900 and time.monotonic()-last<120:
    ready,_,_=select.select([s,com],[],[],.1)
    if s in ready:
     d=s.recv(512)
     if not d:break
     last=time.monotonic();d=w.data(d)
     if d:com.sendall(d)
    if com in ready:
     d=com.recv(1024)
     if not d:break
     total+=len(d)
     if total>4*1024*1024:break
     w.put(d)
   if proc.poll() is not None:w.put(b'\r\nOriginal game finished.\r\n')
  except (OSError,ValueError,EOFError,TimeoutError,subprocess.SubprocessError):pass
  finally:
   if com:com.close()
   if other:other.close()
   if proc:
    try:proc.wait(timeout=1)
    except subprocess.TimeoutExpired:
     try:os.killpg(proc.pid,signal.SIGTERM)
     except ProcessLookupError:pass
     try:proc.wait(timeout=2)
     except subprocess.TimeoutExpired:os.killpg(proc.pid,signal.SIGKILL);proc.wait()
    count('reaped')
   if drop:drop.unlink(missing_ok=True)
   if lock:lock.close()
   s.close();workers.release()
 with socket.socket() as server:
  server.setsockopt(socket.SOL_SOCKET,socket.SO_REUSEADDR,1);server.bind((cfg['bind'],cfg['port']));server.listen(2)
  while True:
   s,_=server.accept();count('accepted')
   if not workers.acquire(blocking=False):s.close();continue
   threading.Thread(target=client,args=(s,),daemon=True).start()
if __name__=='__main__':
 if len(sys.argv)==5 and sys.argv[1]=='--child':
  limits();os.execve(sys.argv[2],[sys.argv[2],'-conf',sys.argv[3],'-socket',sys.argv[4]],os.environ)
 else:main()
