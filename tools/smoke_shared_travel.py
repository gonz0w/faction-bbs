#!/usr/bin/env python3
"""Actual native travel visits and budget gates on a marked owned NetBSD fixture."""
import argparse,hashlib,json,secrets,shlex,subprocess,time,tomllib
from pathlib import Path
from check_restricted_ssh import SSHConsole
from smoke_native_client_ssh import NativeConsole
from smoke_room_browser import MENU
from smoke_board_launcher import LauncherConsole

def run(a):
    assert a.disposable_fixture
    admin=json.loads(a.admin_argv.read_text());member=json.loads(a.member_argv.read_text());guest=json.loads(a.guest_native_argv.read_text());password=a.password_file.read_text().strip();children=[];checks=[];calls=0
    def remote(cmd,data=None):
        x=subprocess.run(admin+['set -e; PATH=/usr/sbin:/sbin:/usr/bin:/bin:/usr/pkg/bin; export PATH; test -f /etc/faction-disposable-test; '+cmd],input=data,capture_output=True,timeout=45)
        if x.returncode:(a.output.parent/'travel-private-error.log').write_bytes(x.stdout+x.stderr)
        assert x.returncode==0,'owned travel fixture operation failed';return x.stdout.decode()
    original=remote('cat /etc/faction/service.toml');assert tomllib.loads(original)['state_dir']=='/var/db/faction-profile-fixture';db='/var/db/faction-profile-fixture/faction.sqlite3'
    def query(sql):
        code='import sqlite3,json; d=sqlite3.connect('+repr('file:'+db+'?mode=ro')+',uri=True); print(json.dumps(d.execute('+repr(sql)+').fetchall()))';return json.loads(remote('python3.13 -c '+shlex.quote(code)))
    def sql(code):remote('python3.13 -c '+shlex.quote('import sqlite3;d=sqlite3.connect('+repr(db)+');d.execute("PRAGMA foreign_keys=ON");'+code+';d.commit()'))
    def configure(text):remote('/etc/rc.d/faction_broker onestop');remote('cat > /etc/faction/service.toml',text.encode());remote('/etc/rc.d/faction_broker onestart')
    def connect(route='native',compact=False,limit=16777216,handle='sysop',pending=False):
        nonlocal calls
        if calls:time.sleep(5.1)
        calls+=1
        options=(['--compact'] if compact else [])+['--file-budget',str(limit)]
        if route=='guest':
            argv=guest.copy();assert argv[-1].endswith("'");argv[-1]=argv[-1][:-1]+' '+' '.join(options)+"'"
        else:argv=[str(a.native_binary.resolve()),'--profile',str(a.native_profile.resolve()),*options]
        c=NativeConsole(argv);children.append(c);c.expect('[L]ogin [R]egister [Q]uit: ');c.send('r' if pending else 'l');c.expect('Handle: ');c.send(handle);c.password(password)
        if pending:c.password(password,'Confirm password (hidden): ');c.expect('Registration pending; sysop approval required.');return c
        c.expect('Press Enter to continue: ');c.send('');c.expect('Command (then Enter): ' if compact else MENU);return c
    def logout(c,compact=False):c.send('q') if compact else c.raw(b'q');c.expect('Logged out.');c.finish()
    def files(c,compact=False):c.send('f') if compact else c.raw(b'f');c.expect('Back to the BBS')
    def done(c):c.expect('Enter to return to Files: ');c.send('');c.expect('Back to the BBS')
    def leave(c,compact=False):c.raw(b'b');c.expect('Command (then Enter): ' if compact else MENU)
    def download_deferred(c,reference):
        before=query('select count(*) from file_transfers');c.raw(b'd');c.expect('File ID, faction-file link or recent #number: ');c.send(reference);c.expect('File deferred: ');done(c);assert query('select count(*) from file_transfers')==before
    payload=b'Faction travel payload / original synthetic data.\n';digest=hashlib.sha256(payload).hexdigest();object_id='c'*32;guest_root='/home/factionbuild/travel159-device';source=a.output.parent/'travel-upload.txt';source.write_bytes(payload);source.chmod(0o600)
    try:
        fuid=remote('id -u factionfiles').strip();auid=remote('id -u factionfileauth').strip();current=original+'files_uid='+fuid+'\nfiles_auth_uid='+auid+'\n';configure(current);remote('/etc/rc.d/faction_broker onestop')
        sql("d.execute('DELETE FROM file_transfers');d.execute('DELETE FROM file_objects');d.execute('DELETE FROM file_area_grants');d.execute('DELETE FROM file_areas');d.execute(\"DELETE FROM sqlite_sequence WHERE name='file_areas'\");d.execute('DELETE FROM post_metadata');d.execute('DELETE FROM posts');d.execute('DELETE FROM unread');d.execute(\"DELETE FROM members WHERE handle='traveler'\");d.execute(\"INSERT INTO file_areas(slug,title,private,quota) VALUES('travel','Travel originals',0,10000)\");d.execute(\"INSERT INTO file_areas(slug,title,private,quota) VALUES('hidden','Hidden originals',1,10000)\");d.execute(\"INSERT INTO file_objects VALUES('"+object_id+"',1,1,'travel-original.txt','Original synthetic data',"+str(len(payload))+",'"+digest+"','published')\");d.execute(\"INSERT INTO file_objects VALUES('"+'d'*32+"',2,1,'hidden-original.txt','Private data',1,'"+'a'*64+"','published')\")")
        reference='faction-file:'+query('select id from file_namespace')[0][0]+'/'+object_id;private_reference=reference[:-32]+'d'*32
        remote('cat > /var/db/faction-files/'+object_id,payload);remote('chown factionfiles:factionfiles /var/db/faction-files/'+object_id+'; chmod 600 /var/db/faction-files/'+object_id)
        remote('install -o root -g wheel -m 755 /home/factionbuild/travel159-target/release/faction-files /usr/local/libexec/faction-files; /etc/rc.d/faction_broker onestart; /etc/rc.d/faction_files_sshd onestart')
        key=remote('cat /etc/ssh/ssh_host_ed25519_key.pub').strip();remote('cat >> '+guest_root+'/member-known-hosts',('[10.0.2.15]:2223 '+key+'\n').encode());remote('cat > '+guest_root+'/upload.txt',payload);remote('chown factionbuild:factionbuild '+guest_root+'/upload.txt; chmod 600 '+guest_root+'/upload.txt')
        # An ordinary dumb SSH terminal must remain free of native OSC advertisement.
        dumb=LauncherConsole(['/usr/bin/env','TERM=dumb',*member]);children.append(dumb);dumb.expect('[L]ogin [R]egister [Q]uit: ');dumb.send('l');dumb.expect('Handle: ');dumb.send('sysop');dumb.password(password);dumb.expect('Press Enter to continue: ');dumb.send('');dumb.expect('Command (then Enter): ');dumb.send('f');dumb.expect('files> ');assert b'\x1b' not in dumb.transcript;dumb.send('back');dumb.expect('Command (then Enter): ');logout(dumb,True);calls+=1;checks.append('actual_ordinary_dumb_ssh_files_stays_escape_free_without_native_hint')
        for compact in [False,True]:
            c=connect(compact=compact)
            if not compact:
                c.raw(b'p');c.expect('Room ID: ');c.send('1');c.expect('Reply to post ID (blank for new message): ');c.send('');c.expect('Subject (blank for generated/reply default): ');c.send('Travel content proof');c.expect('Message (one line, up to 1024 bytes): ');c.send('SAME-AUTHORIZED-CONTENT');c.expect('Posted #');c.expect('Press Enter to return to menu: ');c.send('');c.expect(MENU)
            mark=len(c.transcript);c.send('h') if compact else c.raw(b'h');c.expect('Room ID: ');c.send('1');c.expect('SAME-AUTHORIZED-CONTENT');c.expect('More? [y/N]: ');c.send('n');c.expect('Press Enter to return to menu: ');c.send('');c.expect('Command (then Enter): ' if compact else MENU)
            if compact:assert b'\x1b' not in c.transcript[mark:]
            assert query('select last_read from unread where member_id=1 and room_id=1')==query('select max(id) from posts')
            logout(c,compact)
        checks.append('actual_ordinary_and_compact_visits_show_same_content_and_preserve_read_acknowledgment')
        c=connect(compact=True,limit=len(payload)-1);files(c,True);download_deferred(c,reference);before=query('select count(*) from file_objects');c.raw(b'u');c.expect('Local file to upload: ');c.send(str(source));c.expect('File deferred: ');done(c);assert query('select count(*) from file_objects')==before;leave(c,True);logout(c,True);checks.append('actual_host_native_size_ceiling_defers_download_before_grant_and_upload_before_reservation')
        c=connect(limit=0);files(c);download_deferred(c,reference);leave(c);logout(c);checks.append('actual_zero_budget_defers_files_without_disabling_ordinary_board_use')
        c=connect('guest',True,len(payload));files(c,True);c.raw(b'd');c.expect('File ID, faction-file link or recent #number: ');c.send(reference);c.expect('Save as (new local filename): ');c.send(guest_root+'/download.txt');c.expect('Download to this local path? [y/N]: ');c.send('y');c.expect('Transfer complete. SHA256 '+digest);done(c)
        remote('python3.13 -c '+shlex.quote('from pathlib import Path;import hashlib;assert hashlib.sha256(Path('+repr(guest_root+'/download.txt')+').read_bytes()).hexdigest()=='+repr(digest)))
        checks.append('actual_netbsd_compact_exact_ceiling_download_uses_restricted_sftp_and_checksum')
        c.raw(b'u');c.expect('Local file to upload: ');c.send(guest_root+'/upload.txt');c.expect('Area ID: ');c.send('1');c.expect('Remote filename [');c.expect(']: ');c.send('travel-upload.txt');c.expect('Description (max 256 bytes): ');c.send('Original travel proof');c.expect('[y/N]: ');c.send('y');c.expect('Transfer complete. SHA256 '+digest);c.expect('Upload awaits sysop review; it is not published yet.');done(c);assert query("select state from file_objects where filename='travel-upload.txt'")==[['pending']];checks.append('actual_netbsd_exact_ceiling_upload_preserves_pending_review_and_bounded_transfer')
        c.raw(b'd');c.expect('File ID, faction-file link or recent #number: ');c.send(reference);c.expect('Save as (new local filename): ');c.send(guest_root+'/cancelled.txt');c.expect('Download to this local path? [y/N]: ');before=query('select count(*) from file_transfers');c.send('n');done(c);assert query('select count(*) from file_transfers')==before;remote('test ! -e '+guest_root+'/cancelled.txt');leave(c,True);logout(c,True);checks.append('actual_explicit_cancel_does_not_create_download_grant_or_destination')
        pending=connect(compact=True,handle='traveler',pending=True);pending.finish();time.sleep(5.1);plain=SSHConsole(member);children.append(plain);plain.expect('[L]ogin [R]egister [Q]uit: ');plain.send('l');plain.expect('Handle: ');plain.send('sysop');plain.password(password);plain.expect('Press Enter to continue: ');plain.send('');plain.expect(MENU);plain.raw(b's');plain.expect('[C]reate room: ');plain.send('a');plain.expect('Member handle: ');plain.send('traveler');plain.expect('Status updated; previous sessions expired.');plain.expect('Press Enter to return to menu: ');plain.send('');plain.expect(MENU)
        visitor=connect(compact=True,handle='traveler');files(visitor,True);visitor.raw(b'i');visitor.expect('File ID or faction-file link: ');visitor.send(private_reference);visitor.expect('Operation stopped: ');done(visitor);assert b'hidden-original.txt' not in visitor.transcript;leave(visitor,True);logout(visitor,True);logout(plain);checks.append('actual_compact_visit_preserves_local_approval_and_private_file_denial')
        # New CLI flags must not affect pinned host verification or select fallback routes.
        import tomllib as tl
        profile=tl.loads(a.native_profile.read_text());bad=a.output.parent/'wrong-known-hosts';badkey=a.output.parent/'wrong-key';subprocess.run(['/usr/bin/ssh-keygen','-q','-t','ed25519','-N','','-f',str(badkey)],check=True);bad.write_text('['+profile['host']+']:'+str(profile['port'])+' '+badkey.with_suffix('.pub').read_text());bad.chmod(0o600);wrong=a.output.parent/'wrong-profile.toml';wrong.write_text(a.native_profile.read_text().replace(str(profile['known_hosts']),str(bad)));wrong.chmod(0o600)
        time.sleep(5.1);denied=NativeConsole([str(a.native_binary.resolve()),'--profile',str(wrong),'--compact','--file-budget','0']);children.append(denied);denied.expect('HOST IDENTIFICATION HAS CHANGED');denied.finish(1);assert b'[L]ogin' not in denied.transcript;checks.append('actual_compact_mode_keeps_pinned_host_mismatch_denial_and_no_route_retry')
        assert query('pragma user_version')==[[17]] and all(password.encode() not in c.transcript for c in children)
        a.output.write_text(json.dumps(dict(result='pass',count=len(checks),checks=checks,schema_version=17,normal_sessions=8,admission_refill_ms=5000,payload_bytes=len(payload),capacity_claim=False,wire_budget_claim=False,offline_exchange=False,route_change=False),indent=2,sort_keys=True)+'\n');print('Actual shared travel passed:',len(checks),'controls.')
    except Exception:
        for i,c in enumerate(children):(a.output.parent/('travel-private-transcript-'+str(i))).write_bytes(c.transcript)
        raise
    finally:
        for c in children:c.close()
        remote('/etc/rc.d/faction_files_sshd onestop || true; /etc/rc.d/faction_broker onestop || true');remote('cat > /etc/faction/service.toml',original.encode())
if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__)
    for n in ['admin-argv','member-argv','password-file','native-binary','native-profile','guest-native-argv','output']:p.add_argument('--'+n,type=Path,required=True)
    p.add_argument('--disposable-fixture',action='store_true',required=True);run(p.parse_args())
