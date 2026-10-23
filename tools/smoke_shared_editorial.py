#!/usr/bin/env python3
"""Actual SSH/native authenticated editorial workflow on a marked disposable NetBSD board."""
import argparse,json,secrets,shlex,subprocess,time,tomllib,select
from pathlib import Path
from check_restricted_ssh import SSHConsole
from smoke_native_client_ssh import NativeConsole
from smoke_room_browser import MENU
from smoke_library import LIST,enter
from smoke_editorial import HOME, ISSUE, SOURCE, CONFIRM, FIELDS, send, complete, manuscript, open_issue, create_issue, select as select_source, approve, publish

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
        catalog='/usr/local/share/faction-editorial-test';remote('install -d -o factiond -g factionipc -m 750 '+catalog)
        fuid=remote('id -u factionfiles').strip();auid=remote('id -u factionfileauth').strip();current=original+'library="'+catalog+'"\nfiles_uid='+fuid+'\nfiles_auth_uid='+auid+'\n'
        remote('/etc/rc.d/faction_broker onestop')
        code="import sys,sqlite3;sys.path.insert(0,"+repr(str(a.guest_source/'tools'))+");from smoke_library import seed;d=sqlite3.connect("+repr(db)+");seed(d);d.commit()";remote('python3.13 -c '+shlex.quote(code))
        text='version=1\nenabled=true\nname="Workshop"\ncompact=false\nshelf=[]\nbook=[]\neditorial=true\nmagazine_name="Local zine"\n'
        remote('cat > '+catalog+'/library.toml',text.encode());remote('chown factiond:factionipc '+catalog+'/library.toml; chmod 640 '+catalog+'/library.toml');configure(current)
        def desk(c,plain=False):enter(c,plain);send(c,'w',HOME)
        def return_main(c,plain=False):send(c,'q',LIST);back(c,plain)
        author='author-'+secrets.token_hex(3);reader='reader-'+secrets.token_hex(3)
        for handle in [author,reader]:c=connect(handle=handle,register=True);c.finish()
        c=connect();status(c,author,'a');status(c,reader,'a');logout(c)
        c=connect(handle=author);desk(c);send(c,'s','Submissions> ');c.send('c');manuscript(c,'PRIVATE-EDITORIAL','Original member report.\nORIGINAL-AUTHOR-END',private=True);send(c,'q',HOME);return_main(c);logout(c)
        c=connect(handle=reader);desk(c);send(c,'s','Submissions> ');assert b'PRIVATE-EDITORIAL' not in c.transcript;send(c,'q',HOME);return_main(c);logout(c);checks.append('actual_approved_member_private_source_hidden_from_other_members')
        c=connect(handle=author);desk(c);send(c,'s','Submissions> ');send(c,'v','Submission ID (blank cancels): ');send(c,'1',SOURCE);c.send('e');manuscript(c,'SUBMITTED-EDITORIAL','Member explicitly submits for local review.');send(c,'q',HOME);return_main(c);logout(c)
        c=connect();desk(c);create_issue(c);frame=select_source(c);complete(c,frame,ISSUE);send(c,'publish',CONFIRM);send(c,'yes',CONFIRM);assert query('SELECT state FROM magazine_issues')==[['draft']];send(c,'q',HOME);return_main(c);logout(c);checks.append('actual_submit_select_preview_and_refuse_unapproved_publication')
        c=connect('native',handle=author);desk(c);frame=open_issue(c);approve(c,frame);send(c,'q',HOME);return_main(c);logout(c)
        c=connect();desk(c);open_issue(c);send(c,'e','Slot 1-8 to revise proposal (blank cancels): ');c.send('1');manuscript(c,'APPROVED-EDITORIAL','Exact editor proposal.\nFINAL-APPROVED-END',proposal=True);assert query('SELECT approved=(SELECT token FROM magazine_issues WHERE id=1) FROM magazine_entries')==[[0]];send(c,'q',HOME);return_main(c);logout(c);checks.append('actual_native_author_approval_and_editor_copy_change_invalidates_consent')
        configure(current.replace('ansi=true','ansi=false'));c=connect(plain=True,handle=author);desk(c,True);frame=open_issue(c);approve(c,frame);send(c,'q',HOME);return_main(c,True);logout(c,True);assert b'\x1b' not in c.transcript;configure(current)
        c=connect();desk(c);frame=open_issue(c);publish(c,frame);send(c,'q',HOME);return_main(c);logout(c)
        c=connect(handle=reader);desk(c);frame=open_issue(c);complete(c,frame,ISSUE);assert b'FINAL-APPROVED-END' in c.transcript and ('By '+author).encode() in c.transcript;send(c,'q',HOME);return_main(c);logout(c);assert query('SELECT * FROM unread ORDER BY member_id,room_id')==before_unread;checks.append('actual_plain_exact_consent_publish_credits_and_reading_leave_unread_unchanged')
        remote('install -d -o factiond -g factionipc -m 700 /var/db/faction-edition-test')
        remote("su - factiond -c '/usr/local/libexec/faction --state-dir /var/db/faction-profile-fixture magazine-edition 1 /var/db/faction-edition-test/issue-001-r1.txt'")
        payload=remote('cat /var/db/faction-edition-test/issue-001-r1.txt').encode();assert b'FINAL-APPROVED-END' in payload
        import hashlib;digest=hashlib.sha256(payload).hexdigest();object_id=f'{7:032x}'
        sql("d.execute('UPDATE file_objects SET bytes=?,sha256=? WHERE id=?',("+str(len(payload))+","+repr(digest)+","+repr(object_id)+"))")
        remote('cat > /var/db/faction-files/'+object_id,payload);remote('chown factionfiles:factionfiles /var/db/faction-files/'+object_id+'; chmod 600 /var/db/faction-files/'+object_id)
        remote('install -o root -g wheel -m 755 '+str(a.guest_source)+'-target/release/faction-files /usr/local/libexec/faction-files');remote('/etc/rc.d/faction_files_sshd onestart')
        c=connect();desk(c);open_issue(c);send(c,'o','Published related file ID/link (- removes; blank cancels): ');send(c,object_id,CONFIRM);send(c,'yes',ISSUE);send(c,'q',HOME);return_main(c);logout(c)
        before=query('SELECT count(*) FROM file_transfers');c=connect('native',limit=0);desk(c);open_issue(c);c.send('f');c.expect('Back to the BBS');c.raw(b'd');c.expect('File ID, faction-file link or recent #number: ');assert b'#1 faction-file:' in c.transcript;c.send('#1');c.expect('File deferred: ');c.expect('Enter to return to Files: ');c.send('');c.expect('Back to the BBS');c.raw(b'b');c.expect(MENU);logout(c);assert query('SELECT count(*) FROM file_transfers')==before;checks.append('actual_owner_export_related_file_handoff_keeps_native_recent_link_and_deferral')
        key=remote('cat /etc/ssh/ssh_host_ed25519_key.pub').strip();guest_root=str(a.guest_source)+'-device';remote('cat >> '+guest_root+'/member-known-hosts',('[10.0.2.15]:2223 '+key+'\n').encode())
        c=connect('guest',plain=True,compact=True);desk(c,True);open_issue(c);c.send('f');c.expect('Back to the BBS');c.raw(b'd');c.expect('File ID, faction-file link or recent #number: ');c.send('#1');c.expect('Save as (new local filename): ');c.send(guest_root+'/approved-edition.txt');c.expect('Download to this local path? [y/N]: ');c.send('y');c.expect('Transfer complete. SHA256 '+digest);c.expect('Enter to return to Files: ');c.send('');c.expect('Back to the BBS');c.raw(b'b');c.expect('Command (then Enter): ');logout(c,True)
        remote('python3.13 -c '+shlex.quote('from pathlib import Path;import hashlib;assert hashlib.sha256(Path('+repr(guest_root+'/approved-edition.txt')+').read_bytes()).hexdigest()=='+repr(digest)));checks.append('actual_netbsd_compact_download_matches_exact_consented_edition_via_sftp')
        c=connect(handle=reader);desk(c);frame=open_issue(c);assert b'L:file link' not in frame;send(c,'q',HOME);return_main(c);logout(c)
        owner=connect();files(owner);file_cmd(owner,'allow 2 '+reader);visitor=connect(handle=reader);desk(visitor);open_issue(visitor);send(visitor,'l','Workshop link> ');mark=len(visitor.transcript);file_cmd(owner,'deny 2 '+reader);send(visitor,'l',HOME);assert b'faction-file:' not in visitor.transcript[mark:];return_main(visitor);logout(visitor);file_back(owner);logout(owner);checks.append('actual_related_file_grants_rechecked_on_held_link_without_hiding_articles')
        c=connect(handle=author);desk(c);frame=open_issue(c);approve(c,frame,False);send(c,'q',HOME);return_main(c);logout(c)
        c=connect(handle=reader);desk(c);send(c,'i','Editions> ');assert b'FIRST-ISSUE' not in c.transcript;send(c,'q',HOME);return_main(c);logout(c);assert query('SELECT state FROM magazine_issues')==[['withdrawn']];checks.append('actual_author_revocation_withdraws_issue_existing_download_is_retained')
        owner=connect();visitor=connect(handle=author);desk(visitor);send(visitor,'s','Submissions> ');send(visitor,'v','Submission ID (blank cancels): ');send(visitor,'1',SOURCE);status(owner,author,'v');visitor.send('n');visitor.expect('session expired');visitor.finish(1);logout(owner);checks.append('actual_account_revocation_expires_held_workshop_session')
        assert query('PRAGMA user_version')==[[18]] and all(password.encode() not in c.transcript for c in children)
        a.output.write_text(json.dumps(dict(result='pass',count=len(checks),checks=checks,schema_version=18,normal_sessions=8,admission_refill_ms=5000,payload_bytes=len(payload),capacity_claim=False,public_publication=False,new_transfer_authority=False),indent=2,sort_keys=True)+'\n');print('Actual shared editorial passed:',len(checks),'controls.')
    except Exception:
        for i,c in enumerate(children):(a.output.parent/('editorial-private-transcript-'+str(i))).write_bytes(c.transcript)
        raise
    finally:
        for c in children:c.close()
        remote('/etc/rc.d/faction_files_sshd onestop || true; /etc/rc.d/faction_broker onestop || true');remote('cat > /etc/faction/service.toml',original.encode())
if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__)
    for n in ['admin-argv','member-argv','password-file','native-binary','native-profile','guest-native-argv','output']:p.add_argument('--'+n,type=Path,required=True)
    p.add_argument('--guest-source',type=Path,default=Path('/home/factionbuild/library140'))
    p.add_argument('--disposable-fixture',action='store_true',required=True);run(p.parse_args())
