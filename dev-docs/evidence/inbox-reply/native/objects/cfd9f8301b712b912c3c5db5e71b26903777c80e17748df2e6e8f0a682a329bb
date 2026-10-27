#!/usr/bin/env python3
"""Actual SSH/native discussion search on a marked disposable NetBSD board."""
import argparse,json,secrets,shlex,subprocess,time,tomllib,select
from pathlib import Path
from check_restricted_ssh import SSHConsole
from smoke_native_client_ssh import NativeConsole
from smoke_room_browser import MENU
from smoke_discussion import SETTINGS,LIST,SOURCE,REVIEW,settings,enter,search,view,bookmark

def run(a):
    assert a.disposable_fixture
    admin=json.loads(a.admin_argv.read_text());member=json.loads(a.member_argv.read_text());guest=json.loads(a.guest_native_argv.read_text());password=a.password_file.read_text().strip();children=[];checks=[];calls=0
    def remote(cmd,data=None):
        x=subprocess.run(admin+['set -e; PATH=/usr/sbin:/sbin:/usr/bin:/bin:/usr/pkg/bin; export PATH; test -f /etc/faction-disposable-test; '+cmd],input=data,capture_output=True,timeout=30)
        if x.returncode:(a.output.parent/'discussion-private-error.log').write_bytes(x.stdout+x.stderr)
        assert x.returncode==0,'owned discussion fixture operation failed';return x.stdout.decode()
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
    def post(c,subject,body):
        c.raw(b'p');c.expect('Room ID: ');c.send('1');c.expect('Reply to post ID (blank for new message): ');c.send('');c.expect('Subject (blank for generated/reply default): ');c.send(subject);c.expect('Message (one line, up to 1024 bytes): ');c.send(body);c.expect('Posted #');c.expect('Press Enter to return to menu: ');c.send('');c.expect(MENU)
    def files(c):c.raw(b'f');c.expect('files> ')
    def file_cmd(c,cmd):c.send(cmd);c.expect('files> ')
    def file_back(c):c.send('back');c.expect(MENU)
    def start_bookmark(c,post=1):c.send('a');c.expect('Bookmark post ID (blank cancels): ');c.send(str(post));c.expect(REVIEW)
    def ready_source(c,post=1):c.send('j');c.expect('Open post ID (blank cancels): ');c.send(str(post));c.expect(SOURCE)
    def source_back(c):c.send('q');c.expect(LIST)
    try:
        fuid=remote('id -u factionfiles').strip();auid=remote('id -u factionfileauth').strip();assert fuid.isdecimal() and auid.isdecimal();current_config=original+'files_uid='+fuid+'\nfiles_auth_uid='+auid+'\n';configure(current_config);remote('/etc/rc.d/faction_broker onestop')
        sql("d.execute('DELETE FROM discussion_bookmarks');d.execute('DELETE FROM post_metadata');d.execute('DELETE FROM posts');d.execute('DELETE FROM unread');d.execute('DELETE FROM file_objects');d.execute('DELETE FROM file_areas');d.execute(\"DELETE FROM sqlite_sequence WHERE name IN ('posts','file_areas','catchup_publications')\");d.execute(\"UPDATE discussion_settings SET enabled=0,name='Find & save',compact=0\");d.execute(\"INSERT INTO file_areas(slug,title,private,quota) VALUES('private','Private manuals',1,10000)\");d.execute(\"INSERT INTO file_objects VALUES('"+'c'*32+"',1,1,'secret-manual.txt','Synthetic test data',1,'"+'a'*64+"','published')\")")
        namespace=query('select id from file_namespace')[0][0];reference='faction-file:'+namespace+'/'+'c'*32;remote('/etc/rc.d/faction_broker onestart')
        first=connect();post(first,'Café guide',' '.join('word%03d'%n for n in range(100))+' FINAL-SHARED-FIND');post(first,'Night frequencies','Meet the underground.');post(first,'Library handoff',reference);settings(first);first.send('e');first.expect(SETTINGS);first.send('n');first.expect('Find label (1-24 ASCII): ');first.send('Nightwire archives');first.expect(SETTINGS);back(first);logout(first);assert query('select count(*) from unread')==[[0]]
        checks.append('actual_ssh_optional_named_policy_and_real_post_composition')
        c=connect('native');enter(c);search(c,'underground',1,'sysop','b');assert b'Night frequencies' in c.transcript;search(c,'Café',1,'sysop','s');assert b'guide' in c.transcript;assert query('select count(*) from unread')==[[0]];back(c);logout(c)
        checks.append('actual_host_native_literal_unicode_subject_body_room_author_filters')
        c=connect('guest');enter(c);ready_source(c);complete=b'FINAL-SHARED-FIND' in c.transcript
        for _ in range(40):
            if complete:break
            c.send('n');c.expect(SOURCE);complete=b'FINAL-SHARED-FIND' in c.transcript
        assert complete and query('select count(*) from unread')==[[0]];source_back(c);bookmark(c,1,save=False);assert query('select count(*) from discussion_bookmarks')==[[0]];bookmark(c,1);bookmark(c,1);assert query('select count(*) from discussion_bookmarks')==[[1]];back(c);logout(c)
        checks.append('actual_netbsd_native_readonly_full_source_and_explicit_idempotent_bookmark')
        configure(current_config.replace('ansi=true','ansi=false'));c=connect(plain=True);enter(c,True);search(c,'Café',field='s');c.send('x');c.expect('FACTION-BOOKMARKS/1');c.expect(LIST);back(c,True);logout(c,True);assert b'\x1b' not in c.transcript;configure(current_config)
        checks.append('actual_plain_unicode_query_and_copyable_reference_export_without_ansi')
        handle='reader-'+secrets.token_hex(3);pending=connect(handle=handle,register=True);pending.finish();first=connect();status(first,handle,'a');visitor=connect(handle=handle);enter(visitor);visitor.send('b');visitor.expect('No matching items');visitor.expect(LIST);assert b'Caf' not in visitor.transcript;search(visitor,'guide',1,'sysop','s');bookmark(visitor,1);assert query('select count(*) from discussion_bookmarks')==[[2]];enter(first)
        checks.append('actual_two_members_cannot_list_or_overwrite_each_others_bookmarks')
        start_bookmark(visitor,2);code='import sqlite3,sys;d=sqlite3.connect('+repr(db)+');d.execute("BEGIN IMMEDIATE");print("ready",flush=True);sys.stdin.readline();d.rollback()';lock=subprocess.Popen(admin+['test -f /etc/faction-disposable-test && python3.13 -c '+shlex.quote(code)],stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=subprocess.PIPE)
        try:
            assert select.select([lock.stdout],[],[],15)[0] and lock.stdout.readline()==b'ready\n';visitor.send('s');visitor.expect('database busy');visitor.expect(REVIEW)
        finally:lock.communicate(b'finish\n',timeout=10)
        visitor.send('s');visitor.expect(LIST);assert query('select count(*) from discussion_bookmarks where post=2')==[[1]]
        checks.append('actual_busy_review_stays_readable_and_explicit_retry_saves_once')
        visitor.send('f');visitor.expect('Literal text (1-64 UTF-8 bytes, /cancel): ');visitor.send('guide');visitor.expect('Room ID (blank all, /cancel): ');visitor.resize(40,18);visitor.expect('Room ID (blank all, /cancel): ');visitor.send('1');visitor.expect('Exact author (blank all, /cancel): ');visitor.send('sysop');visitor.expect('Search S:subject B:body A:both: ');visitor.send('s');visitor.expect(LIST);ready_source(visitor);visitor.send('n');visitor.expect(SOURCE);visitor.resize(80,24);visitor.expect(SOURCE);source_back(visitor)
        checks.append('actual_resize_preserves_completed_query_and_readonly_source_cursor')
        search(visitor,'secret-manual',field='b');assert b'No matching items in this scan.' in visitor.transcript;back(first);files(first);file_cmd(first,'allow 1 '+handle);search(visitor,'secret-manual',field='b');assert b'Library handoff' in visitor.transcript;bookmark(visitor,3);ready_source(visitor,3);mark=len(visitor.transcript);file_cmd(first,'deny 1 '+handle);visitor.send('n');visitor.expect('[file unavailable]');visitor.expect(SOURCE);assert b'secret-manual.txt' not in visitor.transcript[mark:];source_back(visitor);visitor.send('b');visitor.expect(LIST);search(visitor,'c'*32,field='b');assert b'No matching items in this scan.' in visitor.transcript;file_back(first)
        checks.append('actual_file_grant_revocation_filters_query_snippets_bookmarks_and_live_source')
        ready_source(visitor,2);sql("d.execute('DELETE FROM post_metadata WHERE post_id=2');d.execute('DELETE FROM posts WHERE id=2')");mark=len(visitor.transcript);visitor.send('n');visitor.expect('not found');visitor.expect(LIST);assert b'Library handoff' not in visitor.transcript[mark:] and query('select count(*) from discussion_bookmarks where post=2')==[[0]]
        checks.append('actual_deleted_source_is_not_replaced_by_a_successor_and_cascades_bookmarks')
        post(first,'Revocation guard','This source remains local.');start_bookmark(visitor,4);status(first,handle,'v');visitor.send('s');visitor.expect('session expired');visitor.finish(1);assert query('select count(*) from discussion_bookmarks where post=4')==[[0]];assert query('select count(*) from unread')==[[0]]
        first.raw(b'n');first.expect('Room ID: ');first.send('1');mark=len(first.transcript)
        for _ in range(40):
            first.expect('More? [y/N]: ')
            if b'FINAL-SHARED-FIND' in first.transcript[mark:]:break
            first.send('y')
        else:raise AssertionError('ordinary room reader final page missing')
        first.send('n');first.expect('Press Enter to return to menu: ');first.send('');first.expect(MENU);assert query('select last_read from unread where member_id=1')==[[1]]
        checks.append('actual_revocation_blocks_bookmark_while_normal_complete_room_read_still_acks')
        enter(first);search(first,'guide');second=connect();settings(second);second.send('e');second.expect(SETTINGS);back(second);logout(second);mark=len(first.transcript);first.send('r');first.expect(MENU);assert b'Caf' not in first.transcript[mark:];logout(first)
        checks.append('actual_disabled_host_feature_hides_held_query_and_source_results')
        assert query('pragma user_version')==[[19]] and all(password.encode() not in c.transcript for c in children)
        a.output.write_text(json.dumps(dict(result='pass',count=len(checks),checks=checks,schema_version=20,normal_sessions=8,admission_refill_ms=5000,capacity_claim=False,server_mail_search=False,automatic_query_retention=False),indent=2,sort_keys=True)+'\n');print('Actual shared discussion discovery passed:',len(checks),'controls.')
    except Exception:
        for i,c in enumerate(children):(a.output.parent/('discussion-private-transcript-'+str(i))).write_bytes(c.transcript)
        raise
    finally:
        for c in children:c.close()
        remote('/etc/rc.d/faction_broker onestop || true');remote('cat > /etc/faction/service.toml',original.encode())
if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__)
    for n in ['admin-argv','member-argv','password-file','native-binary','native-profile','guest-native-argv','output']:p.add_argument('--'+n,type=Path,required=True)
    p.add_argument('--disposable-fixture',action='store_true',required=True);run(p.parse_args())
