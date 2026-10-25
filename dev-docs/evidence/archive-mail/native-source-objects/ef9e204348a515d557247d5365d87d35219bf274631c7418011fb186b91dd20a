#!/usr/bin/env python3
"""Matched worker/shared real SSH scaling ONLY in the marked disposable NetBSD profiling fixture.

No production setup, key deployment, profile secret or transcript is published.
Run following native application/lifecycle checks; uses a separately seeded board.
"""
import argparse
import concurrent.futures
import json
import os
import secrets
import shlex
import signal
import subprocess
import time
import threading
from pathlib import Path
from check_restricted_ssh import SSHConsole
from memory_profile import summarize,interval,headroom


def run(args):
    assert args.disposable_fixture
    assert args.secret_file.stat().st_mode & 0o077 == 0
    secret=args.secret_file.read_text().strip()
    assert len(secret)>=24
    common=['/usr/bin/ssh','-F','/dev/null','-o','BatchMode=yes','-o','IdentitiesOnly=yes','-o','LogLevel=ERROR',
            '-o','ConnectTimeout=5','-o','StrictHostKeyChecking=yes','-o','UserKnownHostsFile='+str(args.known_hosts)]
    admin=common+['-p',str(args.admin_port),'-i',str(args.admin_identity),'root@'+args.host]
    member=common+['-p',str(args.port),'-i',str(args.identity),'factionterm@'+args.host]
    clients=[];records=[]
    def remote(cmd,timeout=30,ok=True,data=None):
        p=subprocess.run(admin+['set -e; PATH=/usr/sbin:/sbin:/usr/bin:/bin:/usr/pkg/bin; export PATH; '+cmd],input=data,capture_output=True,timeout=timeout)
        assert not ok or p.returncode==0, 'profiling fixture operation failed'
        return p
    remote('test -f /etc/faction-disposable-test')
    remote('/etc/rc.d/faction_broker onestatus >/dev/null || /etc/rc.d/faction_broker onestart; /etc/rc.d/faction_sshd onestatus >/dev/null || /etc/rc.d/faction_sshd onestart')
    uid=remote('id -u factiond').stdout.decode().strip();term=remote('id -u factionterm').stdout.decode().strip()
    listener=remote('cat /var/run/faction-sshd.pid').stdout.decode().strip()
    assert uid.isdecimal() and term.isdecimal() and listener.isdecimal()
    def metrics():
        s=json.loads(remote('/usr/local/libexec/faction-netbsd-metrics '+uid+' '+term+' '+listener).stdout)
        summarize(s);return s
    def drop(c):
        if not c.done:
            try:os.kill(c.pid,signal.SIGKILL)
            except ProcessLookupError:pass
            os.close(c.fd)
            try:os.waitpid(c.pid,0)
            except ChildProcessError:pass
            c.done=True
        else:os.close(c.fd)
    def alive(c):
        if c.done:return False
        pid,_=os.waitpid(c.pid,os.WNOHANG)
        if pid:c.done=True;return False
        return True
    def expect(c,label,text):
        nonlocal phase
        phase=label;c.expect(text)
    def open_prompt(n):
        c=SSHConsole(member[:-1]+['-tt',member[-1]]);clients.append(c)
        expect(c,'ssh_access_prompt','[L]ogin [R]egister [Q]uit: ');c.send('l');expect(c,'handle_prompt','Handle: ');c.send(f'profile{n:04}')
        expect(c,'password_prompt','Password (hidden): ');return c
    def login(n):
        c=open_prompt(n);c.send(secret);expect(c,'authenticated_identity','Press Enter to continue: ');c.send('')
        expect(c,'approved_menu','Ctrl-C exits; Esc: Enter to recover.');return c
    def measured(stage,phase,seconds=10):
        before=metrics();start=time.monotonic();samples=[summarize(before)]
        end=start+seconds
        while time.monotonic()<end:
            time.sleep(min(1,max(0,end-time.monotonic())));after=metrics();samples.append(summarize(after))
        elapsed=time.monotonic()-start
        row={'sessions':stage,'distinct_accounts':stage,'phase':phase,'start':samples[0],
             'end':samples[-1],'samples':samples,'interval':interval(before,after,elapsed),
             'all_alive':all(alive(c) for c in clients)}
        assert row['all_alive']
        if stage:
            for sample in samples:
                assert sample['roles'].get('member',{}).get('count',0)==(stage if args.model=='workers' else 0), 'unexpected trusted workers'
                assert sample['roles'].get('broker',{}).get('count')==1, 'missing shared owner/broker'
                assert sample['roles'].get('ssh',{}).get('count',0)>=2*stage+1, 'missing full SSH transports'
                assert sample['roles'].get('frontend',{}).get('count')==stage, 'missing frontends'
        args.output.with_suffix('.partial.json').write_text(json.dumps({'stages':records+[row]},indent=2)+'\n')
        records.append(row);print(f'Profiled {stage} distinct SSH sessions: {phase}',flush=True)
        return after
    # No sysctl change to the developer host; adjustments are explicit fixture
    # inputs to accommodate 250 SSH sessions' process/descriptor overhead.
    original=remote('/sbin/sysctl -n kern.maxproc; /sbin/sysctl -n kern.maxfiles').stdout.decode().splitlines()
    remote('/etc/rc.d/faction_broker onestop; /etc/rc.d/faction_sshd onestatus >/dev/null || /etc/rc.d/faction_sshd onestart')
    remote('install -d -o factiond -g factionipc -m 700 /var/db/faction-profile; touch /var/db/faction-profile/.profile-fixture; chown factiond:factionipc /var/db/faction-profile/.profile-fixture; chmod 600 /var/db/faction-profile/.profile-fixture')
    if args.seed:
        remote('test ! -e /var/db/faction-profile/faction.sqlite3')
        remote("su - factiond -c '/home/factiond/target/release/examples/profile_seed --disposable-state /var/db/faction-profile'",timeout=300,data=secret.encode())
    else:
        remote('test -f /var/db/faction-profile/.profile-fixture && test -f /var/db/faction-profile/faction.sqlite3')
    remote('/sbin/sysctl -w kern.maxproc=2048 kern.maxfiles=16384 >/dev/null')
    # Same protected endpoint/config/member executable; only fixed owner state and
    # explicit experimental cap/cadence differ from the normal eight-worker service.
    command=f'nohup /usr/local/libexec/faction-broker --socket /var/run/faction/terminal.sock --terminal-uid {term} --worker /usr/local/libexec/faction-member --state-dir /var/db/faction-profile --experimental-workers 250 --experimental-launch-ms 100 >/home/factiond/profile-broker.log 2>&1 </dev/null &'
    if args.model=='shared':
        command=f'nohup /usr/local/libexec/shared-bbs --socket /var/run/faction/terminal.sock --terminal-uid {term} --state-dir /var/db/faction-profile >/home/factiond/profile-broker.log 2>&1 </dev/null &'
    def start_server():
        remote('su - factiond -c '+shlex.quote(command));time.sleep(.5)
        pid=remote('cat /var/run/faction/terminal.pid').stdout.decode().strip();assert pid.isdecimal()
        return pid
    server_pid=start_server()
    def stop():
        remote('kill -TERM '+server_pid)
        for _ in range(120):
            if remote('kill -0 '+server_pid,ok=False).returncode!=0:break
            time.sleep(.1)
        else:raise AssertionError('server did not stop within 12 seconds')

    outcome='completed';reached=0;highest=0;failure=None;delivered=[];phase='initial'
    try:
        baseline=measured(0,'seeded_idle_baseline',5)
        for stage in [100,250]:
            if stage>args.max_sessions:break
            try:
                while len(clients)<stage:
                    if len(clients)%8==0 and not headroom(metrics()):
                        outcome='stopped_at_headroom_guard';break
                    phase='approved_login';login(len(clients));highest=len(clients);time.sleep(.11)
                if outcome!='completed':break
                reached=stage;time.sleep(1)
                phase='approved_menu_idle';sample=measured(stage,'approved_menu_idle')
                phase='chat_enter'
                for c in clients:
                    c.raw(b'c');c.expect('chat[#lobby]> ')
                time.sleep(1);measured(stage,'local_chat_idle')
                phase='chat_delivery';start=time.monotonic();clients[0].send('profile bounded delivery')
                for c in clients:c.expect('profile bounded delivery')
                delivered.append({'sessions':stage,'distinct_accounts':stage,'delivery_seconds':round(time.monotonic()-start,3)})
                phase='chat_return_to_menu'
                for c in clients:
                    c.send('/quit');c.expect('Chat closed.')
                    if args.model=='workers':c.expect('Press Enter to return to menu: ');c.send('')
                    c.expect('Ctrl-C exits; Esc: Enter to recover.')
                if not headroom(sample):outcome='stopped_at_headroom_guard';break
            except (AssertionError,TimeoutError,EOFError,OSError) as exc:
                failure={'exception':type(exc).__name__,'phase':phase,'offered_stage':stage,'approved_sessions':highest,'detail':str(exc),'snapshot':summarize(metrics())}
                # Private diagnostic only: generated fixture data, never published.
                args.output.with_suffix('.failure.log').write_bytes(clients[-1].transcript.replace(secret.encode(),b'[REDACTED]'))
                outcome='stopped_at_functional_failure';break
        load_snapshot=summarize(metrics());stop_started=time.monotonic();stop()
        time.sleep(.5);stopped=summarize(metrics())
        assert stopped['roles'].get('member',{}).get('count',0)==0
        assert stopped['roles'].get('frontend',{}).get('count',0)==0
        assert stopped['roles'].get('broker',{}).get('count',0)==0
        load_stop={'admitted_sessions':highest,'snapshot_before_stop':load_snapshot,'seconds':round(time.monotonic()-stop_started,3),'workers_and_frontends_reaped':True}
        for c in clients:drop(c)
        clients.clear();time.sleep(1)
        server_pid=start_server()
        c=login(0);c.raw(b'q');c.expect('Logged out.');c.finish();drop(c);clients.clear()
        recovered=metrics()
        data={'schema':1,'scope':'matched worker/shared native distinct-account SSH scaling; not production capacity',
              'seed':{'approved_accounts':1000,'rooms':50,'posts':100000,'post_body_bytes':1024,'independent_password_salts':True},
              'fixture_limits':{'original_maxproc':int(original[0]),'original_maxfiles':int(original[1]),'experimental_maxproc':2048,'experimental_maxfiles':16384,'worker_limit':250,'launch_refill_ms':100,'launch_burst':8,'credential_slots':2},
              'outcome':outcome,'headroom_guard':'free + max(0, clean_file - wired) >= 128 MiB; zero swap; process/file reserves', 'completed_stage':reached,'highest_observed_approved_sessions':highest,'failure':failure,
              'stages':records,'deliveries':delivered,'model':args.model,'load_stop':load_stop,
              'after_clients_disconnected':summarize(recovered),'fresh_login_after_load':'pass'}
        args.output.write_text(json.dumps(data,indent=2)+'\n')
        print(f'Native scaling recorded: completed stage {reached}; {outcome}. No secrets/transcripts emitted.')
    finally:
        for c in clients:drop(c)
        if remote('kill -0 '+server_pid,ok=False).returncode==0:stop()
        remote(f'/sbin/sysctl -w kern.maxproc={original[0]} kern.maxfiles={original[1]} >/dev/null',ok=False)


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--disposable-fixture',action='store_true');p.add_argument('--host',required=True)
    p.add_argument('--port',type=int,required=True);p.add_argument('--admin-port',type=int,required=True)
    p.add_argument('--identity',type=Path,required=True);p.add_argument('--admin-identity',type=Path,required=True)
    p.add_argument('--known-hosts',type=Path,required=True);p.add_argument('--output',type=Path,required=True)
    p.add_argument('--model',choices=['workers','shared'],required=True)
    p.add_argument('--seed',action='store_true')
    p.add_argument('--secret-file',type=Path,required=True)
    p.add_argument('--max-sessions',type=int,choices=[8,32,64,100,150,200,250],default=250)
    run(p.parse_args())
