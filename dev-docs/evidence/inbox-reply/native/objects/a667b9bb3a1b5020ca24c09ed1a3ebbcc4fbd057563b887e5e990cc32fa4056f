#!/usr/bin/env python3
"""Real shared UI parity/security/failure oracles, marked private NetBSD VM only."""
import argparse,fcntl,json,os,pty,secrets,shlex,signal,struct,subprocess,termios,time
from pathlib import Path
from smoke_local_console import Console

class Terminal(Console):
    def __init__(self,argv,term='xterm-256color'):
        self.pid,self.fd=pty.fork()
        if self.pid==0:
            fcntl.ioctl(0,termios.TIOCSWINSZ,struct.pack('HHHH',24,80,0,0));os.environ['TERM']=term
            os.closerange(3,os.sysconf('SC_OPEN_MAX'));os.execv('/usr/bin/ssh',argv)
        self.buffer,self.transcript,self.done=b'',b'',False
    def resize(self,columns,rows):
        fcntl.ioctl(self.fd,termios.TIOCSWINSZ,struct.pack('HHHH',rows,columns,0,0));os.kill(self.pid,signal.SIGWINCH)

def run(args):
    assert args.disposable_fixture and args.secret_file.stat().st_mode&0o077==0
    secret=args.secret_file.read_text().strip();assert len(secret)>=24
    common=['/usr/bin/ssh','-F','/dev/null','-o','BatchMode=yes','-o','IdentitiesOnly=yes','-o','LogLevel=ERROR','-o','ConnectTimeout=5','-o','StrictHostKeyChecking=yes','-o','UserKnownHostsFile='+str(args.known_hosts)]
    admin=common+['-p',str(args.admin_port),'-i',str(args.admin_identity),'root@'+args.host]
    member=common+['-p',str(args.port),'-i',str(args.identity),'-tt','factionterm@'+args.host]
    children=[];results=[];latencies=[]
    def remote(command,data=None,ok=True):
        x=subprocess.run(admin+['set -e; PATH=/usr/sbin:/sbin:/usr/bin:/bin:/usr/pkg/bin; export PATH; '+command],input=data,capture_output=True,timeout=40)
        assert not ok or x.returncode==0,'private UI fixture action failed';return x
    def passed(name):results.append({'oracle':name,'result':'pass'})
    remote('test -f /etc/faction-disposable-test; test -f /var/db/faction-profile/.profile-fixture; test ! -S /var/run/faction/terminal.sock')
    uid=remote('id -u factionterm').stdout.decode().strip();assert uid.isdecimal()
    binary=remote('sha256 -q /usr/local/libexec/shared-bbs').stdout.decode().strip()
    root='/home/factiond/build/experiments/055-shared-ui/fixtures'
    command=f'nohup /usr/local/libexec/shared-bbs --socket /var/run/faction/terminal.sock --terminal-uid {uid} --state-dir /var/db/faction-profile --theme {root}/base --season {root}/winter >/home/factiond/profile-broker.log 2>&1 </dev/null &'
    def start(extra=''):
        remote('su - factiond -c '+shlex.quote(command+extra));time.sleep(.5)
        pid=remote('cat /var/run/faction/terminal.pid').stdout.decode().strip();assert pid.isdecimal();return pid
    def stop(pid,sig='TERM'):
        remote('kill -'+sig+' '+pid)
        for _ in range(100):
            if remote('kill -0 '+pid,ok=False).returncode!=0:return
            time.sleep(.1)
        raise AssertionError('UI owner did not stop')
    def drop(c):
        if not c.done:
            try:os.kill(c.pid,signal.SIGKILL)
            except ProcessLookupError:pass
            try:os.waitpid(c.pid,0)
            except ChildProcessError:pass
            c.done=True
        os.close(c.fd)
    def enter(choice,name,term='xterm-256color',password=None):
        c=Terminal(member,term);children.append(c)
        c.expect('[L]ogin [R]egister [Q]uit: ');c.send(choice);c.expect('Handle: ');c.send(name);c.expect('Password (hidden): ');c.send(password or secret)
        if choice=='r':c.expect('Confirm password (hidden): ');c.send(password or secret)
        return c
    def menu(c,plain=False):c.expect('Command (then Enter): ' if plain else 'Ctrl-C exits; Esc: Enter to recover.')
    def login(name='profile0000',term='xterm-256color'):
        c=enter('l',name,term);c.expect('WINTER TEST IDENTITY' if term!='dumb' else 'ASCII TEST IDENTITY');c.expect('Winter frequency.');c.expect('Press Enter to continue: ');c.send('');menu(c,term=='dumb');return c
    def pause(c):c.expect('Press Enter to return to menu: ');c.send('');menu(c)
    def status(c,key,name):
        c.raw(b's');c.expect('[C]reate room: ');c.send(key);c.expect('Member handle: ');c.send(name);c.expect('Status updated; previous sessions expired.');pause(c)
    def chat(c):c.raw(b'c');c.expect('chat[#lobby]> ')
    def quit_chat(c):c.send('/quit');c.expect('Chat closed.');menu(c)
    def logout(c):c.raw(b'q');c.expect('Logged out.');c.finish()
    pending='ui_'+secrets.token_hex(4);channel='#ui_'+secrets.token_hex(4);body='shared ui durability '+secrets.token_hex(8)+' café'
    pid=start()
    try:
        c=enter('r',pending);c.expect('Registration pending; sysop approval required.');c.finish();assert b'TEST IDENTITY' not in c.transcript
        c=enter('l',pending);c.expect('Sysop approval required. No community access.');c.finish();assert b'TEST IDENTITY' not in c.transcript
        passed('registration confirmation and pending login cannot see identity or community')
        sysop=login('profile_sysop')
        sysop.raw(b's');sysop.expect('[C]reate room: ');sysop.send('c');sysop.expect('Room slug: ');sysop.send('ui-'+secrets.token_hex(4));sysop.expect('Room title: ');sysop.send('UI parity room');sysop.expect('Created room #');pause(sysop)
        code="import sqlite3; c=sqlite3.connect('/var/db/faction-profile/faction.sqlite3'); print(c.execute('select max(id) from rooms').fetchone()[0])"
        room=remote('/usr/pkg/bin/python3.13 -c '+shlex.quote(code)).stdout.decode().strip();assert room.isdecimal()
        status(sysop,'a',pending);c=login(pending);chat(c)
        status(sysop,'v',pending);c.expect('Action failed:');c.finish(1)
        status(sysop,'a',pending);c=login(pending);logout(c)
        passed('shared sysop approval revocation and fresh reapproval login')
        a=login();b=login('profile0001')
        # The fixture changes the history key to T and reorders history/new.
        a.raw(b'\x1b[B');a.expect('[T]');a.raw(b'\r');a.expect('Room ID: ');a.send('1');a.expect('More? [y/N]: ');a.send('n');pause(a)
        a.raw(b't');a.expect('Room ID: ');a.send('1');a.expect('More? [y/N]: ');a.send('n');pause(a)
        passed('custom menu ordering arrows Enter and displayed hotkey use same room action')
        a.raw(b'r');a.expect('More rooms? [y/N]: ')
        for _ in range(4):a.send('y');a.expect('More rooms? [y/N]: ')
        a.send('y');pause(a);passed('room list pages beyond first ten without truncating output')
        a.raw(b'p');a.expect('Room ID: ');a.send(room);a.expect('Reply to post ID');a.send('');a.expect('Subject (blank for generated/reply default): ');a.send('');a.expect('Message (one line');a.send(body);a.expect('Submission ID: ');a.expect('Posted #');pause(a)
        a.raw(b't');a.expect('Room ID: ');a.send(room);a.expect(body.replace('\u00e9',r'\u{e9}'));a.expect('More? [y/N]: ');a.send('n');pause(a)
        passed('selected non-lobby room durable post submission ID and read page')
        a.raw(b'n');a.expect('Room ID: ');a.send(room);a.expect('No more messages.');pause(a)
        passed('read cursor advances after complete output drains')
        # Fetch only synthetic post metadata; never save actual message contents.
        code="import sqlite3; c=sqlite3.connect('/var/db/faction-profile/faction.sqlite3'); print(c.execute('select max(id) from posts where room_id="+room+"').fetchone()[0])"
        parent=remote('/usr/pkg/bin/python3.13 -c '+shlex.quote(code)).stdout.decode().strip();assert parent.isdecimal()
        a.raw(b'p');a.expect('Room ID: ');a.send(room);a.expect('Reply to post ID');a.send(parent);a.expect('Subject (blank for generated/reply default): ');a.send('');a.expect('Message (one line');a.send('reply parity');a.expect('Posted #');pause(a)
        a.raw(b't');a.expect('Room ID: ');a.send(room);a.expect('(reply to #'+parent+')');a.expect('reply parity');a.expect('More? [y/N]: ');a.send('n');pause(a)
        passed('same-room reply presentation and history')
        a.raw(b'\x1b[9;p\n');a.raw(b'r');a.expect('More rooms? [y/N]: ');a.send('n');pause(a)
        a.raw(b'pINJECTED\n');a.expect('Room ID: ');a.send(room);a.expect('Reply to post ID');a.send('');a.expect('Subject (blank for generated/reply default): ');a.send('');a.expect('Message (one line');a.send('context boundary');a.expect('Posted #');pause(a)
        passed('unknown escape quarantine and menu typeahead cannot become editor input')
        a.raw(b'p');a.expect('Room ID: ');a.send(room);a.expect('Reply to post ID');a.send('');a.expect('Subject (blank for generated/reply default): ');a.send('');a.expect('Message (one line')
        lock="import sqlite3,time; from pathlib import Path; c=sqlite3.connect('/var/db/faction-profile/faction.sqlite3'); c.execute('begin immediate'); Path('/home/factiond/ui-lock-ready').touch(); time.sleep(3); c.rollback()"
        lock_command='nohup /usr/pkg/bin/python3.13 -c '+shlex.quote(lock)+' >/home/factiond/ui-lock.log 2>&1 </dev/null &'
        remote('test ! -e /home/factiond/ui-lock-ready; su - factiond -c '+shlex.quote(lock_command))
        for _ in range(40):
            if remote('test -f /home/factiond/ui-lock-ready',ok=False).returncode==0:break
            time.sleep(.05)
        else:raise AssertionError('synthetic writer lock did not start')
        a.send('retry parity');a.expect('Submission ID: ');a.expect('Retry this same submission? [y/N]: ');time.sleep(3.1);a.send('y');a.expect('Posted #');pause(a)
        code="import sqlite3; c=sqlite3.connect('/var/db/faction-profile/faction.sqlite3'); assert c.execute(\"select count(*) from posts where body='retry parity'\").fetchone()[0]==1"
        remote('/usr/pkg/bin/python3.13 -c '+shlex.quote(code));remote('rm /home/factiond/ui-lock-ready');passed('busy durable post retries the same submission exactly once')
        # A member's disabled S/D cannot select an owner or sysop action.
        a.raw(b'sd');a.raw(b'c');a.expect('chat[#lobby]> ');assert b'Sysop: [L]ist' not in a.transcript
        b.raw(b'c');b.expect('chat[#lobby]> ')
        b.raw(b'unsent draft');time.sleep(.15);marker='incoming '+secrets.token_hex(8);a.send(marker);b.expect(marker);b.expect('unsent draft');b.raw(b'\x7f'*12);b.send('/help');b.expect('/kick #channel handle');passed('live editor preserves draft across incoming chat and exposes complete help')
        a.send('/me emote parity');b.expect('emote parity');a.send('/names');a.expect('Members:');a.send('/list');a.expect('#lobby (board, local)');a.send('/history');a.expect('emote parity')
        a.send('/part');a.expect('Left channel.');a.send('must not redirect');a.expect('join a channel first');a.send('/join #lobby');a.expect('Joined;');passed('chat emotes roster list history part and explicit rejoin')
        chat(sysop);sysop.send('/create '+channel+' private');sysop.expect('Channel created.');sysop.send('/allow '+channel+' profile0000');sysop.expect('Channel grant updated;');a.send('/join '+channel);a.expect('Joined;')
        b.send('/join '+channel);b.expect('Channel access changed or action denied; draft discarded.');b.expect('chat[-]> ')
        a.raw(b'sensitive draft');sysop.send('/deny '+channel+' profile0000');sysop.expect('Channel grant updated;');a.expect('draft discarded.');a.expect('chat[-]> ')
        a.send('must not redirect');a.expect('join a channel first');sysop.send('/allow '+channel+' profile0000');sysop.expect('Channel grant updated;');a.send('/join '+channel);a.expect('Joined;')
        sysop.send('/allow '+channel+' profile0000');sysop.expect('Channel grant updated;');a.expect('draft discarded.');a.send('/join '+channel);a.expect('Joined;')
        sysop.send('/kick '+channel+' profile0000');sysop.expect('Member parted;');a.expect('draft discarded.');a.send('/join '+channel);a.expect('Joined;')
        passed('private foreign connection deny regrant and kick require explicit fresh join without redirect')
        quit_chat(a);quit_chat(b);quit_chat(sysop)
        bad=enter('l','profile0000',password='incorrect-credential-for-test');bad.expect('invalid credentials');bad.finish(1);a.raw(b'r');a.expect('More rooms?');a.send('n');pause(a)
        passed('second connection cannot borrow an approved session')
        sysop.raw(b's');sysop.expect('[C]reate room: ');sysop.send('c');sysop.expect('Room slug: ');sysop.send('ui-'+secrets.token_hex(4));sysop.expect('Room title: ');sysop.send('UI test room');sysop.expect('Created room #');pause(sysop)
        sysop.raw(b's');sysop.expect('[C]reate room: ');sysop.send('l');sysop.expect('More accounts?');sysop.send('n');pause(sysop);passed('shared sysop room creation and paged member listing')
        plain=login(term='dumb');assert b'\x1b[' not in plain.transcript;plain.send('r');plain.expect('More rooms?');plain.send('n');plain.expect('Press Enter to return to menu: ');plain.send('');menu(plain,True);plain.send('c');plain.expect('chat[#lobby]> ');began=len(plain.transcript);plain.raw(b'a\x7f/poll\n');plain.expect('chat[#lobby]> ');assert b'\x08 \x08' in plain.transcript[began:];plain.send('/quit');plain.expect('Chat closed.');menu(plain,True);passed('plain chat backspace edits and renders without ANSI');plain.send('q');plain.expect('Logged out.');plain.finish();passed('dumb TERM keeps ASCII identity and plain hotkeys without ANSI')
        a.resize(30,12);a.expect('Terminal resized; draft discarded.');menu(a,True);a.send('r');a.expect('More rooms?');a.send('n');a.expect('Press Enter to return to menu: ');a.resize(80,24);a.send('');menu(a);passed('menu resize falls back plain and later recovers ANSI')
        chat(a);a.raw(b'discard on resize');a.resize(30,12);a.expect('Terminal resized; draft discarded.');a.send('/quit');a.expect('Chat closed.');menu(a,True);a.resize(80,24);a.send('r');a.expect('More rooms?');a.send('n');a.expect('Press Enter to return to menu: ');a.send('');menu(a);passed('chat resize discards unsent draft without submission')
        chat(a);a.raw(b'\x03');a.expect('Chat closed.');menu(a);passed('chat Ctrl-C returns to menu and closes lease')
        # One hostile busy terminal must not block a distinct member's useful work.
        chat(b)
        end=time.monotonic()+2
        while time.monotonic()<end:
            a.raw(b'\x1b[B\x1b[A'*8)
            began=time.monotonic();b.send('/names');b.expect('Members:');b.expect('chat[#lobby]> ');latencies.append(time.monotonic()-began)
            # Drain the attacker's bounded menu redraws as a fast reader.
            import select
            if select.select([a.fd],[],[],0)[0]:os.read(a.fd,65536)
        assert max(latencies)<2;quit_chat(b);passed('busy menu peer preserves useful chat service with bounded observed latency')
        drop(b);children.remove(b);a.raw(b'r');a.expect('More rooms?');a.send('n');pause(a);passed('frontend crash leaves another member usable')
        probe=remote('/usr/pkg/bin/python3.13 '+root+'/../terminal_probe.py',secret.encode());results.extend(json.loads(probe.stdout)['results'])
        stop(pid,'KILL');a.finish(1);sysop.finish(1);passed('backend crash closes member frontends')
        pid=start();a=login()
        code="import sqlite3; c=sqlite3.connect('/var/db/faction-profile/faction.sqlite3'); assert c.execute('pragma integrity_check').fetchone()[0]=='ok'; assert c.execute('select count(*) from posts where body=?',("+repr(body)+",)).fetchone()[0]==1"
        remote('/usr/pkg/bin/python3.13 -c '+shlex.quote(code));logout(a);passed('backend restart preserves committed post integrity and fresh login')
        c=login();began=len(c.transcript);c.raw(b'\x03');c.finish(1)
        assert b'\x1b[?25h' in c.transcript[began:];passed('responsive ANSI menu cancellation restores visible cursor')
        for c in children:assert secret.encode() not in c.transcript,'hidden password echoed'
        args.output.write_text(json.dumps({'schema':1,'binary_sha256':binary,'results':results,'fairness_seconds':latencies,'scope':'native shared UI parity/fault probe; no production or soak claim'},indent=2)+'\n')
        print(str(len(results))+' shared UI native oracles passed')
    except BaseException:
        (args.output.with_suffix('.private-failure.log')).write_bytes(b'\n'.join(c.transcript[-12000:] for c in children).replace(secret.encode(),b'[REDACTED]'))
        raise
    finally:
        for c in children:drop(c)
        if remote('kill -0 '+pid,ok=False).returncode==0:stop(pid)

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--disposable-fixture',action='store_true')
    for flag in ['host','identity','admin-identity','known-hosts','secret-file','output']:p.add_argument('--'+flag,type=Path if flag!='host' else str,required=True)
    for flag in ['port','admin-port']:p.add_argument('--'+flag,type=int,required=True)
    run(p.parse_args())
