#!/usr/bin/env python3
"""Actual read-only board guides and live current menu/role filtering on a marked disposable NetBSD board."""
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
        if x.returncode:(a.output.parent/'guide-private-error.log').write_bytes(x.stdout+x.stderr)
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
    before_unread=query('SELECT * FROM unread ORDER BY member_id,room_id');before_posts=query('SELECT count(*) FROM posts')
    from smoke_board_guide import END,F1
    from smoke_wall import settings,SETTINGS
    def leave_settings(c):c.send("q");c.expect(MENU)
    def open_guide(c,plain=False,sequence=F1):
        mark=len(c.transcript);c.send('help') if plain else c.raw(sequence);c.expect('Guide command: ' if plain else END);return c.transcript[mark:]
    def guide(c,key,plain=False):
        mark=len(c.transcript);c.send(key) if plain else c.raw(key.encode());c.expect('Guide command: ' if plain else END);return c.transcript[mark:]
    pack='/var/db/faction-guide-pack';theme='version=1\nid="guide-fixture"\nboard="Signal House"\nguide_title="CREW HANDBOOK"\nguide_notes="Ask before sharing files."\n'
    for action,key,label in [('rooms','R','Room decks'),('new','N','New messages'),('history','H','Room history'),('post','P','Post a message'),('chat','Z','Night talk'),('sysop','S','Sysop controls'),('doors','D','Doors'),('terminal','T','Terminal settings'),('quit','Q','Log out')]:theme+=f'[[menu]]\naction="{action}"\nkey="{key}"\nlabel="{label}"\n'
    try:
        remote('install -d -o factiond -g factionipc -m 700 '+pack);remote('cat > '+pack+'/theme.toml',theme.encode());remote('chown factiond:factionipc '+pack+'/theme.toml; chmod 600 '+pack+'/theme.toml')
        current=original+'theme="'+pack+'"\n';configure(current)
        handle='guide-'+secrets.token_hex(3);c=connect(handle=handle,register=True);c.finish();owner=connect();status(owner,handle,'a')
        frame=open_guide(owner);assert b'Local sysop controls' in frame;frame=guide(owner,'s');assert b'LOCAL SYSOP / BBS ROLE' in frame;back(owner);passed('actual_current_sysop_role_gets_separate_bbs_and_host_operator_guidance')
        for route in ['ssh','native','guest']:
            c=connect(route=route,handle=handle);frame=open_guide(c,sequence=b'\x1b[11~' if route=='guest' else F1);assert b'Local sysop controls' not in frame and b'Files and transfers' not in frame
            frame=guide(c,'b')
            for _ in range(3):frame+=guide(c,'n')
            assert b'[Z] Night talk' in frame and b'[S] Sysop controls' not in frame and b'[D] Doors' not in frame
            guide(c,'l');frame=guide(c,'a');assert b'Ask before sharing files.' in frame;back(c);c.raw(b'?');c.expect(MENU);logout(c)
            passed('actual_'+route+'_guide_uses_current_keys_house_notes_role_filters_and_preserves_refresh')
        c=connect(handle=handle);open_guide(c);settings(owner);owner.send('e');owner.expect(SETTINGS);leave_settings(owner);frame=guide(c,'l');assert b'Community places' in frame;guide(c,'f');settings(owner);owner.send('d');owner.expect(SETTINGS);leave_settings(owner);frame=guide(c,'n');assert b'Choose a topic' in frame and b'Community places' not in frame;back(c)
        passed('actual_live_module_disable_removes_current_topic_without_stale_authority')
        open_guide(c);c.resize(40,18);c.expect(MENU);open_guide(c);guide(c,'c');frame=b''
        for _ in range(7):frame+=guide(c,'n')
        assert b'Ctrl-T asks for SEND confirmation.' in frame
        back(c);logout(c);passed('actual_shared_resize_and_compact_room_reference_remain_bounded_and_read_only')
        native=connect(route='native',handle=handle,plain=True,compact=True);frame=open_guide(native,True);frame+=guide(native,'a',True);assert b'\x1b' not in frame;back(native,True);logout(native,True);passed('actual_native_compact_help_is_plain_and_requires_no_new_remote_frame')
        logout(owner);configure(current.replace('ansi=true','ansi=false'));owner=connect(plain=True);c=connect(plain=True,handle=handle);frame=open_guide(c,True);frame+=guide(c,'b',True);assert b'\x1b' not in frame and b'[Z] Night talk' in frame;guide(c,'l',True);passed('actual_plain_ssh_help_has_the_same_reference_without_terminal_controls')
        owner.send('s');owner.expect('[C]reate room: ');owner.send('v');owner.expect('Member handle: ');owner.send(handle);owner.expect('Status updated; previous sessions expired.');owner.expect('Press Enter to return to menu: ');owner.send('');owner.expect('Command (then Enter): ');c.send('n');c.expect('session expired');c.finish(1);logout(owner,True);passed('actual_revocation_ends_a_held_guide_without_a_cached_approval_bypass')
        time.sleep(.2);assert query('PRAGMA user_version')==[[19]] and query('SELECT * FROM unread ORDER BY member_id,room_id')==before_unread and query('SELECT count(*) FROM posts')==before_posts and query('SELECT count(*) FROM room_drafts')==[[0]] and query('SELECT count(*) FROM file_transfers')==[[0]] and all(password.encode() not in c.transcript for c in children)
        passed('actual_guide_keeps_schema_unread_posts_drafts_transfers_and_passwords_unchanged')
        d=dict(result='pass',count=len(checks),checks=checks,schema_version=19,normal_sessions=8,admission_refill_ms=5000,actual_registration_and_approval=True,actual_ssh_and_native=True,current_menu_keys=True,current_enabled_topics=True,classic_plain_fallback=True,question_mark_refresh_preserved=True,read_only=True,capacity_claim=False,hardening_claim=False)
        a.output.write_text(json.dumps(d,indent=2,sort_keys=True)+'\n');print('Actual shared guides passed:',len(checks),'controls.',flush=True)
    except Exception:
        for i,c in enumerate(children):(a.output.parent/('guide-private-transcript-'+str(i))).write_bytes(c.transcript)
        raise
    finally:
        for c in children:c.close()
        remote('/etc/rc.d/faction_broker onestop || true');remote('cat > /etc/faction/service.toml',original.encode())
if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__)
    for n in ['admin-argv','member-argv','password-file','native-binary','native-profile','guest-native-argv','output']:p.add_argument('--'+n,type=Path,required=True)
    p.add_argument('--guest-source',type=Path,required=True);p.add_argument('--disposable-fixture',action='store_true',required=True);run(p.parse_args())
