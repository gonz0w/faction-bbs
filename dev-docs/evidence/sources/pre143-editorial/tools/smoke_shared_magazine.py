#!/usr/bin/env python3
"""Actual SSH/native magazine reader on a marked disposable NetBSD board."""
import argparse,json,secrets,shlex,subprocess,time,tomllib,select
from pathlib import Path
from check_restricted_ssh import SSHConsole
from smoke_native_client_ssh import NativeConsole
from smoke_room_browser import MENU
from smoke_library import LIST,enter
from magazine_fixture import pack

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
        catalog='/usr/local/share/faction-magazine-test';remote('install -d -o factiond -g factionipc -m 750 '+catalog)
        fuid=remote('id -u factionfiles').strip();auid=remote('id -u factionfileauth').strip();current=original+'library="'+catalog+'"\nfiles_uid='+fuid+'\nfiles_auth_uid='+auid+'\n'
        remote('/etc/rc.d/faction_broker onestop')
        code="import sys,sqlite3;sys.path.insert(0,"+repr(str(a.guest_source/'tools'))+");from smoke_library import seed;d=sqlite3.connect("+repr(db)+");seed(d);d.commit()";remote('python3.13 -c '+shlex.quote(code))
        namespace=query('SELECT id FROM file_namespace')[0][0];text=pack(namespace)
        def write_pack(text):remote('cat > '+catalog+'/library.toml',text.encode());remote('chown factiond:factionipc '+catalog+'/library.toml; chmod 640 '+catalog+'/library.toml')
        write_pack(text)
        remote('install -d -o factiond -g factionipc -m 700 /var/db/faction-edition-test')
        remote("su - factiond -c '/usr/local/libexec/faction library-edition "+catalog+" signal-1 /var/db/faction-edition-test/issue-001-r1.txt'")
        payload=remote('cat /var/db/faction-edition-test/issue-001-r1.txt').encode();assert b'ORIGINAL-ARTICLE-END' in payload and b'Fixture author' in payload
        import hashlib;digest=hashlib.sha256(payload).hexdigest();object_id=f'{2:032x}'
        sql("d.execute('UPDATE file_objects SET bytes=?,sha256=? WHERE id=?',("+str(len(payload))+","+repr(digest)+","+repr(object_id)+"))")
        remote('cat > /var/db/faction-files/'+object_id,payload);remote('chown factionfiles:factionfiles /var/db/faction-files/'+object_id+'; chmod 600 /var/db/faction-files/'+object_id)
        remote('install -o root -g wheel -m 755 '+str(a.guest_source)+'-target/release/faction-files /usr/local/libexec/faction-files');configure(current);remote('/etc/rc.d/faction_files_sshd onestart')
        def desk(c,plain=False):enter(c,plain);c.send('m');c.expect('Magazine> ')
        def issue(c):c.send('v');c.expect('Issue ID');c.send('signal-1');c.expect('Contents> ')
        def article(c):c.send('r');c.expect('Article ID');c.send('article-1');c.expect('Magazine page> ')
        def return_main(c,plain=False):c.send('q');c.expect('Magazine> ');c.send('q');c.expect(LIST);back(c,plain)
        handle='reader-'+secrets.token_hex(3);c=connect(handle=handle,register=True);c.finish();c=connect();status(c,handle,'a');post(c,'Magazine discussion','ORIGINAL-MAGAZINE-DISCUSSION');logout(c)
        c=connect(handle=handle);desk(c);assert b'HIDDEN-EDITION' not in c.transcript;c.send('v');c.expect('Issue ID');c.send('signal-2');c.expect('not found');c.expect('Magazine> ');issue(c);c.send('n');c.expect('Contents> ');assert b'article-5' in c.transcript;article(c)
        for _ in range(25):
            if b'ORIGINAL-ARTICLE-END' in c.transcript:break
            c.send('n');c.expect('Magazine page> ')
        assert b'ORIGINAL-ARTICLE-END' in c.transcript and b'HIDDEN-ARTICLE-BODY' not in c.transcript;c.send('c');c.expect('Magazine page> ')
        for _ in range(5):
            if b'Declared consent: board-local' in c.transcript:break
            c.send('n');c.expect('Magazine page> ')
        assert b'Declared consent: board-local' in c.transcript;return_main(c);logout(c);assert query('SELECT * FROM unread ORDER BY member_id,room_id')==before_unread;checks.append('actual_member_published_only_toc_complete_reading_and_credits_leave_unread_unchanged')
        configure(current.replace('ansi=true','ansi=false'));c=connect(plain=True,handle=handle);desk(c,True);issue(c);article(c);c.send('t');c.expect('Contents> ');c.send('l');c.expect('Edition link> ');return_main(c,True);logout(c,True);assert b'\x1b' not in c.transcript;configure(current);checks.append('actual_plain_manual_ssh_magazine_and_current_edition_link_are_escape_free')
        before=query('SELECT count(*) FROM file_transfers');c=connect('native',limit=0);desk(c);issue(c);c.send('f');c.expect('Back to the BBS');c.raw(b'd');c.expect('File ID, faction-file link or recent #number: ');assert b'#1 faction-file:' in c.transcript;c.send('#1');c.expect('File deferred: ');c.expect('Enter to return to Files: ');c.send('');c.expect('Back to the BBS');c.raw(b'b');c.expect(MENU);logout(c);assert query('SELECT count(*) FROM file_transfers')==before;checks.append('actual_host_native_magazine_to_Files_keeps_recent_link_and_zero_budget_deferral')
        key=remote('cat /etc/ssh/ssh_host_ed25519_key.pub').strip();guest_root=str(a.guest_source)+'-device';remote('cat >> '+guest_root+'/member-known-hosts',('[10.0.2.15]:2223 '+key+'\n').encode())
        c=connect('guest',plain=True,compact=True);desk(c,True);issue(c);c.send('f');c.expect('Back to the BBS');c.raw(b'd');c.expect('File ID, faction-file link or recent #number: ');c.send('#1');c.expect('Save as (new local filename): ');c.send(guest_root+'/nightwire-edition.txt');c.expect('Download to this local path? [y/N]: ');c.send('y');c.expect('Transfer complete. SHA256 '+digest);c.expect('Enter to return to Files: ');c.send('');c.expect('Back to the BBS');c.raw(b'b');c.expect('Command (then Enter): ');logout(c,True)
        remote('python3.13 -c '+shlex.quote('from pathlib import Path;import hashlib;assert hashlib.sha256(Path('+repr(guest_root+'/nightwire-edition.txt')+').read_bytes()).hexdigest()=='+repr(digest)));checks.append('actual_netbsd_compact_edition_download_uses_existing_sftp_and_exact_bytes')
        remote('/etc/rc.d/faction_broker onestop');write_pack(pack(namespace,True));remote('/etc/rc.d/faction_broker onestart');owner=connect();files(owner);file_cmd(owner,'allow 2 '+handle);visitor=connect(handle=handle);desk(visitor);issue(visitor);visitor.send('l');visitor.expect('Edition link> ');mark=len(visitor.transcript);file_cmd(owner,'deny 2 '+handle);visitor.send('l');visitor.expect('not found');visitor.expect('Magazine> ');assert b'faction-file:' not in visitor.transcript[mark:];visitor.send('q');visitor.expect(LIST);back(visitor);logout(visitor);file_back(owner);logout(owner);checks.append('actual_live_grant_revocation_removes_selected_optional_edition_reference')
        remote('/etc/rc.d/faction_broker onestop');write_pack(text);remote('/etc/rc.d/faction_broker onestart');c=connect(handle=handle);desk(c);issue(c);c.send('d');c.expect('ORIGINAL-MAGAZINE-DISCUSSION');c.expect('More? [y/N]: ');c.send('n');c.expect('Press Enter to return to menu: ');c.send('');c.expect(MENU);logout(c);assert query('SELECT last_read FROM unread')==query('SELECT max(id) FROM posts');checks.append('actual_magazine_discussion_jump_uses_normal_reader_and_only_that_read_acknowledges')
        remote('/etc/rc.d/faction_broker onestop');write_pack(text.replace('state="published"','state="withdrawn"'));remote('/etc/rc.d/faction_broker onestart');c=connect();assert b'Library shelves' not in c.transcript;logout(c)
        remote('/etc/rc.d/faction_broker onestop');write_pack(text.replace('revision=1','revision=2').replace('ORIGINAL-ARTICLE-START','REVISION-TWO-START'));remote('/etc/rc.d/faction_broker onestart');c=connect();desk(c);issue(c);article(c);assert b'REVISION-TWO-START' in c.transcript and b'ORIGINAL-ARTICLE-START' not in c.transcript;return_main(c);logout(c);checks.append('actual_stopped_restart_withdrawal_and_revision_replace_only_the_live_edition')
        owner=connect();visitor=connect(handle=handle);desk(visitor);issue(visitor);article(visitor);status(owner,handle,'v');visitor.send('n');visitor.expect('session expired');visitor.finish(1);logout(owner);checks.append('actual_live_member_revocation_closes_held_magazine_page')
        remote('/etc/rc.d/faction_broker onestop');write_pack(text.replace('consent="board-local"','consent="pending"'));code="import subprocess;x=subprocess.run(['su','-','factiond','-c','/usr/local/libexec/faction library-check "+catalog+"'],capture_output=True);assert x.returncode!=0";remote('python3.13 -c '+shlex.quote(code));checks.append('actual_owner_unconsented_publication_preflight_rejects')
        assert query('PRAGMA user_version')==[[15]] and all(password.encode() not in c.transcript for c in children)
        a.output.write_text(json.dumps(dict(result='pass',count=len(checks),checks=checks,schema_version=15,normal_sessions=8,admission_refill_ms=5000,payload_bytes=len(payload),capacity_claim=False,contributor_authentication=False,member_submission_queue=False,new_transfer_authority=False,public_publication=False),indent=2,sort_keys=True)+'\n');print('Actual shared magazine passed:',len(checks),'controls.')
    except Exception:
        for i,c in enumerate(children):(a.output.parent/('magazine-private-transcript-'+str(i))).write_bytes(c.transcript)
        raise
    finally:
        for c in children:c.close()
        remote('/etc/rc.d/faction_files_sshd onestop || true; /etc/rc.d/faction_broker onestop || true');remote('cat > /etc/faction/service.toml',original.encode())
if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__)
    for n in ['admin-argv','member-argv','password-file','native-binary','native-profile','guest-native-argv','output']:p.add_argument('--'+n,type=Path,required=True)
    p.add_argument('--guest-source',type=Path,default=Path('/home/factionbuild/library140'))
    p.add_argument('--disposable-fixture',action='store_true',required=True);run(p.parse_args())
