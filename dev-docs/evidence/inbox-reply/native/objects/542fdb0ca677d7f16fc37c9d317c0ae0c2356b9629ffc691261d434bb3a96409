#!/usr/bin/env python3
"""Native NetBSD lifecycle/load extension for check_restricted_ssh's marked guest.

Invoked only after its real 20 application SSH oracles, reusing private in-memory
credentials. No public endpoint or existing installation is a valid fixture.
"""
import json
import os
import shlex
import signal
import time


def exercise(remote, approved, enter, children):
    results, observations = [], {}

    def passed(name): results.append({'oracle': name, 'result': 'pass'})
    def workers():
        return [int(x) for x in remote("ps -ax -o pid= -o comm= | awk '$2==\"/usr/local/libexec/faction-member\" {print $1}'").stdout.split()]
    def wait_workers(count, timeout=5):
        end = time.monotonic()+timeout
        while time.monotonic() < end:
            if len(workers()) == count: return
            time.sleep(.1)
        raise AssertionError('worker count did not reach expected value')
    def broker_pid():
        pid = int(remote('cat /var/run/faction/terminal.pid').stdout)
        assert remote(f'ps -p {pid} -o comm=').stdout.strip() == b'/usr/local/libexec/faction-broker'
        return pid
    def service(action): return remote('/etc/rc.d/faction_broker '+action)
    def detach(c):
        if not c.done:
            os.kill(c.pid, signal.SIGKILL); os.close(c.fd); os.waitpid(c.pid, 0); c.done=True
        children.remove(c)
    def sample():
        rows=[]
        text=remote('ps -ax -o uid= -o pid= -o rss= -o vsz= -o comm=').stdout.decode()
        for line in text.splitlines():
            f=line.split(None,4)
            if len(f)==5 and ('faction-' in f[4] or 'sshd' in f[4]):
                # No PID, endpoint, username or arguments enter saved evidence.
                rows.append({'role': f[4].split()[0].rsplit('/',1)[-1].rstrip(':'), 'rss_kib':int(f[2]), 'vsz_kib':int(f[3])})
        return rows

    remote('test -f /etc/faction-disposable-test')
    service('status')
    assert remote('/etc/rc.d/faction_broker start',ok=False).returncode != 0
    passed('rc_status_and_duplicate_start')
    service('restart'); broker_pid(); wait_workers(0)
    passed('rc_restart_idle')
    sessions=[]
    for _ in range(8):
        c=approved('bobby'); c.raw(b'c'); c.expect('chat[#lobby]> '); sessions.append(c)
    wait_workers(8)
    pid=broker_pid()
    baseline=len(remote(f'/usr/bin/fstat -p {pid}').stdout.splitlines())
    observations['eight_chat_processes']=sample()
    observations['vmstat_summary']=remote("vmstat -s | awk '/pages free$|pages active$|pages inactive$|page size/ {print}'").stdout.decode().splitlines()
    # Observe CPU at a real eight-chat workload, then measure a delivered arrival.
    observations['eight_chat_cpu_sample']=remote('ps -ax -o pcpu= -o comm= | awk \'/faction-/ {sum+=$1} END {print sum+0}\'').stdout.decode().strip()
    started=time.monotonic(); sessions[0].send('bounded eight-member native arrival')
    for c in sessions: c.expect('bounded eight-member native arrival')
    observations['eight_chat_delivery_seconds']=round(time.monotonic()-started,3)
    passed('eight_approved_ssh_members_chat')
    extra=enter(None); extra.expect('Faction SSH: terminal admission denied or session ended.'); extra.finish(1)
    passed('ninth_ssh_member_rejected_at_cap')
    # 100/250 offered local connections while eight real SSH members remain live.
    # These are rejected admission attempts, NOT 100/250 connected members.
    probe="""import socket,time,json
out=[]
for count in [100,250]:
    start=time.monotonic()
    for _ in range(count):
        s=socket.socket(socket.AF_UNIX);s.settimeout(2)
        s.connect('/var/run/faction/terminal.sock');assert s.recv(1)==b'';s.close()
    out.append({'offered':count,'rejected':count,'seconds':round(time.monotonic()-start,3)})
print(json.dumps(out))
"""
    p=remote('su - factionterm -c '+shlex.quote('/usr/pkg/bin/python3.13 -c '+shlex.quote(probe)))
    observations['over_cap_local_offers']=json.loads(p.stdout)
    wait_workers(8)
    after=len(remote(f'/usr/bin/fstat -p {pid}').stdout.splitlines())
    assert baseline==after
    observations['overload_broker_descriptor_rows']={'before':baseline,'after':after}
    sessions[1].send('healthy after rejected offers'); sessions[2].expect('healthy after rejected offers')
    passed('100_and_250_over_cap_offers_preserve_sessions_and_descriptors')
    # A worker failure must not stop the broker or other active sessions.
    victim=workers()[0]; remote(f'kill -KILL {victim}'); wait_workers(7)
    assert broker_pid()==pid
    passed('worker_crash_isolated_and_reaped')
    begin=time.monotonic(); service('stop'); wait_workers(0)
    observations['active_stop_seconds']=round(time.monotonic()-begin,3)
    assert observations['active_stop_seconds'] < 10
    remote('test ! -e /var/run/faction/terminal.sock')
    for c in sessions: detach(c)
    passed('rc_stop_drains_then_reaps_active_members')
    service('start'); c=approved('bobby'); c.raw(b'c'); c.expect('chat[#lobby]> ')
    wait_workers(1); pid=broker_pid()
    begin=time.monotonic();remote(f'kill -KILL {pid}');wait_workers(0)
    observations['broker_loss_worker_exit_seconds']=round(time.monotonic()-begin,3)
    assert observations['broker_loss_worker_exit_seconds']<3
    detach(c)
    remote('test -S /var/run/faction/terminal.sock')
    passed('broker_sigkill_exits_member_without_host_shell')
    service('start'); c=approved('bobby'); c.raw(b'q'); c.expect('Logged out.'); c.finish()
    passed('stale_socket_recovery_and_fresh_authenticated_login')
    # SQLite contention uses an actual held write transaction. A failed post
    # must not corrupt data or bypass login; a subsequent fresh session works.
    c=approved('bobby')
    remote("su - factiond -c 'nohup /usr/pkg/bin/python3.13 -c \"import sqlite3,time; c=sqlite3.connect(\\\"/var/db/faction/faction.sqlite3\\\"); c.execute(\\\"BEGIN IMMEDIATE\\\"); open(\\\"/home/factiond/write-lock\\\",\\\"w\\\").close(); time.sleep(3)\" >/dev/null 2>&1 &' ")
    end=time.monotonic()+2
    while time.monotonic()<end:
        if remote('test -f /home/factiond/write-lock',ok=False).returncode==0:break
        time.sleep(.05)
    else: raise AssertionError('write lock fixture did not become ready')
    c.raw(b'p');c.expect('Room ID: ');c.send('1')
    c.expect('Reply to post ID (blank for new message): ');c.send('')
    c.expect('Subject (blank for generated/reply default): '); c.send('')
    c.expect('Message (one line, up to 1024 bytes): ');c.send('must fail during held write lock')
    c.expect('Database busy. Retry this same submission? [y/N]: '); c.send('n')
    c.expect('Press Enter to return to menu: '); c.send('')
    c.expect('Ctrl-C exits; Esc: Enter to recover.'); c.raw(b'q'); c.expect('Logged out.'); c.finish()
    time.sleep(3.1)
    c=approved('bobby'); c.raw(b'q');c.expect('Logged out.');c.finish()
    passed('sqlite_write_contention_bounded_retry_and_recovery')
    service('stop')
    # A synthetic trusted writer + real member-owned PTY isolates blocked-output
    # supervision. It does not claim the real app implements an output deadline.
    writer='#!/usr/pkg/bin/python3.13\nimport os\nwith open("/home/factiond/output-writer.pid","w") as f:f.write(str(os.getpid()))\nfor i in range(8192):\n with open("/home/factiond/output-progress","w") as f:f.write(str(i))\n os.write(1,b"x"*8192)\n'
    remote('printf %s '+shlex.quote(writer)+' > /usr/local/libexec/faction-test-writer; chmod 755 /usr/local/libexec/faction-test-writer')
    remote('rm -f /home/factiond/output-progress /home/factiond/output-writer.pid /tmp/faction-tty-stalled /tmp/faction-tty-complete /tmp/faction-tty-probe.log')
    uid=remote('id -u factionterm').stdout.decode().strip()
    command=f'nohup /usr/local/libexec/faction-broker --socket /var/run/faction/terminal.sock --terminal-uid {uid} --worker /usr/local/libexec/faction-test-writer --state-dir /var/db/faction --session-seconds 2 >/home/factiond/broker-test.log 2>&1 </dev/null &'
    remote('su - factiond -c '+shlex.quote(command)); time.sleep(.3);broker_pid()
    probe="""import array,fcntl,os,pty,socket,termios,time
m,t=pty.openpty();s=socket.socket(socket.AF_UNIX);s.settimeout(5);s.connect('/var/run/faction/terminal.sock')
s.sendmsg([b'FBSTTY01'+bytes([5])+b'xterm'+bytes(58)],[(socket.SOL_SOCKET,socket.SCM_RIGHTS,array.array('i',[t]))])
time.sleep(1)
queued=array.array('i',[0]);fcntl.ioctl(m,termios.FIONREAD,queued);assert queued[0]>0
open('/tmp/faction-tty-stalled','w').close()
assert s.recv(1)==b''
s.close();os.close(m);os.close(t)
open('/tmp/faction-tty-complete','w').close()
"""
    # Run the probe in the background so root can sample the writer's progress.
    remote('su - factionterm -c '+shlex.quote('nohup /usr/pkg/bin/python3.13 -c '+shlex.quote(probe)+' >/tmp/faction-tty-probe.log 2>&1 &'))
    time.sleep(.6)
    writer_pid=int(remote('cat /home/factiond/output-writer.pid').stdout)
    assert remote(f'ps -p {writer_pid} -o stat=').stdout.strip() not in [b'', b'Z']
    a=remote('cat /home/factiond/output-progress').stdout.strip();time.sleep(.4)
    b=remote('cat /home/factiond/output-progress').stdout.strip()
    assert a==b and int(a)<8191, 'writer was not observably blocked'
    time.sleep(1.5)
    remote("test -f /tmp/faction-tty-stalled; test -f /tmp/faction-tty-complete; test ! -s /tmp/faction-tty-probe.log; ! ps -ax -o comm= | awk '$1==\"/usr/local/libexec/faction-test-writer\" {found=1} END {exit !found}'")
    observations['blocked_writer_completed_chunks']=int(a)
    passed('absolute_session_deadline_reaps_observably_blocked_writer')
    service('stop'); service('start'); broker_pid()
    # Dedicated SSH daemon lifecycle must preserve its restricted configuration.
    remote('/etc/rc.d/faction_sshd status; /etc/rc.d/faction_sshd restart; /etc/rc.d/faction_sshd status')
    passed('dedicated_sshd_rc_restart')
    c=approved('bobby');c.raw(b'q');c.expect('Logged out.');c.finish()
    service('stop');remote('/etc/rc.d/faction_sshd stop')
    passed('fresh_login_after_service_failures_and_ordered_stop')
    return {'schema':1,'scope':'disposable NetBSD eight-member SSH and lifecycle/fault tests',
            'results':results,'observations':observations}
