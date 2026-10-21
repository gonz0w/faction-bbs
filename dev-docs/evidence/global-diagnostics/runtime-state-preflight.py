#!/usr/bin/env python3
"""Explicit root preflight/configuration for the experimental NetBSD SSH gate.

No firewall, allocator, NTP, overlay, RC enablement or process policy is changed.
"""
import argparse,hashlib,json,os,pwd,re,stat,subprocess,tomllib
from pathlib import Path
CONFIG=Path('/etc/faction/ssh-gate.toml')
LIBRARY=Path('/usr/local/libexec/faction-ssh-gate.so')
TOOL=Path('/usr/local/libexec/faction-ssh-gate')
SSH=Path('/usr/sbin/sshd')
SSH_CONFIG='/etc/ssh/faction/key_admission_sshd_config'
STATE=Path('/var/run/faction-gate/admission.state')

def protected(path,mode=None,maximum=None):
    for parent in path.parents:
        s=parent.lstat();assert stat.S_ISDIR(s.st_mode) and s.st_uid==0 and not s.st_mode&0o022
    s=path.lstat();assert stat.S_ISREG(s.st_mode) and s.st_uid==0 and s.st_nlink==1 and not s.st_mode&0o022
    assert not s.st_mode&0o6000
    if mode is not None:assert stat.S_IMODE(s.st_mode)==mode
    if maximum is not None:assert 0<s.st_size<=maximum
    return s

def call(args):return subprocess.check_output(args,timeout=5)
def effective(path):
    rows=call([str(SSH),'-T','-f',path]).decode().splitlines()
    return {k:v for k,v in (row.split(' ',1) for row in rows)}

def quiescent():
    p=Path('/var/run/faction-sshd.pid')
    if p.exists():
        protected(p,maximum=32);value=p.read_text().strip();assert value.isdecimal() and int(value)>1
        try:os.kill(int(value),0)
        except ProcessLookupError:pass
        else:raise AssertionError('stop the member listener before configuring the gate')
    assert json.loads(call(['/usr/local/libexec/faction-key-admission','--inspect']))['active']==0,'drain old member transports first'
    terminal=pwd.getpwnam('factionterm').pw_uid
    for row in call(['/bin/ps','-ax','-o','ppid=','-o','ruid=','-o','uid=','-o','comm=']).decode().splitlines():
        ppid,ruid,uid,comm=row.split(None,3)
        assert int(ruid)!=terminal and int(uid)!=terminal,'surviving terminal identity prevents setup'
        assert not (int(ppid)==1 and comm.startswith('sshd-session')),'orphan SSH monitor prevents setup'

def check(configure=False,refill_ms=5000,experimental_capacity=False,reset_budget=False):
    assert os.getuid()==os.geteuid()==0
    # The original key/force-command/channel/UID/approval/resource boundary stays.
    subprocess.run(['/usr/bin/env','-i','PATH=/usr/pkg/bin:/usr/sbin:/sbin:/usr/bin:/bin','/usr/pkg/bin/python3.13','-I','-B','/usr/local/libexec/check_key_admission_config.py','--initialize'],check=True,timeout=8,capture_output=True)
    for path in [LIBRARY,TOOL]:protected(path,0o755,16*1024*1024)
    assert stat.S_IMODE(protected(SSH,maximum=16*1024*1024).st_mode) in [0o555,0o755]
    assert LIBRARY.read_bytes()[:4]==b'\x7fELF'
    symbols=call(['/usr/bin/nm','-D',str(SSH)]).decode()
    for name in ['accept','listen','execv']:assert re.search(r' U '+name+r'$',symbols,re.M),'unverified base SSH symbol: '+name
    # A stopped listener may leave authenticated monitors, which the key gate
    # still counts. Refuse a new preauth cohort while orphan sshd-auth processes
    # from an old listener survive. Unrelated live-listener admin SSH is excluded.
    rows={}
    for row in call(['/bin/ps','-ax','-o','pid=','-o','ppid=','-o','comm=']).decode().splitlines():
        pid,ppid,comm=row.split(None,2);rows[int(pid)]=(int(ppid),comm)
    for ppid,comm in rows.values():
        if comm.startswith('sshd-auth') and ppid in rows:
            parent_ppid,parent_comm=rows[ppid]
            assert not (parent_ppid==1 and parent_comm.startswith('sshd-session')),'orphan preauthentication cohort prevents restart'
    current=effective(SSH_CONFIG);base=effective('/dev/null')
    for key in ['persourcepenalties','persourcenetblocksize','persourcepenaltyexemptlist']:
        assert current.get(key)==base.get(key),'native source policy must remain unchanged: '+key
    assert current['persourcepenalties']!='no'
    root=STATE.parent
    for parent in root.parents:
        s=parent.lstat();assert stat.S_ISDIR(s.st_mode) and s.st_uid==0 and not s.st_mode&0o022
    if configure:
        assert 100<=refill_ms<=60000 and (experimental_capacity or refill_ms>=5000)
        quiescent();os.umask(0o077)
        if not root.exists():root.mkdir(mode=0o700)
    s=root.lstat();assert stat.S_ISDIR(s.st_mode) and s.st_uid==0 and stat.S_IMODE(s.st_mode)==0o700
    proposed={'version':1,'refill_ms':refill_ms,'experimental_capacity':experimental_capacity,
              'sshd_sha256':hashlib.sha256(SSH.read_bytes()).hexdigest(),'module_sha256':hashlib.sha256(LIBRARY.read_bytes()).hexdigest()}
    if configure:
        if CONFIG.exists():protected(CONFIG,0o600,8192)
        args=[str(TOOL),'--reset' if reset_budget else '--initialize','--refill-ms',str(refill_ms)]
        if experimental_capacity:args.append('--experimental-capacity')
        subprocess.run(args,check=True,timeout=5,capture_output=True)
        text=''.join(k+' = '+json.dumps(v)+'\n' for k,v in proposed.items())
        fd=os.open(CONFIG,os.O_WRONLY|os.O_CREAT|os.O_NOFOLLOW|os.O_TRUNC,0o600)
        with os.fdopen(fd,'w') as out:out.write(text);out.flush();os.fsync(out.fileno())
    protected(CONFIG,0o600,8192);config=tomllib.loads(CONFIG.read_text())
    assert set(config)==set(proposed) and type(config['version']) is int and config['version']==1
    assert type(config['refill_ms']) is int and 100<=config['refill_ms']<=60000
    assert type(config['experimental_capacity']) is bool and (config['experimental_capacity'] or config['refill_ms']>=5000)
    assert config['sshd_sha256']==proposed['sshd_sha256'] and config['module_sha256']==proposed['module_sha256']
    protected(STATE,0o600,120);state=json.loads(call([str(TOOL),'--inspect']))
    assert state['version']==1 and state['refill_ms']==config['refill_ms'] and state['source_burst']==2 and state['source_slots']==32
    print('Experimental listener gate verified; base source penalties unchanged; no host policy changed.')
if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--configure',action='store_true');p.add_argument('--reset-budget',action='store_true');p.add_argument('--refill-ms',type=int,default=5000);p.add_argument('--experimental-capacity',action='store_true');a=p.parse_args()
    assert not a.reset_budget or a.configure
    check(a.configure,a.refill_ms,a.experimental_capacity,a.reset_budget)
