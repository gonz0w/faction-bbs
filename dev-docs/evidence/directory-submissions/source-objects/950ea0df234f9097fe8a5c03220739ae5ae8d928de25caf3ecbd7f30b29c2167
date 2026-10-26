#!/usr/bin/env python3
"""Actual Who/Lounge/group privacy and activity in local/worker ANSI/plain visits."""
import argparse,json,os,secrets,select,sqlite3,tempfile,time
from pathlib import Path
from smoke_local_console import Console
from smoke_member_worker import Worker
from smoke_room_browser import MENU
from smoke_writing_desk import begin,action_end,PROMPT


def run(a):
    children=[];checks=[];password=secrets.token_hex(20)
    def passed(name):checks.append(name);print(name,flush=True)
    with tempfile.TemporaryDirectory(prefix='faction-who-') as tmp:
        state=Path(tmp)/'board'
        def query(sql):
            with sqlite3.connect(f'file:{state}/faction.sqlite3?mode=ro',uri=True) as d:return d.execute(sql).fetchall()
        def start(handle,plain=False,worker=False):
            c=Worker(a.member,state,columns=30 if plain else 80,rows=12 if plain else 24) if worker else Console(a.binary,state,'login',['--plain' if plain else '--ansi']);children.append(c)
            if worker:c.expect('[L]ogin [R]egister [Q]uit: ');c.send('l')
            c.expect('Handle: ');c.send(handle);c.password(password)
            target='Command (then Enter): ' if plain else MENU;deadline=time.monotonic()+15
            while target.encode() not in c.buffer:
                if b'Press Enter to continue: ' in c.buffer:c.expect('Press Enter to continue: ');c.send('')
                assert time.monotonic()<deadline,'menu readiness timeout'
                if select.select([c.fd],[],[],.05)[0]:
                    raw=os.read(c.fd,8192);assert raw,'unexpected EOF';c.buffer+=raw;c.transcript+=raw
            c.expect(target);return c
        def key(c,value,plain=False):c.send(value) if plain else c.raw(value.encode())
        def cmd(c,text,prompt):mark=len(c.transcript);c.send(text);c.expect(prompt);return c.transcript[mark:]
        def panel(c,plain=False):key(c,'who' if plain else 'w',plain);c.expect('Who> ')
        def lobby(c,plain=False):cmd(c,'q','Command (then Enter): ' if plain else MENU)
        def logout(c,plain=False):key(c,'q',plain);c.expect('Logged out.');c.finish()
        def group():
            frame=cmd(owner,'/who','Not a delivery guarantee.');owner.expect('chat[#lobby]> ');return frame
        try:
            c=Console(a.binary,state,'init');children.append(c);c.expect('Handle: ');c.send('sysop');c.password(password,'New password (12+ characters, hidden): ');c.password(password,'Confirm password (hidden): ');c.expect('Lobby created.');c.finish()
            modes=[('alice',False,False),('bobby',False,True),('charlie',True,False),('dora',True,True)]
            for handle,_,_ in modes:
                c=Console(a.binary,state,'register');children.append(c);c.expect('Handle: ');c.send(handle);c.password(password,'New password (12+ characters, hidden): ');c.password(password,'Confirm password (hidden): ');c.expect('Registered pending sysop approval.');c.finish()
            owner=start('sysop',True)
            for handle,_,_ in modes:
                owner.send('s');owner.expect('[C]reate room: ');owner.send('a');owner.expect('Member handle: ');owner.send(handle);owner.expect('Status updated; previous sessions expired.');owner.expect('Command (then Enter): ')
            owner.send('c');owner.expect('chat[#lobby]> ');cmd(owner,'/social on','Social configuration updated.');owner.expect('chat[#lobby]> ');cmd(owner,'/available Operator','chat[#lobby]> ')
            for handle,plain,worker in modes:
                c=start(handle,plain,worker);panel(c,plain);frame=cmd(c,'r','Who> ');assert b'Your visibility: hidden' in frame and (handle+' |').encode() not in group();passed(handle+'_who_is_distinct_and_fresh_visit_starts_hidden')
                for mode in ['available','busy','away']:
                    cmd(c,'/'+mode+' Original status','Who> ');frame=group();assert (handle+' | Who | '+mode+' | Original status').encode() in frame
                lobby(c,plain);assert (handle+' | Menu | away').encode() in group();key(c,'c',plain);c.expect('chat[#lobby]> ');frame=cmd(c,'/who','Not a delivery guarantee.');c.expect('chat[#lobby]> ');assert (handle+' | Chat | away').encode() in frame;cmd(c,'/available Chat signal','chat[#lobby]> ');cmd(c,'/quit','Chat closed.');action_end(c,plain);passed(handle+'_group_and_main_who_share_activity_and_separate_availability')
                key(c,'l',plain);c.expect('Lounge> ');frame=cmd(c,'r','Lounge> ');assert b'Your advertisement: available / Chat signal' in frame and b'Current requests/replies' in frame
                cmd(c,'/busy Lounge signal','Lounge> ');assert (handle+' | Lounge | busy | Lounge signal').encode() in group();cmd(c,'/hide','Lounge> ');assert (handle+' |').encode() not in group();lobby(c,plain);panel(c,plain);frame=cmd(c,'r','Who> ');assert b'Your visibility: hidden' in frame;cmd(c,'/available Room visitor','Who> ');lobby(c,plain);passed(handle+'_lounge_keeps_its_own_attributes_and_hide_is_universal')
                begin(c,plain,subject='Private unfinished subject');c.send('Kept first.');c.expect(PROMPT);c.raw(b'unsent');time.sleep(.2);frame=group();assert (handle+' | Rooms | available | Room visitor').encode() in frame and b'Private unfinished subject' not in frame and b'unsent' not in frame
                c.send(' kept.');c.expect(PROMPT);frame=cmd(c,'.preview',PROMPT);assert b'Kept first.' in frame and b'unsent kept.' in frame;c.send('.cancel');c.expect('Draft cancelled');action_end(c,plain);passed(handle+'_room_location_keeps_subject_body_and_pending_input_private')
                panel(c,plain);cmd(c,'/block sysop','Who> ');assert (handle+' |').encode() not in group();cmd(c,'/unblock sysop','Who> ');assert (handle+' | Who | available').encode() in group();lobby(c,plain);logout(c,plain);assert (handle+' |').encode() not in group()
                c=start(handle,plain,worker);panel(c,plain);frame=cmd(c,'r','Who> ');assert b'Your visibility: hidden' in frame and (handle+' |').encode() not in group();lobby(c,plain);logout(c,plain)
                if plain:assert b'\x1b' not in c.transcript
                passed(handle+'_current_block_logout_and_reconnect_never_restore_visibility')
            cmd(owner,'/quit','Chat closed.');action_end(owner,True);logout(owner,True)
            assert query('PRAGMA user_version')==[(19,)] and query('SELECT count(*) FROM visit_activity')==[(0,)] and query('SELECT count(*) FROM chat_sessions')==[(0,)] and query('SELECT count(*) FROM unread')==[(0,)] and query('SELECT count(*) FROM file_transfers')==[(0,)] and all(password.encode() not in c.transcript for c in children)
            passed('live_activity_cleans_up_without_unread_transfer_or_password_side_effects')
        except Exception:
            if a.diagnostics:
                a.diagnostics.mkdir(parents=True,exist_ok=True,mode=0o700)
                for i,c in enumerate(children):(a.diagnostics/('private-transcript-'+str(i))).write_bytes(c.transcript)
            raise
        finally:
            for c in children:c.close()
    result=dict(result='pass',count=len(checks),checks=checks,schema_version=19,actual_registration_and_approval=True,universal_visibility_and_availability=True,distinct_interfaces=True,broad_host_activity=True,private_locations=False,capacity_claim=False,hardening_claim=False)
    if a.output:a.output.write_text(json.dumps(result,indent=2,sort_keys=True)+'\n')
    print('Who local/worker/plain passed:',len(checks),'controls.',flush=True)
if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('binary',type=Path);p.add_argument('member',type=Path);p.add_argument('--output',type=Path);p.add_argument('--diagnostics',type=Path);a=p.parse_args();a.binary=a.binary.resolve();a.member=a.member.resolve();run(a)
