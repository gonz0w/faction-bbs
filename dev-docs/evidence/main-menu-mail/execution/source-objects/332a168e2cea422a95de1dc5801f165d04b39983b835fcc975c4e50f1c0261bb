#!/usr/bin/env python3
"""Read-only native preflight for the opt-in base-OpenSSH key admission profile."""
import argparse,json,os,pwd,shlex,stat,subprocess
from pathlib import Path

def check(initialize=False,migrate=False):
    assert os.getuid()==os.geteuid()==0
    config=Path('/etc/ssh/faction/key_admission_sshd_config')
    for path in [config,*config.parents]:
        s=path.lstat();assert s.st_uid==0 and not s.st_mode&0o022 and not stat.S_ISLNK(s.st_mode)
    s=config.stat();assert stat.S_ISREG(s.st_mode) and stat.S_IMODE(s.st_mode)==0o600 and s.st_nlink==1 and s.st_size<=16384
    for line in config.read_text().splitlines():
        if line.strip() and not line.lstrip().startswith('#'):assert line.split()[0].lower() not in ['include','match']
    helper=pwd.getpwnam('factionauth');terminal=pwd.getpwnam('factionterm');owner=pwd.getpwnam('factiond')
    assert len({0,helper.pw_uid,terminal.pw_uid,owner.pw_uid})==4
    runtime=Path('/var/run/faction-auth')
    for parent in runtime.parents:
        st=parent.lstat();assert stat.S_ISDIR(st.st_mode) and st.st_uid==0 and not st.st_mode&0o022
    if runtime.exists():
        directory=runtime.lstat()
        assert stat.S_ISDIR(directory.st_mode) and directory.st_uid==helper.pw_uid and stat.S_IMODE(directory.st_mode)==0o700
    else:assert initialize
    p=subprocess.run(['/usr/sbin/sshd','-T','-f',str(config)],capture_output=True,text=True,timeout=5,check=True)
    effective=dict(line.split(' ',1) for line in p.stdout.splitlines())
    required={'authenticationmethods':'publickey','authorizedkeyscommanduser':'factionauth',
        'allowusers':'factionterm','permitrootlogin':'no','passwordauthentication':'no','kbdinteractiveauthentication':'no',
        'pubkeyauthentication':'yes','permitemptypasswords':'no','hostbasedauthentication':'no','trustedusercakeys':'none','authorizedprincipalsfile':'none','authorizedprincipalscommand':'none',
        'permituserenvironment':'no','permituserrc':'no','permittty':'yes','disableforwarding':'yes','allowtcpforwarding':'no',
        'allowstreamlocalforwarding':'no','allowagentforwarding':'no','x11forwarding':'no','permittunnel':'no',
        'maxauthtries':'3','maxsessions':'1','maxstartups':'3:30:8','logingracetime':'30','unusedconnectiontimeout':'30',
        'forcecommand':f'/usr/local/libexec/faction-isolated/faction-tty --socket /var/run/faction/terminal.sock --owner-uid {owner.pw_uid}'}
    for key,value in required.items():assert effective[key]==value,'SSH admission boundary differs: '+key
    # This base release omits AuthorizedKeysFile entirely when its list is empty.
    assert effective.get('authorizedkeysfile','none')=='none'
    command=shlex.split(effective['authorizedkeyscommand'])
    assert command[:3]==['/usr/local/libexec/faction-key-admission','--target-uid','%U']
    allowed=['--capacity','--refill-ms','--experimental-capacity'];values={};i=3
    while i<len(command):
        key=command[i];assert key in allowed and key not in values
        if key=='--experimental-capacity':values[key]=True;i+=1;continue
        assert i+1<len(command) and command[i+1].isdigit();values[key]=int(command[i+1]);i+=2
    capacity=values.get('--capacity',8);refill=values.get('--refill-ms',5000)
    assert 1<=capacity<=250 and 100<=refill<=60000
    assert values.get('--experimental-capacity',False) or (capacity<=8 and refill>=5000)
    binary=Path('/usr/local/libexec/faction-key-admission').lstat()
    assert stat.S_ISREG(binary.st_mode) and binary.st_uid==0 and stat.S_IMODE(binary.st_mode)==0o755 and binary.st_nlink==1
    keys=Path('/etc/ssh/faction/authorized_keys').lstat()
    assert stat.S_ISREG(keys.st_mode) and keys.st_uid==0 and not keys.st_mode&0o022 and keys.st_nlink==1 and 0<keys.st_size<=65536
    state=runtime/'admission.state'
    if not state.exists() or migrate:
        assert initialize or migrate,'missing admission state requires quiescent initialization'
        pidfile=Path('/var/run/faction-sshd.pid')
        if pidfile.exists():
            value=pidfile.read_text().strip();assert value.isdigit()
            if int(value)>1:
                try:os.kill(int(value),0)
                except ProcessLookupError:pass
                else:raise AssertionError('live member listener prevents fresh authority')
        # Conservative fail-closed recovery: reject terminal-UID processes or
        # any orphaned SSH session, even if an unrelated admin session blocks it.
        rows=subprocess.check_output(['/bin/ps','-ax','-o','pid=','-o','ppid=','-o','ruid=','-o','uid=','-o','comm='],text=True).splitlines()
        for row in rows:
            fields=row.split(None,4);assert len(fields)==5
            pid,ppid,ruid,uid,comm=fields
            assert int(ruid)!=terminal.pw_uid and int(uid)!=terminal.pw_uid,'live terminal identity prevents initialization'
            assert not (int(ppid)==1 and comm.startswith('sshd-session')),'orphan SSH session prevents initialization'
        if not runtime.exists():
            runtime.mkdir(mode=0o700);os.chown(runtime,helper.pw_uid,helper.pw_gid)
        subprocess.run([command[0],'--migrate-v1' if migrate else '--reset',*command[3:]],check=True,timeout=5)
    if state.exists():
        current=json.loads(subprocess.check_output(['/usr/local/libexec/faction-key-admission','--inspect'],timeout=5))
        assert not current['active'] or (current['capacity']==capacity and current['refill_ms']==refill)
    print('Base SSH key admission configuration verified; no host settings changed.')

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);g=p.add_mutually_exclusive_group();g.add_argument('--initialize',action='store_true');g.add_argument('--migrate-v1',action='store_true');a=p.parse_args();check(a.initialize,a.migrate_v1)
