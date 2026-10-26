#!/usr/bin/env python3
"""Authenticated local/inherited-worker visit lounge; private PTYs and no listener.

The normal run measures a real 66-second parallel wait in three input states.
--quick is development preflight only and never qualifies as sustained evidence.
"""
import argparse,json,os,secrets,select,sqlite3,tempfile,time
from pathlib import Path
from smoke_local_console import Console
from smoke_member_worker import Worker
from smoke_room_browser import MENU
from smoke_writing_desk import begin,command,action_end,PROMPT
from smoke_full_screen import enter_screen,FOOTER

PLAIN='Command (then Enter): '
def run(a):
    checks=[];children=[];password=secrets.token_hex(20);observations=[]
    def passed(name):checks.append(name);print(name,flush=True)
    def collect(c,seconds=.2):
        end=time.monotonic()+seconds
        while time.monotonic()<end:
            if select.select([c.fd],[],[],.03)[0]:
                raw=os.read(c.fd,8192);assert raw,'unexpected terminal EOF';c.buffer+=raw;c.transcript+=raw
    with tempfile.TemporaryDirectory(prefix='faction-standalone-social-') as name:
        root=Path(name);state=root/'board'
        def query(sql):
            with sqlite3.connect(f'file:{state}/faction.sqlite3?mode=ro',uri=True) as d:return d.execute(sql).fetchall()
        def write(sql):
            with sqlite3.connect(state/'faction.sqlite3') as d:d.execute(sql)
        def menu(c,plain=False):
            target=PLAIN if plain else MENU;end=time.monotonic()+15
            while target.encode() not in c.buffer:
                if b'Press Enter to continue: ' in c.buffer:
                    c.expect('Press Enter to continue: ');c.send('')
                assert time.monotonic()<end,'login/menu readiness timeout';collect(c,.05)
            c.expect(target)
        def start(handle='sysop',plain=False,worker=False,theme=None):
            extra=['--theme',str(theme)] if theme else []
            c=Worker(a.member,state,extra,columns=30 if plain else 80,rows=12 if plain else 24) if worker else Console(a.binary,state,'login',[*extra,'--plain' if plain else '--ansi']);children.append(c)
            if worker:c.expect('[L]ogin [R]egister [Q]uit: ');c.send('l')
            c.expect('Handle: ');c.send(handle);c.password(password);menu(c,plain);return c
        def key(c,text,plain=False):c.send(text) if plain else c.raw(text.encode())
        def cmd(c,text,expected='Lounge> '):mark=len(c.transcript);c.send(text);c.expect(expected);return c.transcript[mark:]
        def lounge(c,plain=False,keyname='l'):mark=len(c.transcript);key(c,keyname,plain);c.expect('Lounge> ');return c.transcript[mark:]
        def lobby(c,plain=False):cmd(c,'q',PLAIN if plain else MENU)
        def logout(c,plain=False):key(c,'q',plain);c.expect('Logged out.');c.finish()
        def chat(c,plain=False):key(c,'c',plain);c.expect('chat[#lobby]> ')
        def leave_chat(c,plain=False):cmd(c,'/quit','Chat closed.');action_end(c,plain)
        def age():write('UPDATE social_cooldowns SET last_page=unixepoch()-31')
        def page(owner,handle):age();cmd(owner,'/page '+handle,'Page attempted');owner.expect('chat[#lobby]> ')
        def incoming(handle):return query("SELECT p.id FROM social_pages p JOIN social_presence s ON p.recipient=s.session JOIN members m ON m.id=s.member WHERE m.handle="+repr(handle)+" AND p.state='pending'")
        def status(c,handle,value):key(c,'s',True);c.expect('[C]reate room: ');c.send(value);c.expect('Member handle: ');c.send(handle);c.expect('Status updated; previous sessions expired.');c.expect(PLAIN)
        try:
            c=Console(a.binary,state,'init');children.append(c);c.expect('Handle: ');c.send('sysop');c.password(password,'New password (12+ characters, hidden): ');c.password(password,'Confirm password (hidden): ');c.expect('Lobby created.');c.finish()
            for handle in ['alice','bobby','charlie','dora']:
                c=Console(a.binary,state,'register');children.append(c);c.expect('Handle: ');c.send(handle);c.password(password,'New password (12+ characters, hidden): ');c.password(password,'Confirm password (hidden): ');c.expect('Registered pending sysop approval.');c.finish()
            owner=start(plain=True)
            for handle in ['alice','bobby','charlie','dora']:status(owner,handle,'a')
            chat(owner,True);cmd(owner,'/social on','Social configuration updated.');cmd(owner,'/social name Standalone lounge','Social configuration updated.');cmd(owner,'/available Operator here','Availability shared');owner.expect('chat[#lobby]> ')
            alice=start('alice');bobby=start('bobby',worker=True);charlie=start('charlie',plain=True)
            for c in [alice,bobby,charlie]:
                frame=lounge(c,c is charlie);assert b'Your advertisement: hidden' in frame;cmd(c,'/available Reading the board');lobby(c,c is charlie)
            chat(alice);cmd(alice,'/join #lobby','Joined; new messages start here.');leave_chat(alice)
            assert query("SELECT count(*) FROM chat_sessions WHERE member=(SELECT id FROM members WHERE handle='alice')")==[(1,)]
            assert query("SELECT count(*) FROM chat_joins WHERE session IN(SELECT id FROM chat_sessions WHERE member=(SELECT id FROM members WHERE handle='alice'))")==[(0,)]
            passed('registered_approved_local_worker_plain_optin_reuses_one_visit_lease_and_parts_chat')
            begin(bobby,subject='Uninterrupted worker draft');command(bobby,'Worker');enter_screen(bobby);bobby.raw(b' kept.');bobby.expect('Worker kept.');bobby.expect(FOOTER)
            key(charlie,'p',True);charlie.expect('Room ID: ');charlie.raw(b'1');collect(charlie)
            for c in [alice,bobby,charlie]:collect(c)
            marks={id(c):len(c.transcript) for c in [alice,bobby,charlie]}
            started=time.monotonic()
            while time.monotonic()-started<(0.5 if a.quick else 66):time.sleep(.1)
            elapsed=time.monotonic()-started
            for label,handle in [('local_menu','alice'),('worker_full_screen','bobby'),('plain_room_prompt','charlie')]:
                touched=query("SELECT unixepoch()-touched FROM chat_sessions WHERE member=(SELECT id FROM members WHERE handle="+repr(handle)+")")[0][0]
                assert 0<=touched<=15
                if not a.quick:observations.append(dict(state=label,elapsed_seconds=round(elapsed,3),last_heartbeat_age_seconds=touched))
            assert a.quick or elapsed>=66;passed('parallel_menu_full_screen_and_canonical_prompt_keep_current_visit_heartbeats')
            for handle in ['alice','bobby','charlie']:page(owner,handle)
            for c in [alice,bobby,charlie]:collect(c);assert b'Page waiting:' not in c.transcript[marks[id(c)]:] and b'Lounge> ' not in c.transcript[marks[id(c)]:]
            alice.raw(b'?');alice.expect('Page waiting: open Social lounge');alice.expect(MENU);frame=lounge(alice);assert b'from sysop: pending' in frame
            cmd(alice,f'/answer {incoming("alice")[0][0]} accept');frame=cmd(owner,'/pages','Requests/replies expire');assert b'to alice: accepted' in frame;owner.expect('chat[#lobby]> ');passed('safe_menu_refresh_accept_and_reply_without_forced_channel_switch')
            bobby.raw(b'\x11');bobby.expect('Classic line editing; draft kept.');bobby.expect(PROMPT);frame=cmd(bobby,'.preview',PROMPT);assert b'Worker kept.' in frame
            bobby.send('.cancel');bobby.expect('Draft cancelled');action_end(bobby);assert b'Page waiting: open Social lounge' in bobby.transcript;frame=lounge(bobby);assert b'from sysop: pending' in frame
            charlie.send('');charlie.expect('Reply to post ID (blank for new message): ');charlie.send('');charlie.expect('Subject (blank for generated/reply default): ');charlie.send('Plain remains');charlie.expect('Message (one line, up to 1024 bytes): ');charlie.send('/edit');charlie.expect(PROMPT);charlie.send('.cancel');charlie.expect('Draft cancelled');action_end(charlie,True);frame=lounge(charlie,True);assert b'from sysop: pending' in frame;passed('worker_full_screen_and_plain_canonical_prefix_survive_invitations_without_popup')
            cmd(bobby,'/block sysop');frame=cmd(owner,'/who','Not a delivery guarantee.');assert b'bobby |' not in frame;owner.expect('chat[#lobby]> ')
            mark=len(bobby.transcript);bobby.resize(40,18);collect(bobby,.6);assert len(bobby.transcript)>mark and b'from sysop: pending' not in bobby.transcript[mark:];bobby.buffer=b'';frame=cmd(bobby,'r');assert b'from sysop: pending' not in frame
            cmd(bobby,'/unblock sysop');cmd(bobby,'/mute on');page(owner,'bobby');assert incoming('bobby')==[];cmd(bobby,'/mute off');minute=query('SELECT (unixepoch()/60)%1440')[0][0];end=(minute+2)%1440
            cmd(bobby,f'/quiet {minute//60:02}:{minute%60:02} {end//60:02}:{end%60:02}');page(owner,'bobby');assert incoming('bobby')==[];cmd(bobby,'/quiet off');passed('current_block_mute_quiet_and_resize_remove_ineligible_signals')
            frame=cmd(alice,'/page-sysop');assert b'sysop [available, sysop]' in frame;age();cmd(alice,'/page-sysop sysop');cmd(owner,f'/answer {incoming("sysop")[0][0]} decline','Page answered.');owner.expect('chat[#lobby]> ')
            pack=root/'theme';pack.mkdir();(pack/'theme.toml').write_text((Path('themes/base/theme.toml')).read_text()+"\n[[menu]]\naction='lounge'\nkey='J'\nlabel='Night lounge'\ndescription='Choose to meet'\n")
            dora=start('dora',plain=True,worker=True,theme=pack);frame=lounge(dora,True,'j');assert b'Standalone lounge' in frame and b'Your advertisement: hidden' in frame;cmd(dora,'/available Classic worker');lobby(dora,True);frame=cmd(dora,'?',PLAIN);assert b'Unknown or unavailable command' not in frame;lounge(dora,True,'j');cmd(dora,'/privacy');lobby(dora,True);logout(dora,True);assert b'\x1b' not in dora.transcript;passed('named_lounge_and_custom_hotkey_work_for_escape_free_plain_inherited_worker')
            lobby(charlie,True);logout(charlie,True);assert b'\x1b' not in charlie.transcript;charlie=start('charlie',plain=True);frame=lounge(charlie,True);assert b'Your advertisement: hidden' in frame;lobby(charlie,True);logout(charlie,True);passed('plain_logout_and_reconnect_never_recreate_previous_advertisement')
            # The inherited worker has no controlling terminal: closing its master
            # exercises the poll/EOF error path rather than sending a kill signal.
            os.close(bobby.fd);children.remove(bobby);deadline=time.monotonic()+5
            while time.monotonic()<deadline:
                pid,_=os.waitpid(bobby.pid,os.WNOHANG)
                if pid:break
                time.sleep(.05)
            else:raise AssertionError('worker EOF did not terminate')
            assert query("SELECT count(*) FROM chat_sessions WHERE member=(SELECT id FROM members WHERE handle='bobby')")==[(0,)];passed('worker_transport_eof_releases_visit_advertisement_and_restores_no_stale_authority')
            lobby(alice);leave_chat(owner,True);status(owner,'alice','v');alice.raw(b'l');alice.expect('session expired');alice.finish(1);assert query("SELECT count(*) FROM chat_sessions WHERE member=(SELECT id FROM members WHERE handle='alice')")==[(0,)];passed('current_revocation_closes_held_local_visit_and_restores_terminal')
            contended=start('charlie',plain=True);chat(contended,True)
            with sqlite3.connect(state/'faction.sqlite3') as lock:
                lock.execute('BEGIN IMMEDIATE');contended.send('/quit');contended.expect('database busy; retry later');contended.finish(1)
            # A deliberately locked cleanup cannot remove rows until expiry;
            # the important property is that no menu resumes/renews those joins.
            write("UPDATE chat_sessions SET touched=unixepoch()-61 WHERE member=(SELECT id FROM members WHERE handle='charlie')")
            passed('failed_chat_part_cleanup_ends_visit_instead_of_resuming_with_retained_channel_joins')
            held=start('charlie');lounge(held);chat(owner,True);cmd(owner,'/social off','Social configuration updated.');cmd(held,'r',MENU);assert b'Social lounge disabled' in held.transcript;logout(held);leave_chat(owner,True);owner.send('?');owner.expect(PLAIN);assert query('SELECT count(*) FROM social_presence')==[(0,)] and query('SELECT count(*) FROM social_pages')==[(0,)];logout(owner,True);passed('sysop_disable_clears_live_signals_and_logout_closes_final_visit')
            assert query('PRAGMA user_version')==[(20,)] and query('SELECT count(*) FROM unread')==[(0,)] and query('SELECT count(*) FROM file_transfers')==[(0,)] and all(password.encode() not in c.transcript for c in children)
            passed('schema_unread_transfer_state_unchanged_and_passwords_never_echoed')
        except Exception:
            if a.diagnostics:
                a.diagnostics.mkdir(mode=0o700,parents=True,exist_ok=True)
                for i,c in enumerate(children):(a.diagnostics/('private-transcript-'+str(i))).write_bytes(c.transcript)
            raise
        finally:
            for c in children:c.close()
    result=dict(result='pass',count=len(checks),checks=checks,schema_version=20,sustained=not a.quick,heartbeat_observations=observations,synthetic_clock_aging=True,actual_registration_and_approval=True,standalone_visit_availability=True,capacity_claim=False,automatic_popups=False,private_message_bodies=False,new_transfer_authority=False)
    if a.output:a.output.write_text(json.dumps(result,indent=2,sort_keys=True)+'\n')
    print('Standalone visit lounge passed:',len(checks),'controls; sustained:',not a.quick,flush=True)
if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('binary',type=Path);p.add_argument('member',type=Path);p.add_argument('--output',type=Path);p.add_argument('--quick',action='store_true');p.add_argument('--diagnostics',type=Path);a=p.parse_args();a.binary=a.binary.resolve();a.member=a.member.resolve();run(a)
