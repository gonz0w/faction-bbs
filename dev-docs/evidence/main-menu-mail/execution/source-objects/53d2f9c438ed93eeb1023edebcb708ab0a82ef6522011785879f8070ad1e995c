#!/usr/bin/env python3
"""Shared-board authorization and fault oracles ONLY in the marked NetBSD guest."""
import argparse
import json
import os
import shlex
import signal
import secrets
import subprocess
import time
from pathlib import Path
from check_restricted_ssh import SSHConsole


def run(args):
    assert args.disposable_fixture and args.secret_file.stat().st_mode&0o077==0
    secret=args.secret_file.read_text().strip();assert len(secret)>=24
    common=['/usr/bin/ssh','-F','/dev/null','-o','BatchMode=yes','-o','IdentitiesOnly=yes','-o','LogLevel=ERROR',
            '-o','ConnectTimeout=5','-o','StrictHostKeyChecking=yes','-o','UserKnownHostsFile='+str(args.known_hosts)]
    admin=common+['-p',str(args.admin_port),'-i',str(args.admin_identity),'root@'+args.host]
    member=common+['-p',str(args.port),'-i',str(args.identity),'factionterm@'+args.host]
    children=[];results=[]
    def remote(command,data=None,ok=True):
        p=subprocess.run(admin+['set -e; PATH=/usr/sbin:/sbin:/usr/bin:/bin:/usr/pkg/bin; export PATH; '+command],input=data,capture_output=True,timeout=30)
        assert not ok or p.returncode==0,'shared fixture operation failed';return p
    remote('test -f /etc/faction-disposable-test; test -f /var/db/faction-profile/.profile-fixture')
    term=remote('id -u factionterm').stdout.decode().strip();assert term.isdecimal()
    command=f'nohup /usr/local/libexec/shared-bbs --socket /var/run/faction/terminal.sock --terminal-uid {term} --state-dir /var/db/faction-profile >/home/factiond/profile-broker.log 2>&1 </dev/null &'
    def start():
        remote('su - factiond -c '+shlex.quote(command));time.sleep(.5)
        pid=remote('cat /var/run/faction/terminal.pid').stdout.decode().strip();assert pid.isdecimal();return pid
    def stop(pid,sig='TERM'):
        remote('kill -'+sig+' '+pid)
        for _ in range(80):
            if remote('kill -0 '+pid,ok=False).returncode!=0:return
            time.sleep(.1)
        raise AssertionError('shared backend did not stop')
    def drop(c):
        if not c.done:
            try:os.kill(c.pid,signal.SIGKILL)
            except ProcessLookupError:pass
            os.close(c.fd)
            try:os.waitpid(c.pid,0)
            except ChildProcessError:pass
            c.done=True
        else:os.close(c.fd)
    def passed(name):results.append({'oracle':name,'result':'pass'})
    def login(name='profile0000',password=None,approved=True):
        c=SSHConsole(member[:-1]+['-tt',member[-1]]);children.append(c)
        c.expect('[L]ogin [R]egister [Q]uit: ');c.send('l');c.expect('Handle: ');c.send(name);c.expect('Password (hidden): ');c.send(password or secret)
        if approved:c.expect('Press Enter to continue: ');c.send('');c.expect('Ctrl-C exits; Esc: Enter to recover.')
        return c
    def status(action,name):
        c=SSHConsole(admin[:-1]+['-tt',admin[-1],"su - factiond -c '/usr/local/libexec/faction --state-dir /var/db/faction-profile login'"]);children.append(c)
        c.expect('Handle: ');c.send('profile_sysop');c.expect('Password (hidden): ');c.send(secret);c.expect('Command (then Enter): ')
        c.send('s');c.expect('[C]reate room: ');c.send(action);c.expect('Member handle: ');c.send(name)
        c.expect('Status updated; previous sessions expired.');c.expect('Command (then Enter): ');c.send('q');c.expect('Logged out.');c.finish()
    def owner_chat(commands):
        c=SSHConsole(admin[:-1]+['-tt',admin[-1],"su - factiond -c '/usr/local/libexec/faction --state-dir /var/db/faction-profile login'"]);children.append(c)
        c.expect('Handle: ');c.send('profile_sysop');c.expect('Password (hidden): ');c.send(secret);c.expect('Command (then Enter): ')
        c.send('c');c.expect('chat[#lobby]> ')
        for command in commands:
            c.send(command)
            c.expect('Channel created.' if command.startswith('/create ') else 'Channel grant updated; target must join again.')
            c.expect('chat[#lobby]> ')
        c.send('/quit');c.expect('Chat closed.');c.expect('Command (then Enter): ');c.send('q');c.expect('Logged out.');c.finish()
    pending='pending_'+secrets.token_hex(4)
    channel='#probe_'+secrets.token_hex(4)
    pid=start()
    try:
        # Registration cannot see identity/rooms. Approval is a separate local sysop action.
        c=SSHConsole(member[:-1]+['-tt',member[-1]]);children.append(c)
        c.expect('[L]ogin [R]egister [Q]uit: ');c.send('r');c.expect('Handle: ');c.send(pending);c.expect('Password (hidden): ');c.send(secret)
        c.expect('Registration pending; sysop approval required.');c.finish()
        c=login(pending,approved=False);c.expect('Sysop approval required. No community access.');c.finish()
        assert b'Press Enter to continue' not in c.transcript;passed('pending registration and login have no community access')
        status('a',pending);c=login(pending);c.raw(b'c');c.expect('chat[#lobby]> ')
        status('v',pending);c.expect('Action failed:');c.finish(1);passed('active chat revocation rechecks current core epoch')
        status('a',pending);c=login(pending);c.raw(b'q');c.expect('Logged out.');c.finish();passed('reapproval requires fresh login')
        owner_chat(['/create '+channel+' private','/allow '+channel+' profile0000'])
        c=login();c.raw(b'c');c.expect('chat[#lobby]> ');c.send('/join '+channel);c.expect('Joined.');c.expect('chat['+channel+']> ')
        owner_chat(['/deny '+channel+' profile0000']);c.expect('Action failed:');c.finish(1);passed('private channel grant removal terminates selected connection without redirect')
        c=login('profile0001');c.raw(b'c');c.expect('chat[#lobby]> ');c.send('/join '+channel);c.expect('Action failed:');c.finish(1);passed('private channel foreign account denied')
        owner_chat(['/allow '+channel+' profile0000']);c=login();c.raw(b'c');c.expect('chat[#lobby]> ');c.send('/join '+channel);c.expect('Joined.');c.expect('chat['+channel+']> ')
        owner_chat(['/allow '+channel+' profile0000']);c.expect('Action failed:');c.finish(1);passed('regrant does not revive prior private join')
        a=login();b=login('profile0000','incorrect-credential-for-test',False);b.expect('invalid credentials');b.finish(1)
        a.raw(b'r');a.expect('Ctrl-C exits; Esc: Enter to recover.');passed('failed second connection cannot borrow approved first session')
        a.raw(b'\x1b[B');a.expect('New messages');a.raw(b'\x1b[A');a.expect('Message rooms')
        a.raw(b'p');a.expect('Lobby message');a.send('shared durability oracle');a.expect('Posted #');a.expect('Ctrl-C exits; Esc: Enter to recover.');passed('arrows hotkeys and core durable post')
        # Frontend crash must release its connection; another member stays usable.
        b=login('profile0001');drop(b);children.remove(b);time.sleep(.5)
        a.raw(b'r');a.expect('Ctrl-C exits; Esc: Enter to recover.');passed('frontend crash isolated from another member')
        probe=remote('/usr/pkg/bin/python3.13 /home/factiond/build/experiments/055-shared-board/terminal_probe.py',secret.encode())
        results.extend(json.loads(probe.stdout)['results'])
        a.raw(b'r');a.expect('Ctrl-C exits; Esc: Enter to recover.');passed('other SSH member remains usable after slow reader')
        stop(pid,'KILL');a.finish(1);passed('backend crash closes member frontend')
        pid=start();c=login()
        code="import sqlite3; c=sqlite3.connect('/var/db/faction-profile/faction.sqlite3'); assert c.execute(\"select count(*) from posts where body='shared durability oracle'\").fetchone()[0]==1; assert c.execute('pragma integrity_check').fetchone()[0]=='ok'"
        remote('/usr/pkg/bin/python3.13 -c '+shlex.quote(code));passed('backend crash restart preserves committed post and integrity')
        c.raw(b'q');c.expect('Logged out.');c.finish();passed('fresh login and logout after backend crash')
        for child in children:assert secret.encode() not in child.transcript,'password echoed'
        args.output.write_text(json.dumps({'schema':1,'results':results,'scope':'focused experiment; full UI/grant/fairness adoption gates remain'},indent=2)+'\n')
        print(str(len(results))+' shared-board native authorization/terminal/fault oracles passed')
    finally:
        for child in children:drop(child)
        if remote('kill -0 '+pid,ok=False).returncode==0:stop(pid)


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--disposable-fixture',action='store_true');p.add_argument('--host',required=True)
    p.add_argument('--port',type=int,required=True);p.add_argument('--admin-port',type=int,required=True)
    p.add_argument('--identity',type=Path,required=True);p.add_argument('--admin-identity',type=Path,required=True)
    p.add_argument('--known-hosts',type=Path,required=True);p.add_argument('--secret-file',type=Path,required=True);p.add_argument('--output',type=Path,required=True)
    run(p.parse_args())
