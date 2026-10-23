#!/usr/bin/env python3
"""Opt-in idle notifications on actual local/worker ANSI/plain PTYs; no listener."""
import argparse,json,os,secrets,select,sqlite3,tempfile,time
from pathlib import Path
from smoke_local_console import Console
from smoke_member_worker import Worker
from smoke_room_browser import MENU
from smoke_writing_desk import begin,command,action_end,PROMPT
from smoke_full_screen import enter_screen,FOOTER
PLAIN='Command (then Enter): '
NOTICE='Page waiting: open Social lounge'

def run(a):
    children=[];checks=[];password=secrets.token_hex(20)
    def passed(name):checks.append(name);print(name,flush=True)
    def collect(c,seconds=.2):
        end=time.monotonic()+seconds
        while time.monotonic()<end:
            if select.select([c.fd],[],[],.03)[0]:
                data=os.read(c.fd,8192);assert data,'unexpected EOF';c.buffer+=data;c.transcript+=data
    with tempfile.TemporaryDirectory(prefix='faction-idle-notices-') as name:
        state=Path(name)/'board'
        def query(sql):
            with sqlite3.connect(f'file:{state}/faction.sqlite3?mode=ro',uri=True) as d:return d.execute(sql).fetchall()
        def write(sql):
            with sqlite3.connect(state/'faction.sqlite3') as d:d.execute(sql)
        def ready(c,plain=False):
            target=PLAIN if plain else MENU;deadline=time.monotonic()+15
            while target.encode() not in c.buffer:
                if b'Press Enter to continue: ' in c.buffer:c.expect('Press Enter to continue: ');c.send('')
                assert time.monotonic()<deadline,'menu readiness timeout';collect(c,.05)
            c.expect(target)
        def start(handle,plain=False,worker=False):
            c=Worker(a.member,state,columns=30 if plain else 80,rows=12 if plain else 24) if worker else Console(a.binary,state,'login',['--plain' if plain else '--ansi']);children.append(c)
            if worker:c.expect('[L]ogin [R]egister [Q]uit: ');c.send('l')
            c.expect('Handle: ');c.send(handle);c.password(password);ready(c,plain);return c
        def key(c,text,plain=False):c.send(text) if plain else c.raw(text.encode())
        def cmd(c,text,expected='Lounge> '):mark=len(c.transcript);c.send(text);c.expect(expected);return c.transcript[mark:]
        def lounge(c,plain=False):key(c,'l',plain);c.expect('Lounge> ')
        def lobby(c,plain=False):cmd(c,'q',PLAIN if plain else MENU)
        def logout(c,plain=False):key(c,'q',plain);c.expect('Logged out.');c.finish()
        def rearm(c,plain=False,on=True):
            lounge(c,plain);cmd(c,'/hide');cmd(c,'/available Reading');cmd(c,'/notify '+('on' if on else 'off'));lobby(c,plain);collect(c)
        def page(handle):
            write('UPDATE social_cooldowns SET last_page=unixepoch()-31');cmd(owner,'/page '+handle,'Page attempted');owner.expect('chat[#lobby]> ')
        def silent(c,mark):collect(c,3.2);assert NOTICE.encode() not in c.transcript[mark:] and b'Lounge> ' not in c.transcript[mark:]
        def notified(c,mark,plain=False):
            c.expect(NOTICE);c.expect(PLAIN if plain else MENU);collect(c,3.2);assert c.transcript[mark:].count(NOTICE.encode())==1,'idle signal repeated';assert b'\x07' not in c.transcript[mark:]
        try:
            c=Console(a.binary,state,'init');children.append(c);c.expect('Handle: ');c.send('sysop');c.password(password,'New password (12+ characters, hidden): ');c.password(password,'Confirm password (hidden): ');c.expect('Lobby created.');c.finish()
            modes=[('alice',False,False),('bobby',False,True),('charlie',True,False),('dora',True,True)]
            for handle,_,_ in modes:
                c=Console(a.binary,state,'register');children.append(c);c.expect('Handle: ');c.send(handle);c.password(password,'New password (12+ characters, hidden): ');c.password(password,'Confirm password (hidden): ');c.expect('Registered pending sysop approval.');c.finish()
            owner=start('sysop',True)
            for handle,_,_ in modes:
                key(owner,'s',True);owner.expect('[C]reate room: ');owner.send('a');owner.expect('Member handle: ');owner.send(handle);owner.expect('Status updated; previous sessions expired.');owner.expect(PLAIN)
            key(owner,'c',True);owner.expect('chat[#lobby]> ');cmd(owner,'/social on','Social configuration updated.');cmd(owner,'/available Operator','Availability shared');owner.expect('chat[#lobby]> ')
            for handle,plain,worker in modes:
                c=start(handle,plain,worker);lounge(c,plain);frame=cmd(c,'/notify');assert b'Idle page notifications: off' in frame;cmd(c,'/available Reading');lobby(c,plain);collect(c);mark=len(c.transcript);page(handle);silent(c,mark);passed(handle+'_default_off_never_announces_idle_page')
                rearm(c,plain);mark=len(c.transcript);page(handle);notified(c,mark,plain);passed(handle+'_explicit_optin_coalesces_one_quiet_current_idle_notice')
                rearm(c,plain);mark=len(c.transcript)
                if plain:
                    c.raw(b'c');collect(c);page(handle);silent(c,mark);c.send('');c.expect('chat[#lobby]> ');cmd(c,'/notify off','Idle page notifications: off');c.expect('chat[#lobby]> ');cmd(c,'/quit','Chat closed.');action_end(c,True)
                else:
                    # A partial control sequence must keep notification output
                    # quarantined; the recovery Enter must not choose an action.
                    c.raw(b'\x1b');page(handle);silent(c,mark);c.raw(b'\r');notified(c,mark)
                    # Preserve arrow focus across the next automatic frame.
                    rearm(c);c.raw(b'\x1b[B');collect(c);mark=len(c.transcript);page(handle);notified(c,mark);c.raw(b'\r');c.expect('Room ID: ');c.send('0');c.expect('positive ID required');action_end(c)
                passed(handle+'_partial_input_or_control_sequence_and_menu_focus_survive')
                rearm(c,plain,False);mark=len(c.transcript);page(handle);silent(c,mark);lounge(c,plain);cmd(c,'/notify on');lobby(c,plain);logout(c,plain)
                if plain:assert b'\x1b' not in c.transcript
                c=start(handle,plain,worker);lounge(c,plain);frame=cmd(c,'/notify');assert b'Idle page notifications: off' in frame and b'Your advertisement: hidden' in frame;lobby(c,plain);logout(c,plain);passed(handle+'_off_and_reconnect_reset_notification_consent')
            c=start('alice');rearm(c);begin(c,subject='Quiet optional notices');command(c,'Kept first line.');c.raw(b'unsubmitted');collect(c);mark=len(c.transcript);page('alice');silent(c,mark);c.send(' kept.');c.expect(PROMPT);frame=cmd(c,'.preview',PROMPT);assert b'Kept first line.' in frame and b'unsubmitted kept.' in frame;c.send('.cancel');c.expect('Draft cancelled');action_end(c);passed('enabled_notifications_leave_composer_and_unsent_line_untouched')
            rearm(c);begin(c);command(c,'Full screen remains.');enter_screen(c);c.raw(b' kept');c.expect(FOOTER);collect(c);mark=len(c.transcript);page('alice');silent(c,mark);write('UPDATE social_pages SET created=unixepoch()-61');c.raw(b'\x11');c.expect('Classic line editing; draft kept.');c.expect(PROMPT);frame=cmd(c,'.preview',PROMPT);assert b'Full screen remains. kept' in frame;c.send('.cancel');c.expect('Draft cancelled');action_end(c);mark=len(c.transcript);silent(c,mark);passed('full_screen_input_survives_and_expired_deferred_signal_never_announces')
            rearm(c);lounge(c);cmd(c,'/mute on');lobby(c);mark=len(c.transcript);page('alice');silent(c,mark);lounge(c);cmd(c,'/mute off');cmd(c,'/block sysop');lobby(c);mark=len(c.transcript);page('alice');silent(c,mark);lounge(c);cmd(c,'/unblock sysop');minute=query('SELECT (unixepoch()/60)%1440')[0][0];end=(minute+2)%1440;cmd(c,f'/quiet {minute//60:02}:{minute%60:02} {end//60:02}:{end%60:02}');lobby(c);mark=len(c.transcript);page('alice');silent(c,mark);lounge(c);cmd(c,'/quiet off');lobby(c);logout(c);passed('current_mute_block_and_quiet_hours_suppress_optional_idle_notices')
            cmd(owner,'/quit','Chat closed.');owner.expect(PLAIN);logout(owner,True)
            assert query('PRAGMA user_version')==[(18,)] and query('SELECT count(*) FROM unread')==[(0,)] and query('SELECT count(*) FROM file_transfers')==[(0,)] and query('SELECT count(*) FROM chat_sessions')==[(0,)] and all(password.encode() not in c.transcript for c in children);passed('logout_clears_live_authority_and_schema_unread_transfer_password_invariants_hold')
        except Exception:
            if a.diagnostics:
                a.diagnostics.mkdir(mode=0o700,parents=True,exist_ok=True)
                for i,c in enumerate(children):(a.diagnostics/('private-transcript-'+str(i))).write_bytes(c.transcript)
            raise
        finally:
            for c in children:c.close()
    result=dict(result='pass',count=len(checks),checks=checks,schema_version=18,actual_registration_and_approval=True,synthetic_clock_aging=True,default_off=True,visit_only=True,idle_main_menu_only=True,plain_escape_free=True,capacity_claim=False,private_message_bodies=False,forced_switch=False,bells=False)
    if a.output:a.output.write_text(json.dumps(result,indent=2,sort_keys=True)+'\n')
    print('Idle notification local/worker/plain passed:',len(checks),'controls.',flush=True)
if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('binary',type=Path);p.add_argument('member',type=Path);p.add_argument('--output',type=Path);p.add_argument('--diagnostics',type=Path);a=p.parse_args();a.binary=a.binary.resolve();a.member=a.member.resolve();run(a)
