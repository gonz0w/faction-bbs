#!/usr/bin/env python3
"""Private synthetic local/worker poll journeys through actual PTYs."""
import argparse,json,secrets,sqlite3,tempfile
from pathlib import Path
from smoke_local_console import Console
from smoke_member_worker import Worker
from smoke_room_browser import MENU
SETTINGS='Poll settings> '
LIST='Polls> '
CARD='Poll> '
TEXT='Poll text> '
REVIEW='Poll review> '
def settings(c,plain=False):
    c.send('s') if plain else c.raw(b's');c.expect('[C]reate room: ');c.send('p');c.expect(SETTINGS)
def enter(c,plain=False):
    c.send('v') if plain else c.raw(b'v');c.expect(LIST)
def view(c,poll=1):
    c.send('v');c.expect('Poll ID (blank cancels): ');c.send(str(poll));c.expect(CARD)
def create(c,mode='v',question='Which night for ANSI club?'):
    c.send('c');c.expect('Question (1-96 ASCII, blank keeps): ');c.send(question)
    for option in ['Friday','Saturday','Sunday','.']:
        c.expect("Choice (1-48 ASCII), '.' finishes: ");c.send(option)
    c.expect('Results A:always V:after vote C:after close (blank keeps): ');c.send(mode);c.expect(TEXT);c.send('Bring your terminal.');c.expect(TEXT);c.send('.');c.expect(REVIEW)
def vote(c,choice=1,save=True):
    c.send('v');c.expect('Choice number (0 withdraws): ');c.send(str(choice));c.expect('Ballot review> ');c.send('s' if save else 'c');c.expect(CARD)
def leave(c,plain=False):
    c.send('q')
    if not plain:c.expect('Press Enter to return to menu: ');c.send('')
    c.expect('Command (then Enter): ' if plain else MENU)
