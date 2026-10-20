#!/usr/bin/env python3
"""Verify persistent kernel locks are shared across host/nullfs path views."""
import errno,fcntl,json,os,pwd
from pathlib import Path
assert os.geteuid()==0 and Path('/etc/faction-disposable-test').is_file()
owner=pwd.getpwnam('factiond');state=Path('/var/db/faction-profile')
assert (state/'.profile-fixture').is_file()
paths=[state/'credential-0.lock',state/'credential-1.lock',Path('/var/run/faction/terminal.pid')]
for path in paths:assert path.is_file() and not path.is_symlink()
def identity(sealed):
    if sealed:os.chroot('/var/chroot/faction');os.chdir('/')
    os.setgroups([owner.pw_gid]);os.setgid(owner.pw_gid);os.setuid(owner.pw_uid)
for reverse in [False,True]:
    ready_r,ready_w=os.pipe();release_r,release_w=os.pipe()
    holder=os.fork()
    if holder==0:
        os.close(ready_r);os.close(release_w);identity(reverse);held=[]
        for path in paths:
            f=path.open('r+');fcntl.flock(f,fcntl.LOCK_EX|fcntl.LOCK_NB);held.append(f)
        os.write(ready_w,b'R');os.close(ready_w);os.read(release_r,1);os._exit(0)
    os.close(ready_w);os.close(release_r)
    try:
        assert os.read(ready_r,1)==b'R'
        checker=os.fork()
        if checker==0:
            identity(not reverse)
            for target in paths:
                with target.open('r+') as f:
                    try:fcntl.flock(f,fcntl.LOCK_EX|fcntl.LOCK_NB)
                    except OSError as error:assert error.errno in [errno.EAGAIN,errno.EWOULDBLOCK]
                    else:os._exit(1)
            os._exit(0)
        _,status=os.waitpid(checker,0);assert os.waitstatus_to_exitcode(status)==0
    finally:
        os.close(ready_r);os.close(release_w);_,status=os.waitpid(holder,0);assert os.waitstatus_to_exitcode(status)==0
print(json.dumps({'schema':1,'bidirectional_credentials_and_pid_locks':'pass','lock_inodes_unlinked':False}))
