#!/usr/bin/env python3
"""Direct minimal frontend contract oracles: marked disposable NetBSD guest only."""
import array
import json
import os
import pty
import select
import pwd
import signal
import socket
import stat
import tempfile
import termios
import time
from pathlib import Path


def run():
    assert os.geteuid()==0 and Path('/etc/faction-disposable-test').is_file()
    owner=pwd.getpwnam('factiond');terminal=pwd.getpwnam('factionterm')
    binary='/usr/local/libexec/faction-tty';results=[]
    def passed(name):results.append({'oracle':name,'result':'pass'})
    with tempfile.TemporaryDirectory(prefix='faction-frontend-',dir='/var/run') as root:
        root=Path(root);os.chown(root,owner.pw_uid,owner.pw_gid);root.chmod(0o710)
        path=root/'terminal.sock'
        with socket.socket(socket.AF_UNIX,socket.SOCK_STREAM) as server:
            os.setegid(owner.pw_gid);os.seteuid(owner.pw_uid)
            try:server.bind(str(path));path.chmod(0o660);server.listen(4)
            finally:os.seteuid(0);os.setegid(0)
            server.settimeout(3)
            base={'SSH_CONNECTION':'disposable fixture','TERM':'xterm-256color'}
            def spawn(args=None,env=None,identity=None,mixed=None,leak=False):
                pid,fd=pty.fork()
                if pid==0:
                    os.fchown(0,terminal.pw_uid,terminal.pw_gid)
                    if mixed=='stdout':
                        other,slave=pty.openpty();os.fchown(slave,terminal.pw_uid,terminal.pw_gid);os.dup2(slave,1)
                    if mixed=='stderr':os.dup2(os.open('/dev/null',os.O_WRONLY),2)
                    if mixed=='owner':os.fchown(0,owner.pw_uid,owner.pw_gid)
                    os.closerange(3,os.sysconf('SC_OPEN_MAX'))
                    if leak:
                        extra=os.open('/dev/null',os.O_RDONLY);os.dup2(extra,64,inheritable=True);os.close(extra)
                    os.setgroups([terminal.pw_gid]+([0] if identity=='groups' else []))
                    if identity=='gid':os.setregid(terminal.pw_gid,0)
                    else:os.setgid(terminal.pw_gid)
                    if identity=='uid':os.setreuid(0,terminal.pw_uid)
                    else:os.setuid(terminal.pw_uid)
                    os.execve(binary,[binary]+(args if args is not None else ['--socket',str(path),'--owner-uid',str(owner.pw_uid)]),base if env is None else env)
                return pid,fd
            def finish(pid,fd,code):
                try:
                    end=time.monotonic()+5
                    while time.monotonic()<end:
                        p,status=os.waitpid(pid,os.WNOHANG)
                        if p:
                            assert os.waitstatus_to_exitcode(status)==code
                            assert termios.tcgetattr(fd)[3]&termios.ECHO,'TTY restoration failed'
                            return
                        if select.select([fd],[],[],.01)[0]:
                            try:os.read(fd,8192)
                            except OSError:pass
                    os.kill(pid,signal.SIGKILL);os.waitpid(pid,0);raise AssertionError('frontend did not exit')
                finally:os.close(fd)
            for name,status in [('actual_tty_descriptor_and_success_restore',b'\0'),('backend_error_restores_tty',b'\1'),('backend_eof_restores_tty',None),('sigpipe_ignored_and_tty_restored',b'\0'),('inherited_fd_is_closed',b'\0'),('trailing_control_denied',b'\0x'),('invalid_status_denied',b'\x7f'),('success_without_eof_times_out',b'\0')]:
                pid,fd=spawn(leak=name=='inherited_fd_is_closed')
                with server.accept()[0] as control:
                    control.settimeout(3)
                    packet,rights,flags,_=control.recvmsg(72,socket.CMSG_SPACE(4))
                    assert flags==0 and len(rights)==1
                    level,kind,data=rights[0];assert level==socket.SOL_SOCKET and kind==socket.SCM_RIGHTS
                    fds=array.array('i');fds.frombytes(data);assert len(fds)==1
                    tty=fds[0]
                    try:
                        while len(packet)<72:
                            data=control.recv(72-len(packet));assert data;packet+=data
                        assert packet==b'FBSTTY01'+bytes([14])+b'xterm-256color'+bytes(49)
                        assert os.isatty(tty) and os.fstat(tty).st_uid==terminal.pw_uid
                        attrs=termios.tcgetattr(tty);attrs[3]&=~termios.ECHO;termios.tcsetattr(tty,termios.TCSANOW,attrs)
                        if name=='inherited_fd_is_closed':
                            import subprocess
                            table=subprocess.run(['/usr/bin/fstat','-p',str(pid)],capture_output=True,check=True).stdout.decode()
                            numbers=[int(line.split()[3]) for line in table.splitlines()[1:] if len(line.split())>3 and line.split()[3].isdigit()]
                            assert 64 not in numbers and set(numbers)<={0,1,2,3,4}
                        if name.startswith('sigpipe'):os.kill(pid,signal.SIGPIPE);time.sleep(.02)
                        if status is not None:control.sendall(status)
                        if name=='success_without_eof_times_out':time.sleep(2.3)
                    finally:os.close(tty)
                finish(pid,fd,0 if status==b'\0' and name!='success_without_eof_times_out' else 1);passed(name)
            for mode in ['uid','gid','groups']:
                pid,fd=spawn(identity=mode);finish(pid,fd,1);passed('elevated_or_extra_'+mode+'_denied')
            for mixed in ['stdout','stderr','owner']:
                pid,fd=spawn(mixed=mixed);finish(pid,fd,1);passed('mixed_or_foreign_tty_'+mixed+'_denied')
            cases=[('empty_exec_presence_denied',None,{**base,'SSH_ORIGINAL_COMMAND':''}),
                   ('missing_transport_environment_denied',None,{'TERM':'xterm'}),
                   ('terminal_controls_denied',None,{**base,'TERM':'xterm\x1b[2J'}),
                   ('empty_terminal_denied',None,{**base,'TERM':''}),
                   ('oversized_terminal_denied',None,{**base,'TERM':'x'*64}),
                   ('root_owner_denied',['--socket',str(path),'--owner-uid','0'],None),
                   ('same_uid_owner_denied',['--socket',str(path),'--owner-uid',str(terminal.pw_uid)],None),
                   ('owner_overflow_denied',['--socket',str(path),'--owner-uid','4294967296'],None),
                   ('negative_owner_denied',['--socket',str(path),'--owner-uid','-1'],None),
                   ('owner_suffix_denied',['--socket',str(path),'--owner-uid',str(owner.pw_uid)+'x'],None),
                   ('unknown_arguments_denied',['--owner-tools'],None),
                   ('oversized_path_denied',['--socket','/'+('x'*300),'--owner-uid',str(owner.pw_uid)],None)]
            for name,args,env in cases:
                pid,fd=spawn(args,env);finish(pid,fd,1);passed(name)
            root.chmod(0o770);pid,fd=spawn();finish(pid,fd,1);root.chmod(0o710);passed('writable_parent_denied')
            os.chown(path,terminal.pw_uid,terminal.pw_gid);pid,fd=spawn();finish(pid,fd,1);os.chown(path,owner.pw_uid,owner.pw_gid);passed('wrong_socket_owner_denied')
            path.chmod(0o666);pid,fd=spawn();finish(pid,fd,1);path.chmod(0o660);passed('world_accessible_socket_denied')
            link=root/'link.sock';link.symlink_to(path);pid,fd=spawn(['--socket',str(link),'--owner-uid',str(owner.pw_uid)]);finish(pid,fd,1);passed('socket_symlink_denied')
    with tempfile.TemporaryDirectory(prefix='faction-false-peer-',dir='/var/run') as root:
        root=Path(root);os.chown(root,owner.pw_uid,owner.pw_gid);root.chmod(0o710);path=root/'terminal.sock'
        with socket.socket(socket.AF_UNIX,socket.SOCK_STREAM) as server:
            server.bind(str(path));os.chown(path,owner.pw_uid,owner.pw_gid);path.chmod(0o660);server.listen(1);server.settimeout(3)
            pid,fd=spawn()
            with server.accept()[0] as control:
                control.settimeout(3);assert control.recv(72)==b'', 'TTY leaked to wrong kernel peer'
            finish(pid,fd,1);passed('root_peer_with_forged_owner_metadata_denied_before_rights')
    print(json.dumps({'schema':1,'results':results,'scope':'direct fixture contract plus actual SSH regressions; not production adoption'}))


if __name__=='__main__':run()
