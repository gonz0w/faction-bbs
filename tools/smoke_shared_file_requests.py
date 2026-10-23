#!/usr/bin/env python3
"""Actual SSH/native file-request desk on a marked disposable NetBSD board."""
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
    before_unread=query('SELECT * FROM unread ORDER BY member_id,room_id')
    try:
        catalog='/usr/local/share/faction-request-test';remote('install -d -o factiond -g factionipc -m 750 '+catalog)
        fuid=remote('id -u factionfiles').strip();auid=remote('id -u factionfileauth').strip();current=original+'library="'+catalog+'"\nfiles_uid='+fuid+'\nfiles_auth_uid='+auid+'\n'
        remote('/etc/rc.d/faction_broker onestop')
        code="import sys,sqlite3;sys.path.insert(0,"+repr(str(a.guest_source/'tools'))+");from smoke_library import seed;d=sqlite3.connect("+repr(db)+");n=d.execute('SELECT count(*) FROM file_areas').fetchone()[0];seed(d) if n==0 else None;d.commit()";remote('python3.13 -c '+shlex.quote(code))
        namespace=query('SELECT id FROM file_namespace')[0][0];text='version=1\nenabled=true\nname="Request library"\ncompact=false\nrequests=true\nrequests_name="Wanted files"\nshelf=[]\nbook=[]\n'
        remote('cat > '+catalog+'/library.toml',text.encode());remote('chown factiond:factionipc '+catalog+'/library.toml; chmod 640 '+catalog+'/library.toml')
        payload=b'Original requested NetBSD manual.\n';import hashlib;digest=hashlib.sha256(payload).hexdigest();object_id=f'{2:032x}'
        sql("d.execute('UPDATE file_objects SET bytes=?,sha256=? WHERE id=?',("+str(len(payload))+","+repr(digest)+","+repr(object_id)+"))")
        remote('cat > /var/db/faction-files/'+object_id,payload);remote('chown factionfiles:factionfiles /var/db/faction-files/'+object_id+'; chmod 600 /var/db/faction-files/'+object_id)
        remote('install -o root -g wheel -m 755 '+str(a.guest_source)+'-target/release/faction-files /usr/local/libexec/faction-files');configure(current);remote('/etc/rc.d/faction_files_sshd onestart')
        def desk(c,plain=False):enter(c,plain);c.send('r');c.expect('Requests> ')
        def create(c,confirm=True):
            for value,prompt in [('c','Request title'),('Wanted NetBSD manual','What are you looking for?'),('Original permitted documentation please','Post request? YES or Q to cancel: ')]:c.send(value);c.expect(prompt)
            c.send('yes' if confirm else 'q');c.expect('Request> ' if confirm else 'Requests> ')
        def request(c,id):c.send('v');c.expect('Request ID');c.send(str(id));c.expect('Request> ')
        def return_main(c,plain=False):c.send('q');c.expect('Requests> ');c.send('q');c.expect(LIST);back(c,plain)
        def offer(c,object):
            for value,prompt in [('u','Published local file ID'),(object,'Declared distribution rights'),('BSD-3-Clause original fixture','Source/provenance'),('Written by fixture author','Fulfill request? YES or Q to cancel: ')]:c.send(value);c.expect(prompt)
        handle='requester-'+secrets.token_hex(3);c=connect(handle=handle,register=True);c.finish();c=connect();status(c,handle,'a');logout(c)
        c=connect(handle=handle);desk(c);create(c,False);assert query('SELECT count(*) FROM file_requests')==[[0]];create(c);id=query('SELECT id FROM file_requests')[0][0];return_main(c);logout(c);checks.append('actual_ssh_member_request_review_cancel_create_and_persisted_reconnect')
        c=connect();desk(c);request(c,id);offer(c,f'{7:032x}');c.send('yes');c.expect('file unavailable to requester');c.expect('Fulfill request? YES or Q to cancel: ');assert query('SELECT state FROM file_requests WHERE id='+str(id))==[['open']];c.send('q');c.expect('Requests> ');request(c,id);offer(c,object_id);c.send('yes');c.expect('Requests> ');request(c,id);c.send('l');c.expect('Request link> ');assert ('faction-file:'+namespace+'/'+object_id).encode() in c.transcript;c.send('q');c.expect('Request> ');return_main(c);logout(c);checks.append('actual_sysop_fulfillment_rejects_requester_private_access_then_confirms_published_file')
        before=query('SELECT count(*) FROM file_transfers');c=connect('native',limit=0);desk(c);request(c,id);c.send('f');c.expect('Back to the BBS');c.raw(b'd');c.expect('File ID, faction-file link or recent #number: ');assert b'#1 faction-file:' in c.transcript;c.send('#1');c.expect('File deferred: ');c.expect('Enter to return to Files: ');c.send('');c.expect('Back to the BBS');c.raw(b'b');c.expect(MENU);logout(c);assert query('SELECT count(*) FROM file_transfers')==before;checks.append('actual_native_request_to_Files_recent_link_respects_zero_payload_ceiling')
        key=remote('cat /etc/ssh/ssh_host_ed25519_key.pub').strip();guest_root=str(a.guest_source)+'-device';remote('cat >> '+guest_root+'/member-known-hosts',('[10.0.2.15]:2223 '+key+'\n').encode())
        c=connect('guest',plain=True,compact=True);desk(c,True);request(c,id);c.send('f');c.expect('Back to the BBS');c.raw(b'd');c.expect('File ID, faction-file link or recent #number: ');c.send('#1');c.expect('Save as (new local filename): ');c.send(guest_root+'/requested-manual.txt');c.expect('Download to this local path? [y/N]: ');c.send('y');c.expect('Transfer complete. SHA256 '+digest);c.expect('Enter to return to Files: ');c.send('');c.expect('Back to the BBS');c.raw(b'b');c.expect('Command (then Enter): ');logout(c,True)
        remote('python3.13 -c '+shlex.quote('from pathlib import Path;import hashlib;assert hashlib.sha256(Path('+repr(guest_root+'/requested-manual.txt')+').read_bytes()).hexdigest()=='+repr(digest)));checks.append('actual_netbsd_compact_request_fulfillment_download_uses_restricted_sftp_and_exact_bytes')
        configure(current.replace('ansi=true','ansi=false'));c=connect(plain=True,handle=handle);desk(c,True);request(c,id);c.send('n');c.expect('Request> ');c.send('l');c.expect('Request link> ');c.send('q');c.expect('Request> ');return_main(c,True);logout(c,True);assert b'\x1b' not in c.transcript;configure(current);checks.append('actual_plain_manual_ssh_request_details_and_links_are_escape_free')
        owner=connect();files(owner);file_cmd(owner,'allow 2 '+handle);file_back(owner);visitor=connect(handle=handle);desk(visitor);create(visitor);id2=query('SELECT max(id) FROM file_requests')[0][0];return_main(visitor);logout(visitor);desk(owner);request(owner,id2);offer(owner,f'{7:032x}');owner.send('yes');owner.expect('Requests> ');request(owner,id2);return_main(owner)
        visitor=connect(handle=handle);desk(visitor);request(visitor,id2);visitor.send('l');visitor.expect('Request link> ');visitor.send('q');visitor.expect('Request> ');files(owner);file_cmd(owner,'deny 2 '+handle);mark=len(visitor.transcript);visitor.send('l');visitor.expect('use displayed request commands');visitor.expect('Request> ');assert b'faction-file:' not in visitor.transcript[mark:] and b'Written by fixture author' not in visitor.transcript[mark:];file_back(owner);checks.append('actual_live_area_revocation_removes_held_fulfillment_link_and_provenance')
        visitor.send('x');visitor.expect('Close request? YES or Q: ');desk(owner);request(owner,id2);owner.send('x');owner.expect('Close request? YES or Q: ');owner.send('yes');owner.expect('Requests> ');visitor.send('yes');visitor.expect('request changed; reopen it');visitor.expect('Requests> ');assert query('SELECT state FROM file_requests WHERE id='+str(id2))==[['closed']];checks.append('actual_concurrent_close_rejects_stale_review_without_replacing_revision')
        request(owner,id2);owner.send('z');owner.expect('Delete request permanently');owner.send('yes');owner.expect('Requests> ');assert query('SELECT count(*) FROM file_requests WHERE id='+str(id2))==[[0]];owner.send('q');owner.expect(LIST);back(owner);request(visitor,id);status(owner,handle,'v');visitor.send('l');visitor.expect('session expired');visitor.finish(1);logout(owner);checks.append('actual_sysop_explicit_delete_and_live_member_revocation_keep_current_authority')
        assert query('PRAGMA user_version')==[[16]] and query('SELECT * FROM unread ORDER BY member_id,room_id')==before_unread and all(password.encode() not in c.transcript for c in children)
        a.output.write_text(json.dumps(dict(result='pass',count=len(checks),checks=checks,schema_version=16,normal_sessions=8,admission_refill_ms=5000,payload_bytes=len(payload),capacity_claim=False,new_transfer_authority=False,public_library=False),indent=2,sort_keys=True)+'\n');print('Actual shared file requests passed:',len(checks),'controls.')
    except Exception:
        for i,c in enumerate(children):(a.output.parent/('requests-private-transcript-'+str(i))).write_bytes(c.transcript)
        raise
    finally:
        for c in children:c.close()
        remote('/etc/rc.d/faction_files_sshd onestop || true; /etc/rc.d/faction_broker onestop || true');remote('cat > /etc/faction/service.toml',original.encode())
if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__)
    for n in ['admin-argv','member-argv','password-file','native-binary','native-profile','guest-native-argv','output']:p.add_argument('--'+n,type=Path,required=True)
    p.add_argument('--guest-source',type=Path,default=Path('/home/factionbuild/library140'))
    p.add_argument('--disposable-fixture',action='store_true',required=True);run(p.parse_args())
