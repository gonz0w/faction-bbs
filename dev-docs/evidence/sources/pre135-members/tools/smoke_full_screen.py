#!/usr/bin/env python3
"""Full-screen and explicit saved-room drafts on private local/worker PTYs."""
import argparse,json,secrets,sqlite3,tempfile
from pathlib import Path
from smoke_local_console import Console
from smoke_member_worker import Worker
from smoke_room_browser import MENU
from smoke_writing_desk import PROMPT,begin,command,action_end
FOOTER='Arrows edit; ^A/^E start/end'
SAVED='Saved room draft: [R]esume [D]elete and start new [Q]uit: '

def open_saved(c,plain=False):
    c.send('p') if plain else c.raw(b'p');c.expect('Room ID: ');c.send('1');c.expect(SAVED);c.send('r');c.expect(PROMPT)

def enter_screen(c):c.send('.screen');c.expect(FOOTER)

def edit_save(c):
    begin(c,subject='Saved night signal');command(c,'first');enter_screen(c)
    c.raw(b'\x01bright ');c.expect('bright first');c.expect(FOOTER)
    c.raw(b'\x05\rtail');c.expect('tail');c.expect(FOOTER)
    c.raw(b'\x10');c.expect('DRAFT PREVIEW (not sent)');c.expect('bright first');c.expect('tail');c.expect(FOOTER)
    c.raw(b'\x13');c.expect('Checkpoint saved on this board for 30 days');c.expect(FOOTER)

def resume_send(c,count,shared=False):
    before=count();open_saved(c);c.send('.preview');c.expect('Subject: Saved night signal');c.expect('bright first');c.expect('tail');c.expect(PROMPT);assert count()==before
    enter_screen(c);c.raw(b'\x14');c.expect('Type SEND to post, or Enter to keep editing: ');c.send('no');c.expect(FOOTER);assert count()==before
    c.raw(b'\x14');c.expect('Type SEND to post, or Enter to keep editing: ');c.send('SEND');c.expect('Posted #');action_end(c,shared=shared);assert count()==before+1

def run(a):
    checks=[];children=[];password=secrets.token_hex(20)
    with tempfile.TemporaryDirectory(prefix='faction-screen-') as tmp:
        state=Path(tmp)/'board'
        def query(sql):
            with sqlite3.connect(f'file:{state}/faction.sqlite3?mode=ro',uri=True) as db:return db.execute(sql).fetchall()
        def count():return query('select count(*) from posts')[0][0]
        def start(plain=False,worker=False):
            c=Worker(a.member,state) if worker else Console(a.binary,state,'login',['--plain' if plain else '--ansi']);children.append(c)
            if worker:c.expect('[L]ogin [R]egister [Q]uit: ');c.send('l')
            c.expect('Handle: ');c.send('sysop');c.password(password)
            if not plain:c.expect('Press Enter to continue: ');c.send('')
            c.expect('Command (then Enter): ' if plain else MENU);return c
        def logout(c,plain=False):
            c.send('q') if plain else c.raw(b'q');c.expect('Logged out.');c.finish()
        try:
            c=Console(a.binary,state,'init');children.append(c);c.expect('Handle: ');c.send('sysop');c.password(password,'New password (12+ characters, hidden): ');c.password(password,'Confirm password (hidden): ');c.expect('Lobby created.');c.finish()
            c=start();edit_save(c);assert count()==0 and query('select body from room_drafts')==[('bright first\ntail',)];checks.append('full_screen_caret_paragraph_preview_and_explicit_checkpoint')
            c.close();children.remove(c);c=start();resume_send(c,count);logout(c);assert query('select count(*) from room_drafts')==[(0,)];checks.append('disconnect_resume_preserves_subject_and_send_confirmation_cleans_checkpoint')
            c=start(True);begin(c,True);c.send('.screen');c.expect('Full-screen unavailable');c.expect(PROMPT);command(c,'Plain checkpoint.');command(c,'.save');c.send('.leave');c.expect('last explicit checkpoint kept');action_end(c,True);logout(c,True);checks.append('plain_writer_can_save_without_ansi')
            c=start(True);open_saved(c,True);command(c,'.preview');c.send('.cancel');c.expect('Draft cancelled');action_end(c,True);logout(c,True);assert b'\x1b' not in c.transcript and query('select count(*) from room_drafts')==[(0,)];checks.append('plain_resume_preview_and_explicit_cancel_delete_checkpoint')
            c=start(worker=True);begin(c);command(c,'Worker body.');enter_screen(c);c.raw(b'\x11');c.expect('Classic line editing; draft kept.');c.expect(PROMPT);command(c,'.save');c.send('.leave');c.expect('last explicit checkpoint kept');action_end(c);logout(c);checks.append('worker_full_screen_returns_to_classic_and_saves')
            c=start();open_saved(c);c.send('.cancel');c.expect('Draft cancelled');action_end(c);begin(c);command(c,'Resize keeps me.');enter_screen(c);c.resize(40,18);c.raw(b'\x1b[D');c.expect(FOOTER);c.resize(30,12);c.raw(b'\x1b[D');c.expect('Press Enter to resume line editing: ');c.send('');c.expect('Classic line editing; draft kept.');c.expect(PROMPT);command(c,'.save');c.send('.leave');c.expect('last explicit checkpoint kept');c.expect('Press Enter to return to menu: ');c.send('');c.expect('Command (then Enter): ');logout(c,True);assert query('select body from room_drafts')==[('Resize keeps me.',)],repr(query('select body from room_drafts'));checks.append('resize_reflows_then_falls_back_with_complete_draft_intact')
            c=start();open_saved(c);enter_screen(c);c.raw(b'\x1b[999~');c.expect('Press Enter to resume line editing: ');c.send('');c.expect('Classic line editing; draft kept.');c.expect(PROMPT);assert count()==1;c.send('.cancel');c.expect('Draft cancelled');action_end(c);logout(c);checks.append('unsupported_input_returns_to_classic_without_send_or_lost_checkpoint')
            assert query('pragma user_version')==[(9,)] and query('select count(*) from unread')==[(0,)]
            assert all(password.encode() not in c.transcript for c in children)
        finally:
            for c in children:c.close()
    d=dict(result='pass',count=len(checks),checks=checks,schema_version=9,capacity_claim=False)
    if a.output:a.output.write_text(json.dumps(d,indent=2,sort_keys=True)+'\n')
    print('Full-screen/saved-draft local/worker checks passed:',len(checks))

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('binary',type=Path);p.add_argument('member',type=Path);p.add_argument('--output',type=Path);a=p.parse_args();a.binary=a.binary.resolve();a.member=a.member.resolve();run(a)
