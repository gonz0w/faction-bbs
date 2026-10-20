#!/usr/bin/env python3
"""Native root fixture orchestrator: real sealed processes and live UID packet rules."""
import errno,json,os,pty,pwd,select,signal,socket,subprocess,threading,time
from pathlib import Path
assert os.geteuid()==0 and Path('/etc/faction-disposable-test').is_file()
owner=pwd.getpwnam('factiond');term=pwd.getpwnam('factionterm')
base=Path('/usr/local/libexec/faction-isolated');root=Path('/var/lib/faction-isolation/service')
source=Path('/home/factiond/build');flags=['cc','-std=c11','-O2','-Wall','-Wextra','-Werror','-fstack-protector-strong','-D_FORTIFY_SOURCE=2','-fPIE','-pie','-Wl,-z,relro,-z,now']
for role in ['owner','terminal']:
    output=base/('probe-'+role)
    cmd=flags+(['-DFACTION_TERMINAL=1'] if role=='terminal' else [])+[str(source/'experiments/055-isolation/isolation_probe.c'),str(source/'native/faction-isolation/startup.c'),str(source/'crates/faction-unix/src/process_security.c'),'-o',str(output)]
    subprocess.run(cmd,check=True);output.chmod(0o4755 if role=='terminal' else 0o700)
# A known-valid static ELF, visible inside the noexec root. No interpreter fallback.
payload=Path('/var/tmp/faction-noexec-payload.c');payload.write_text('int main(void){return 47;}\n')
subprocess.run(['cc','-static',str(payload),'-o',str(root/'probe-executable')],check=True)
(root/'probe-executable').chmod(0o555)
assert subprocess.run([str(root/'probe-executable')]).returncode==47
listeners=[];out_ports=[];incoming=[];incoming_ports=[];reservations=[];blocked_deliveries=[];running=True
for family,address in [(socket.AF_INET,'127.0.0.1'),(socket.AF_INET6,'::1')]:
    for _ in range(32):
        tcp=socket.socket(family,socket.SOCK_STREAM);udp=socket.socket(family,socket.SOCK_DGRAM)
        if family==socket.AF_INET6:
            for candidate in [tcp,udp]:candidate.setsockopt(socket.IPPROTO_IPV6,socket.IPV6_V6ONLY,1)
        udp.bind((address,0));port=udp.getsockname()[1]
        try:tcp.bind((address,port));tcp.listen(8);break
        except OSError:tcp.close();udp.close()
    else:raise AssertionError('no controlled TCP/UDP endpoint available')
    listeners.extend([tcp,udp]);out_ports.append(port)
    reserve=socket.socket(family,socket.SOCK_DGRAM);reserve.bind((address,0));incoming_ports.append(reserve.getsockname()[1]);reservations.append(reserve)
    incoming.append((udp,(address,incoming_ports[-1])))
    raw=socket.socket(family,socket.SOCK_RAW,socket.IPPROTO_ICMPV6 if family==socket.AF_INET6 else socket.IPPROTO_ICMP);raw.close()
def server():
    while running:
        for sock in select.select(listeners,[],[],.03)[0]:
            if sock.type==socket.SOCK_STREAM:
                peer,_=sock.accept();peer.close()
            else:
                data,address=sock.recvfrom(100)
                if data==b'blocked':blocked_deliveries.append(True)
                sock.sendto(data,address)
        for sock,target in incoming:sock.sendto(b'unsolicited',target)
worker=threading.Thread(target=server);worker.start();results=[]
try:
    # Same endpoints actually work for root; a missing listener cannot fake denial.
    for family,address,port in [(socket.AF_INET,'127.0.0.1',out_ports[0]),(socket.AF_INET6,'::1',out_ports[1])]:
        c=socket.socket(family,socket.SOCK_STREAM);c.settimeout(1);c.connect((address,port));c.close()
        c=socket.socket(family,socket.SOCK_DGRAM);c.settimeout(1);c.sendto(b'control',(address,port));assert c.recv(20)==b'control';c.close()
    for reserve in reservations:reserve.close()
    def execute(binary,args,terminal=False):
        log=Path('/var/tmp/faction-isolation-boundary.log');log.touch(mode=0o600);log.chmod(0o600)
        if not terminal:
            with log.open('wb') as out:
                x=subprocess.run([str(binary),*args],stdin=subprocess.DEVNULL,stdout=out,stderr=subprocess.STDOUT,timeout=12)
            assert x.returncode==0,'private owner boundary probe failed: '+log.read_text()[-300:]
            return json.loads(log.read_text())
        pid,fd=pty.fork()
        if pid==0:
            os.fchown(0,term.pw_uid,term.pw_gid);os.setgroups([term.pw_gid]);os.setgid(term.pw_gid);os.setuid(term.pw_uid)
            os.execve(str(binary),[str(binary),*args],{'TERM':'xterm','SSH_CONNECTION':'private fixture'})
        data=b'';deadline=time.monotonic()+12
        while time.monotonic()<deadline:
            if select.select([fd],[],[],.1)[0]:
                try:part=os.read(fd,4096)
                except OSError as error:
                    assert error.errno==errno.EIO;break
                if not part:break
                data+=part;assert len(data)<16384
        else:os.kill(pid,signal.SIGKILL)
        _,status=os.waitpid(pid,0);os.close(fd)
        assert os.waitstatus_to_exitcode(status)==0,'private terminal boundary probe failed: '+data.decode(errors='replace')[-300:]
        return json.loads(data)
    args=[str(x) for x in out_ports+incoming_ports]
    results=[execute(base/'probe-owner',args),execute(base/'probe-terminal',args,True)]
    allocators={}
    for role in ['owner','terminal']:
        output=base/('allocator-'+role)
        subprocess.run(flags+(['-DFACTION_TERMINAL=1'] if role=='terminal' else [])+[str(source/'experiments/058-headroom/allocator.c'),str(source/'native/faction-isolation/startup.c'),'-o',str(output)],check=True)
        output.chmod(0o4755 if role=='terminal' else 0o700)
        allocators[role]=execute(output,[],role=='terminal')
    assert not blocked_deliveries,'a supposedly denied packet reached the root control receiver'
    print(json.dumps({'schema':1,'positive_root_controls':'pass','blocked_packets_received':0,'results':results,'sealed_allocator_readbacks':allocators}))
finally:
    running=False;worker.join(2)
    for sock in listeners:sock.close()
