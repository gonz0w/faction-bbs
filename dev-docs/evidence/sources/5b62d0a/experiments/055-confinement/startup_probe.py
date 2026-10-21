#!/usr/bin/env python3
"""Marked native guest only: actual installed service rejects unsafe launch state."""
import hashlib,json,os,pwd,signal,subprocess,time
from pathlib import Path
assert os.geteuid()==0 and Path('/etc/faction-disposable-test').is_file()
owner=pwd.getpwnam('factiond');terminal=pwd.getpwnam('factionterm');config=Path('/etc/faction/service.toml')
assert not Path('/var/run/faction/terminal.sock').exists()
config.write_text(f'version=1\nterminal_uid={terminal.pw_uid}\nstate_dir="/var/db/faction-profile"\n');config.chmod(0o644)
state=Path('/var/db/faction-profile/faction.sqlite3');before=hashlib.sha256(state.read_bytes()).hexdigest()
results=[]
for mode in ['descriptor','uid','gid','groups']:
    child=os.fork()
    if child==0:
        for fd in range(3):os.dup2(os.open('/dev/null',os.O_RDWR),fd)
        os.closerange(3,os.sysconf('SC_OPEN_MAX'))
        if mode=='descriptor':
            extra=os.open('/dev/null',os.O_RDONLY);os.dup2(extra,64,inheritable=True);os.close(extra)
        os.setgroups([owner.pw_gid]+([0] if mode=='groups' else []))
        if mode=='gid':os.setregid(owner.pw_gid,0)
        else:os.setgid(owner.pw_gid)
        if mode=='uid':os.setreuid(0,owner.pw_uid)
        else:os.setuid(owner.pw_uid)
        os.execve('/usr/local/libexec/faction-service',['faction-service','--config',str(config)],{})
    deadline=time.monotonic()+5
    while time.monotonic()<deadline:
        pid,status=os.waitpid(child,os.WNOHANG)
        if pid:
            assert os.waitstatus_to_exitcode(status)==1,'unsafe launch was not denied'
            break
        time.sleep(.02)
    else:
        os.kill(child,signal.SIGKILL);os.waitpid(child,0);raise AssertionError('unsafe launch remained live')
    assert not Path('/var/run/faction/terminal.sock').exists()
    assert hashlib.sha256(state.read_bytes()).hexdigest()==before,'unsafe launch changed board'
    results.append({'oracle':'installed service denies inherited '+mode+' authority before board/listen','result':'pass'})
# Same configuration must actually start when launched by the sanitized rc.d path.
Path('/etc/rc.conf.d/faction_broker').write_text('faction_broker=YES\nfaction_broker_backend=shared\n')
for command in ['onestart','onestatus','onestop']:
    subprocess.run(['/etc/rc.d/faction_broker',command],stdout=subprocess.DEVNULL,check=True,timeout=15)
results.append({'oracle':'same configuration starts and stops under clean rc.d launch','result':'pass'})
print(json.dumps({'schema':1,'results':results}))
