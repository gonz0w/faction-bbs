#!/usr/bin/env python3
"""Explicit experimental NetBSD profile setup/preflight; never applies firewall rules."""
import argparse,fcntl,grp,os,pwd,re,stat,subprocess,tomllib
from pathlib import Path

BASE=Path('/var/lib/faction-isolation');ROOTS={'service':Path('/var/chroot/faction')}
MIRROR_OWNER=None
CONFIG=Path('/etc/faction/service.toml');IDS=Path('/etc/faction/isolation.ids');NPF=Path('/etc/faction/npf-isolation.conf')

def path(value):
    assert isinstance(value,str) and re.fullmatch(r'/[A-Za-z0-9_/-]{1,239}',value)
    p=Path(value);assert str(p)==value and not any(x in ['.','..'] or x.startswith('.') for x in p.parts)
    return p

def protected(p,directory=False,owner=0):
    st=p.lstat();assert not stat.S_ISLNK(st.st_mode) and st.st_uid==owner and not st.st_mode&0o022,str(p)
    assert stat.S_ISDIR(st.st_mode) if directory else stat.S_ISREG(st.st_mode) and st.st_nlink==1
    return st

def parents(p):
    for ancestor in reversed(p.parents):protected(ancestor,True)

def read(p,limit=262144,owner=0):
    parents(p);protected(p,owner=owner)
    fd=os.open(p,os.O_RDONLY|os.O_NOFOLLOW|os.O_NONBLOCK|os.O_CLOEXEC)
    try:
        st=os.fstat(fd);assert st.st_uid==owner and stat.S_ISREG(st.st_mode) and st.st_nlink==1 and not st.st_mode&0o022
        raw=os.read(fd,limit+1);assert len(raw)<=limit;return raw
    finally:os.close(fd)

def directory(p):
    if not p.exists():directory(p.parent);p.mkdir(mode=0o755)
    actual=p.lstat().st_uid
    expected=MIRROR_OWNER if p.is_relative_to(BASE/'service') and actual==MIRROR_OWNER else 0
    protected(p,True,expected)

def write(p,raw,mode=0o644,owner=0):
    directory(p.parent)
    if p.exists() or p.is_symlink():protected(p,owner=owner)
    temp=p.with_name(p.name+'.new')
    fd=os.open(temp,os.O_WRONLY|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW|os.O_CLOEXEC,0o600)
    try:
        with os.fdopen(fd,'wb') as out:out.write(raw);out.flush();os.fsync(out.fileno());os.fchmod(out.fileno(),mode);os.fchown(out.fileno(),owner,0)
        os.replace(temp,p)
    finally:
        if temp.exists():temp.unlink()

def command(argv):
    x=subprocess.run(argv,capture_output=True,timeout=20);assert x.returncode==0,x.stderr.decode(errors='replace')[-500:]
    assert len(x.stdout)<=65536;return x.stdout.decode()

def canonical(text):
    return ' '.join(re.sub(r'#.*','',text).split())

def network():
    policy=read(NPF).decode();expected=canonical(command(['/sbin/npfctl','validate',str(NPF)]))
    actual=command(['/sbin/npfctl','show'])
    assert re.search(r'^# filtering:\s*active\s*$',actual,re.M),'NPF is inactive'
    assert canonical(actual)==expected,'active NPF policy differs; no automatic firewall replacement'
    flows=command(['/sbin/npfctl','list','-n']);assert not canonical(flows),'existing NPF states require explicit operator review'
    assert 'stateful' not in policy and 'skip' not in policy

