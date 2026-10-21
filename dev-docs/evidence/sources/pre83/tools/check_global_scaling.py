#!/usr/bin/env python3
"""Maintained rc.d shared service headroom scaling ONLY in the marked disposable NetBSD profiling fixture.

No production setup, key deployment, profile secret or transcript is published.
Run following native application/lifecycle checks; uses a separately seeded board.
"""
import argparse
import concurrent.futures
import json
import os
import secrets
import socket
import shlex
import signal
import subprocess
import time
import threading
from pathlib import Path
from check_restricted_ssh import SSHConsole
from global_workload import active
from memory_profile import summarize,interval,headroom
from credential_peak import Stream,allocation_window
from transport_memory import guard_decision,admission_batches
from headroom_memory import live_limits as original_live_limits,sample_guard

def pack_live_limits(raw):
    zombies=raw.pop("ssh_zombie_processes")
    assert type(zombies) is int and 0<=zombies<=250
    ledger=original_live_limits(raw)
    ledger["ssh_zombie_processes"]=zombies
    return ledger


def run(args):
    assert args.disposable_fixture
    assert args.secret_file.stat().st_mode & 0o077 == 0
    secret=args.secret_file.read_text().strip()
    assert len(secret)>=24
    common=['/usr/bin/ssh','-F','/dev/null','-o','BatchMode=yes','-o','IdentitiesOnly=yes','-o','LogLevel=ERROR',
            '-o','ConnectTimeout=5','-o','StrictHostKeyChecking=yes','-o','UserKnownHostsFile='+str(args.known_hosts)]
    admin=common+['-p',str(args.admin_port),'-i',str(args.admin_identity),'root@'+args.host]
    member=common+['-p',str(args.port),'-i',str(args.identity),'factionterm@'+args.host]
    clients=[];records=[];decisions=[];credentials=[];calibrations=[];transport_states=[];live_limits=[];owner_credentials=[];loaded_controls=[];ui_work=[];at_capacity=[];global_activity=None
    def remote(cmd,timeout=30,ok=True,data=None):
        p=subprocess.run(admin+['set -e; PATH=/usr/sbin:/sbin:/usr/bin:/bin:/usr/pkg/bin; export PATH; '+cmd],input=data,capture_output=True,timeout=timeout)
        assert not ok or p.returncode==0, 'profiling fixture operation failed'
        return p
    remote('test -f /etc/faction-disposable-test')
    remote('if /etc/rc.d/faction_ssh_gate onestatus >/dev/null 2>&1; then /etc/rc.d/faction_ssh_gate onestop; fi')
    change="from pathlib import Path; p=Path('/etc/ssh/faction/key_admission_sshd_config'); rows=p.read_text().splitlines(); rows=[('AuthorizedKeysCommand /usr/local/libexec/faction-key-admission --target-uid %U --capacity 250 --refill-ms 250 --experimental-capacity' if line.startswith('AuthorizedKeysCommand ') else line) for line in rows]; p.write_text(chr(10).join(rows)+chr(10)); p.chmod(0o600)"
    remote('/usr/pkg/bin/python3.13 -c '+shlex.quote(change))
    remote('/usr/local/libexec/faction-key-admission --reset --capacity 250 --refill-ms 250 --experimental-capacity')
    remote('/etc/rc.d/faction_isolated onestatus >/dev/null || /etc/rc.d/faction_isolated onestart; /etc/rc.d/faction_ssh_gate onestatus >/dev/null || /etc/rc.d/faction_ssh_gate onestart')
    uid=remote('id -u factiond').stdout.decode().strip();term=remote('id -u factionterm').stdout.decode().strip()
    listener=remote('cat /var/run/faction-sshd.pid').stdout.decode().strip()
    admission_before=json.loads(remote('/usr/local/libexec/faction-key-admission --inspect').stdout)
    global_before=json.loads(remote('/usr/local/libexec/faction-ssh-gate --inspect').stdout)
    assert global_before['refill_ms']==250
    assert admission_before['capacity']==250 and admission_before['refill_ms']==250 and admission_before['burst']==8
    allocator=json.loads(remote('/usr/local/libexec/faction-headroom-allocator').stdout)
    expected=(1,False,0)
    assert allocator['retain']==(args.allocator=='retained'), 'retain profile did not take effect'
    assert (allocator['narenas'],allocator['tcache'],allocator['dirty_decay_ms'])==expected, 'allocator profile did not take effect'
    assert uid.isdecimal() and term.isdecimal() and listener.isdecimal()
    metric='/usr/local/libexec/faction-netbsd-metrics '+uid+' '+term+' '+listener
    def metrics():
        s=json.loads(remote(metric).stdout)
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
        time.sleep(.55)
        c=SSHConsole(member[:-1]+['-tt',member[-1]]);c.profile=n;clients.append(c)
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
        if stage:
            live_limits.append({'sessions':stage,'phase':phase,'readback':pack_live_limits(json.loads(remote('/usr/local/libexec/faction-live-limits '+uid+' '+term+' '+listener).stdout))})
        records.append(row);print(f'Profiled {stage} distinct SSH sessions: {phase}',flush=True)
        return after
    # No sysctl change to the developer host; adjustments are explicit fixture
    # inputs to accommodate 250 SSH sessions' process/descriptor overhead.
    original=remote('/sbin/sysctl -n kern.maxproc; /sbin/sysctl -n kern.maxfiles').stdout.decode().splitlines()
    remote('/etc/rc.d/faction_isolated onestop; /etc/rc.d/faction_ssh_gate onestatus >/dev/null || /etc/rc.d/faction_ssh_gate onestart')
    remote('install -d -o factiond -g factionipc -m 700 /var/db/faction-profile; touch /var/db/faction-profile/.profile-fixture; chown factiond:factionipc /var/db/faction-profile/.profile-fixture; chmod 600 /var/db/faction-profile/.profile-fixture')
    if args.seed:
        remote('test ! -e /var/db/faction-profile/faction.sqlite3')
        remote("su - factiond -c '/home/factiond/target/release/examples/profile_seed --disposable-state /var/db/faction-profile'",timeout=300,data=secret.encode())
    else:
        remote('test -f /var/db/faction-profile/.profile-fixture && test -f /var/db/faction-profile/faction.sqlite3')
    remote('/sbin/sysctl -w kern.maxproc=2048 kern.maxfiles=16384 >/dev/null')
    # Explicit root-owned experimental configuration, started by the installed rc.d service.
    assert args.model=='shared'
    root='/etc/faction/packs'
    config=f'version=1\nterminal_uid={term}\nstate_dir="/var/db/faction-profile"\nmax_sessions=250\nlaunch_ms=100\nexperimental_capacity=true\ntheme="{root}/base"\nseason="{root}/winter"\n'
    remote('cat > /etc/faction/service.toml; chmod 644 /etc/faction/service.toml; /usr/pkg/bin/python3.13 /usr/local/libexec/faction-isolated/prepare_isolation.py --configure',data=config.encode())
    remote('printf "faction_isolated=YES\\n" > /etc/rc.conf.d/faction_isolated')
    def start_server():
        remote('/etc/rc.d/faction_isolated onestart; /etc/rc.d/faction_isolated onestatus');time.sleep(.2)
        pid=remote('cat /var/run/faction/terminal.pid').stdout.decode().strip();assert pid.isdecimal()
        return pid
    server_pid=start_server()
    def stop():
        remote('/etc/rc.d/faction_isolated onestop')
        assert remote('/etc/rc.d/faction_isolated onestatus',ok=False).returncode!=0

    outcome='completed';reached=0;highest=0;failure=None;delivered=[];phase='initial'
    try:
        baseline=measured(0,'seeded_idle_baseline',5)
        if args.attribution:
            # No channel/PTY means authenticated SSH only. -N cannot run a host
            # command; it adds no alternate shell/command permission.
            transports=[]
            try:
                before=metrics();transport_start=time.monotonic()
                for _ in range(32):
                    transports.append(subprocess.Popen(member[:-1]+['-N',member[-1]],stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL))
                    time.sleep(.55)
                time.sleep(2);after=metrics()
                assert all(p.poll() is None for p in transports)
                assert summarize(after)['roles']['ssh']['count']>=65
                transport_states.append({'state':'authenticated_transport_no_channel','transports':32,'snapshot':summarize(after),'delta':interval(before,after,time.monotonic()-transport_start)})
            finally:
                for p in transports:p.kill();p.wait()
            time.sleep(2)
            for n in range(32):open_prompt(n);time.sleep(.55)
            transport_states.append({'state':'handed_off_password_prompt','transports':32,'snapshot':summarize(metrics())})
            for c in clients:drop(c)
            clients.clear();time.sleep(2)
        # Same native sampler and touched-page controls as research 0029.
        for mib in [0,19,38]:
            stream=Stream(admin+[metric+' --stream 10 150'])
            try:
                allocation=json.loads(remote('/usr/local/libexec/faction-memory-calibrate '+str(mib)+' 200').stdout)
                observed=stream.finish();observed['calibration']=allocation_window(observed,allocation);calibrations.append(observed)
            finally:stream.abort()
        for stage in [32,100,200,250]:
            if stage>args.max_sessions:break
            try:
                while len(clients)<stage:
                    if len(clients)%8==0:
                        decision=guard_decision(metrics(),len(clients),'before_admission_batch')
                        decisions.append(decision)
                        if not decision['passed']:
                            outcome='stopped_at_headroom_guard';break
                    phase='approved_login';login(len(clients));highest=len(clients);time.sleep(.55)
                if outcome!='completed':break
                time.sleep(1)
                phase='approved_menu_idle';sample=measured(stage,'approved_menu_idle')
                phase='chat_enter'
                for c in clients:
                    c.raw(b'c');c.expect('chat[#lobby]> ')
                time.sleep(1);measured(stage,'local_chat_idle')
                phase='chat_delivery';marker='profile delivery '+str(stage)+' '+secrets.token_hex(8)
                start=time.monotonic();clients[0].send(marker)
                for c in clients:c.expect(marker)
                delivered.append({'sessions':stage,'distinct_accounts':stage,'delivery_seconds':round(time.monotonic()-start,3)})
                if stage>=200:
                    phase='high_load_credentials'
                    # Replace two transports, rather than exceed the 250 cap.
                    # All other distinct approved members stay in useful chat.
                    replace_ids=[c.profile for c in clients[:2]]
                    for c in clients[:2]:
                        c.send('/quit');c.expect('Chat closed.');c.expect('Ctrl-C exits; Esc: Enter to recover.')
                        c.raw(b'q');c.expect('Logged out.');c.finish();drop(c)
                    clients[:2]=[];time.sleep(.5)
                    decision=guard_decision(metrics(),len(clients),'before_replacement_credentials');decisions.append(decision)
                    if not decision['passed']:
                        outcome='stopped_at_headroom_guard';highest=stage;break
                    wrong=[open_prompt(n) for n in replace_ids]
                    for c in wrong:c.send('incorrect-credential-for-test')
                    for c in wrong:
                        c.expect('invalid credentials');c.finish(1);drop(c);clients.remove(c)
                    time.sleep(.5)
                    failed_snapshot=summarize(metrics())
                    assert failed_snapshot['roles']['frontend']['count']==stage-2
                    assert all(alive(c) for c in clients)
                    pair=[open_prompt(n) for n in replace_ids]
                    stream=Stream(admin+[metric+' --stream 10 150'])
                    try:
                        time.sleep(.2);began=time.monotonic()
                        for c in pair:c.send(secret)
                        for c in pair:
                            c.expect('Press Enter to continue: ')
                            assert secret.encode() not in c.transcript,'hidden password echoed'
                            c.send('');c.expect('Ctrl-C exits; Esc: Enter to recover.');c.raw(b'c');c.expect('chat[#lobby]> ')
                        outcome_seconds=time.monotonic()-began
                        observed=stream.finish();observed.update({'cohort_in_chat':stage-2,'successful_credentials':2,'failed_credentials':2,'failed_transports_reaped':True,'after_failure_snapshot':failed_snapshot,'hidden_input':'pass','host_submit_to_menu_seconds':outcome_seconds})
                        credentials.append(observed)
                    finally:stream.abort()
                    marker='profile after credentials '+str(stage)+' '+secrets.token_hex(8)
                    start=time.monotonic();clients[0].send(marker)
                    for c in clients:c.expect(marker)
                    assert all(alive(c) for c in clients)
                    delivered.append({'sessions':stage,'distinct_accounts':stage,'phase':'after_two_credentials','delivery_seconds':round(time.monotonic()-start,3)})
                if stage==250:
                    before_admission=json.loads(remote('/usr/local/libexec/faction-key-admission --inspect').stdout)
                    for _ in range(2):
                        time.sleep(.55)
                        extra=subprocess.run(member[:-1]+['-N',member[-1]],capture_output=True,timeout=8)
                        assert extra.returncode!=0
                    time.sleep(1.1)
                    after_admission=json.loads(remote('/usr/local/libexec/faction-key-admission --inspect').stdout)
                    assert after_admission['active']==after_admission['peak']==250
                    assert after_admission['launches']==before_admission['launches']
                    assert after_admission['full']==before_admission['full']+2
                    at_capacity.append({'extra_transports_denied':2,'before':before_admission,'after':after_admission})
                    # Complete UI actions while 249 distinct peers remain in chat.
                    ui_stream=Stream(admin+[metric+' --stream 10 150'])
                    try:
                        phase='full_ui_at_250';c=clients[-1];began=time.monotonic()
                        c.send('/quit');c.expect('Chat closed.');c.expect('Ctrl-C exits; Esc: Enter to recover.')
                        c.raw(b'r');c.expect('More rooms? [y/N]: ');c.send('n');c.expect('Press Enter to return to menu: ');c.send('');c.expect('Ctrl-C exits; Esc: Enter to recover.')
                        c.raw(b'p');c.expect('Room ID: ');c.send('1');c.expect('Reply to post ID');c.send('');c.expect('Message (one line');c.send('UI load '+secrets.token_hex(8));c.expect('Submission ID: ');c.expect('Posted #');c.expect('Press Enter to return to menu: ');c.send('');c.expect('Ctrl-C exits; Esc: Enter to recover.')
                        c.raw(b'n');c.expect('Room ID: ');c.send('50');c.expect('More? [y/N]: ');c.send('n');c.expect('Press Enter to return to menu: ');c.send('');c.expect('Ctrl-C exits; Esc: Enter to recover.');c.raw(b'c');c.expect('chat[#lobby]> ')
                        observed=ui_stream.finish();observed.update({'connected_members':250,'peers_in_chat':249,'room_list':'pass','selected_room_page':'pass','post_submission':'pass','return_to_live_chat':'pass','elapsed_seconds':time.monotonic()-began});ui_work.append(observed)
                    finally:ui_stream.abort()
                    # Independent fresh owner processes use the unchanged kernel
                    # credential slots while all 250 member transports stay in chat.
                    # These are private fixture administration, never member RPC.
                    owners=[]
                    try:
                        for n in [998,999]:
                            c=SSHConsole(admin[:-1]+['-tt',admin[-1],"su - factiond -c '/usr/local/libexec/faction --state-dir /var/db/faction-profile login --plain'"])
                            owners.append(c);c.expect('Handle: ');c.send(f'profile{n:04}');c.expect('Password (hidden): ')
                        live_limits.append({'sessions':stage,'phase':'two_owner_password_prompts','readback':pack_live_limits(json.loads(remote('/usr/local/libexec/faction-live-limits '+uid+' '+term+' '+listener).stdout))})
                        stream=Stream(admin+[metric+' --stream 10 150'])
                        try:
                            time.sleep(.2);began=time.monotonic()
                            for c in owners:c.send(secret)
                            for c in owners:
                                c.expect('Command (then Enter): ');assert secret.encode() not in c.transcript
                            observed=stream.finish();observed.update({'cohort_in_chat':stage,'successful_owner_credentials':2,'hidden_input':'pass','host_submit_to_menu_seconds':time.monotonic()-began})
                            owner_credentials.append(observed)
                        finally:stream.abort()
                    finally:
                        for c in owners:
                            try:c.send('q');c.expect('Logged out.');c.finish()
                            finally:drop(c)
                    assert all(alive(c) for c in clients)
                    marker='profile after owner credentials '+secrets.token_hex(8);clients[0].send(marker)
                    for c in clients:c.expect(marker)
                    delivered.append({'sessions':stage,'distinct_accounts':stage,'phase':'after_two_owner_credentials'})
                    # Additional known touched pages under all 250 members.
                    # A synthetic control, not a cold Argon2 upper-bound claim.
                    for mib in [0,19,38]:
                        stream=Stream(admin+[metric+' --stream 10 150'])
                        try:
                            allocation=json.loads(remote('/usr/local/libexec/faction-memory-calibrate '+str(mib)+' 200').stdout)
                            observed=stream.finish();observed['calibration']=allocation_window(observed,allocation)
                            observed['cohort_in_chat']=stage;loaded_controls.append(observed)
                        finally:stream.abort()
                if stage==250:
                    phase='global_tcp_pressure'
                    global_activity=active(clients,remote,metric,admin)
                phase='chat_return_to_menu'
                for c in clients:
                    c.send('/quit');c.expect('Chat closed.')
                    if args.model=='workers':c.expect('Press Enter to return to menu: ');c.send('')
                    c.expect('Ctrl-C exits; Esc: Enter to recover.')
                reached=stage
                decision=guard_decision(metrics(),len(clients),'after_completed_stage')
                decisions.append(decision)
                if not decision['passed']:outcome='stopped_at_headroom_guard';break
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
        admission_after=json.loads(remote('/usr/local/libexec/faction-key-admission --inspect').stdout)
        if outcome=='completed':
            assert admission_after['peak']==250 and admission_after['full']==2 and admission_after['rate']==0
        data={'transport_accounting':'ssh role includes base listener, every SSH descendant and any transient key helper; no resident admission process',
              'global_before':global_before,'global_after':json.loads(remote('/usr/local/libexec/faction-ssh-gate --inspect').stdout),'at_capacity_probe':at_capacity,'admission_before':admission_before,'admission_after':admission_after,'schema':1,'scope':'experimental listener gate + key admission + isolated rc.d service; short workloads, not production capacity',
              'seed':{'approved_accounts':1000,'rooms':50,'posts':100000,'post_body_bytes':1024,'independent_password_salts':True},
              'fixture_limits':{'original_maxproc':int(original[0]),'original_maxfiles':int(original[1]),'experimental_maxproc':2048,'experimental_maxfiles':16384,'worker_limit':250,'launch_refill_ms':100,'launch_burst':8,'credential_slots':2},
              'outcome':outcome,'headroom_guard':'free + max(0, clean_file - wired) >= 128 MiB; zero swap; process/file reserves', 'completed_stage':reached,'highest_observed_approved_sessions':highest,'failure':failure,
              'stages':records,'deliveries':delivered,'model':args.model,'load_stop':load_stop,
              'admission_batches':admission_batches(decisions),'allocator_profile':args.allocator,'allocator_readback':allocator,'guard_decisions':decisions,
              'global_activity':global_activity,'transport_states':transport_states,'calibrations':calibrations,'high_load_credentials':credentials,
              'full_ui_at_250':ui_work,'live_process_limits':live_limits,'owner_credentials':owner_credentials,'loaded_controls':loaded_controls,
              'stress_guard_results':[{'kind':kind,'run':i,'minimum':min(sample_guard(s)['reclaimable_headroom_bytes'] for s in observed['samples']), 'all_sampled_guards_passed':all(sample_guard(s)['passed'] for s in observed['samples'])} for kind,runs in [('member_credentials',credentials),('owner_credentials',owner_credentials),('loaded_controls',loaded_controls),('full_ui',ui_work)] for i,observed in enumerate(runs)],
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
    p.add_argument('--allocator',choices=['compact','retained'],required=True)
    p.add_argument('--attribution',action='store_true')
    p.add_argument('--seed',action='store_true')
    p.add_argument('--secret-file',type=Path,required=True)
    p.add_argument('--max-sessions',type=int,choices=[8,32,64,100,150,200,250],default=250)
    run(p.parse_args())
