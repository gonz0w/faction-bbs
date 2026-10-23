#!/usr/bin/env python3
"""Actual SSH/native catch-up on an explicitly marked disposable NetBSD board."""
import argparse,json,secrets,shlex,subprocess,time,tomllib,select
from pathlib import Path
from check_restricted_ssh import SSHConsole
from smoke_native_client_ssh import NativeConsole
from smoke_room_browser import MENU
from smoke_catchup import SETTINGS,LIST,settings,enter,section,jump

def run(a):
    assert a.disposable_fixture
    admin=json.loads(a.admin_argv.read_text());member=json.loads(a.member_argv.read_text());guest=json.loads(a.guest_native_argv.read_text());password=a.password_file.read_text().strip();children=[];checks=[];calls=0
    def remote(cmd,data=None):
        x=subprocess.run(admin+['set -e; PATH=/usr/sbin:/sbin:/usr/bin:/bin:/usr/pkg/bin; export PATH; test -f /etc/faction-disposable-test; '+cmd],input=data,capture_output=True,timeout=30)
        if x.returncode:(a.output.parent/'catchup-private-error.log').write_bytes(x.stdout+x.stderr)
        assert x.returncode==0,'owned catch-up fixture operation failed';return x.stdout.decode()
    original=remote('cat /etc/faction/service.toml');assert tomllib.loads(original)['state_dir']=='/var/db/faction-profile-fixture';db='/var/db/faction-profile-fixture/faction.sqlite3'
    def query(sql):
        code='import sqlite3,json; d=sqlite3.connect('+repr('file:'+db+'?mode=ro')+',uri=True); print(json.dumps(d.execute('+repr(sql)+').fetchall()))';return json.loads(remote('python3.13 -c '+shlex.quote(code)))
    def connect(route='ssh',plain=False,handle='sysop',register=False):
        nonlocal calls
        if calls:time.sleep(5.1)
        calls+=1;c=SSHConsole(member) if route=='ssh' else NativeConsole(guest if route=='guest' else [str(a.native_binary.resolve()),'--profile',str(a.native_profile.resolve())]);children.append(c)
        c.expect('[L]ogin [R]egister [Q]uit: ');c.send('r' if register else 'l');c.expect('Handle: ');c.send(handle);c.password(password)
        if register:c.password(password,'Confirm password (hidden): ');c.expect('Registration pending; sysop approval required.');return c
        c.expect('Press Enter to continue: ');c.send('');c.expect('Command (then Enter): ' if plain else MENU);return c
    def logout(c,plain=False):c.send('q') if plain else c.raw(b'q');c.expect('Logged out.');c.finish()
    def back(c,plain=False):c.send('q');c.expect('Command (then Enter): ' if plain else MENU)
    def configure(text):remote('/etc/rc.d/faction_broker onestop');remote('cat > /etc/faction/service.toml',text.encode());remote('/etc/rc.d/faction_broker onestart')
    def status(c,handle,key):c.raw(b's');c.expect('[C]reate room: ');c.send(key);c.expect('Member handle: ');c.send(handle);c.expect('Status updated; previous sessions expired.');c.expect('Press Enter to return to menu: ');c.send('');c.expect(MENU)
    def sql(code):remote('python3.13 -c '+shlex.quote('import sqlite3;d=sqlite3.connect('+repr(db)+');d.execute("PRAGMA foreign_keys=ON");'+code+';d.commit()'))
    def post(c,body,reply=''):
        c.raw(b'p');c.expect('Room ID: ');c.send('1');c.expect('Reply to post ID (blank for new message): ');c.send(reply);c.expect('Subject (blank for generated/reply default): ');c.send('Catch-up frequency');c.expect('Message (one line, up to 1024 bytes): ');c.send(body);c.expect('Posted #');c.expect('Press Enter to return to menu: ');c.send('');c.expect(MENU)
    def read_back(c):c.expect('Press Enter to return to menu: ');c.send('');c.expect(MENU)
    def file_cmd(c,cmd):c.send(cmd);c.expect('files> ')
    try:
        remote('/etc/rc.d/faction_broker onestop')
        sql("d.execute('DELETE FROM catchup_members');d.execute('DELETE FROM gatherings');d.execute('DELETE FROM post_metadata');d.execute('DELETE FROM posts');d.execute('DELETE FROM unread');d.execute('DELETE FROM file_objects');d.execute('DELETE FROM file_areas');d.execute(\"DELETE FROM sqlite_sequence WHERE name IN ('posts','gatherings','file_areas','catchup_publications')\");d.execute(\"UPDATE catchup_settings SET enabled=0,name='Catch up',compact=0,notice=''\");d.execute('UPDATE gathering_settings SET enabled=1');d.execute(\"INSERT INTO gatherings(organizer,request_id,revision,title,body,starts_at,ends_at,place,cancelled,hidden,changed_at) VALUES(1,'"+'e'*32+"','"+'f'*32+"','Catch-up club','Bring ANSI art',unixepoch()+3600,unixepoch()+7200,'room:1',0,0,unixepoch())\")")
        remote('/etc/rc.d/faction_broker onestart')
        first=connect();assert query('select count(*) from catchup_members')==[[0]];post(first,' '.join('word%03d'%i for i in range(100))+' FINAL-CATCHUP');settings(first);first.send('e');first.expect(SETTINGS);first.send('n');first.expect('Catch-up label (1-24 ASCII): ');first.send('Nightwire catchup');first.expect(SETTINGS);first.send('a');first.expect('Announcement text> ');first.send('Friday ANSI club.');first.expect('Announcement text> ');first.send('.');first.expect('Announcement review> ');assert query('select notice from catchup_settings')==[['']];first.send('s');first.expect(SETTINGS);back(first);enter(first);assert query('select count(*) from catchup_members')==[[0]];section(first,'files');assert b'Files unavailable in this session.' in first.transcript;back(first);logout(first)
        checks.append('actual_ssh_optional_policy_notice_save_and_no_automatic_visit_tracking')
        c=connect('native');enter(c);section(c,'rooms');assert b'1 unread' in c.transcript;section(c,'replies');assert query('select count(*) from unread')==[[0]];back(c);logout(c);checks.append('actual_host_native_previews_leave_room_read_ledger_untouched')
        c=connect('guest');enter(c);c.send('c');c.expect('Visit review> ');c.send('c');c.expect(LIST);assert query('select count(*) from catchup_members')==[[0]];c.send('c');c.expect('Visit review> ');c.send('s');c.expect(LIST);assert query('select count(*) from catchup_members where saved_at>0')==[[1]];back(c);logout(c);checks.append('actual_netbsd_native_explicit_checkpoint_cancel_and_save')
        configure(original.replace('ansi=true','ansi=false'));c=connect(plain=True);enter(c,True);section(c,'rooms');back(c,True);logout(c,True);assert b'\x1b' not in c.transcript;configure(original);checks.append('actual_plain_catchup_contains_no_ansi')
        handle='reader-'+secrets.token_hex(3);pending=connect(handle=handle,register=True);pending.finish();first=connect();status(first,handle,'a');visitor=connect(handle=handle);post(visitor,'A direct reply from the board.','1');enter(visitor);visitor.send('s');visitor.expect('Subscriptions> ');visitor.send('+1');visitor.expect('Subscriptions> ');visitor.send('q');visitor.expect(LIST);enter(first);section(first,'replies');assert b'1 direct replies' in first.transcript;assert query('select count(*) from catchup_favorites where member=1')==[[0]] and query('select count(*) from unread')==[[0]];back(first);back(visitor);logout(visitor);checks.append('actual_two_members_direct_replies_and_independent_favorites')
        enter(first);first.resize(40,18);first.expect(LIST);section(first,'rooms');jump(first,1);first.expect('More? [y/N]: ');first.send('n');read_back(first);assert query('select count(*) from unread')==[[0]];enter(first);section(first,'rooms');jump(first,1);mark=len(first.transcript)
        for _ in range(30):
            first.expect('More? [y/N]: ')
            if b'FINAL-CATCHUP' in first.transcript[mark:]:break
            first.send('y')
        else:raise AssertionError('complete catch-up source read missing')
        first.send('n');read_back(first);assert query('select last_read from unread where member_id=1')==[[1]];enter(first);first.resize(80,24);first.expect(LIST);checks.append('actual_source_jump_partial_read_preserves_and_complete_drain_acknowledges')
        second=connect();enter(second);second.send('s');second.expect('Subscriptions> ');second.send('+1');second.expect('Subscriptions> ');second.send('q');second.expect(LIST);back(second);logout(second);first.send('c');first.expect('Visit review> ');first.send('s');first.expect('Visit changed elsewhere');first.expect(LIST);assert query('select room from catchup_favorites where member=1')==[[1]];checks.append('actual_concurrent_preferences_refuse_stale_checkpoint')
        first.send('c');first.expect('Visit review> ');code='import sqlite3,sys;d=sqlite3.connect('+repr(db)+');d.execute("BEGIN IMMEDIATE");print("ready",flush=True);sys.stdin.readline();d.rollback()';lock=subprocess.Popen(admin+['test -f /etc/faction-disposable-test && python3.13 -c '+shlex.quote(code)],stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=subprocess.PIPE)
        try:
            assert select.select([lock.stdout],[],[],15)[0] and lock.stdout.readline()==b'ready\n';first.send('s');first.expect('database busy');first.expect('Visit review> ')
        finally:lock.communicate(b'finish\n',timeout=10)
        first.send('s');first.expect(LIST);first.resize(40,18);first.expect(LIST);first.resize(80,24);first.expect(LIST);checks.append('actual_busy_checkpoint_retry_and_resize_keep_explicit_intent')
        section(first,'gatherings');jump(first,1);first.expect('Catch-up club');first.expect('Gathering> ');first.send('q');first.expect('Gatherings> ');back(first);logout(first);checks.append('actual_gathering_jump_uses_existing_current_event_detail')
        fuid=remote('id -u factionfiles').strip();auid=remote('id -u factionfileauth').strip();assert fuid.isdecimal() and auid.isdecimal();file_config=original+'files_uid='+fuid+'\nfiles_auth_uid='+auid+'\n';configure(file_config);remote('/etc/rc.d/faction_broker onestop')
        sql("d.execute(\"INSERT INTO file_areas(slug,title,private,quota) VALUES('catchup-public','Public files',0,10000)\");d.execute(\"INSERT INTO file_areas(slug,title,private,quota) VALUES('catchup-private','Private files',1,10000)\");d.execute(\"INSERT INTO file_objects VALUES('"+'a'*32+"',1,1,'public.txt','Synthetic public file',1,'"+'a'*64+"','published')\");d.execute(\"INSERT INTO file_objects VALUES('"+'b'*32+"',2,1,'private.txt','Synthetic private file',1,'"+'b'*64+"','published')\")")
        remote('/etc/rc.d/faction_broker onestart');first=connect();visitor=connect(handle=handle);enter(visitor);section(visitor,'files');assert b'public.txt' in visitor.transcript and b'private.txt' not in visitor.transcript;first.raw(b'f');first.expect('files> ');file_cmd(first,'allow 2 '+handle);visitor.send('r');visitor.expect('private.txt');visitor.expect(LIST);visitor.send('j');visitor.expect('File object ID (blank cancels): ');file_cmd(first,'deny 2 '+handle);visitor.send('b'*32);visitor.expect('not found');visitor.expect(LIST);first.send('back');first.expect(MENU);checks.append('actual_catalog_preview_and_file_jump_recheck_revoked_private_grant')
        visitor.send('j');visitor.expect('File object ID (blank cancels): ');visitor.send('a'*32);visitor.expect('public.txt');visitor.expect('faction-file:');visitor.send('back');visitor.expect(MENU);enter(visitor);visitor.send('c');visitor.expect('Visit review> ');status(first,handle,'v');visitor.send('s');visitor.expect('session expired');visitor.finish(1);logout(first);checks.append('actual_public_file_jump_emits_existing_reference_and_revocation_blocks_checkpoint')
        assert query('pragma user_version')==[[18]] and all(password.encode() not in c.transcript for c in children)
        a.output.write_text(json.dumps(dict(result='pass',count=len(checks),checks=checks,schema_version=18,normal_sessions=8,admission_refill_ms=5000,capacity_claim=False,automatic_login_tracking=False,server_mail_decryption=False,fresh_sftp_byte_transfer_claim=False),indent=2,sort_keys=True)+'\n');print('Actual shared catch-up passed:',len(checks),'controls.')
    except Exception:
        for i,c in enumerate(children):(a.output.parent/('catchup-private-transcript-'+str(i))).write_bytes(c.transcript)
        raise
    finally:
        for c in children:c.close()
        remote('/etc/rc.d/faction_broker onestop || true');remote('cat > /etc/faction/service.toml',original.encode())
if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__)
    for n in ['admin-argv','member-argv','password-file','native-binary','native-profile','guest-native-argv','output']:p.add_argument('--'+n,type=Path,required=True)
    p.add_argument('--disposable-fixture',action='store_true',required=True);run(p.parse_args())
