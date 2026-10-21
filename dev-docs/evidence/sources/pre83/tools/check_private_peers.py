#!/usr/bin/env python3
"""Two private peers behind one NAT: destructive marked two-VM fixture only.

No provider/deployment support; native wg remains experimental. Connection argv
JSON files, SSH identities and generated tunnel keys are private fixture inputs.
"""
import argparse,json,shlex,subprocess,threading,time,os
from pathlib import Path
from check_restricted_ssh import SSHConsole

def run(a):
    assert a.disposable_fixture
    server=json.loads(a.server_admin_argv.read_text());peer=json.loads(a.peer_admin_argv.read_text())
    assert server!=peer and all(isinstance(x,str) for x in server+peer)
    def command(argv,cmd,data=None,ok=True,timeout=30):
        x=subprocess.run(argv+['set -e; PATH=/usr/sbin:/sbin:/usr/bin:/bin:/usr/pkg/bin; export PATH; '+cmd],input=data,capture_output=True,timeout=timeout)
        assert not ok or x.returncode==0,'private peer fixture operation failed'
        return x
    def sr(cmd,data=None,ok=True):return command(server,cmd,data,ok)
    def pr(cmd,data=None,ok=True,timeout=30):return command(peer,cmd,data,ok,timeout)
    port=a.udp_port
    for argv in [server,peer]:
        assert b'if_wg ' in command(argv,'test -f /etc/faction-disposable-test; /sbin/modstat -n if_wg').stdout
    assert json.loads(sr('/usr/local/libexec/faction-key-admission --inspect').stdout)['active']==0
    sr('test -f /etc/faction-disposable-test; install -d -m 700 /root/faction-wg; umask 077; wg-keygen > /root/faction-wg/server; wg-keygen --pub < /root/faction-wg/server > /root/faction-wg/server.pub')
    pr('test -f /etc/faction-disposable-test; mkdir -p /root/faction-wg; umask 077; wg-keygen > /root/faction-wg/a; wg-keygen > /root/faction-wg/b; wg-keygen --pub < /root/faction-wg/a > /root/faction-wg/a.pub; wg-keygen --pub < /root/faction-wg/b > /root/faction-wg/b.pub')
    pub=sr('cat /root/faction-wg/server.pub').stdout.decode().strip();peer_a=pr('cat /root/faction-wg/a.pub').stdout.decode().strip();peer_b=pr('cat /root/faction-wg/b.pub').stdout.decode().strip()
    cmd='ifconfig wg0 create; ifconfig wg0 inet 172.28.0.1/24; ifconfig wg0 inet 172.29.0.1/24 alias; wgconfig wg0 set private-key /root/faction-wg/server; wgconfig wg0 set listen-port 51820; wgconfig wg0 add peer A '+shlex.quote(peer_a)+' --allowed-ips=172.28.0.2/32; wgconfig wg0 add peer B '+shlex.quote(peer_b)+' --allowed-ips=172.29.0.2/32; ifconfig wg0 up'
    sr(cmd)
    for n,network,key in [(0,28,'a'),(1,29,'b')]:
     pr(f'ifconfig wg{n} create; ifconfig wg{n} inet 172.{network}.0.2/24; wgconfig wg{n} set private-key /root/faction-wg/{key}; wgconfig wg{n} add peer server '+shlex.quote(pub)+f' --allowed-ips=172.{network}.0.1/32 --endpoint=10.0.2.2:{port}; ifconfig wg{n} up')
    # Exact host-key trust originates from primary trusted admin/serial, never TOFU.
    host=sr('cat /etc/ssh/ssh_host_ed25519_key.pub').stdout.decode().split();known=''.join('[172.'+str(n)+'.0.1]:2222 '+' '.join(host[:2])+'\n' for n in [28,29])
    pr('umask 077; cat > /root/faction-wg/known_hosts',known.encode())
    for key in ['member_key','unknown_key']:pr('umask 077; cat > /root/faction-wg/'+key,getattr(a,key).read_bytes())
    sr('/etc/rc.d/faction_ssh_gate onestop');original=sr('cat /etc/ssh/faction/key_admission_sshd_config').stdout
    code="from pathlib import Path;p=Path('/etc/ssh/faction/key_admission_sshd_config');r=p.read_text().splitlines();r=[x for x in r if not (x.startswith('ListenAddress 172.') or x=='ListenAddress 127.0.0.1')];r=[('AuthorizedKeysCommand /usr/local/libexec/faction-key-admission --target-uid %U --refill-ms 250 --experimental-capacity' if x.startswith('AuthorizedKeysCommand ') else x) for x in r];r=[('LogLevel VERBOSE' if x.startswith('LogLevel ') else x) for x in r];p.write_text('ListenAddress 127.0.0.1\\nListenAddress 172.28.0.1\\nListenAddress 172.29.0.1\\n'+'\\n'.join(r)+'\\n');p.chmod(0o600)"
    sr('/usr/pkg/bin/python3.13 -c '+shlex.quote(code))
    sr('/usr/local/libexec/faction-key-admission --reset --refill-ms 250 --experimental-capacity; /usr/pkg/bin/python3.13 -I -B /usr/local/libexec/prepare_ssh_gate.py --configure --reset-budget --refill-ms 250 --experimental-capacity; /etc/rc.d/faction_ssh_gate onestart')
    base=['/usr/bin/ssh','-F','/dev/null','-o','BatchMode=yes','-o','IdentitiesOnly=yes','-o','ConnectTimeout=3','-o','StrictHostKeyChecking=yes','-o','UserKnownHostsFile=/root/faction-wg/known_hosts','-p','2222']
    def inner(network,key='member_key',extra=()):return shlex.join(base+['-i','/root/faction-wg/'+key]+list(extra)+['factionterm@172.'+str(network)+'.0.1'])
    # Verify the actual TCP terminal path; ICMP is a separate policy/protocol.
    for network in [28,29]:
        console=SSHConsole(peer[:-1]+['-tt',peer[-1],inner(network,extra=('-tt',))])
        try:
            console.expect('[L]ogin [R]egister [Q]uit: ');console.send('q');console.finish()
        finally:console.close()
    state=sr('wgconfig wg0 show all').stdout.decode();endpoints=[line.strip() for line in state.splitlines() if 'endpoint:' in line]
    assert len(endpoints)==2 and all('(none)' not in line for line in endpoints)
    outer=[line.split('endpoint:',1)[1].rsplit(':',1)[0] for line in endpoints];assert outer[0]==outer[1]
    configured=dict(two_authenticated_peers=True,same_outer_address=True,distinct_inner_addresses=True,allowed_ips_single_host=True,server_interfaces=1,peer_interfaces=2,native_experimental=True)
    logs_before=len(sr('cat /var/log/authlog').stdout)
    stop=threading.Event();attempts=[];errors=[]
    def attack():
        try:
            while not stop.is_set() and len(attempts)<80:
                x=pr(inner(28,'unknown_key',('-N',)),ok=False,timeout=10);attempts.append(x.returncode);time.sleep(.6)
        except Exception as exc:errors.append(type(exc).__name__)
    thread=threading.Thread(target=attack,daemon=True);thread.start();console=None
    try:
        # Only inspect the ephemeral private fixture log. Publish booleans,
        # never host addresses, key fingerprints, commands or log text.
        deadline=time.monotonic()+50;seen=False
        while time.monotonic()<deadline:
            log=sr('cat /var/log/authlog').stdout[logs_before:].decode(errors='replace')
            seen=any('penalt' in line.lower() and '172.28.0.2' in line for line in log.splitlines())
            if seen:break
            time.sleep(.6)
        assert seen and not errors,'native penalty did not activate in bounded fixture'
        # Wait past the source bucket spacing, then require a fresh BBS login
        # while the other authenticated private identity keeps attacking.
        time.sleep(.6);before=len(attempts)
        console=SSHConsole(peer[:-1]+['-tt',peer[-1],inner(29,extra=('-tt',))])
        console.expect('[L]ogin [R]egister [Q]uit: ');console.send('l');console.expect('Handle: ');console.send('profile0000');console.expect('Password (hidden): ')
        secret=a.secret_file.read_text().strip();console.send(secret);console.expect('Press Enter to continue: ');console.send('');console.expect('Ctrl-C exits; Esc: Enter to recover.');console.raw(b'c');console.expect('chat[#lobby]> ');console.send('/help');console.expect('/kick #channel handle')
        time.sleep(2);assert len(attempts)>before and not errors
        console.send('/quit');console.expect('Chat closed.');console.expect('Ctrl-C exits; Esc: Enter to recover.');console.raw(b'q');console.expect('Logged out.');console.finish();console.close();console=None
        # Datagram controls avoid mistaking an asymmetric SSH reply path for
        # proof of source anti-spoofing. Both real sources arrive; B's forged
        # source under A's key must not reach the receiver, followed by B again.
        receiver="""import socket,time,json
s=socket.socket(socket.AF_INET,socket.SOCK_DGRAM);s.bind(('0.0.0.0',23456));s.settimeout(.3)
from pathlib import Path
Path('/root/faction-wg/receiver-ready').touch()
seen={'a':0,'b':0,'forged':0};end=time.monotonic()+10
while time.monotonic()<end:
 try:payload,source=s.recvfrom(64)
 except socket.timeout:continue
 if payload==b'a' and source[0]=='172.28.0.2':seen['a']+=1
 elif payload==b'b' and source[0]=='172.29.0.2':seen['b']+=1
 elif payload==b'forged':seen['forged']+=1
 else:raise AssertionError('unexpected private datagram')
print(json.dumps(seen))
"""
        sr('rm -f /root/faction-wg/receiver-ready')
        rx=subprocess.Popen(server+['/usr/pkg/bin/python3.13 -I -c '+shlex.quote(receiver)],stdout=subprocess.PIPE,stderr=subprocess.PIPE)
        try:
            for _ in range(30):
                if sr('test -f /root/faction-wg/receiver-ready',ok=False).returncode==0:break
                time.sleep(.1)
            def datagram(tag,source,destination):
                pr('printf %s '+shlex.quote(tag)+' | nc -u -w 1 -s '+source+' '+destination+' 23456',timeout=5)
            datagram('a','172.28.0.2','172.28.0.1');datagram('b','172.29.0.2','172.29.0.1')
            outbound_before=pr('netstat -ibn -I wg0').stdout
            datagram('forged','172.29.0.2','172.28.0.1')
            outbound_after=pr('netstat -ibn -I wg0').stdout
            assert outbound_before!=outbound_after,'forged datagram was not offered to the tunnel'
            datagram('b','172.29.0.2','172.29.0.1')
            out,err=rx.communicate(timeout=12);assert rx.returncode==0
            datagrams=json.loads(out);assert datagrams=={'a':1,'b':2,'forged':0}
        finally:
            if rx.poll() is None:rx.kill();rx.wait()
        configured['datagram_source_controls']=datagrams
        configured['forged_datagram_offered_to_tunnel']=True
        # B's source deliberately sent under A's tunnel key must fail.
        spoof=pr(inner(28,extra=('-b','172.29.0.2','-N')),ok=False,timeout=8);assert spoof.returncode!=0
                # Preserve successful B flow after the negative anti-spoof control.
        console=SSHConsole(peer[:-1]+['-tt',peer[-1],inner(29,extra=('-tt',))]);console.expect('[L]ogin [R]egister [Q]uit: ');console.send('q');console.finish();console.close();console=None
        log=sr('cat /var/log/authlog').stdout[logs_before:].decode(errors='replace')
        assert any('penalt' in line.lower() and '172.28.0.2' in line for line in log.splitlines())
        configured.update(native_penalty_observed=True,fresh_approved_bbs_login_during_attack=True,useful_chat_during_attack=True,attack_continued_during_b_login=True,source_spoof_denied=True,other_peer_positive_after_spoof=True,attempts=len(attempts),authenticated_attackers=False,listener_reset_during_attack=False,registry_reset_during_attack=False,penalties_weakened=False,server_securelevel=int(sr('sysctl -n kern.securelevel').stdout))
        a.output.write_text(json.dumps(configured,indent=2)+'\n');print('Private identity separation, useful approved login and source anti-spoof controls passed')
    finally:
        stop.set();thread.join(12)
        if console:console.close()
        # Keep the two server peer entries for following memory accounting, but
        # restore quiet logging. Restart only after the measured interval ends.
        sr('/etc/rc.d/faction_ssh_gate onestop');sr('cat > /etc/ssh/faction/key_admission_sshd_config',original);sr('/etc/rc.d/faction_ssh_gate onestart')
if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--disposable-fixture',action='store_true');p.add_argument('--udp-port',type=int,required=True)
    for n in ['server-admin-argv','peer-admin-argv','member-key','unknown-key','secret-file','output']:p.add_argument('--'+n,type=Path,required=True)
    run(p.parse_args())
