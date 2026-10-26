#!/usr/bin/env python3
"""Actual SSH/native opt-in social availability and bodyless invitations on a marked disposable NetBSD board."""
import argparse,json,secrets,shlex,subprocess,time,tomllib,select
from pathlib import Path
from check_restricted_ssh import SSHConsole
from smoke_native_client_ssh import NativeConsole
from smoke_room_browser import MENU
from smoke_library import LIST,enter


def run(a):
    assert a.disposable_fixture
    admin=json.loads(a.admin_argv.read_text());member=json.loads(a.member_argv.read_text());guest=json.loads(a.guest_native_argv.read_text());password=a.password_file.read_text().strip();children=[];checks=[];calls=0
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
    def post(c,subject,body):
        c.raw(b'p');c.expect('Room ID: ');c.send('1');c.expect('Reply to post ID (blank for new message): ');c.send('');c.expect('Subject (blank for generated/reply default): ');c.send(subject);c.expect('Message (one line, up to 1024 bytes): ');c.send(body);c.expect('Posted #');c.expect('Press Enter to return to menu: ');c.send('');c.expect(MENU)
    def files(c):c.raw(b'f');c.expect('files> ')
    def file_cmd(c,cmd):c.send(cmd);c.expect('files> ')
    def file_back(c):c.send('back');c.expect(MENU)


    before_unread=query('SELECT * FROM unread ORDER BY member_id,room_id')
    try:
        def cmd(c,line,expected):
            mark=len(c.transcript);c.send(line);c.expect(expected);return c.transcript[mark:]
        def chat(c,plain=False):
            c.send('c') if plain else c.raw(b'c');c.expect('chat[#lobby]> ')
        def close(c,plain=False):
            cmd(c,'/quit','Chat closed.');c.expect('Command (then Enter): ' if plain else MENU)
        def rate_age():sql("d.execute('UPDATE social_cooldowns SET last_page=unixepoch()-31')")
        author='friend-'+secrets.token_hex(3);reader='visitor-'+secrets.token_hex(3)
        for handle in [author,reader]:c=connect(handle=handle,register=True);c.finish()
        owner=connect();status(owner,author,'a');status(owner,reader,'a');chat(owner)
        cmd(owner,'/social','[disabled]');cmd(owner,'/social on','Social configuration updated.');cmd(owner,'/social name Night lounge','Social configuration updated.')
        friend=connect(handle=author);chat(friend);visitor=connect('native',handle=reader);chat(visitor);frame=cmd(visitor,'/who','Not a delivery guarantee.');assert (author+' |').encode() not in frame
        cmd(owner,'/create #studio private','Channel created.');cmd(owner,'/allow #studio '+author,'Channel grant updated; target must join again.');cmd(friend,'/join #studio','Joined; new messages start here.')
        cmd(friend,'/available Original public status','Availability shared');cmd(visitor,'/available Ready for chat','Availability shared');frame=cmd(visitor,'/who','Not a delivery guarantee.');assert (author+' | Chat | available').encode() in frame and b'#studio' not in frame;checks.append('actual_ssh_and_native_explicit_optin_hide_private_channel_and_unadvertised_members')
        friend.raw(b'kept private draft');friend.expect('chat[#studio]> kept private draft');cmd(visitor,'/page '+author,'Page attempted');friend.send('');friend.expect(author+'> kept private draft');frame=cmd(friend,'/pages','Requests/replies expire');assert ('from '+reader+': pending').encode() in frame
        id=query("SELECT id FROM social_pages WHERE state='pending'")[0][0];cmd(friend,f'/answer {id} accept','Page answered.');friend.expect('chat[#studio]> ');frame=cmd(visitor,'/pages','Requests/replies expire');assert ('to '+author+': accepted').encode() in frame;cmd(visitor,'/page '+author,'page cooldown');checks.append('actual_bodyless_page_accept_and_cooldown_preserve_draft_and_channel_selection')
        cmd(friend,'/block '+reader,'Local block preference updated');frame=cmd(visitor,'/who','Not a delivery guarantee.');assert (author+' |').encode() not in frame;rate_age();cmd(visitor,'/page '+author,'Page attempted');assert query("SELECT count(*) FROM social_pages")==[[0]];cmd(friend,'/unblock '+reader,'Local block preference updated')
        cmd(friend,'/mute on','Page preference updated');rate_age();cmd(visitor,'/page '+author,'Page attempted');assert query("SELECT count(*) FROM social_pages")==[[0]];cmd(friend,'/mute off','Page preference updated')
        minute=query("SELECT (unixepoch()/60)%1440")[0][0];end=(minute+2)%1440;cmd(friend,f'/quiet {minute//60:02}:{minute%60:02} {end//60:02}:{end%60:02}','UTC quiet hours updated');rate_age();cmd(visitor,'/page '+author,'Page attempted');assert query("SELECT count(*) FROM social_pages")==[[0]];cmd(friend,'/quiet off','UTC quiet hours updated');checks.append('actual_mutual_block_mute_and_utc_quiet_hours_suppress_invitations_without_existence_oracles')
        frame=cmd(friend,'/page-sysop','Not a delivery guarantee.');assert b'sysop [' not in frame;cmd(owner,'/available Operator on duty','Availability shared');frame=cmd(friend,'/page-sysop','Not a delivery guarantee.');assert b'sysop [available, sysop]' in frame;cmd(friend,'/page-sysop sysop','Page attempted');id=query("SELECT id FROM social_pages WHERE state='pending'")[0][0];cmd(owner,f'/answer {id} decline','Page answered.');frame=cmd(friend,'/pages','Requests/replies expire');assert b'to sysop: declined' in frame;checks.append('actual_sysop_availability_and_decline_are_local_signals_without_host_privileges')
        sql("d.execute('UPDATE social_pages SET created=unixepoch()-61')");frame=cmd(friend,'/pages','Requests/replies expire');assert b'to sysop: declined' not in frame
        cmd(friend,'/block unknown','Local block preference updated');close(friend);logout(friend);frame=cmd(visitor,'/who','Not a delivery guarantee.');assert (author+' |').encode() not in frame;close(visitor);logout(visitor);close(owner);logout(owner);checks.append('actual_aged_request_expiry_disconnect_and_reconnect_never_recreate_presence')
        configure(original.replace('ansi=true','ansi=false'));plain=connect(plain=True,handle=author);chat(plain,True);cmd(plain,'/privacy','Your blocked handles: unknown');cmd(plain,'/available Plain visitor','Availability shared');cmd(plain,'/who','Not a delivery guarantee.');close(plain,True);logout(plain,True);assert b'\x1b' not in plain.transcript;configure(original);checks.append('actual_plain_ssh_preferences_availability_and_pages_use_escape_free_presentation')
        owner=connect();chat(owner);friend=connect(handle=author);chat(friend);cmd(friend,'/available Test revocation','Availability shared');close(owner);status(owner,author,'v');friend.send('/who');friend.expect('session expired');friend.finish(1);chat(owner);frame=cmd(owner,'/who','Not a delivery guarantee.');assert (author+' |').encode() not in frame;cmd(owner,'/social off','Social configuration updated.');cmd(owner,'/who','social lounge disabled by sysop');close(owner);logout(owner);checks.append('actual_current_account_revocation_expires_held_social_entry_and_sysop_disable_clears_visibility')
        assert query('PRAGMA user_version')==[[19]] and query('SELECT * FROM unread ORDER BY member_id,room_id')==before_unread and query('SELECT count(*) FROM file_transfers')==[[0]] and all(password.encode() not in c.transcript for c in children)
        a.output.write_text(json.dumps(dict(result='pass',count=len(checks),checks=checks,schema_version=20,normal_sessions=8,admission_refill_ms=5000,synthetic_clock_aging=True,capacity_claim=False,private_message_bodies=False,boardwide_presence=False,automatic_notifications=False),indent=2,sort_keys=True)+'\n');print('Actual shared social lounge passed:',len(checks),'controls.')
    except Exception:
        for i,c in enumerate(children):(a.output.parent/('social-private-transcript-'+str(i))).write_bytes(c.transcript)
        raise
    finally:
        for c in children:c.close()
        remote('/etc/rc.d/faction_broker onestop || true');remote('cat > /etc/faction/service.toml',original.encode())
if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__)
    for n in ['admin-argv','member-argv','password-file','native-binary','native-profile','guest-native-argv','output']:p.add_argument('--'+n,type=Path,required=True)
    p.add_argument('--guest-source',type=Path,default=Path('/home/factionbuild/library140'))
    p.add_argument('--disposable-fixture',action='store_true',required=True);run(p.parse_args())
