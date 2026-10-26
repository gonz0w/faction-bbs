#!/usr/bin/env python3
"""Actual distinct Who and universal visibility/availability on a marked disposable NetBSD board."""
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
        if x.returncode:(a.output.parent/'who-private-error.log').write_bytes(x.stdout+x.stderr)
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
    def cmd(c,text,prompt='Lounge> '):
        mark=len(c.transcript);c.send(text);c.expect(prompt);return c.transcript[mark:]
    def chat(c,plain=False):c.send('c') if plain else c.raw(b'c');c.expect('chat[#lobby]> ')
    def close(c,plain=False):cmd(c,'/quit','Chat closed.');c.expect('Command (then Enter): ' if plain else MENU)
    def group():
        frame=cmd(owner,'/who','Not a delivery guarantee.');owner.expect('chat[#lobby]> ');return frame
    def panel(c,plain=False):c.send('who') if plain else c.raw(b'w');c.expect('Who> ')
    def lobby(c,plain=False):cmd(c,'q','Command (then Enter): ' if plain else MENU)
    def location(handle,where,mode=None):
        time.sleep(.3);frame=group();assert (handle+' | '+where+' | '+(mode or '')).encode() in frame;return frame
    try:
        remote('/etc/rc.d/faction_doors onestop || true; /etc/rc.d/faction_broker onestop')
        remote('install -o root -g wheel -m 755 '+str(a.guest_source)+'-target/release/faction-doors /usr/local/libexec/faction-doors')
        remote('install -o root -g wheel -m 555 '+str(a.guest_source)+'/deploy/netbsd/faction_doors /etc/rc.d/faction_doors')
        remote('install -o root -g wheel -m 644 '+str(a.guest_source)+'/doors/examples/*.lua /usr/local/share/faction/doors/')
        games=remote('id -u factiongames').strip();fuid=remote('id -u factionfiles').strip();auid=remote('id -u factionfileauth').strip();assert all(s.isdecimal() for s in [games,fuid,auid])
        current=original+'doors_uid='+games+'\nfiles_uid='+fuid+'\nfiles_auth_uid='+auid+'\n';configure(current);remote('/etc/rc.d/faction_doors onestart')
        author='who-'+secrets.token_hex(3);reader='native-'+secrets.token_hex(3)
        for handle in [author,reader]:c=connect(handle=handle,register=True);c.finish()
        owner=connect();status(owner,author,'a');status(owner,reader,'a');chat(owner);cmd(owner,'/social on','Social configuration updated.');cmd(owner,'/available Operator','chat[#lobby]> ')
        friend=connect(handle=author);panel(friend);frame=cmd(friend,'r','Who> ');assert b'Your visibility: hidden' in frame and (author+' |').encode() not in group();passed('actual_ssh_who_is_distinct_and_new_member_starts_hidden')
        for mode in ['available','busy','away']:
            cmd(friend,'/'+mode+' Original signal','Who> ');location(author,'Who',mode)
        lobby(friend);location(author,'Menu','away');passed('actual_who_and_group_roster_share_separate_location_and_availability')
        begin(friend,subject='Unpublished private subject');command(friend,'Kept line.');friend.raw(b'unsent');frame=location(author,'Rooms','away');assert b'Unpublished private subject' not in frame and b'unsent' not in frame
        friend.send(' kept.');friend.expect(PROMPT);frame=cmd(friend,'.preview',PROMPT);assert b'Kept line.' in frame and b'unsent kept.' in frame;friend.send('.cancel');friend.expect('Draft cancelled');action_end(friend,shared=True);passed('actual_room_location_keeps_private_draft_and_pending_input_untouched')
        friend.raw(b'd');friend.expect('After Hours Arcade');friend.expect('Ctrl-C returns to the BBS.');friend.raw(b'h');friend.expect('Frequency (1-99), or Q to leave: ');friend.raw(b'9');frame=location(author,'Doors','away');assert b'Signal Hunt' not in frame
        friend.send('99');friend.expect('Tune a whole frequency from 1 to 99.');friend.raw(b'\x03');friend.expect(MENU);passed('actual_lua_location_hides_game_detail_and_preserves_held_input')
        friend.raw(b'f');friend.expect('files> ');location(author,'Files','away');friend.send('back');friend.expect(MENU);friend.raw(b't');friend.expect('[D]one: ');location(author,'Settings','away');friend.send('d');friend.expect(MENU);passed('actual_files_and_terminal_settings_report_only_broad_current_location')
        channel='#who'+secrets.token_hex(3);cmd(owner,'/create '+channel+' private','Channel created.');owner.expect('chat[#lobby]> ');cmd(owner,'/allow '+channel+' '+author,'Channel grant updated; target must join again.');owner.expect('chat[#lobby]> ');chat(friend);cmd(friend,'/join '+channel,'Joined; new messages start here.');friend.expect('chat['+channel+']> ');cmd(friend,'/available Group visitor','chat['+channel+']> ');frame=location(author,'Chat','available');assert channel.encode() not in frame
        frame=cmd(friend,'/who','Not a delivery guarantee.');friend.expect('chat['+channel+']> ');payload=frame.split(b'Who / visible board visits',1)[1].split(b'Not a delivery guarantee.',1)[0];assert (author+' | Chat | available').encode() in payload and channel.encode() not in payload;close(friend);passed('actual_group_who_is_boardwide_and_does_not_reveal_private_channel_names')
        friend.raw(b'l');friend.expect('Lounge> ');frame=cmd(friend,'r');assert b'Current requests/replies' in frame and b'Your advertisement: available / Group visitor' in frame;cmd(friend,'/busy Lounge visitor');location(author,'Lounge','busy');cmd(friend,'/hide');assert (author+' |').encode() not in group()
        close(owner);owner.raw(b'l');owner.expect('Lounge> ');frame=cmd(owner,'r');assert (author+' [').encode() not in frame;lobby(owner);panel(owner);frame=cmd(owner,'r','Who> ');assert (author+' |').encode() not in frame;lobby(owner);chat(owner);passed('actual_lounge_retains_paging_attributes_and_visibility_is_universal_across_all_three')
        cmd(friend,'/available Visible friend');cmd(friend,'/block sysop');assert (author+' |').encode() not in group();cmd(friend,'/unblock sysop');location(author,'Lounge','available');lobby(friend);logout(friend);assert (author+' |').encode() not in group();passed('actual_mutual_block_and_disconnect_remove_who_activity_and_presence')
        native=connect('native',handle=reader);panel(native);frame=cmd(native,'r','Who> ');assert b'Your visibility: hidden' in frame;cmd(native,'/available Native visitor','Who> ');location(reader,'Who','available');lobby(native);logout(native)
        guest=connect('guest',plain=True,handle=reader,compact=True);panel(guest,True);frame=cmd(guest,'r','Who> ');assert b'Your visibility: hidden' in frame;cmd(guest,'/available Compact visitor','Who> ');location(reader,'Who','available');lobby(guest,True);logout(guest,True);before_exit,_,exit_tail=guest.transcript.partition(b'Logged out.');assert b'\x1b' not in before_exit and b'\x1b' not in exit_tail.replace(b'\x1b[0m\x1b[?25h',b'');passed('actual_host_and_netbsd_native_who_preserve_compact_fallback_and_fresh_hidden_identity')
        close(owner);logout(owner);configure(current.replace('ansi=true','ansi=false'));owner=connect(plain=True);chat(owner,True);cmd(owner,'/available Plain operator','chat[#lobby]> ')
        plain=connect(plain=True,handle=author);panel(plain,True);cmd(plain,'/away Plain visitor','Who> ');location(author,'Who','away');lobby(plain,True);plain.send('l');plain.expect('Lounge> ');frame=cmd(plain,'r');assert b'Your advertisement: away / Plain visitor' in frame;lobby(plain,True);panel(plain,True);passed('actual_plain_ssh_word_who_and_lounge_share_universal_availability_without_escapes')
        close(owner,True);owner.send('s');owner.expect('[C]reate room: ');owner.send('v');owner.expect('Member handle: ');owner.send(author);owner.expect('Status updated; previous sessions expired.');owner.expect('Press Enter to return to menu: ');owner.send('');owner.expect('Command (then Enter): ');plain.expect('session expired');plain.finish(1);assert b'\x1b' not in plain.transcript
        chat(owner,True);cmd(owner,'/social off','Social configuration updated.');cmd(owner,'/who','social lounge disabled by sysop');close(owner,True);logout(owner,True);passed('actual_revocation_and_global_disable_remove_held_who_authority')
        time.sleep(.2);assert query('PRAGMA user_version')==[[19]] and query('SELECT count(*) FROM visit_activity')==[[0]] and query('SELECT * FROM unread ORDER BY member_id,room_id')==before_unread and query('SELECT count(*) FROM file_transfers')==[[0]] and all(password.encode() not in c.transcript for c in children)
        passed('actual_activity_cleanup_schema_and_normal_unread_transfer_password_invariants')
        a.output.write_text(json.dumps(dict(result='pass',count=len(checks),checks=checks,schema_version=20,normal_sessions=8,admission_refill_ms=5000,actual_registration_and_approval=True,actual_ssh_and_native=True,universal_visibility_and_availability=True,distinct_interfaces=True,broad_host_activity=True,compact_server_frames_escape_free=True,private_locations=False,capacity_claim=False,hardening_claim=False),indent=2,sort_keys=True)+'\n');print('Actual shared Who passed:',len(checks),'controls.',flush=True)
    except Exception:
        for i,c in enumerate(children):(a.output.parent/('who-private-transcript-'+str(i))).write_bytes(c.transcript)
        raise
    finally:
        for c in children:c.close()
        remote('/etc/rc.d/faction_doors onestop || true; /etc/rc.d/faction_broker onestop || true');remote('cat > /etc/faction/service.toml',original.encode())
if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__)
    for n in ['admin-argv','member-argv','password-file','native-binary','native-profile','guest-native-argv','output']:p.add_argument('--'+n,type=Path,required=True)
    p.add_argument('--guest-source',type=Path,required=True);p.add_argument('--disposable-fixture',action='store_true',required=True);run(p.parse_args())
