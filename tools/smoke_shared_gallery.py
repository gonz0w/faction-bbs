#!/usr/bin/env python3
"""Actual SSH/native curated art on a marked disposable NetBSD board."""
import argparse,json,secrets,shlex,subprocess,time,tomllib,select
from pathlib import Path
from check_restricted_ssh import SSHConsole
from smoke_native_client_ssh import NativeConsole
from smoke_room_browser import MENU
from smoke_gallery import LIST,INFO,ART,enter,view

def run(a):
    assert a.disposable_fixture
    admin=json.loads(a.admin_argv.read_text());member=json.loads(a.member_argv.read_text());guest=json.loads(a.guest_native_argv.read_text());password=a.password_file.read_text().strip();children=[];checks=[];calls=0
    def remote(cmd,data=None):
        x=subprocess.run(admin+['set -e; PATH=/usr/sbin:/sbin:/usr/bin:/bin:/usr/pkg/bin; export PATH; test -f /etc/faction-disposable-test; '+cmd],input=data,capture_output=True,timeout=30)
        if x.returncode:(a.output.parent/'gallery-private-error.log').write_bytes(x.stdout+x.stderr)
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
        owner=remote('id -u factiond').strip();pack='/usr/local/share/faction-gallery-test'
        remote('install -d -o factiond -g factionipc -m 750 '+pack)
        source=Path(__file__).resolve().parents[1]/'gallery/nightwire'
        for f in source.iterdir():
            if f.is_file():remote('cat > '+pack+'/'+f.name,f.read_bytes())
        remote('chown -R factiond:factionipc '+pack+'; chmod 640 '+pack+'/*')
        fuid=remote('id -u factionfiles').strip();auid=remote('id -u factionfileauth').strip()
        current_config=original+'gallery="'+pack+'"\nfiles_uid='+fuid+'\nfiles_auth_uid='+auid+'\n';configure(current_config)
        first=connect();enter(first);first.send('e');first.expect('Exhibit ID (blank cancels): ');first.send('seasons');first.expect(LIST);view(first,'autumn');first.send('n');first.expect(INFO);assert b'Provenance:' in first.transcript;first.send('a');first.expect(ART);assert b'Page 1/3' in first.transcript
        for n in [2,3,3]:first.send('n');first.expect(ART);assert ('Page '+str(n)+'/3').encode() in first.transcript
        first.send('q');first.expect(LIST);back(first);logout(first);checks.append('actual_ssh_curated_exhibit_artist_rights_and_three_bounded_canonical_pages')
        c=connect('native');enter(c);view(c);c.send('a');c.expect(ART);c.send('t');c.expect('text edition');c.expect(ART);c.send('i');c.expect(INFO);c.send('q');c.expect(LIST);back(c);logout(c);checks.append('actual_host_native_art_text_toggle_and_artist_notes')
        c=connect('guest');enter(c);view(c,'winter');c.send('a');c.expect(ART);c.send('n');c.expect(ART);c.send('q');c.expect(LIST);back(c);logout(c);checks.append('actual_netbsd_native_uses_same_gallery_frames_and_paging')
        configure(current_config.replace('ansi=true','ansi=false'));c=connect(plain=True);enter(c,True);view(c,'signal');c.send('a');c.expect('text edition');c.expect(ART);assert b'NIGHTWIRE' in c.transcript;c.send('q');c.expect(LIST);back(c,True);logout(c,True);assert b'\x1b' not in c.transcript;configure(current_config);checks.append('actual_plain_ascii_art_edition_never_emits_ansi')
        c=connect();enter(c);view(c);c.send('a');c.expect(ART);mark=len(c.transcript);c.resize(40,18);c.expect('text edition');c.expect(ART);assert b'Page ' not in c.transcript[mark:];c.resize(80,24);c.expect('Page 1/3');c.expect(ART);c.send('q');c.expect(LIST);back(c);logout(c);checks.append('actual_resize_switches_to_text_and_restarts_fitting_art_page')
        # An explicit file offer in a curator-owned synthetic copy grants no access.
        remote('/etc/rc.d/faction_broker onestop')
        sql("d.execute(\"INSERT INTO file_areas(slug,title,private,quota) VALUES('private-art','Private art',1,10000)\");d.execute(\"INSERT INTO file_objects VALUES('\"+'c'*32+\"',1,1,'licensed-art.ans','Synthetic reference only',1,'\"+'a'*64+\"','published')\")")
        namespace=query('select id from file_namespace')[0][0];reference='faction-file:'+namespace+'/'+'c'*32
        manifest=(source/'gallery.toml').read_text().replace('download_permission = false','download_permission = true\ndownload_ref = "'+reference+'"',1);remote('cat > '+pack+'/gallery.toml',manifest.encode());remote('/etc/rc.d/faction_broker onestart')
        handle='art-'+secrets.token_hex(3);pending=connect(handle=handle,register=True);pending.finish();first=connect();status(first,handle,'a');visitor=connect(handle=handle);enter(visitor);view(visitor);visitor.send('l');visitor.expect('Download unavailable on this board.');visitor.expect('Gallery file> ');assert reference.encode() not in visitor.transcript;checks.append('actual_optional_download_cannot_reveal_private_filename_or_reference')
        files(first);file_cmd(first,'allow 1 '+handle);visitor.send('i');visitor.expect(INFO);visitor.send('l');visitor.expect(reference);visitor.expect('Gallery file> ');mark=len(visitor.transcript);file_cmd(first,'deny 1 '+handle);visitor.send('i');visitor.expect(INFO);visitor.send('l');visitor.expect('Download unavailable on this board.');visitor.expect('Gallery file> ');assert b'licensed-art.ans' not in visitor.transcript[mark:] and reference.encode() not in visitor.transcript[mark:];file_back(first);checks.append('actual_download_reference_rechecks_live_file_grant_and_revocation')
        status(first,handle,'v');visitor.send('i');visitor.expect('session expired');visitor.finish(1);logout(first);assert query('select count(*) from unread')==[[0]];checks.append('actual_member_revocation_stops_gallery_without_room_read_acknowledgment')
        # Owner removal/format change takes effect on a reviewed stopped restart.
        remote('/etc/rc.d/faction_broker onestop');remote('cat > '+pack+'/gallery.toml',manifest.replace('formats = ["ansi", "ascii"]','formats = ["ascii"]').encode());remote('/etc/rc.d/faction_broker onestart');c=connect();enter(c);assert b'nightwire: Nightwire' not in c.transcript and b'signal: Signal notes' in c.transcript;back(c);logout(c);checks.append('actual_owner_format_removal_changes_catalog_on_stopped_restart')
        remote('/etc/rc.d/faction_broker onestop');remote('cat > '+pack+'/gallery.toml',manifest.replace('enabled = true','enabled = false').encode());remote('/etc/rc.d/faction_broker onestart');c=connect();assert b'Art gallery' not in c.transcript;logout(c);checks.append('actual_disabled_pack_removes_main_menu_capability')
        remote('/etc/rc.d/faction_broker onestop');remote('cat > '+pack+'/gallery.toml',manifest.replace('display_permission = true','display_permission = false',1).encode());code="import subprocess; x=subprocess.run(['su','-','factiond','-c','/usr/local/libexec/faction gallery-check "+pack+"'],capture_output=True);assert x.returncode!=0";remote('python3.13 -c '+shlex.quote(code));checks.append('actual_netbsd_offline_preflight_rejects_missing_display_rights')
        assert query('pragma user_version')==[[16]] and all(password.encode() not in c.transcript for c in children)
        a.output.write_text(json.dumps(dict(result='pass',count=len(checks),checks=checks,schema_version=16,normal_sessions=8,admission_refill_ms=5000,capacity_claim=False,raw_member_art=False,public_art_publication=False),indent=2,sort_keys=True)+'\n');print('Actual shared gallery passed:',len(checks),'controls.')
    except Exception:
        for i,c in enumerate(children):(a.output.parent/('gallery-private-transcript-'+str(i))).write_bytes(c.transcript)
        raise
    finally:
        for c in children:c.close()
        remote('/etc/rc.d/faction_broker onestop || true');remote('cat > /etc/faction/service.toml',original.encode())
if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__)
    for n in ['admin-argv','member-argv','password-file','native-binary','native-profile','guest-native-argv','output']:p.add_argument('--'+n,type=Path,required=True)
    p.add_argument('--disposable-fixture',action='store_true',required=True);run(p.parse_args())
