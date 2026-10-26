#!/usr/bin/env python3
"""Local/worker discussion discovery and bookmark journeys on disposable data."""
import argparse,json,secrets,sqlite3,tempfile
from pathlib import Path
from smoke_local_console import Console
from smoke_member_worker import Worker
from smoke_room_browser import MENU
SETTINGS='Find settings> '
LIST='Find> '
SOURCE='Source> '
REVIEW='Bookmark review> '
def settings(c,plain=False):
    c.send('s') if plain else c.raw(b's');c.expect('[C]reate room: ');c.send('b');c.expect(SETTINGS)
def enter(c,plain=False):
    c.send('b') if plain else c.raw(b'b');c.expect(LIST)
def search(c,text,room='',author='',field='a'):
    c.send('f');c.expect('Literal text (1-64 UTF-8 bytes, /cancel): ');c.send(text);c.expect('Room ID (blank all, /cancel): ');c.send(str(room));c.expect('Exact author (blank all, /cancel): ');c.send(author);c.expect('Search S:subject B:body A:both: ');c.send(field);c.expect(LIST)
def view(c,post=1):c.send('j');c.expect('Open post ID (blank cancels): ');c.send(str(post));c.expect(SOURCE)
def bookmark(c,post=1,add=True,save=True):
    c.send('a' if add else 'd');c.expect('Bookmark post ID (blank cancels): ' if add else 'Remove post ID (blank cancels): ');c.send(str(post));c.expect(REVIEW);c.send('s' if save else 'c');c.expect(LIST)
def leave(c,plain=False):
    c.send('q')
    if not plain:c.expect('Press Enter to return to menu: ');c.send('')
    c.expect('Command (then Enter): ' if plain else MENU)
def run(a):
    checks=[];children=[];password=secrets.token_hex(20)
    with tempfile.TemporaryDirectory(prefix='faction-discussion-pty-') as directory:
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
        def post(c,subject,body):
            c.raw(b'p');c.expect('Room ID: ');c.send('1');c.expect('Reply to post ID (blank for new message): ');c.send('');c.expect('Subject (blank for generated/reply default): ');c.send(subject);c.expect('Message (one line, up to 1024 bytes): ');c.send(body);c.expect('Posted #');c.expect('Press Enter to return to menu: ');c.send('');c.expect(MENU)
        try:
            c=Console(a.binary,state,'init');children.append(c);c.expect('Handle: ');c.send('sysop');c.password(password,'New password (12+ characters, hidden): ');c.password(password,'Confirm password (hidden): ');c.expect('Lobby created.');c.finish()
            c=start();assert query('select enabled from discussion_settings')==[(0,)];post(c,'Café guide',' '.join('word%03d'%n for n in range(100))+' FINAL-FIND');post(c,'Night frequencies','Meet the underground.');settings(c);c.send('e');c.expect(SETTINGS);c.send('n');c.expect('Find label (1-24 ASCII): ');c.send('Nightwire archives');c.expect(SETTINGS);leave(c);enter(c);checks.append('optional_named_find_policy_preserves_existing_login_and_menu')
            search(c,'guide',1,'sysop','s');assert b'Caf\\u{e9} guide' in c.transcript;search(c,'underground',field='b');assert b'Night frequencies' in c.transcript and query('select count(*) from unread')==[(0,)];checks.append('literal_subject_body_room_author_filters_and_previews_leave_unread_alone')
            bookmark(c,1,save=False);assert query('select count(*) from discussion_bookmarks')==[(0,)];bookmark(c,1);bookmark(c,1);assert query('select post from discussion_bookmarks')==[(1,)];c.send('b');c.expect('Your bookmarks');c.expect(LIST);c.send('x');c.expect('FACTION-BOOKMARKS/1');c.expect(LIST);assert query('select count(*) from unread')==[(0,)];checks.append('explicit_bookmark_review_cancel_idempotent_save_and_copyable_export')
            leave(c);quit(c);c=start(compact=True);enter(c);view(c,1);complete=b'FINAL-FIND' in c.transcript
            for _ in range(30):
                if complete:break
                c.send('n');c.expect(SOURCE);complete=b'FINAL-FIND' in c.transcript
            assert complete and query('select count(*) from unread')==[(0,)];c.send('p');c.expect(SOURCE);c.send('q');c.expect(LIST);leave(c);quit(c);checks.append('compact_exact_source_paging_is_readonly_even_after_complete_rendering')
            c=start(True);enter(c,True);search(c,'Café',1,'sysop','s');c.send('b');c.expect(LIST);leave(c,True);quit(c,True);assert b'\x1b' not in c.transcript;checks.append('plain_utf8_literal_query_and_inert_subjects_emit_no_ansi')
            c=start(worker=True);enter(c);c.send('b');c.expect(LIST);bookmark(c,1,False);assert query('select count(*) from discussion_bookmarks')==[(0,)];leave(c);quit(c);checks.append('worker_uses_same_member_bookmarks_and_explicit_removal')
            c=start();settings(c);c.send('l');c.expect(SETTINGS);leave(c);enter(c);c.send('f');c.expect('Literal text (1-64 UTF-8 bytes, /cancel): ');c.send('/cancel');c.expect(LIST);search(c,'missing needle');assert b'No matching items in this scan.' in c.transcript;leave(c);settings(c);c.send('e');c.expect(SETTINGS);leave(c);quit(c);checks.append('compact_cards_cancel_empty_scan_and_disabled_menu')
            c=Console(a.binary,state,'register');children.append(c);c.expect('Handle: ');c.send('pending');c.password(password,'New password (12+ characters, hidden): ');c.password(password,'Confirm password (hidden): ');c.expect('Registered pending sysop approval.');c.finish();c=Console(a.binary,state,'login',['--ansi']);children.append(c);c.expect('Handle: ');c.send('pending');c.password(password);c.expect('No community access.');c.finish();assert b'Find>' not in c.transcript and query('pragma user_version')==[(20,)] and all(password.encode() not in c.transcript for c in children);checks.append('pending_member_denied_schema_sixteen_and_no_secret_echo')
        finally:
            for c in children:c.close()
    result=dict(result='pass',count=len(checks),checks=checks,schema_version=20,capacity_claim=False,server_mail_search=False,automatic_query_retention=False)
    if a.output:a.output.write_text(json.dumps(result,indent=2,sort_keys=True)+'\n')
    print('Local/worker discussion discovery passed:',len(checks),'controls.')
if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('binary',type=Path);p.add_argument('member',type=Path);p.add_argument('--output',type=Path);a=p.parse_args();a.binary=a.binary.resolve();a.member=a.member.resolve();run(a)
