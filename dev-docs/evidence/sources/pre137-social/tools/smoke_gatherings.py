#!/usr/bin/env python3
"""Real local/worker gathering journeys in an owned private synthetic board."""
import argparse
import json
from pathlib import Path
import secrets
import sqlite3
import tempfile
from smoke_local_console import Console
from smoke_member_worker import Worker
from smoke_room_browser import MENU

SETTINGS='Gathering settings> '
LIST='Gatherings> '
CARD='Gathering> '
TEXT='Gathering text> '
REVIEW='Gathering review> '


def settings(c,plain=False):
    c.send('s') if plain else c.raw(b's');c.expect('[C]reate room: ');c.send('g');c.expect(SETTINGS)
def enter(c,plain=False):
    c.send('g') if plain else c.raw(b'g');c.expect(LIST)
def view(c,event=1):
    c.send('v');c.expect('Gathering ID (blank cancels): ');c.send(str(event));c.expect(CARD)
def create(c,when,place='room:1',title='Afterhours terminal night'):
    c.send('c');c.expect('Title (1-64 ASCII, blank keeps): ');c.send(title)
    c.expect('Start UTC YYYY-MM-DD HH:MM (blank keeps): ');c.send(when)
    c.expect('Duration 1-1440 minutes (blank keeps): ');c.send('120')
    c.expect("Place: room:ID, #channel, '-' none, blank keeps: ");c.send(place)
    c.expect(TEXT);c.send('Bring your ANSI art.');c.expect(TEXT);c.send('.');c.expect(REVIEW)
def rsvp(c,response=1,save=True):
    c.send('r');c.expect('RSVP: 1 going / 2 maybe / 0 clear: ');c.send(str(response));c.expect('RSVP review> ')
    c.send('s' if save else 'c');c.expect(CARD)
def leave(c,plain=False):
    c.send('q')
    if not plain:c.expect('Press Enter to return to menu: ');c.send('')
    c.expect('Command (then Enter): ' if plain else MENU)


