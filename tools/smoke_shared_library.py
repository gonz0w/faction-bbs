#!/usr/bin/env python3
"""Actual SSH/native curated library on a marked disposable NetBSD board."""
import argparse,json,secrets,shlex,subprocess,time,tomllib,select
from pathlib import Path
from check_restricted_ssh import SSHConsole
from smoke_native_client_ssh import NativeConsole
from smoke_room_browser import MENU
from smoke_library import LIST,INFO,LINK,enter,view,pack

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
    def start_bookmark(c,post=1):c.send('a');c.expect('Bookmark post ID (blank cancels): ');c.send(str(post));c.expect(REVIEW)
    def ready_source(c,post=1):c.send('j');c.expect('Open post ID (blank cancels): ');c.send(str(post));c.expect(SOURCE)
    def source_back(c):c.send('q');c.expect(LIST)
    try:
        catalog='/usr/local/share/faction-library-test';remote('install -d -o factiond -g factionipc -m 750 '+catalog)
        fuid=remote('id -u factionfiles').strip();auid=remote('id -u factionfileauth').strip();current=original+'library="'+catalog+'"\nfiles_uid='+fuid+'\nfiles_auth_uid='+auid+'\n'
        remote('/etc/rc.d/faction_broker onestop')
        code="import sys,sqlite3;sys.path.insert(0,"+repr(str(a.guest_source / "tools"))+");from smoke_library import seed;d=sqlite3.connect("+repr(db)+");seed(d);d.execute(\"INSERT INTO rooms VALUES(2,'old-discussion','Old discussion')\");d.commit()";remote('python3.13 -c '+shlex.quote(code))
        namespace=query('SELECT id FROM file_namespace')[0][0];parts=pack(namespace).split('[[book]]');parts=[p.replace('room=1','room=2') if 'id="book-3"' in p else p for p in parts];text='[[book]]'.join(parts)
        remote('cat > '+catalog+'/library.toml',text.encode());remote('chown factiond:factionipc '+catalog+'/library.toml; chmod 640 '+catalog+'/library.toml')
        payload=b'Faction library manual.\n';import hashlib;digest=hashlib.sha256(payload).hexdigest();object_id=f'{1:032x}'
        sql("d.execute('UPDATE file_objects SET bytes=?,sha256=? WHERE id=?',( "+str(len(payload))+","+repr(digest)+","+repr(object_id)+"))")
        remote('cat > /var/db/faction-files/'+object_id,payload);remote('chown factionfiles:factionfiles /var/db/faction-files/'+object_id+'; chmod 600 /var/db/faction-files/'+object_id)
        remote('install -o root -g wheel -m 755 '+str(a.guest_source)+'-target/release/faction-files /usr/local/libexec/faction-files');configure(current);remote('/etc/rc.d/faction_files_sshd onestart')
        first=connect();post(first,'Library discussion','SHELF-DISCUSSION-CONTENT');enter(first);view(first);first.send('n');first.expect(INFO);assert b'Provenance:' in first.transcript;first.send('l');first.expect(LINK);assert ('faction-file:'+namespace+'/').encode() in first.transcript;first.send('q');first.expect(LIST);back(first);logout(first);checks.append('actual_ssh_named_shelves_rights_provenance_and_current_copyable_file_link')
        handle='reader-'+secrets.token_hex(3);pending=connect(handle=handle,register=True);pending.finish();first=connect();status(first,handle,'a');logout(first);visitor=connect(handle=handle);enter(visitor);visitor.send('n');visitor.expect(LIST);visitor.send('e');visitor.expect('Shelf ID (blank cancels): ');assert b'Hidden collection' not in visitor.transcript
        for value in ['PRIVATE-LIBRARY-NOTE','manual-7.txt','manual-8.txt','manual-9.txt','book-10']:assert value.encode() not in visitor.transcript
        visitor.send('-');visitor.expect(LIST);back(visitor);logout(visitor);checks.append('actual_member_shelves_hide_private_pending_removed_and_foreign_metadata_before_paging')
        # Host native transfer handoff must populate its existing recent-link list.
        before=query('SELECT count(*) FROM file_transfers');c=connect('native',limit=0);enter(c);view(c);c.send('f');c.expect('Back to the BBS');c.raw(b'd');c.expect('File ID, faction-file link or recent #number: ');assert b'#1 faction-file:' in c.transcript;c.send('#1');c.expect('File deferred: ');c.expect('Enter to return to Files: ');c.send('');c.expect('Back to the BBS');c.raw(b'b');c.expect(MENU);logout(c);assert query('SELECT count(*) FROM file_transfers')==before;checks.append('actual_host_native_library_to_Files_keeps_recent_link_and_zero_budget_deferral')
        key=remote('cat /etc/ssh/ssh_host_ed25519_key.pub').strip();guest_root=str(a.guest_source)+'-device';remote('cat >> '+guest_root+'/member-known-hosts',('[10.0.2.15]:2223 '+key+'\n').encode())
        c=connect('guest',plain=True,compact=True);enter(c,True);view(c);c.send('f');c.expect('Back to the BBS');c.raw(b'd');c.expect('File ID, faction-file link or recent #number: ');c.send('#1');c.expect('Save as (new local filename): ');c.send(guest_root+'/library-manual.txt');c.expect('Download to this local path? [y/N]: ');c.send('y');c.expect('Transfer complete. SHA256 '+digest);c.expect('Enter to return to Files: ');c.send('');c.expect('Back to the BBS');c.raw(b'b');c.expect('Command (then Enter): ');logout(c,True)
        remote('python3.13 -c '+shlex.quote('from pathlib import Path;import hashlib;assert hashlib.sha256(Path('+repr(guest_root+'/library-manual.txt')+').read_bytes()).hexdigest()=='+repr(digest)));checks.append('actual_netbsd_compact_library_recent_link_download_uses_existing_restricted_sftp')
        configure(current.replace('ansi=true','ansi=false'));c=connect(plain=True,handle=handle);enter(c,True);view(c);c.send('l');c.expect(LINK);c.send('q');c.expect(LIST);back(c,True);logout(c,True);assert b'\x1b' not in c.transcript;configure(current);checks.append('actual_plain_manual_ssh_library_notes_and_links_are_escape_free')
        first=connect();visitor=connect(handle=handle);enter(visitor);files(first);file_cmd(first,'allow 2 '+handle);visitor.send('e');visitor.expect('Shelf ID (blank cancels): ');visitor.send('hidden');visitor.expect(LIST);view(visitor,'book-7');assert b'PRIVATE-LIBRARY-NOTE' in visitor.transcript;mark=len(visitor.transcript);file_cmd(first,'deny 2 '+handle);visitor.send('n');visitor.expect('not found');visitor.expect(LIST);assert b'PRIVATE-LIBRARY-NOTE' not in visitor.transcript[mark:] and b'Hidden collection' not in visitor.transcript[mark:];checks.append('actual_live_grant_revocation_hides_selected_notes_and_private_shelf_names')
        view(visitor);file_cmd(first,'remove '+object_id);mark=len(visitor.transcript);visitor.send('l');visitor.expect('not found');visitor.expect(LIST);assert b'Library entry>' not in visitor.transcript[mark:] and b'manual-1.txt' not in visitor.transcript[mark:];file_back(first);checks.append('actual_publication_withdrawal_returns_to_catalog_without_opening_a_successor')
        assert query('SELECT count(*) FROM unread')==[[0]];view(visitor,'book-2');visitor.send('d');visitor.expect('SHELF-DISCUSSION-CONTENT');visitor.expect('More? [y/N]: ');visitor.send('n');visitor.expect('Press Enter to return to menu: ');visitor.send('');visitor.expect(MENU);assert query('SELECT last_read FROM unread')==query('SELECT max(id) FROM posts');checks.append('actual_discussion_jump_uses_existing_reader_and_only_that_read_acknowledges')
        enter(visitor);view(visitor,'book-3');sql("d.execute('DELETE FROM rooms WHERE id=2')");visitor.send('d');visitor.expect('not found');visitor.expect(LIST);view(visitor,'book-2');status(first,handle,'v');visitor.send('l');visitor.expect('session expired');visitor.finish(1);logout(first);checks.append('actual_deleted_discussion_and_member_revocation_recheck_current_authority')
        remote('/etc/rc.d/faction_broker onestop');remote('cat > '+catalog+'/library.toml',text.replace('enabled=true','enabled=false').encode());remote('/etc/rc.d/faction_broker onestart');c=connect();assert b'Library shelves' not in c.transcript;logout(c)
        remote('/etc/rc.d/faction_broker onestop');remote('cat > '+catalog+'/library.toml',text.replace('rights="BSD-3-Clause synthetic fixture only"','rights=""',1).encode());code="import subprocess;x=subprocess.run(['su','-','factiond','-c','/usr/local/libexec/faction library-check "+catalog+"'],capture_output=True);assert x.returncode!=0";remote('python3.13 -c '+shlex.quote(code));checks.append('actual_owner_disabled_pack_and_invalid_rights_preflight_change_no_member_authority')
        assert query('PRAGMA user_version')==[[18]] and all(password.encode() not in c.transcript for c in children)
        a.output.write_text(json.dumps(dict(result='pass',count=len(checks),checks=checks,schema_version=18,normal_sessions=8,admission_refill_ms=5000,payload_bytes=len(payload),capacity_claim=False,request_desk=False,new_transfer_authority=False,public_library=False),indent=2,sort_keys=True)+'\n');print('Actual shared library passed:',len(checks),'controls.')
    except Exception:
        for i,c in enumerate(children):(a.output.parent/('library-private-transcript-'+str(i))).write_bytes(c.transcript)
        raise
    finally:
        for c in children:c.close()
        remote('/etc/rc.d/faction_files_sshd onestop || true; /etc/rc.d/faction_broker onestop || true');remote('cat > /etc/faction/service.toml',original.encode())
if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__)
    for n in ['admin-argv','member-argv','password-file','native-binary','native-profile','guest-native-argv','output']:p.add_argument('--'+n,type=Path,required=True)
    p.add_argument('--guest-source',type=Path,default=Path('/home/factionbuild/library140'))
    p.add_argument('--disposable-fixture',action='store_true',required=True);run(p.parse_args())
