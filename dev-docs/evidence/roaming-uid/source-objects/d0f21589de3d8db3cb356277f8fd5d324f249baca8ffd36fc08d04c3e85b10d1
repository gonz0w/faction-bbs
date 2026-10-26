#!/usr/bin/env python3
"""Actual shared board-wide availability, safe paging and outside-chat heartbeat on a marked disposable NetBSD board."""
import argparse,json,secrets,shlex,subprocess,time,tomllib,select
from pathlib import Path
from check_restricted_ssh import SSHConsole
from smoke_native_client_ssh import NativeConsole
from smoke_room_browser import MENU
from smoke_writing_desk import begin,command,action_end,PROMPT
import os


def run(a):
    assert a.disposable_fixture
    admin=json.loads(a.admin_argv.read_text());member=json.loads(a.member_argv.read_text());guest=json.loads(a.guest_native_argv.read_text());password=a.password_file.read_text().strip();children=[];checks=[];calls=0
    def passed(name):
        checks.append(name);print(name,flush=True)
    def remote(cmd,data=None):
        x=subprocess.run(admin+['set -e; PATH=/usr/sbin:/sbin:/usr/bin:/bin:/usr/pkg/bin; export PATH; test -f /etc/faction-disposable-test; '+cmd],input=data,capture_output=True,timeout=30)
        if x.returncode:(a.output.parent/'library-private-error.log').write_bytes(x.stdout+x.stderr)
        assert x.returncode==0,'owned discussion fixture operation failed';return x.stdout.decode()
    original=remote('cat /etc/faction/service.toml');assert tomllib.loads(original)['state_dir']=='/var/db/faction-profile-fixture';db='/var/db/faction-profile-fixture/faction.sqlite3'
    def query(sql):
        code='import sqlite3,json; d=sqlite3.connect('+repr('file:'+db+'?mode=ro')+',uri=True); print(json.dumps(d.execute('+repr(sql)+').fetchall()))';return json.loads(remote('python3.13 -c '+shlex.quote(code)))
    def connect(route='ssh',plain=False,handle='sysop',register=False,compact=False,limit=None):
        nonlocal calls
        if calls:time.sleep(5.1)
        calls+=1
        options=(['--compact'] if compact else [])+(['--file-budget',str(limit)] if limit is not None else [])
        argv=guest.copy() if route=='guest' else [str(a.native_binary.resolve()),'--profile',str(a.native_profile.resolve())]
        if route=='guest':
            assert argv[-1].endswith("'");argv[-1]=argv[-1][:-1]+' '+' '.join(options)+"'"
        else:argv+=options
        c=SSHConsole(member) if route=='ssh' else NativeConsole(argv);children.append(c)
        c.expect('[L]ogin [R]egister [Q]uit: ');c.send('r' if register else 'l');c.expect('Handle: ');c.send(handle);c.password(password)
        if register:c.password(password,'Confirm password (hidden): ');c.expect('Registration pending; sysop approval required.');return c
        c.expect('Press Enter to continue: ');c.send('');c.expect('Command (then Enter): ' if plain else MENU);return c
    def logout(c,plain=False):c.send('q') if plain else c.raw(b'q');c.expect('Logged out.');c.finish()
    def back(c,plain=False):c.send('q');c.expect('Command (then Enter): ' if plain else MENU)
    def configure(text):remote('/etc/rc.d/faction_broker onestop');remote('cat > /etc/faction/service.toml',text.encode());remote('/etc/rc.d/faction_broker onestart')
    def status(c,handle,key):c.raw(b's');c.expect('[C]reate room: ');c.send(key);c.expect('Member handle: ');c.send(handle);c.expect('Status updated; previous sessions expired.');c.expect('Press Enter to return to menu: ');c.send('');c.expect(MENU)
    def sql(code):remote('python3.13 -c '+shlex.quote('import sqlite3;d=sqlite3.connect('+repr(db)+');d.execute("PRAGMA foreign_keys=ON");'+code+';d.commit()'))
    before_unread=query('SELECT * FROM unread ORDER BY member_id,room_id')
    heartbeat=[]
    try:
        # Rebuild/install the existing optional C/Lua door only on this owned clone.
        remote('/etc/rc.d/faction_doors onestop || true; /etc/rc.d/faction_broker onestop')
        remote('install -o root -g wheel -m 755 '+str(a.guest_source)+'-target/release/faction-doors /usr/local/libexec/faction-doors')
        remote('install -o root -g wheel -m 555 '+str(a.guest_source)+'/deploy/netbsd/faction_doors /etc/rc.d/faction_doors')
        remote('install -o root -g wheel -m 644 '+str(a.guest_source)+'/doors/examples/*.lua /usr/local/share/faction/doors/')
        games=remote('id -u factiongames').strip();assert games.isdecimal()
        current=original+'doors_uid='+games+'\n';configure(current);remote('/etc/rc.d/faction_doors onestart')
        def cmd(c,line,expected):
            mark=len(c.transcript);c.send(line);c.expect(expected);return c.transcript[mark:]
        def chat(c,plain=False):
            c.send('c') if plain else c.raw(b'c');c.expect('chat[#lobby]> ')
        def close(c,plain=False):
            cmd(c,'/quit','Chat closed.');c.expect('Command (then Enter): ' if plain else MENU)
        def lounge(c,plain=False):
            mark=len(c.transcript);c.send('l') if plain else c.raw(b'l');c.expect('Lounge> ');return c.transcript[mark:]
        def lobby(c,plain=False):cmd(c,'q','Command (then Enter): ' if plain else MENU)
        def rate_age():sql("d.execute('UPDATE social_cooldowns SET last_page=unixepoch()-31')")
        def collect(c,seconds=.25):
            end=time.monotonic()+seconds
            while time.monotonic()<end:
                if select.select([c.fd],[],[],.05)[0]:
                    raw=os.read(c.fd,8192);assert raw,'connection unexpectedly closed'
                    c.buffer+=raw;c.transcript+=raw
        def wait_outside(label,handle):
            start=time.monotonic()
            while time.monotonic()-start<66:time.sleep(.5)
            elapsed=time.monotonic()-start
            age=query("SELECT unixepoch()-touched FROM chat_sessions WHERE member=(SELECT id FROM members WHERE handle="+repr(handle)+")")[0][0]
            assert 0<=age<=15 and elapsed>=66
            heartbeat.append(dict(state=label,elapsed_seconds=round(elapsed,3),last_heartbeat_age_seconds=age));print('Measured outside-chat heartbeat:',label,round(elapsed,3),'seconds; age',age,flush=True)
        author='reader-'+secrets.token_hex(3);reader='friend-'+secrets.token_hex(3);channel='#studio'+secrets.token_hex(3)
        for handle in [author,reader]:c=connect(handle=handle,register=True);c.finish()
        owner=connect();status(owner,author,'a');status(owner,reader,'a');chat(owner)
        cmd(owner,'/social on','Social configuration updated.');cmd(owner,'/social name Night lounge','Social configuration updated.')
        cmd(owner,'/available Operator in the BBS','Availability shared');cmd(owner,'/create '+channel+' private','Channel created.');cmd(owner,'/allow '+channel+' '+author,'Channel grant updated; target must join again.');close(owner)
        friend=connect(handle=author);frame=lounge(friend);assert b'Your advertisement: hidden' in frame;cmd(friend,'/available Reading and exploring','Lounge> ');lobby(friend)
        visitor=connect('native',handle=reader);frame=lounge(visitor);assert b'Your advertisement: hidden' in frame;cmd(visitor,'/available Ready to meet','Lounge> ')
        chat(friend);cmd(friend,'/join '+channel,'Joined; new messages start here.');cmd(friend,'Before leaving chat',author+'> Before leaving chat');close(friend)
        assert query("SELECT count(*) FROM chat_sessions WHERE member=(SELECT id FROM members WHERE handle="+repr(author)+")")==[[1]]
        assert query("SELECT count(*) FROM chat_joins WHERE session IN(SELECT id FROM chat_sessions WHERE member=(SELECT id FROM members WHERE handle="+repr(author)+"))")==[[0]]
        frame=cmd(visitor,'/who','Lounge> ');assert (author+' [available]').encode() in frame and channel.encode() not in frame;passed('actual_lounge_optin_and_chat_exit_keep_one_connection_lease_without_channel_presence_leak')
        wait_outside('main_menu',author);frame=cmd(visitor,'/who','Lounge> ');assert (author+' [available]').encode() in frame;cmd(visitor,'/page '+author,'Lounge> ')
        collect(friend);mark=len(friend.transcript);friend.raw(b'?');friend.expect('Page waiting: open Social lounge');friend.expect(MENU)
        frame=lounge(friend);assert ('from '+reader+': pending').encode() in frame;id=query("SELECT id FROM social_pages WHERE state='pending'")[0][0];cmd(friend,f'/answer {id} accept','Lounge> ');lobby(friend);frame=cmd(visitor,'/pages','Lounge> ');assert ('to '+author+': accepted').encode() in frame;passed('actual_idle_menu_heartbeat_beyond_sixty_seconds_and_deliberate_safe_page_badge')
        begin(friend,subject='Quiet lounge editor');command(friend,'Kept first line.');friend.raw(b'unsent line');collect(friend);mark=len(friend.transcript)
        wait_outside('room_composer',author);frame=cmd(visitor,'/who','Lounge> ');assert (author+' [available]').encode() in frame;cmd(visitor,'/page '+author,'Lounge> ');collect(friend);assert b'Page waiting:' not in friend.transcript[mark:] and b'Lounge> ' not in friend.transcript[mark:]
        friend.send(' kept.');friend.expect(PROMPT);command(friend,'.preview');assert b'Kept first line.' in friend.transcript and b'unsent line kept.' in friend.transcript
        friend.send('.cancel');friend.expect('Draft cancelled');action_end(friend,shared=True);assert b'Page waiting: open Social lounge' in friend.transcript;passed('actual_composer_heartbeat_and_paging_preserve_complete_and_unsubmitted_draft_text')
        frame=lounge(friend);assert ('from '+reader+': pending').encode() in frame;id=query("SELECT id FROM social_pages WHERE state='pending' ORDER BY id DESC")[0][0];cmd(friend,f'/answer {id} decline','Lounge> ');lobby(friend)
        friend.raw(b'd');friend.expect('After Hours Arcade');friend.expect('Ctrl-C returns to the BBS.');friend.raw(b'h');friend.expect('Frequency (1-99), or Q to leave: ');friend.raw(b'9');collect(friend);mark=len(friend.transcript)
        wait_outside('lua_game_input',author);frame=cmd(visitor,'/who','Lounge> ');assert (author+' [available]').encode() in frame;cmd(visitor,'/page '+author,'Lounge> ');collect(friend);assert b'Page waiting:' not in friend.transcript[mark:] and b'Lounge> ' not in friend.transcript[mark:]
        friend.send('99');friend.expect('Signal Hunt // pirate radio after midnight');friend.expect('Tune a whole frequency from 1 to 99.');friend.raw(b'\x03');friend.expect(MENU);frame=lounge(friend);assert ('from '+reader+': pending').encode() in frame;passed('actual_lua_game_heartbeat_and_invitations_preserve_pending_input_without_forced_screen_switch')
        cmd(friend,'/block '+reader,'Lounge> ');frame=cmd(visitor,'/who','Lounge> ');assert (author+' [').encode() not in frame
        mark=len(friend.transcript);friend.resize(40,18);collect(friend,.6);assert len(friend.transcript)>mark and ('from '+reader+': pending').encode() not in friend.transcript[mark:];friend.buffer=b'';frame=cmd(friend,'r','Lounge> ');assert ('from '+reader+': pending').encode() not in frame
        cmd(friend,'/unblock '+reader,'Lounge> ');cmd(friend,'/mute on','Lounge> ');rate_age();cmd(visitor,'/page '+author,'Lounge> ');assert query("SELECT count(*) FROM social_pages WHERE state='pending'")==[[0]]
        cmd(friend,'/mute off','Lounge> ');minute=query("SELECT (unixepoch()/60)%1440")[0][0];end=(minute+2)%1440
        cmd(friend,f'/quiet {minute//60:02}:{minute%60:02} {end//60:02}:{end%60:02}','Lounge> ');rate_age();cmd(visitor,'/page '+author,'Lounge> ');assert query("SELECT count(*) FROM social_pages WHERE state='pending'")==[[0]];cmd(friend,'/quiet off','Lounge> ');passed('actual_outside_chat_block_mute_quiet_and_resize_refetch_current_signal_visibility')
        cmd(friend,'/page-sysop','Lounge> ');assert b'sysop [available, sysop]' in friend.transcript[-2500:];rate_age();cmd(friend,'/page-sysop sysop','Lounge> ');owner.raw(b'?');owner.expect('Page waiting: open Social lounge');owner.expect(MENU);lounge(owner);id=query("SELECT id FROM social_pages WHERE state='pending' ORDER BY id DESC")[0][0];cmd(owner,f'/answer {id} decline','Lounge> ');lobby(owner);passed('actual_page_sysop_is_available_outside_chat_without_os_console_or_host_authority')
        # A fresh plain shared connection never inherits its old advertisement.
        lobby(friend);logout(friend);frame=cmd(visitor,'/who','Lounge> ');assert (author+' [').encode() not in frame;lobby(visitor);logout(visitor);logout(owner)
        configure(current.replace('ansi=true','ansi=false'));plain=connect(plain=True,handle=author);frame=lounge(plain,True);assert b'Your advertisement: hidden' in frame;cmd(plain,'/available Plain visitor','Lounge> ');lobby(plain,True);plain.send('?');plain.expect('Command (then Enter): ');lounge(plain,True);cmd(plain,'/privacy','Lounge> ');lobby(plain,True);logout(plain,True);assert b'\x1b' not in plain.transcript;configure(current);passed('actual_plain_outside_chat_lounge_menu_refresh_and_disconnect_keep_classic_fallback')
        owner=connect();friend=connect(handle=author);lounge(friend);cmd(friend,'/available Revocation check','Lounge> ');lobby(friend);status(owner,author,'v');friend.raw(b'l');friend.expect('session expired');friend.finish(1)
        lounge(owner);cmd(owner,'/social off',MENU);assert b'Social lounge disabled' in owner.transcript;logout(owner);passed('actual_revocation_and_disable_remove_outside_chat_authority_and_pending_signals')
        assert query('PRAGMA user_version')==[[19]] and query('SELECT * FROM unread ORDER BY member_id,room_id')==before_unread and all(password.encode() not in c.transcript for c in children)
        a.output.write_text(json.dumps(dict(result='pass',count=len(checks),checks=checks,schema_version=20,normal_sessions=8,admission_refill_ms=5000,heartbeat_observations=heartbeat,synthetic_clock_aging=True,capacity_claim=False,private_message_bodies=False,outside_chat_shared_availability=True,automatic_popups=False,legacy_outside_chat=False),indent=2,sort_keys=True)+'\n');print('Actual board-wide shared social passed:',len(checks),'controls.')
    except Exception:
        for i,c in enumerate(children):(a.output.parent/('boardwide-private-transcript-'+str(i))).write_bytes(c.transcript)
        raise
    finally:
        for c in children:c.close()
        remote('/etc/rc.d/faction_doors onestop || true; /etc/rc.d/faction_broker onestop || true');remote('cat > /etc/faction/service.toml',original.encode())
if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__)
    for n in ['admin-argv','member-argv','password-file','native-binary','native-profile','guest-native-argv','output']:p.add_argument('--'+n,type=Path,required=True)
    p.add_argument('--guest-source',type=Path,required=True)
    p.add_argument('--disposable-fixture',action='store_true',required=True);run(p.parse_args())
