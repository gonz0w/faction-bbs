#!/usr/bin/env python3
"""Real channel/command denial through the experimental listener; private fixture only."""
import argparse,json,os,secrets,select,subprocess,time
from pathlib import Path
from check_restricted_ssh import SSHConsole

def run(a):
    assert a.disposable_fixture
    common=['/usr/bin/ssh','-F','/dev/null','-o','BatchMode=yes','-o','IdentitiesOnly=yes','-o','LogLevel=ERROR',
        '-o','ConnectTimeout=5','-o','StrictHostKeyChecking=yes','-o','UserKnownHostsFile='+str(a.known_hosts)]
    admin=common+['-p',str(a.admin_port),'-i',str(a.admin_identity),'root@'+a.host]
    member=common+['-p',str(a.port),'-i',str(a.identity)]
    destination='factionterm@'+a.host;results=[];clients=[]
    def remote(command):
        p=subprocess.run(admin+[command],capture_output=True,timeout=20);assert p.returncode==0
        return p.stdout
    def passed(name):results.append({'oracle':name,'result':'pass'});print('Passed: '+name,flush=True)
    def call(flags,command=()):
        time.sleep(.55)
        return subprocess.run(member+list(flags)+[destination]+list(command),capture_output=True,timeout=8)
    remote('test -f /etc/faction-disposable-test; test -S /var/run/faction/terminal.sock')
    control=a.output.parent/('mux-'+secrets.token_hex(4))
    try:
        for command in [('id',),('/bin/sh',),('/usr/local/libexec/faction','export','/tmp/admission-leak')]:
            assert call(['-T'],command).returncode!=0
        remote('test ! -e /tmp/admission-leak');passed('no-PTY command shell and owner export requests fail')
        assert call(['-T','-s'],['sftp']).returncode!=0
        passed('SFTP subsystem cannot create a file-transfer or host-command path')
        assert call(['-N','-o','ExitOnForwardFailure=yes','-R','0:127.0.0.1:22']).returncode!=0
        assert call(['-W','127.0.0.1:22']).returncode!=0
        passed('remote forwarding and direct-tcpip channel requests are denied')
        time.sleep(.55);c=SSHConsole(member+['-tt',destination,'id']);clients.append(c)
        c.finish(1);assert b'uid=' not in c.transcript and b'[L]ogin' not in c.transcript
        passed('PTY exec request is rejected by the forced native frontend')
        assert not control.exists()
        time.sleep(.5);before=json.loads(remote('/usr/local/libexec/faction-key-admission --inspect'))
        time.sleep(.55);c=SSHConsole(member+['-M','-S',str(control),'-tt',destination]);clients.append(c)
        c.expect('[L]ogin [R]egister [Q]uit: ')
        assert control.exists() and call(['-S',str(control),'-O','check']).returncode==0
        second=call(['-S',str(control),'-o','ProxyCommand=false','-tt']);assert second.returncode!=0
        assert b'Session open refused by peer' in second.stderr
        assert b'[L]ogin' not in second.stdout
        # A failed multiplex request must not fall back to a second transport.
        time.sleep(1.1);after=json.loads(remote('/usr/local/libexec/faction-key-admission --inspect'))
        assert after['launches']==before['launches']+1
        c.send('q');c.finish();passed('MaxSessions one denies a second channel on an existing transport')
        assert not control.exists()
        # Base SSH keeps authenticated sessions when its listener restarts.
        # The persistent birth registry must keep their slots authoritative.
        time.sleep(2.1);active=[]
        for _ in range(8):
            time.sleep(.55);c=SSHConsole(member+['-tt',destination]);clients.append(c);active.append(c)
            c.expect('[L]ogin [R]egister [Q]uit: ');time.sleep(.55)
        remote('/etc/rc.d/faction_ssh_gate onestop; /etc/rc.d/faction_ssh_gate onestart')
        refused=call(['-N']);assert refused.returncode!=0
        state=json.loads(remote('/usr/local/libexec/faction-key-admission --inspect'));assert state['active']==8
        for c in active:c.send('q');c.finish()
        passed('base listener restart preserves old PTYs and cannot reset their authenticated cap')
        a.output.write_text(json.dumps({'schema':1,'results':results,'scope':'native restricted channels through listener-only gate and base sshd'},indent=2)+'\n')
    finally:
        for c in clients:c.close()
        control.unlink(missing_ok=True)

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--disposable-fixture',action='store_true');p.add_argument('--host',required=True)
    for n in ['identity','admin-identity','known-hosts','output']:p.add_argument('--'+n,type=Path,required=True)
    for n in ['port','admin-port']:p.add_argument('--'+n,type=int,required=True)
    run(p.parse_args())