def run(a):
    checks=[];children=[];password=secrets.token_hex(20)
    with tempfile.TemporaryDirectory(prefix='faction-gathering-pty-') as directory:
        state=Path(directory)/'board'
        def query(sql):
            with sqlite3.connect(f'file:{state}/faction.sqlite3?mode=ro',uri=True) as db:return db.execute(sql).fetchall()
        def start(plain=False,worker=False,compact=False,notice=False):
            c=Worker(a.member,state,columns=40 if compact else 80,rows=18 if compact else 24) if worker else Console(a.binary,state,'login',['--plain' if plain else '--ansi'],columns=40 if compact else 80,rows=18 if compact else 24)
            children.append(c)
            if worker:c.expect('[L]ogin [R]egister [Q]uit: ');c.send('l')
            c.expect('Handle: ');c.send('sysop');c.password(password)
            if notice:c.expect('Open Gatherings from the menu.')
            if notice or (not plain and not compact):c.expect('Press Enter to continue: ');c.send('')
            c.expect('Command (then Enter): ' if plain else MENU);return c
        def quit(c,plain=False):
            c.send('q') if plain else c.raw(b'q');c.expect('Logged out.');c.finish()
        try:
            c=Console(a.binary,state,'init');children.append(c);c.expect('Handle: ');c.send('sysop');c.password(password,'New password (12+ characters, hidden): ');c.password(password,'Confirm password (hidden): ');c.expect('Lobby created.');c.finish()
            c=start();assert query('select enabled from gathering_settings')==[(0,)]
            c.raw(b'p');c.expect('Room ID: ');c.send('1');c.expect('Reply to post ID (blank for new message): ');c.send('');c.expect('Subject (blank for generated/reply default): ');c.send('Gathering room');c.expect('Message (one line, up to 1024 bytes): ');c.send('Meet us here.');c.expect('Posted #');c.expect('Press Enter to return to menu: ');c.send('');c.expect(MENU)
            settings(c);c.send('e');c.expect(SETTINGS);c.send('n');c.expect('Gathering label (1-24 ASCII): ');c.send('Afterhours agenda');c.expect(SETTINGS);c.send('o');c.expect(SETTINGS);c.send('r');c.expect(SETTINGS);leave(c)
            when=query("select strftime('%Y-%m-%d %H:%M','now','+1 hour')")[0][0]
            enter(c);create(c,when);assert query('select count(*) from gatherings')==[(0,)]
            c.send('s');c.expect('Gathering saved locally.');c.expect(CARD);assert query('select count(*) from gatherings')==[(1,)]
            checks.append('ansi_settings_named_event_utc_notes_preview_and_explicit_save')
            rsvp(c,1,False);assert query('select count(*) from gathering_rsvps')==[(0,)]
            rsvp(c,1);rsvp(c,1);assert query('select count(*) from gathering_rsvps')==[(1,)]
            rsvp(c,2);assert query('select response from gathering_rsvps')==[(2,)]
            assert query('select count(*) from unread')==[(0,)]
            checks.append('rsvp_explicit_cancel_single_repeat_change_and_no_unread_side_effect')
            c.send('q');c.expect(LIST);leave(c);quit(c)
            c=start(True,notice=True);enter(c,True);view(c);rsvp(c,0);assert query('select count(*) from gathering_rsvps')==[(0,)]
            c.send('q');c.expect(LIST);leave(c,True);quit(c,True);assert b'\x1b' not in c.transcript
            checks.append('plain_login_reminder_view_and_response_clear_without_ansi')
            c=start(worker=True,notice=True);enter(c);view(c);c.send('j');c.expect('Room #1 | chronological messages');c.expect('Meet us here.');c.expect('More? [y/N]: ');c.send('n');c.expect('Press Enter to return to menu: ');c.send('');c.expect(MENU);quit(c)
            checks.append('worker_approved_login_reminder_and_deliberate_local_room_jump')
            c=start(compact=True,notice=True);enter(c);view(c);c.send('e');c.expect('Title (1-64 ASCII, blank keeps): ');c.send('Compact event edit');c.expect('Start UTC YYYY-MM-DD HH:MM (blank keeps): ');c.send('');c.expect('Duration 1-1440 minutes (blank keeps): ');c.send('');c.expect("Place: room:ID, #channel, '-' none, blank keeps: ");c.send('#lobby');c.expect(TEXT);c.send('.');c.expect(REVIEW);c.send('s');c.expect(CARD);c.send('j');c.expect('chat[#lobby]> ');c.send('/quit');c.expect('Press Enter to return to menu: ');c.send('');c.expect(MENU);quit(c)
            checks.append('compact_edit_and_explicit_local_channel_jump_return')
            c=start(notice=True);enter(c);view(c);c.send('c');c.expect('Type CANCEL (anything else returns): ');c.send('no');c.expect(CARD);assert query('select cancelled from gatherings')==[(0,)]
            c.send('c');c.expect('Type CANCEL (anything else returns): ');c.send('CANCEL');c.expect('Gathering cancelled');c.expect(CARD);c.send('j');c.expect('event is cancelled, hidden or ended');c.expect(CARD);c.send('q');c.expect(LIST);leave(c);quit(c)
            checks.append('cancel_requires_exact_confirmation_and_prevents_jump')
            c=start();settings(c);c.send('m');c.expect('Gathering ID and hide|show|remove: ');c.send('1 remove');c.expect('Type DELETE (anything else cancels): ');c.send('no');c.expect(SETTINGS);assert query('select count(*) from gatherings')==[(1,)]
            c.send('m');c.expect('Gathering ID and hide|show|remove: ');c.send('1 remove');c.expect('Type DELETE (anything else cancels): ');c.send('DELETE');c.expect('Gathering and all responses removed.');c.expect(SETTINGS);assert query('select count(*) from gatherings')==[(0,)]
            c.send('e');c.expect(SETTINGS);leave(c);quit(c)
            checks.append('sysop_stable_id_remove_requires_delete_and_feature_can_disable')
            c=Console(a.binary,state,'register');children.append(c);c.expect('Handle: ');c.send('pending');c.password(password,'New password (12+ characters, hidden): ');c.password(password,'Confirm password (hidden): ');c.expect('Registered pending sysop approval.');c.finish()
            c=Console(a.binary,state,'login',['--ansi']);children.append(c);c.expect('Handle: ');c.send('pending');c.password(password);c.expect('No community access.');c.finish();assert b'Gatherings>' not in c.transcript
            assert query('pragma user_version')==[(16,)] and all(password.encode() not in c.transcript for c in children)
            checks.append('pending_member_never_sees_gatherings_or_login_reminder')
        finally:
            for c in children:c.close()
    result=dict(result='pass',count=len(checks),checks=checks,schema_version=16,capacity_claim=False,external_calendar=False)
    if a.output:a.output.write_text(json.dumps(result,indent=2,sort_keys=True)+'\n')
    print('Local/worker gatherings passed:',len(checks),'controls.')
if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('binary',type=Path);p.add_argument('member',type=Path);p.add_argument('--output',type=Path);a=p.parse_args();a.binary=a.binary.resolve();a.member=a.member.resolve();run(a)