def main():
    parser=argparse.ArgumentParser(description=__doc__);group=parser.add_mutually_exclusive_group(required=True)
    group.add_argument('--configure',action='store_true');group.add_argument('--verify',action='store_true');args=parser.parse_args()
    assert os.geteuid()==0 and os.uname().sysname=='NetBSD'
    global MIRROR_OWNER
    owner=pwd.getpwnam('factiond');MIRROR_OWNER=owner.pw_uid;terminal=pwd.getpwnam('factionterm')
    assert owner.pw_uid and terminal.pw_uid and owner.pw_uid!=terminal.pw_uid and owner.pw_gid==terminal.pw_gid and owner.pw_gid
    permitted={owner.pw_name,terminal.pw_name}
    assert set(grp.getgrgid(owner.pw_gid).gr_mem)<=permitted,'IPC group must be dedicated'
    assert all(a.pw_name in permitted for a in pwd.getpwall() if a.pw_gid==owner.pw_gid),'IPC group must be dedicated'
    raw=read(CONFIG,8192);config=tomllib.loads(raw.decode());state=path(config['state_dir'])
    assert config['terminal_uid']==terminal.pw_uid
    parents(state);protected(state,True,owner.pw_uid)
    run=Path('/var/run/faction');parents(run)
    if not run.exists():run.mkdir(mode=0o710);os.chown(run,owner.pw_uid,owner.pw_gid)
    st=protected(run,True,owner.pw_uid);assert st.st_mode&0o777==0o710
    lock=None;pid=run/'terminal.pid'
    if pid.exists():
        lock=os.open(pid,os.O_RDWR|os.O_NOFOLLOW|os.O_CLOEXEC);st=os.fstat(lock)
        assert stat.S_ISREG(st.st_mode) and st.st_uid==owner.pw_uid and st.st_nlink==1 and st.st_mode&0o777==0o600
        fcntl.flock(lock,fcntl.LOCK_EX|fcntl.LOCK_NB) # Never unlink a live lock.
    policy=f'{owner.pw_uid}\n{terminal.pw_uid}\n{owner.pw_gid}\n{state}\n'.encode()
    npf=f'set bpf.jit off\ngroup default {{\n block final all group {owner.pw_gid}\n block final all user {owner.pw_uid}\n block final all user {terminal.pw_uid}\n pass final all\n}}\n'.encode()
    if args.configure:
        if IDS.exists():assert read(IDS,511).splitlines()[-1]==str(state).encode(),'state change requires explicit teardown'
        write(IDS,policy,0o600)
        if NPF.exists():assert read(NPF)==npf,'existing NPF candidate differs'
        else:write(NPF,npf,0o600)
        for role in ROOTS:directory(BASE/role);directory(ROOTS[role]);directory(BASE/role/'var/run/faction');BASE.chmod(0o700)
        directory(BASE/'service'/str(state).lstrip('/'))
        write(BASE/'service/etc/faction/service.toml',raw,0o400,owner.pw_uid)
        for key in ['theme','season']:
            if key not in config:continue
            source=path(config[key]);parents(source);protected(source,True)
            mirror=BASE/'service'/str(source).lstrip('/')
            directory(mirror);os.chown(mirror,owner.pw_uid,0);mirror.chmod(0o500)
            entries=list(source.rglob('*'));assert len(entries)<=256
            for entry in entries:
                relative=entry.relative_to(source);target=BASE/'service'/str(source).lstrip('/')/relative
                if entry.is_dir() and not entry.is_symlink():protected(entry,True);directory(target);os.chown(target,owner.pw_uid,0);target.chmod(0o500)
                else:write(target,read(entry,1048576),0o400,owner.pw_uid)
    else:
        assert read(IDS,511)==policy
        assert read(BASE/'service/etc/faction/service.toml',8192,owner.pw_uid)==raw,'configuration changed; explicitly configure before start'
        assert read(NPF)==npf;network()
    mount_lines=command(['/sbin/mount']).splitlines()
    mounts={}
    for line in mount_lines:
        match=re.fullmatch(r'(\S+) on (\S+) type (\S+) \(([^)]*)\)',line)
        if match:mounts[match[2]]=(match[1],match[3],set(match[4].split(', ')))
    wanted=[(BASE/role,ROOTS[role],True) for role in ROOTS]
    wanted += [(run,ROOTS[role]/'var/run/faction',False) for role in ROOTS]
    wanted += [(state,ROOTS['service']/str(state).lstrip('/'),False)]
    for source,target,readonly in wanted:
        opts={'noexec','nosuid','nodev','ro' if readonly else 'rw'}
        if str(target) not in mounts:
            assert args.configure or target.exists()
            command(['/sbin/mount','-t','null','-o',','.join(sorted(opts)),str(source),str(target)])
        else:
            row=mounts[str(target)];assert row[0]==str(source) and row[1]=='null' and {'noexec','nosuid','nodev'}<=row[2] and ('read-only' in row[2])==readonly,'unexpected mount at '+str(target)
        actual=os.statvfs(target).f_flag;flags=os.ST_NOEXEC|os.ST_NOSUID|os.ST_NODEV|(os.ST_RDONLY if readonly else 0)
        assert actual&flags==flags
    if lock is not None:os.close(lock)
    print('Isolation layout configured; apply/review NPF explicitly before start.' if args.configure else 'Isolation mounts/configuration/active stateless GID/UID policy verified.')

if __name__=='__main__':main()
