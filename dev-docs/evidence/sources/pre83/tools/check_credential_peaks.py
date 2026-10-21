#!/usr/bin/env python3
"""Persistent credential peak campaign ONLY in a marked disposable NetBSD VM."""
import argparse
import json
import os
import select
import shlex
import signal
import subprocess
import time
from pathlib import Path
from check_restricted_ssh import SSHConsole
from credential_peak import Stream,allocation_window
from memory_profile import headroom


def run(args):
    assert args.disposable_fixture and args.secret_file.stat().st_mode&0o077==0
    secret=args.secret_file.read_text().strip();assert len(secret)>=24
    common=['/usr/bin/ssh','-F','/dev/null','-o','BatchMode=yes','-o','IdentitiesOnly=yes','-o','LogLevel=ERROR',
            '-o','ConnectTimeout=5','-o','StrictHostKeyChecking=yes','-o','UserKnownHostsFile='+str(args.known_hosts)]
    admin=common+['-p',str(args.admin_port),'-i',str(args.admin_identity),'root@'+args.host]
    member=common+['-p',str(args.port),'-i',str(args.identity),'factionterm@'+args.host]
    def remote(command,data=None):
        p=subprocess.run(admin+['set -e; PATH=/usr/sbin:/sbin:/usr/bin:/bin:/usr/pkg/bin; export PATH; '+command],input=data,capture_output=True,timeout=30)
        assert p.returncode==0,'credential fixture operation failed';return p.stdout
    remote('test -f /etc/faction-disposable-test && test -f /var/db/faction-profile/.profile-fixture')
    uid=remote('id -u factiond').decode().strip();term=remote('id -u factionterm').decode().strip();listener=remote('cat /var/run/faction-sshd.pid').decode().strip()
    assert uid.isdecimal() and term.isdecimal() and listener.isdecimal()
    remote("su - factiond -c 'umask 077; touch /var/db/faction-profile/credential-0.lock /var/db/faction-profile/credential-1.lock'")
    metric='/usr/local/libexec/faction-netbsd-metrics '+uid+' '+term+' '+listener
    assert headroom(json.loads(remote(metric)))
    def stream():return Stream(admin+[metric+' --stream 10 300'])
    calibrations=[]
    for mib in [0,19,38]:
        s=stream()
        try:
            allocation=json.loads(remote('/usr/local/libexec/faction-memory-calibrate '+str(mib)+' 200'))
            data=s.finish();data['calibration']=allocation_window(data,allocation);calibrations.append(data)
        finally:s.abort()
        print('Native persistent sampler calibration recorded: '+str(mib)+' MiB',flush=True)
    clients=[];bursts=[]
    def drop(c):
        if not c.done:
            try:os.kill(c.pid,signal.SIGKILL)
            except ProcessLookupError:pass
            os.close(c.fd)
            try:os.waitpid(c.pid,0)
            except ChildProcessError:pass
            c.done=True
        else:os.close(c.fd)
    def prompt(n):
        c=SSHConsole(member[:-1]+['-tt',member[-1]]);clients.append(c)
        c.expect('[L]ogin [R]egister [Q]uit: ');c.send('l');c.expect('Handle: ');c.send(f'profile{n:04}')
        c.expect('Password (hidden): ');return c
    try:
        # External kernel-slot holders do not alter the KDF or collector. They
        # prove saturation and release while eight real prompts remain admitted.
        for case in ['both_slots_held','wrong_password','valid_password','valid_password_repeat']:
            holder=None
            if case=='both_slots_held':
                code="import fcntl,sys; from pathlib import Path; p=Path('/var/db/faction-profile'); a=open(p/'credential-0.lock','r+'); b=open(p/'credential-1.lock','r+'); fcntl.flock(a,fcntl.LOCK_EX); fcntl.flock(b,fcntl.LOCK_EX); print('ready',flush=True); sys.stdin.read(1)"
                holder=subprocess.Popen(admin+['/usr/pkg/bin/python3.13 -c '+shlex.quote(code)],stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=subprocess.PIPE)
                assert holder.stdout.readline()==b'ready\n'
            s=None
            try:
                for n in range(8):prompt(n);time.sleep(.11)
                s=stream();time.sleep(.25);began=time.monotonic()
                offered=secret if case!='wrong_password' else 'incorrect-credential-for-test'
                for c in clients:c.send(offered)
                counts={'successful':0,'budget_rejected':0,'credentials_rejected':0}
                for c in clients:
                    deadline=time.monotonic()+10
                    while not any(t in c.buffer for t in [b'Press Enter to continue: ',b'credential work busy; reconnect later',b'invalid credentials']):
                        if time.monotonic()>deadline:raise AssertionError('credential outcome timed out')
                        if select.select([c.fd],[],[],.1)[0]:
                            try:data=os.read(c.fd,8192)
                            except OSError:data=b''
                            if not data:break
                            c.buffer+=data;c.transcript+=data
                    if b'Press Enter to continue: ' in c.buffer:counts['successful']+=1
                    elif b'credential work busy; reconnect later' in c.buffer:counts['budget_rejected']+=1
                    elif b'invalid credentials' in c.buffer:counts['credentials_rejected']+=1
                    else:raise AssertionError('unclassified credential outcome')
                    assert offered.encode() not in c.transcript,'hidden input was echoed'
                assert sum(counts.values())==8
                if case=='both_slots_held':assert counts['budget_rejected']==8
                elif case=='wrong_password':assert counts['successful']==0 and counts['credentials_rejected']>=1
                else:assert counts['successful']>=1
                outcome_seconds=time.monotonic()-began
                submit_since_stream=began-s.started
                observed=s.finish();observed.update({'case':case,'offered':8,**counts,'host_submit_to_outcomes_seconds':outcome_seconds,
                    'host_submit_since_stream_start_seconds':submit_since_stream,
                    'observed_max_anon_bytes':max(row['vm_bytes']['anon'] for row in observed['samples']),
                    'first_sample_anon_bytes':observed['samples'][0]['vm_bytes']['anon'],
                    'observed_peak_delta_from_first_sample_bytes':max(row['vm_bytes']['anon'] for row in observed['samples'])-observed['samples'][0]['vm_bytes']['anon']})
                bursts.append(observed)
            finally:
                if s:s.abort()
                for c in clients:drop(c)
                clients.clear()
                if holder:
                    holder.stdin.write(b'x');holder.stdin.close();holder.wait(timeout=5);holder.stdout.close();holder.stderr.close()
            time.sleep(2)
            assert headroom(json.loads(remote(metric)))
            print('Persistent native credential burst recorded: '+case,flush=True)
        # Fresh approved identity after saturation, bad credentials and disconnects.
        c=prompt(0);c.send(secret);c.expect('Press Enter to continue: ');c.send('');c.expect('Ctrl-C exits; Esc: Enter to recover.')
        c.raw(b'q');c.expect('Logged out.');c.finish();drop(c);clients.clear()
        args.output.write_text(json.dumps({'schema':1,'credential_slots':2,'argon2':{'algorithm':'argon2id','version':19,'memory_kib':19456,'iterations':2,'lanes':1},
            'calibrations':calibrations,'bursts':bursts,'fresh_login_after_release':'pass','hidden_input':'pass',
            'scope':'sampled lower bounds; not instantaneous upper bounds or 250-user support'},indent=2)+'\n')
    finally:
        for c in clients:drop(c)


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--disposable-fixture',action='store_true');p.add_argument('--host',required=True)
    p.add_argument('--port',type=int,required=True);p.add_argument('--admin-port',type=int,required=True)
    p.add_argument('--identity',type=Path,required=True);p.add_argument('--admin-identity',type=Path,required=True)
    p.add_argument('--known-hosts',type=Path,required=True);p.add_argument('--secret-file',type=Path,required=True);p.add_argument('--output',type=Path,required=True)
    run(p.parse_args())