def run(a):
    children=[];checks=[];password=secrets.token_hex(20)
    with tempfile.TemporaryDirectory(prefix='faction-polls-pty-') as directory:
        state=Path(directory)/'board'
        def query(sql):
            with sqlite3.connect(f'file:{state}/faction.sqlite3?mode=ro',uri=True) as db:return db.execute(sql).fetchall()
        def start(plain=False,worker=False,compact=False):
            c=Worker(a.member,state,columns=40 if compact else 80,rows=18 if compact else 24) if worker else Console(a.binary,state,'login',['--plain' if plain else '--ansi'],columns=40 if compact else 80,rows=18 if compact else 24);children.append(c)
            if worker:c.expect('[L]ogin [R]egister [Q]uit: ');c.send('l')
            c.expect('Handle: ');c.send('sysop');c.password(password)
            if not plain and not compact:c.expect('Press Enter to continue: ');c.send('')
            c.expect('Command (then Enter): ' if plain else MENU);return c
        def quit(c,plain=False):c.send('q') if plain else c.raw(b'q');c.expect('Logged out.');c.finish()
        def to_main(c,plain=False):c.send('q');c.expect(LIST);leave(c,plain)
        try:
            c=Console(a.binary,state,'init');children.append(c);c.expect('Handle: ');c.send('sysop');c.password(password,'New password (12+ characters, hidden): ');c.password(password,'Confirm password (hidden): ');c.expect('Lobby created.');c.finish()
            c=start();assert query('select enabled from poll_settings')==[(0,)];settings(c);c.send('e');c.expect(SETTINGS);c.send('n');c.expect('Poll label (1-24 ASCII): ');c.send('Nightwire questions');c.expect(SETTINGS);c.send('o');c.expect(SETTINGS);leave(c)
            enter(c);create(c);assert query('select count(*) from polls')==[(0,)];c.send('d');c.expect('Bring your terminal.');c.expect('Poll note preview> ');c.send('q');c.expect(REVIEW);c.send('s');c.expect('Poll saved locally.');c.expect(CARD);assert query('select count(*) from polls')==[(1,)];checks.append('ansi_policy_question_choices_notes_preview_and_explicit_save')
            vote(c,1,False);assert query('select count(*) from poll_ballots')==[(0,)];vote(c,1);vote(c,1);assert query('select count(*) from poll_ballots')==[(1,)];vote(c,2);assert query('select choice from poll_ballots')==[(2,)];vote(c,0);assert query('select count(*) from poll_ballots')==[(0,)];c.send('e');c.expect('choose a displayed poll command');c.expect(CARD);assert query('select ever_voted from polls')==[(1,)];checks.append('explicit_ballot_cancel_repeat_change_withdraw_and_permanent_wording_lock')
            to_main(c);quit(c);c=start(True);enter(c,True);view(c);vote(c,1)
            to_main(c,True);quit(c,True);assert b'\x1b' not in c.transcript;checks.append('plain_vote_and_text_cell_results_emit_no_ansi')
            c=start(worker=True);enter(c);view(c);c.send('n');c.expect('3: Sunday');c.expect(CARD);c.send('n');c.expect('3: Sunday');c.expect(CARD);c.send('p');c.expect('1: Friday');c.expect(CARD);c.send('d');c.expect('Bring your terminal.');c.expect('Poll notes> ');c.send('q');c.expect(CARD);to_main(c);quit(c);checks.append('worker_paged_choices_notes_and_return_use_same_workflow')
            c=start(compact=True);enter(c);create(c,'c','Next club theme?');c.send('s');c.expect(CARD);vote(c,2);assert query('select count(*) from poll_ballots where poll=2')==[(1,)];c.send('f');c.expect('Type CLOSE (anything else returns): ');c.send('no');c.expect(CARD);assert query('select state from polls where id=2')==[(0,)];c.send('f');c.expect('Type CLOSE (anything else returns): ');c.send('CLOSE');c.expect('Poll CLOSED.');c.expect(CARD);assert query('select state from polls where id=2')==[(1,)];c.send('n');c.expect('2: Saturday');c.expect('1/1');c.expect(CARD);to_main(c);quit(c);checks.append('compact_after_close_results_and_exact_close_confirmation')
            c=start();enter(c);view(c);c.send('c');c.expect('Type CANCEL (anything else returns): ');c.send('CANCEL');c.expect('Poll CANCELLED.');c.expect(CARD);assert query('select state from polls where id=1')==[(2,)];to_main(c);settings(c);c.send('m');c.expect('Poll ID and hide|show|remove: ');c.send('1 remove');c.expect('Type DELETE (anything else cancels): ');c.send('no');c.expect(SETTINGS);assert query('select count(*) from polls')==[(2,)];c.send('m');c.expect('Poll ID and hide|show|remove: ');c.send('1 remove');c.expect('Type DELETE (anything else cancels): ');c.send('DELETE');c.expect('Poll and ballots removed.');c.expect(SETTINGS);assert query('select count(*) from poll_ballots where poll=1')==[(0,)];c.send('e');c.expect(SETTINGS);leave(c);quit(c);checks.append('cancelled_poll_and_explicit_moderator_removal_cascade_ballots')
            c=Console(a.binary,state,'register');children.append(c);c.expect('Handle: ');c.send('pending');c.password(password,'New password (12+ characters, hidden): ');c.password(password,'Confirm password (hidden): ');c.expect('Registered pending sysop approval.');c.finish();c=Console(a.binary,state,'login',['--ansi']);children.append(c);c.expect('Handle: ');c.send('pending');c.password(password);c.expect('No community access.');c.finish();assert b'Polls>' not in c.transcript;checks.append('pending_member_cannot_enter_voting_desk')
            assert query('pragma user_version')==[(18,)] and query('select count(*) from unread')==[(0,)] and all(password.encode() not in c.transcript for c in children);checks.append('schema_sixteen_and_no_room_unread_or_secret_echo_side_effects')
        finally:
            for c in children:c.close()
    result=dict(result='pass',count=len(checks),checks=checks,schema_version=18,capacity_claim=False,secret_ballot_claim=False)
    if a.output:a.output.write_text(json.dumps(result,indent=2,sort_keys=True)+'\n')
    print('Local/worker polls passed:',len(checks),'controls.')
if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('binary',type=Path);p.add_argument('member',type=Path);p.add_argument('--output',type=Path);a=p.parse_args();a.binary=a.binary.resolve();a.member=a.member.resolve();run(a)
