#!/usr/bin/env python3
"""Opt-in room writing on private local/worker PTYs; no network listener."""
import argparse,json,secrets,sqlite3,tempfile
from pathlib import Path
from smoke_local_console import Console
from smoke_member_worker import Worker
from smoke_room_browser import MENU
PROMPT='Write> '
BODY='Message (one line, up to 1024 bytes): '

def begin(c,plain=False,reply='',subject='Writing together'):
    c.send('p') if plain else c.raw(b'p')
    c.expect('Room ID: ');c.send('1');c.expect('Reply to post ID (blank for new message): ');c.send(reply)
    c.expect('Subject (blank for generated/reply default): ');c.send(subject);c.expect(BODY);c.send('/edit');c.expect(PROMPT)

def action_end(c,plain=False,shared=False):
    if not plain or shared:c.expect('Press Enter to return to menu: ');c.send('')
    c.expect('Command (then Enter): ' if plain else MENU)

def command(c,text):c.send(text);c.expect(PROMPT)

def exercise(c,count,plain=False,shared=False):
    checks=[];baseline=count()
    begin(c,plain)
    command(c,'.send');command(c,'First signal.');command(c,'discard me');command(c,'.undo');command(c,'');command(c,'Second signal.');command(c,'.preview')
    assert count()==baseline,'preview submitted a message'
    assert b'DRAFT PREVIEW (not sent)' in c.transcript and b'Second signal.' in c.transcript
    c.send('.send');c.expect('Posted #');action_end(c,plain,shared);assert count()==baseline+1;checks.append('multiline_preview_undo_blank_paragraph_and_explicit_send')
    # Fixture's first post is the known reply; it is freshly fetched through Board.
    begin(c,plain,reply='1',subject='');command(c,'.quote');command(c,'My reply.');command(c,'.preview');assert count()==baseline+1
    c.send('.send');c.expect('Posted #');action_end(c,plain,shared);assert count()==baseline+2;checks.append('authorized_reply_quote_and_subject_default')
    begin(c,plain);command(c,'Never sent.');c.send('.cancel');c.expect('Draft cancelled; nothing sent.');action_end(c,plain,shared);assert count()==baseline+2;checks.append('cancel_does_not_store_or_send')
    begin(c,plain);command(c,'x'*1000);c.send('y'*24);c.expect('draft limit:');c.expect(PROMPT);c.send('.preview');c.expect('Preview more? [y/N]: ');c.send('n');c.expect(PROMPT);assert count()==baseline+2;c.send('.cancel');c.expect('Draft cancelled');action_end(c,plain,shared);checks.append('oversize_line_keeps_previous_draft')
    begin(c,plain)
    for _ in range(32):command(c,'')
    c.send('line 33');c.expect('draft limit:');c.expect(PROMPT);command(c,'.undo');command(c,'line 32');c.send('.cancel');c.expect('Draft cancelled');action_end(c,plain,shared);checks.append('line_limit_and_recovery')
    return checks

def run(a):
    checks=[];children=[];password=secrets.token_hex(20)
    with tempfile.TemporaryDirectory(prefix='faction-writing-') as tmp:
        state=Path(tmp)/'board'
        def count():
            with sqlite3.connect(f'file:{state}/faction.sqlite3?mode=ro',uri=True) as db:return db.execute('select count(*) from posts').fetchone()[0]
        def start(mode='ansi',worker=False):
            c=Worker(a.member,state) if worker else Console(a.binary,state,'login',['--plain' if mode=='plain' else '--ansi']);children.append(c)
            if worker:c.expect('[L]ogin [R]egister [Q]uit: ');c.send('l')
            c.expect('Handle: ');c.send('sysop');c.password(password)
            if mode!='plain':c.expect('Press Enter to continue: ');c.send('')
            c.expect('Command (then Enter): ' if mode=='plain' else MENU);return c
        def logout(c,plain=False):
            c.send('q') if plain else c.raw(b'q');c.expect('Logged out.');c.finish()
        try:
            c=Console(a.binary,state,'init');children.append(c);c.expect('Handle: ');c.send('sysop');c.password(password,'New password (12+ characters, hidden): ');c.password(password,'Confirm password (hidden): ');c.expect('Lobby created.');c.finish()
            c=start();checks+=exercise(c,count);logout(c)
            c=start('plain');checks+=['plain_'+s for s in exercise(c,count,True)];logout(c,True);assert b'\x1b' not in c.transcript
            c=start(worker=True);begin(c);command(c,'Worker signal.');c.send('.send');c.expect('Posted #');action_end(c);logout(c);checks.append('worker_inherited_terminal_sends_multiline_draft')
            before=count();c=start();begin(c);command(c,'Lost on disconnect.');c.close();children.remove(c);c=start();begin(c);c.send('.send');c.expect('empty draft');c.expect(PROMPT);c.send('.cancel');c.expect('Draft cancelled');action_end(c);logout(c);assert count()==before;checks.append('disconnect_draft_not_persisted_or_resurrected')
            c=start();c.raw(b't');c.expect('[D]one: ');c.send('u');c.expect('[D]one: ');c.send('d');action_end(c);begin(c,subject='Échos 界');command(c,'Café 界');command(c,'.preview');assert 'Café 界'.encode() in c.transcript;c.send('.send');c.expect('Posted #');action_end(c);logout(c);checks.append('unicode_subject_and_body_follow_selected_capabilities')
            with sqlite3.connect(f'file:{state}/faction.sqlite3?mode=ro',uri=True) as db:
                assert db.execute('select body from posts where id=1').fetchone()[0]=='First signal.\n\nSecond signal.'
                assert db.execute('select body from posts where id=2').fetchone()[0]=='> First signal.\n> \n> Second signal.\nMy reply.'
                assert db.execute('pragma user_version').fetchone()[0]==20
            assert all(password.encode() not in c.transcript for c in children)
        finally:
            for c in children:c.close()
    out=dict(result='pass',count=len(checks),checks=checks,capacity_claim=False,persistent_drafts=False,schema_change=False)
    if a.output:a.output.write_text(json.dumps(out,indent=2,sort_keys=True)+'\n')
    print(f'Room writing desk passed {len(checks)} local/worker controls.')

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('binary',type=Path);p.add_argument('member',type=Path);p.add_argument('--output',type=Path);a=p.parse_args();a.binary=a.binary.resolve();a.member=a.member.resolve();run(a)
