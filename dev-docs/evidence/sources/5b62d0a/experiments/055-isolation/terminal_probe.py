#!/usr/bin/env python3
"""Private native root oracle: use the actual shared nullfs socket namespace."""
import array,fcntl,json,os,pty,pwd,select,socket,struct,sys,termios,time
from pathlib import Path
assert os.geteuid()==0 and Path('/etc/faction-disposable-test').is_file()
# Python and all extension modules/accounts are loaded before entering the root.
# Test-only preopened PTYs replace pathname-based /dev access after chroot.
source=Path('/home/factiond/build/experiments/055-confinement/terminal_probe.py').read_text()
accounts={name:pwd.getpwnam(name) for name in ['factiond','factionterm']}
pairs=[pty.openpty() for _ in range(5)]
for _,slave in pairs:os.fchown(slave,accounts['factionterm'].pw_uid,accounts['factionterm'].pw_gid)
original=pwd.getpwnam
pwd.getpwnam=lambda name:accounts[name]
pty.openpty=lambda:pairs.pop(0)
os.chroot('/var/chroot/faction');os.chdir('/')
exec(compile(source,'preserved-terminal-oracles','exec'))
assert not pairs
