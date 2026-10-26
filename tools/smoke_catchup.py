#!/usr/bin/env python3
"""Private synthetic catch-up journeys through local and worker PTYs."""
import argparse,json,secrets,sqlite3,tempfile
from pathlib import Path
from smoke_local_console import Console
from smoke_member_worker import Worker
from smoke_room_browser import MENU
SETTINGS='Catch-up settings> '
LIST='Catch-up> '

def settings(c,plain=False):
    c.send('s') if plain else c.raw(b's');c.expect('[C]reate room: ');c.send('u');c.expect(SETTINGS)
def enter(c,plain=False):
    c.send('u') if plain else c.raw(b'u');c.expect(LIST)
def section(c,name):c.send('m '+name);c.expect('Section: '+name);c.expect(LIST)
def jump(c,key):c.send('j');c.expect('Source ID (blank cancels): ');c.send(str(key))
def leave(c,plain=False):
    c.send('q')
    if not plain:c.expect('Press Enter to return to menu: ');c.send('')
    c.expect('Command (then Enter): ' if plain else MENU)
def run(a):
    checks=[];children=[];password=secrets.token_hex(20)
    with tempfile.TemporaryDirectory(prefix='faction-catchup-pty-') as tmp:
        state=Path(tmp)/'board'
        def query(sql):
            with sqlite3.connect(f'file:{state}/faction.sqlite3?mode=ro',uri=True) as db:return db.execute(sql).fetchall()
        def start(plain=False,worker=False,compact=False):
            c=Worker(a.member,state,columns=40 if compact else 80,rows=18 if compact else 24) if worker else Console(a.binary,state,'login',['--plain' if plain else '--ansi'],columns=40 if compact else 80,rows=18 if compact else 24);children.append(c)
            if worker:c.expect('[L]ogin [R]egister [Q]uit: ');c.send('l')
            c.expect('Handle: ');c.send('sysop');c.password(password)
            if not plain and not compact:c.expect('Press Enter to continue: ');c.send('')
            c.expect('Command (then Enter): ' if plain else MENU);return c
        def quit(c,plain=False):c.send('q') if plain else c.raw(b'q');c.expect('Logged out.');c.finish()
        try:
            c=Console(a.binary,state,'init');children.append(c);c.expect('Handle: ');c.send('sysop');c.password(password,'New password (12+ characters, hidden): ');c.password(password,'Confirm password (hidden): ');c.expect('Lobby created.');c.finish()
            c=start();assert query('select enabled from catchup_settings')==[(0,)] and query('select count(*) from catchup_members')==[(0,)]
            c.raw(b'p');c.expect('Room ID: ');c.send('1');c.expect('Reply to post ID (blank for new message): ');c.send('');c.expect('Subject (blank for generated/reply default): ');c.send('Welcome back');c.expect('Message (one line, up to 1024 bytes): ');c.send('Catch up with the underground.');c.expect('Posted #');c.expect('Press Enter to return to menu: ');c.send('');c.expect(MENU)
            settings(c);c.send('e');c.expect(SETTINGS);c.send('n');c.expect('Catch-up label (1-24 ASCII): ');c.send('Nightwire catchup');c.expect(SETTINGS);c.send('a');c.expect('Announcement text> ');c.send('Friday is ANSI night.');c.expect('Announcement text> ');c.send('.');c.expect('Announcement review> ');assert query('select notice from catchup_settings')==[('',)];c.send('s');c.expect(SETTINGS);leave(c)
            enter(c);assert b'Friday is ANSI night.' in c.transcript;assert query('select count(*) from catchup_members')==[(0,)];checks.append('optional_named_policy_and_notice_explicit_save_without_login_tracking')
            section(c,'rooms');assert b'1 unread' in c.transcript;section(c,'replies');assert query('select count(*) from unread')==[(0,)];c.send('c');c.expect('Visit review> ');c.send('c');c.expect(LIST);assert query('select count(*) from catchup_members')==[(0,)];c.send('c');c.expect('Visit review> ');c.send('s');c.expect('Visit saved; room unread unchanged.');c.expect(LIST);assert query('select count(*) from catchup_members where saved_at>0')==[(1,)] and query('select count(*) from unread')==[(0,)];checks.append('room_reply_previews_and_cancel_or_explicit_checkpoint_leave_unread_alone')
            c.send('s');c.expect('Subscriptions> ');c.send('+1');c.expect('Subscriptions> ');c.send('none');c.expect('Subscriptions> ');c.send('q');c.expect(LIST);section(c,'rooms');assert b'No eligible items on this page.' in c.transcript;c.send('s');c.expect('Subscriptions> ');c.send('+1');c.expect('Subscriptions> ');c.send('all');c.expect('Subscriptions> ');c.send('q');c.expect(LIST);assert query('select room from catchup_favorites')==[(1,)];checks.append('explicit_favorites_none_and_all_preferences_persist_locally')
            section(c,'files');assert b'Files unavailable in this session.' in c.transcript;section(c,'rooms');jump(c,1);c.expect('Catch up with the underground.');c.expect('More? [y/N]: ');c.send('n');c.expect('Press Enter to return to menu: ');c.send('');c.expect(MENU);assert query('select last_read from unread')==[(1,)];quit(c);checks.append('file_absence_and_authorized_room_jump_reuse_completed_read_ack')
            c=start(True);enter(c,True);section(c,'rooms');c.send('s');c.expect('Subscriptions> ');c.send('fav');c.expect('Subscriptions> ');c.send('q');c.expect(LIST);leave(c,True);quit(c,True);assert b'\x1b' not in c.transcript;checks.append('plain_browsing_and_subscription_controls_emit_no_ansi')
            c=start(worker=True);enter(c);section(c,'rooms');assert b'0 unread' in c.transcript;leave(c);quit(c);checks.append('worker_uses_same_catchup_preferences_and_read_ledger')
            c=start(compact=True);settings(c);c.send('l');c.expect(SETTINGS);c.send('o');c.expect("Comma-separated keys, '-' disables all: ");c.send('rooms,notice');c.expect(SETTINGS);leave(c);enter(c);assert b'Section: rooms' in c.transcript;c.send('m files');c.expect('section is disabled');c.expect(LIST);leave(c);settings(c);c.send('e');c.expect(SETTINGS);leave(c);quit(c);checks.append('compact_layout_order_disabled_module_and_host_optional_menu')
            c=Console(a.binary,state,'register');children.append(c);c.expect('Handle: ');c.send('pending');c.password(password,'New password (12+ characters, hidden): ');c.password(password,'Confirm password (hidden): ');c.expect('Registered pending sysop approval.');c.finish();c=Console(a.binary,state,'login',['--ansi']);children.append(c);c.expect('Handle: ');c.send('pending');c.password(password);c.expect('No community access.');c.finish();assert b'Catch-up>' not in c.transcript and query('pragma user_version')==[(20,)] and all(password.encode() not in c.transcript for c in children);checks.append('pending_member_denied_schema_sixteen_and_no_secret_echo')
        finally:
            for c in children:c.close()
    result=dict(result='pass',count=len(checks),checks=checks,schema_version=20,capacity_claim=False,automatic_login_tracking=False,server_mail_decryption=False)
    if a.output:a.output.write_text(json.dumps(result,indent=2,sort_keys=True)+'\n')
    print('Local/worker catch-up passed:',len(checks),'controls.')
if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('binary',type=Path);p.add_argument('member',type=Path);p.add_argument('--output',type=Path);a=p.parse_args();a.binary=a.binary.resolve();a.member=a.member.resolve();run(a)
