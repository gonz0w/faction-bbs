#!/usr/bin/env python3
"""Real local/worker PTYs: modern text, explicit preferences and isolated probes."""
import argparse, secrets, sqlite3, tempfile
from pathlib import Path
from smoke_local_console import Console
from smoke_member_worker import Worker
PROMPT='[D]one: '
QUERY='\x1b[?2026$p'

def smoke(binary, member):
    children=[];checks=[];password=secrets.token_hex(20)
    with tempfile.TemporaryDirectory(prefix='faction-modern-') as root:
        board=Path(root)/'board'
        def login(mode):
            if mode=='worker':
                c=Worker(member,board);children.append(c);c.expect('[L]ogin [R]egister [Q]uit: ');c.send('l')
            else:
                c=Console(binary,board,'login',['--ansi'] if mode=='ansi' else [],columns=40,rows=18);children.append(c)
            c.expect('Handle: ');c.send('sysop');c.password(password)
            if mode=='worker':c.expect('Press Enter to continue: ');c.send('')
            c.expect('Command (then Enter): ' if mode=='plain' else 'Ctrl-C exits; Esc: Enter to recover.')
            return c
        def key(c,mode,k):c.send(k) if mode=='plain' else c.raw(k.encode())
        def back(c,mode):
            if mode!='plain':c.expect('Press Enter to return to menu: ');c.send('')
            c.expect('Command (then Enter): ' if mode=='plain' else 'Ctrl-C exits; Esc: Enter to recover.')
        def settings(c,mode):key(c,mode,'t');c.expect(PROMPT)
        def choose(c,k):c.send(k);c.expect(PROMPT)
        def done(c,mode):c.send('d');back(c,mode)
        def post(c,mode,subject,body):
            key(c,mode,'p');c.expect('Room ID: ');c.send('1');c.expect('Reply to post ID (blank for new message): ');c.send('')
            c.expect('Subject (blank for generated/reply default): ')
            if subject=='Café 界 🦀':c.raw('Café 界'.encode()+b'\x7f');c.send('界 🦀')
            else:c.send(subject)
            c.expect('Message (one line, up to 1024 bytes): ');c.send(body);c.expect('Posted #');back(c,mode)
        try:
            c=Console(binary,board,'init');children.append(c);c.expect('Handle: ');c.send('sysop')
            c.password(password,'New password (12+ characters, hidden): ');c.password(password,'Confirm password (hidden): ');c.expect('Lobby created.');c.finish()
            for mode in ['plain','ansi','worker']:
                c=login(mode);settings(c,mode)
                choose(c,'u');choose(c,'2');choose(c,'3');choose(c,'b');choose(c,'a')
                assert '▁▂▃▄'.encode() in c.transcript and '⡀⡄'.encode() in c.transcript
                c.send('p')
                if mode!='plain':
                    c.expect(QUERY);c.raw(b'\x1b[?20');c.raw(b'26;2$y');c.expect('sync: true');c.expect(PROMPT)
                else:c.expect('ANSI disabled; probe refused.');c.expect(PROMPT);assert QUERY.encode() not in c.transcript
                done(c,mode)
                start=len(c.transcript);post(c,mode,'Café 界 🦀','界🦀é '*70+'FINAL-MODERN')
                key(c,mode,'n');c.expect('Room ID: ');c.send('1');read=len(c.transcript)
                for _ in range(40):
                    c.expect('Subject: Café 界 🦀');c.expect('More? [y/N]: ')
                    if b'FINAL-MODERN' in c.transcript[read:]:break
                    c.send('y')
                else:raise AssertionError('Unicode page failed to finish')
                assert c.transcript[read:].count('界🦀é'.encode())==70
                c.send('n');back(c,mode)
                text=c.transcript[start:]
                if mode!='plain':assert b'38;2;' in text and b'\x1b[?2026h' in text and b'\x1b[?2026l' in text
                checks.append(mode+'_native_unicode_cells_color_graphs_and_fallback')
                settings(c,mode);choose(c,'c');done(c,mode)
                post(c,mode,'café','classic body')
                key(c,mode,'n');c.expect('Room ID: ');c.send('1');c.expect('Subject: caf\\u{e9}');c.expect('More? [y/N]: ');c.send('n');back(c,mode)
                key(c,mode,'q');c.expect('Logged out.');c.finish()
            for mode in ['ansi','worker']:
                c=login(mode);settings(c,mode);c.send('p');c.expect(QUERY);c.expect('sync: false');c.expect(PROMPT)
                done(c,mode);key(c,mode,'q');c.expect('Logged out.');c.finish()
                for reply in [b'\x1b[?2026;2$yP\n',b'\x1b',b'\x1b]52;c;bad\x07']:
                    c=login(mode);settings(c,mode);c.send('p');c.expect(QUERY);c.raw(reply);c.finish(1)
                    assert b'Room ID: ' not in c.transcript
                c=login(mode);settings(c,mode);c.send('p');c.expect(QUERY);c.expect(PROMPT)
                c.raw(b'\x1b[?2026;2$yP\n');c.finish(1);assert b'Room ID: ' not in c.transcript
                checks.append(mode+'_unknown_malformed_incomplete_and_late_probe_isolation')
            with sqlite3.connect(board/'faction.sqlite3') as db:assert db.execute('SELECT count(*) FROM posts').fetchone()[0]==6
            print(f'Modern terminal PTYs passed {len(checks)} controls: native UTF-8/cells, color/graphs, classic recovery and isolated known/unknown/malformed/late probes.')
            return checks
        finally:
            for c in children:c.close()

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('binary',type=Path);p.add_argument('member',type=Path);a=p.parse_args();smoke(a.binary.resolve(),a.member.resolve())
