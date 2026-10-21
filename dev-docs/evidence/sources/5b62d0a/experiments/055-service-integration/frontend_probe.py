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
            server.bind(str(path));os.chown(path,owner.pw_uid,owner.pw_gid);path.chmod(0o660);server.listen(4);server.settimeout(3)
            base={'SSH_CONNECTION':'disposable fixture','TERM':'xterm-256color'}
            def spawn(args=None,env=None):
                pid,fd=pty.fork()
                if pid==0:
                    os.fchown(0,terminal.pw_uid,terminal.pw_gid)
                    os.closerange(3,os.sysconf('SC_OPEN_MAX'))
                    os.setgroups([terminal.pw_gid]);os.setgid(terminal.pw_gid);os.setuid(terminal.pw_uid)
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
            for name,status in [('actual_tty_descriptor_and_success_restore',b'\0'),('backend_error_restores_tty',b'\1'),('backend_eof_restores_tty',None),('sigpipe_ignored_and_tty_restored',b'\0')]:
                pid,fd=spawn()
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
                        if name.startswith('sigpipe'):os.kill(pid,signal.SIGPIPE);time.sleep(.02)
                        if status is not None:control.sendall(status)
                    finally:os.close(tty)
                finish(pid,fd,0 if status==b'\0' else 1);passed(name)
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
    print(json.dumps({'schema':1,'results':results,'scope':'direct fixture contract plus actual SSH regressions; not production adoption'}))


if __name__=='__main__':run()
