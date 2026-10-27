#!/usr/bin/env python3
"""Actual optional idle page notifications over SSH/native; marked disposable NetBSD only."""
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
    def remote(cmd,data=None):
        x=subprocess.run(admin+['set -e; PATH=/usr/sbin:/sbin:/usr/bin:/bin:/usr/pkg/bin; export PATH; test -f /etc/faction-disposable-test; '+cmd],input=data,capture_output=True,timeout=30)
        if x.returncode:(a.output.parent/'idle-private-error.log').write_bytes(x.stdout+x.stderr)
        assert x.returncode==0,'owned notification fixture operation failed';return x.stdout.decode()
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
    def passed(name):checks.append(name);print(name,flush=True)
    def collect(c,seconds=.2):
        end=time.monotonic()+seconds
        while time.monotonic()<end:
            if select.select([c.fd],[],[],.03)[0]:
                data=os.read(c.fd,8192);assert data,'unexpected terminal EOF';c.buffer+=data;c.transcript+=data
    def cmd(c,text,expected='Lounge> '):mark=len(c.transcript);c.send(text);c.expect(expected);return c.transcript[mark:]
    def lounge(c,plain=False):c.send('l') if plain else c.raw(b'l');c.expect('Lounge> ')
    def lobby(c,plain=False):cmd(c,'q','Command (then Enter): ' if plain else MENU)
    def rearm(c,plain=False,on=True):
        lounge(c,plain);cmd(c,'/hide');cmd(c,'/available Browsing');cmd(c,'/notify '+('on' if on else 'off'));lobby(c,plain);collect(c)
    def page(owner,handle):sql("d.execute('UPDATE social_cooldowns SET last_page=unixepoch()-31')");cmd(owner,'/page '+handle)
    def silent(c,mark):collect(c,3.2);assert b'Page waiting: open Social lounge' not in c.transcript[mark:] and b'Lounge> ' not in c.transcript[mark:]
    def notice(c,mark,plain=False):
        c.expect('Page waiting: open Social lounge');c.expect('Command (then Enter): ' if plain else MENU);collect(c,3.2);assert c.transcript[mark:].count(b'Page waiting: open Social lounge')==1 and b'\x07' not in c.transcript[mark:]
    def incoming(handle):return query("SELECT p.id FROM social_pages p JOIN social_presence s ON p.recipient=s.session JOIN members m ON m.id=s.member WHERE m.handle="+repr(handle)+" AND p.state='pending'")
    try:
        remote('/etc/rc.d/faction_doors onestop || true; /etc/rc.d/faction_broker onestop')
        remote('install -o root -g wheel -m 755 '+str(a.guest_source)+'-target/release/faction-doors /usr/local/libexec/faction-doors')
        remote('install -o root -g wheel -m 555 '+str(a.guest_source)+'/deploy/netbsd/faction_doors /etc/rc.d/faction_doors')
        remote('install -o root -g wheel -m 644 '+str(a.guest_source)+'/doors/examples/*.lua /usr/local/share/faction/doors/')
        games=remote('id -u factiongames').strip();assert games.isdecimal();current=original+'doors_uid='+games+'\n';configure(current);remote('/etc/rc.d/faction_doors onestart')
        author='idle-'+secrets.token_hex(3);reader='native-'+secrets.token_hex(3)
        for handle in [author,reader]:c=connect(handle=handle,register=True);c.finish()
        owner=connect();status(owner,author,'a');status(owner,reader,'a');owner.raw(b'c');owner.expect('chat[#lobby]> ');cmd(owner,'/social on','Social configuration updated.');cmd(owner,'/available Operator','Availability shared');cmd(owner,'/quit','Chat closed.');owner.expect(MENU);lounge(owner)
        friend=connect(handle=author);lounge(friend);frame=cmd(friend,'/notify');assert b'Idle page notifications: off' in frame;cmd(friend,'/available Exploring');lobby(friend);collect(friend);mark=len(friend.transcript);page(owner,author);silent(friend,mark)
        rearm(friend);mark=len(friend.transcript);page(owner,author);notice(friend,mark);passed('actual_ssh_default_off_and_optin_one_quiet_coalesced_idle_notice')
        rearm(friend);friend.raw(b'\x1b');mark=len(friend.transcript);page(owner,author);silent(friend,mark);friend.raw(b'\r');notice(friend,mark)
        rearm(friend);friend.raw(b'\x1b[B');collect(friend);mark=len(friend.transcript);page(owner,author);notice(friend,mark);friend.raw(b'\r');friend.expect('Room ID: ');friend.send('0');friend.expect('Press Enter to return to menu: ');friend.send('');friend.expect(MENU);passed('actual_ssh_partial_escape_quarantine_and_arrow_focus_survive_idle_frames')
        rearm(friend);begin(friend,subject='Quiet idle pages');command(friend,'Kept first.');friend.raw(b'unsent');collect(friend);mark=len(friend.transcript);page(owner,author);silent(friend,mark);friend.send(' kept.');friend.expect(PROMPT);frame=cmd(friend,'.preview',PROMPT);assert b'Kept first.' in frame and b'unsent kept.' in frame;friend.send('.cancel');friend.expect('Draft cancelled');action_end(friend,shared=True);passed('actual_enabled_notifications_preserve_composer_and_unsubmitted_input')
        rearm(friend);friend.raw(b'd');friend.expect('After Hours Arcade');friend.expect('Ctrl-C returns to the BBS.');friend.raw(b'h');friend.expect('Frequency (1-99), or Q to leave: ');friend.raw(b'9');collect(friend);mark=len(friend.transcript);page(owner,author);silent(friend,mark);friend.send('99');friend.expect('Signal Hunt // pirate radio after midnight');friend.expect('Tune a whole frequency from 1 to 99.');friend.raw(b'\x03');friend.expect(MENU);passed('actual_enabled_notifications_leave_lua_game_and_held_input_untouched')
        rearm(friend);friend.raw(b't');friend.expect('[D]one: ');mark=len(friend.transcript);page(owner,author);friend.send('p');friend.expect('\x1b[?2026$p');friend.expect('[D]one: ');silent(friend,mark);friend.send('d');friend.expect(MENU);passed('actual_page_waits_through_terminal_settings_and_sync_probe_without_injected_command')
        lounge(friend);frame=cmd(friend,'/pages');assert b'from sysop: pending' in frame;id=incoming(author)[0][0];cmd(owner,'/notify on');lobby(owner);mark=len(owner.transcript);cmd(friend,f'/answer {id} accept');notice(owner,mark);lounge(owner);cmd(owner,'/notify off');lobby(friend);logout(friend);passed('actual_explicit_outgoing_answer_announces_only_at_idle_menu')
        native=connect('native',handle=reader);lounge(native);cmd(native,'/available Native visitor');cmd(native,'/notify on');lobby(native);mark=len(native.transcript);page(owner,reader);notice(native,mark);lounge(native);cmd(native,'/hide');lobby(native);logout(native);passed('actual_host_native_client_receives_idle_notification_without_extra_authority')
        guest=connect('guest',plain=True,handle=reader,compact=True);lounge(guest,True);frame=cmd(guest,'/notify');assert b'Idle page notifications: off' in frame and b'Your advertisement: hidden' in frame;cmd(guest,'/available Compact visitor');cmd(guest,'/notify on');lobby(guest,True);mark=len(guest.transcript);page(owner,reader);notice(guest,mark,True)
        rearm(guest,True);guest.raw(b'c');collect(guest);mark=len(guest.transcript);page(owner,reader);silent(guest,mark);guest.send('');guest.expect('chat[#lobby]> ');cmd(guest,'/notify off','Idle page notifications: off');cmd(guest,'/quit','Chat closed.');guest.expect('Command (then Enter): ');logout(guest,True);before_exit,_,exit_tail=guest.transcript.partition(b'Logged out.');assert b'\x1b' not in before_exit and exit_tail.count(b'\x1b[0m\x1b[?25h')==1 and b'\x1b' not in exit_tail.replace(b'\x1b[0m\x1b[?25h',b'');passed('actual_netbsd_native_compact_client_resets_consent_and_preserves_plain_partial_command')
        lobby(owner);logout(owner);configure(current.replace('ansi=true','ansi=false'))
        owner=connect(plain=True);owner.send('c');owner.expect('chat[#lobby]> ');cmd(owner,'/available Plain operator','Availability shared');cmd(owner,'/quit','Chat closed.');owner.expect('Command (then Enter): ');lounge(owner,True)
        plain=connect(plain=True,handle=author);lounge(plain,True);frame=cmd(plain,'/notify');assert b'Idle page notifications: off' in frame;cmd(plain,'/available Plain visitor');cmd(plain,'/notify on');lobby(plain,True);mark=len(plain.transcript);page(owner,author);notice(plain,mark,True)
        rearm(plain,True);plain.raw(b'c');collect(plain);mark=len(plain.transcript);page(owner,author);silent(plain,mark);plain.send('');plain.expect('chat[#lobby]> ');cmd(plain,'/quit','Chat closed.');plain.expect('Command (then Enter): ');passed('actual_plain_ssh_idle_notice_has_no_escapes_and_keeps_partial_menu_command')
        lounge(plain,True);cmd(plain,'/hide');cmd(plain,'/available Plain visitor');cmd(plain,'/block sysop');lobby(plain,True);mark=len(plain.transcript);page(owner,author);silent(plain,mark);lounge(plain,True);cmd(plain,'/unblock sysop');lobby(plain,True)
        # Expire a pending signal while a protected canonical room prompt owns input.
        plain.send('p');plain.expect('Room ID: ');page(owner,author);sql("d.execute('UPDATE social_pages SET created=unixepoch()-61')");plain.send('0');plain.expect('Press Enter to return to menu: ');plain.send('');plain.expect('Command (then Enter): ');mark=len(plain.transcript);silent(plain,mark);passed('actual_current_block_and_expired_deferred_signal_never_create_idle_notification')
        lobby(owner,True);owner.send('s');owner.expect('[C]reate room: ');owner.send('v');owner.expect('Member handle: ');owner.send(author);owner.expect('Status updated; previous sessions expired.');owner.expect('Press Enter to return to menu: ');owner.send('');owner.expect('Command (then Enter): ');plain.expect('session expired');plain.finish(1);assert b'\x1b' not in plain.transcript
        owner.send('c');owner.expect('chat[#lobby]> ');cmd(owner,'/social off','Social configuration updated.');cmd(owner,'/quit','Chat closed.');owner.expect('Command (then Enter): ');logout(owner,True);passed('actual_notification_poll_rechecks_current_revocation_and_sysop_disable')
        assert query('PRAGMA user_version')==[[19]] and query('SELECT * FROM unread ORDER BY member_id,room_id')==before_unread and query('SELECT count(*) FROM file_transfers')==[[0]] and all(password.encode() not in c.transcript for c in children)
        passed('actual_normal_budget_schema_unread_transfer_and_hidden_password_invariants')
        a.output.write_text(json.dumps(dict(result='pass',count=len(checks),checks=checks,schema_version=20,normal_sessions=8,admission_refill_ms=5000,synthetic_clock_aging=True,default_off=True,visit_only=True,idle_main_menu_only=True,actual_ssh_and_native=True,compact_server_frames_escape_free=True,native_client_exit_reset=True,capacity_claim=False,private_message_bodies=False,forced_switch=False,bells=False),indent=2,sort_keys=True)+'\n');print('Actual shared idle notifications passed:',len(checks),'controls.',flush=True)
    except Exception:
        for i,c in enumerate(children):(a.output.parent/('idle-private-transcript-'+str(i))).write_bytes(c.transcript)
        raise
    finally:
        for c in children:c.close()
        remote('/etc/rc.d/faction_doors onestop || true; /etc/rc.d/faction_broker onestop || true');remote('cat > /etc/faction/service.toml',original.encode())
if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__)
    for n in ['admin-argv','member-argv','password-file','native-binary','native-profile','guest-native-argv','output']:p.add_argument('--'+n,type=Path,required=True)
    p.add_argument('--guest-source',type=Path,required=True);p.add_argument('--disposable-fixture',action='store_true',required=True);run(p.parse_args())
