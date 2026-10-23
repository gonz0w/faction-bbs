#!/usr/bin/env python3
"""Named-room local/worker PTYs on private synthetic state; no network service."""
import argparse,json,secrets,tempfile
from pathlib import Path
from smoke_local_console import Console
from smoke_member_worker import Worker

MENU='Ctrl-C exits; Esc: Enter to recover.'
BROWSE='N/P:page U:next unread Q:main'
CARD='N/Enter:new H:history P:post'

def run(a):
    checks=[];children=[];password=secrets.token_hex(20)
    with tempfile.TemporaryDirectory(prefix='faction-room-browser-') as tmp:
        state=Path(tmp)/'board'
        def start(command='login',handle='sysop',extra=(),columns=80,rows=24):
            c=Console(a.binary,state,command,extra,columns=columns,rows=rows);children.append(c);c.expect('Handle: ');c.send(handle);return c
        def login(mode='ansi',handle='sysop'):
            c=start(handle=handle,extra=['--plain' if mode=='plain' else '--ansi'],columns=40 if mode=='compact' else 80,rows=18 if mode=='compact' else 24);c.password(password)
            if mode=='plain':c.expect('Command (then Enter): ')
            elif mode=='compact':c.expect(MENU)
            else:c.expect('Press Enter to continue: ');c.send('');c.expect(MENU)
            return c
        def pause(c):c.expect('Press Enter to return to menu: ');c.send('');c.expect(MENU)
        def logout(c,plain=False):
            c.send('q') if plain else c.raw(b'q');c.expect('Logged out.');c.finish()
        def setup(command,handle):
            c=start(command,handle);c.password(password,'New password (12+ characters, hidden): ');c.password(password,'Confirm password (hidden): ');c.expect('Lobby created.' if command=='init' else 'Registered pending sysop approval.');c.finish()
        def card(c,index=2):
            c.raw(b'r');c.expect(BROWSE);c.raw(bytes([ord('a')+index-1]));c.expect(f'ROOM #{index}');c.expect(CARD)
        def post(c,body):
            mark=len(c.transcript);card(c);c.raw(b'p');c.expect('Post in room #2');c.expect('Reply to post ID (blank for new message): ');assert b'Room ID: ' not in c.transcript[mark:];c.send('');c.expect('Subject (blank for generated/reply default): ');c.send('Night frequency');c.expect('Message (one line, up to 1024 bytes): ');c.send(body);c.expect('Posted #');pause(c)
        def admin(action):
            c=login('plain');c.send('s');c.expect('[C]reate room: ');c.send(action);c.expect('Member handle: ');c.send('member');c.expect('Status updated; previous sessions expired.');c.expect('Command (then Enter): ');logout(c,True)
        try:
            setup('init','sysop');setup('register','member')
            c=start(handle='member',extra=['--ansi']);c.password(password);c.expect('No community access.');c.finish();assert b'ROOMS /' not in c.transcript;checks.append('pending_member_cannot_enter_room_browser')
            c=login('plain')
            for i in range(2,10):
                c.send('s');c.expect('[C]reate room: ');c.send('c');c.expect('Room slug: ');c.send('radio' if i==2 else 'room-'+str(i));c.expect('Room title: ');c.send('Night Radio' if i==2 else 'Room '+str(i));c.expect(f'Created room #{i}.');c.expect('Command (then Enter): ')
            logout(c,True)
            c=login();c.raw(b'r');c.expect('Page 1 / 3');c.expect(BROWSE);c.raw(b'\x1b[B');c.expect('> [B] #2');c.raw(b'\r');c.expect('ROOM #2');c.expect('Night Radio');c.expect(CARD);c.raw(b'l');c.expect(BROWSE);c.raw(b'n');c.expect('Page 2 / 3');c.raw(b'a');c.expect('ROOM #5');c.expect(CARD);c.raw(b'q');c.expect(MENU);checks.append('named_arrows_enter_hotkeys_cards_and_page_return')
            post(c,'A signal from the selected room.');checks.append('selected_room_post_skips_numeric_room_prompt')
            c.raw(b'r');c.expect('#2 (1 unread)');c.expect(BROWSE);c.raw(b'u');c.expect('> [B] #2');c.raw(b'\r');c.expect(CARD);c.raw(b'n');c.expect('Room #2 | chronological messages');c.expect('A signal from the selected room.');c.expect('More? [y/N]: ');c.send('n');pause(c);c.raw(b'r');c.expect('#2 (0 unread)');c.expect(BROWSE);c.raw(b'q');c.expect(MENU);checks.append('next_unread_and_selected_new_reader_acknowledge_completed_post')
            card(c);c.raw(b'h');c.expect('Room #2 | chronological messages');c.expect('Night frequency');c.expect('More? [y/N]: ');c.send('n');pause(c);checks.append('selected_history_uses_existing_chronological_reader')
            post(c,' '.join('word%03d'%i for i in range(100))+' END-MARKER');logout(c)
            c=login('compact');card(c);c.raw(b'n');c.expect('More? [y/N]: ');c.send('n');pause(c);c.raw(b'r');c.expect('#2 (1 unread)');c.expect(BROWSE);c.raw(b'q');c.expect(MENU);logout(c);checks.append('compact_partial_message_retains_unread_state')
            c=login('plain');c.send('r');c.expect('radio - Night Radio');c.expect('Command (then Enter): ');logout(c,True);assert b'\x1b' not in c.transcript;checks.append('plain_numeric_room_workflow_remains_escape_free')
            c=Worker(a.member,state);children.append(c);c.expect('[L]ogin [R]egister [Q]uit: ');c.send('l');c.expect('Handle: ');c.send('sysop');c.password(password);c.expect('Press Enter to continue: ');c.send('');c.expect(MENU);card(c);c.raw(b'q');c.expect(MENU);logout(c);checks.append('inherited_terminal_worker_uses_same_room_presenter')
            c=login();c.raw(b'r');c.expect(BROWSE);c.resize(40,18);c.expect(MENU);logout(c);checks.append('browser_resize_returns_to_working_main_menu')
            admin('a');c=login(handle='member');card(c);admin('v');mark=len(c.transcript);c.raw(b'p');c.finish(1);assert b'Reply to post ID' not in c.transcript[mark:];checks.append('revocation_prevents_selected_room_compose')
            assert all(password.encode() not in c.transcript for c in children)
        finally:
            for c in children:c.close()
    result=dict(result='pass',count=len(checks),checks=checks,capacity_claim=False,federation_claim=False)
    if a.output:a.output.write_text(json.dumps(result,indent=2,sort_keys=True)+'\n')
    print(f'Local/worker room browser passed {len(checks)} controls.')

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('binary',type=Path);p.add_argument('member',type=Path);p.add_argument('--output',type=Path);a=p.parse_args();a.binary=a.binary.resolve();a.member=a.member.resolve();run(a)
