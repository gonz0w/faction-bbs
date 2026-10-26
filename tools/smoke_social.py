#!/usr/bin/env python3
"""Real simultaneous local/worker opt-in lounge and bodyless page journeys."""
import argparse,json,secrets,sqlite3,tempfile
from pathlib import Path
from smoke_local_console import Console
from smoke_member_worker import Worker
from smoke_room_browser import MENU
def run(a):
    checks=[];children=[];password=secrets.token_hex(20)
    with tempfile.TemporaryDirectory(prefix='faction-social-pty-') as name:
        root=Path(name);state=root/'board'
        def query(sql):
            with sqlite3.connect(f'file:{state}/faction.sqlite3?mode=ro',uri=True) as d:return d.execute(sql).fetchall()
        def write(sql):
            with sqlite3.connect(state/'faction.sqlite3') as d:d.execute(sql)
        def start(handle='sysop',plain=False,worker=False):
            c=Worker(a.member,state,columns=40,rows=18) if worker else Console(a.binary,state,'login',['--plain' if plain else '--ansi'],columns=40,rows=18);children.append(c)
            if worker:c.expect('[L]ogin [R]egister [Q]uit: ');c.send('l')
            c.expect('Handle: ');c.send(handle);c.password(password)
            if handle=='pending':return c
            c.expect('Command (then Enter): ' if plain else MENU);c.send('c') if plain else c.raw(b'c');c.expect('chat[#lobby]> ');return c
        def cmd(c,line,expect):mark=len(c.transcript);c.send(line);c.expect(expect);return c.transcript[mark:]
        def stop(c,plain=False):
            cmd(c,'/quit','Chat closed.')
            if not plain:c.expect('Press Enter to return to menu: ');c.send('')
            c.expect('Command (then Enter): ' if plain else MENU);c.send('q') if plain else c.raw(b'q');c.expect('Logged out.');c.finish()
        try:
            c=Console(a.binary,state,'init');children.append(c);c.expect('Handle: ');c.send('sysop');c.password(password,'New password (12+ characters, hidden): ');c.password(password,'Confirm password (hidden): ');c.expect('Lobby created.');c.finish()
            with sqlite3.connect(state/'faction.sqlite3') as d:
                for handle,status in [('alice','approved'),('bobby','approved'),('pending','pending')]:d.execute("INSERT INTO members(handle,password_hash,role,status) SELECT ?,password_hash,'member',? FROM members WHERE id=1",(handle,status))
            owner=start();cmd(owner,'/social','[disabled]');cmd(owner,'/social on','Social configuration updated.');cmd(owner,'/social name Original lounge','Social configuration updated.')
            alice=start('alice');bob=start('bobby',True);frame=cmd(bob,'/who','Not a delivery guarantee.');assert b'alice |' not in frame
            cmd(owner,'/create #studio private','Channel created.');cmd(owner,'/allow #studio alice','Channel grant updated; target must join again.');cmd(alice,'/join #studio','Joined; new messages start here.');alice.expect('chat[#studio]> ')
            cmd(alice,'/available Original public status','Availability shared');cmd(bob,'/available Ready for chat','Availability shared');frame=cmd(bob,'/who','Not a delivery guarantee.');assert b'alice | Chat | available' in frame and b'#studio' not in frame;checks.append('default_hidden_explicit_available_public_status_without_private_channel_disclosure')
            alice.raw(b'draft stays private');alice.expect('chat[#studio]> draft stays private');cmd(bob,'/page alice','Page attempted');alice.send('');alice.expect('alice> draft stays private');frame=cmd(alice,'/pages','Requests/replies expire');id=query("SELECT id FROM social_pages")[0][0];assert b'from bobby: pending' in frame;cmd(alice,f'/answer {id} accept','Page answered.');alice.expect('chat[#studio]> ');frame=cmd(bob,'/pages','Requests/replies expire');assert b'to alice: accepted' in frame;cmd(bob,'/page alice','page cooldown');checks.append('bodyless_invitation_accept_keeps_draft_and_selected_private_channel_and_bounds_attempts')
            cmd(alice,'/block bobby','Local block preference updated');frame=cmd(bob,'/who','Not a delivery guarantee.');assert b'alice |' not in frame;write('UPDATE social_cooldowns SET last_page=unixepoch()-31');cmd(bob,'/page alice','Page attempted');frame=cmd(bob,'/pages','Requests/replies expire');assert b'Page 1 ' not in frame
            cmd(alice,'/unblock bobby','Local block preference updated');cmd(alice,'/mute on','Page preference updated');frame=cmd(alice,'/privacy','Your blocked handles: ');assert b'Pages muted: true' in frame;cmd(alice,'/quiet 22:00 07:00','UTC quiet hours updated');cmd(alice,'/block unknown','Local block preference updated');stop(alice)
            alice=start('alice',True);frame=cmd(alice,'/privacy','Your blocked handles: unknown');assert b'Pages muted: true' in frame and b'22:00 to 07:00' in frame;frame=cmd(bob,'/who','Not a delivery guarantee.');assert b'alice |' not in frame;checks.append('mutual_block_suppresses_roster_and_pages_private_controls_survive_hidden_reconnect')
            cmd(alice,'/quiet off','UTC quiet hours updated');cmd(alice,'/mute off','Page preference updated');cmd(alice,'/available Back again','Availability shared')
            frame=cmd(alice,'/page-sysop','Not a delivery guarantee.');assert b'sysop [' not in frame;cmd(owner,'/available Operator available','Availability shared');frame=cmd(alice,'/page-sysop','Not a delivery guarantee.');assert b'sysop [available, sysop]' in frame
            cmd(alice,'/page-sysop sysop','Page attempted');id=query("SELECT id FROM social_pages WHERE state='pending'")[0][0];cmd(owner,f'/answer {id} decline','Page answered.');frame=cmd(alice,'/pages','Requests/replies expire');assert b'to sysop: declined' in frame;checks.append('page_sysop_requires_operator_optin_and_explicit_decline_never_grants_os_authority')
            write('UPDATE social_pages SET created=unixepoch()-61');frame=cmd(alice,'/pages','Requests/replies expire');assert b'to sysop: declined' not in frame;cmd(alice,'/hide','Lounge availability hidden');stop(alice,True);assert b'\x1b' not in alice.transcript;stop(bob,True);assert b'\x1b' not in bob.transcript;checks.append('expired_signal_hidden_cleanup_and_plain_commands_emit_no_escapes')
            worker=start('alice',worker=True);cmd(worker,'/available Worker presence','Availability shared');frame=cmd(owner,'/who','Not a delivery guarantee.');assert b'alice | Chat | available' in frame;stop(worker);frame=cmd(owner,'/who','Not a delivery guarantee.');assert b'alice |' not in frame;checks.append('inherited_worker_reuses_presenter_and_logout_removes_advertisement')
            cmd(owner,'/social off','Social configuration updated.');cmd(owner,'/who','social lounge disabled by sysop');stop(owner);checks.append('sysop_name_and_disable_controls_remove_live_advertisements')
            c=start('pending');c.expect('No community access.');c.finish();assert query('PRAGMA user_version')==[(19,)] and query('SELECT count(*) FROM unread')==[(0,)] and query('SELECT count(*) FROM file_transfers')==[(0,)] and all(password.encode() not in c.transcript for c in children);checks.append('pending_denied_schema_18_no_room_transfer_or_password_echo_side_effects')
        finally:
            for c in children:c.close()
    result=dict(result='pass',count=len(checks),checks=checks,schema_version=19,synthetic_clock_aging=True,capacity_claim=False,private_message_bodies=False,boardwide_presence=False,automatic_notifications=False)
    if a.output:a.output.write_text(json.dumps(result,indent=2,sort_keys=True)+'\n')
    print('Local/worker social lounge passed:',len(checks),'controls.')
if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('binary',type=Path);p.add_argument('member',type=Path);p.add_argument('--output',type=Path);a=p.parse_args();a.binary=a.binary.resolve();a.member=a.member.resolve();run(a)
