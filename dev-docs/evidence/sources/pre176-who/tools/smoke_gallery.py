#!/usr/bin/env python3
"""Local/worker curated art journeys on disposable boards; no network or art rights inference."""
import argparse,json,secrets,shutil,sqlite3,subprocess,tempfile
from pathlib import Path
from smoke_local_console import Console
from smoke_member_worker import Worker
from smoke_room_browser import MENU
LIST='Gallery> '
INFO='Work> '
ART='Art> '
def enter(c,plain=False):
    c.send('a') if plain else c.raw(b'a');c.expect(LIST)
def view(c,work='nightwire'):
    c.send('v');c.expect('Work ID (blank cancels): ');c.send(work);c.expect(INFO)
def run(a):
    checks=[];children=[];password=secrets.token_hex(20)
    with tempfile.TemporaryDirectory(prefix='faction-gallery-pty-') as directory:
        root=Path(directory);state=root/'board';pack=root/'gallery';shutil.copytree(Path(__file__).resolve().parents[1]/'gallery/nightwire',pack)
        def query(sql):
            with sqlite3.connect(f'file:{state}/faction.sqlite3?mode=ro',uri=True) as db:return db.execute(sql).fetchall()
        def start(plain=False,worker=False,narrow=False,enabled=True,handle='sysop'):
            flags=['--gallery',str(pack)] if enabled else []
            c=Worker(a.member,state,flags,columns=40 if narrow else 80,rows=18 if narrow else 24) if worker else Console(a.binary,state,'login',['--plain' if plain else '--ansi',*flags],columns=40 if narrow else 80,rows=18 if narrow else 24);children.append(c)
            if worker:c.expect('[L]ogin [R]egister [Q]uit: ');c.send('l')
            c.expect('Handle: ');c.send(handle);c.password(password)
            if handle!='sysop':return c
            if not plain and not narrow:c.expect('Press Enter to continue: ');c.send('')
            c.expect('Command (then Enter): ' if plain else MENU);return c
        def quit(c,plain=False):c.send('q') if plain else c.raw(b'q');c.expect('Logged out.');c.finish()
        def back(c,plain=False):c.send('q');c.expect('Command (then Enter): ' if plain else MENU)
        try:
            c=Console(a.binary,state,'init');children.append(c);c.expect('Handle: ');c.send('sysop');c.password(password,'New password (12+ characters, hidden): ');c.password(password,'Confirm password (hidden): ');c.expect('Lobby created.');c.finish()
            result=subprocess.run([str(a.binary),'gallery-check',str(pack)],capture_output=True);assert result.returncode==0 and b'4 curated works' in result.stdout
            c=start();assert b'Art gallery' in c.transcript;enter(c);c.send('e');c.expect('Exhibit ID (blank cancels): ');c.send('seasons');c.expect(LIST);assert b'Autumn frequencies' in c.transcript;view(c,'autumn');assert b'Artist: Faction contributors' in c.transcript;c.send('n');c.expect(INFO);assert b'Provenance:' in c.transcript;checks.append('optional_menu_catalog_exhibit_and_artist_rights_notes')
            c.send('a');c.expect(ART);assert b'Page 1/3' in c.transcript
            for n in [2,3,3]:c.send('n');c.expect(ART);assert ('Page '+str(n)+'/3').encode() in c.transcript
            c.send('p');c.expect(ART);c.send('t');c.expect('text edition');c.expect(ART);c.send('i');c.expect(INFO);c.send('q');c.expect(LIST);back(c);quit(c);checks.append('canonical_three_page_art_clamps_and_text_toggle_returns_to_notes')
            c=start(True);enter(c,True);view(c,'signal');c.send('a');c.expect('text edition');c.expect(ART);assert b'NIGHTWIRE' in c.transcript;c.send('q');c.expect(LIST);back(c,True);quit(c,True);assert b'\x1b' not in c.transcript;checks.append('ascii_work_and_plain_text_edition_emit_no_ansi')
            c=start(narrow=True);enter(c);view(c);c.send('a');c.expect('text edition');c.expect(ART);c.send('n');c.expect(ART);c.send('p');c.expect(ART);c.send('q');c.expect(LIST);back(c);quit(c);checks.append('narrow_terminal_uses_paged_explicit_text_edition')
            c=start(worker=True);enter(c);view(c,'winter');c.send('a');c.expect(ART);c.send('q');c.expect(LIST);back(c);quit(c);checks.append('broker_worker_reuses_identical_gallery_workflow')
            c=start(enabled=False);assert b'Art gallery' not in c.transcript;quit(c);text=(pack/'gallery.toml').read_text();(pack/'gallery.toml').write_text(text.replace('enabled = true','enabled = false'));c=start();assert b'Art gallery' not in c.transcript;quit(c);(pack/'gallery.toml').write_text(text);checks.append('absent_and_owner_disabled_catalogs_do_not_add_menu_authority')
            (pack/'gallery.toml').write_text(text.replace('display_permission = true','display_permission = false',1));result=subprocess.run([str(a.binary),'gallery-check',str(pack)],capture_output=True);assert result.returncode!=0;result=subprocess.run([str(a.binary),'--state-dir',str(state),'login','--gallery',str(pack)],input=b'',capture_output=True);assert result.returncode!=0 and b'Handle:' not in result.stdout;(pack/'gallery.toml').write_text(text);checks.append('invalid_curator_rights_rejected_before_login_or_display')
            c=Console(a.binary,state,'register');children.append(c);c.expect('Handle: ');c.send('pending');c.password(password,'New password (12+ characters, hidden): ');c.password(password,'Confirm password (hidden): ');c.expect('Registered pending sysop approval.');c.finish();c=start(handle='pending');c.expect('No community access.');c.finish();assert b'Gallery>' not in c.transcript and query('select count(*) from unread')==[(0,)] and query('pragma user_version')==[(17,)] and all(password.encode() not in c.transcript for c in children);checks.append('pending_member_denied_unchanged_schema_and_read_ledger_no_secret_echo')
        finally:
            for c in children:c.close()
    result=dict(result='pass',count=len(checks),checks=checks,schema_version=17,capacity_claim=False,raw_member_art=False,public_art_publication=False)
    if a.output:a.output.write_text(json.dumps(result,indent=2,sort_keys=True)+'\n')
    print('Local/worker gallery passed:',len(checks),'controls.')
if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('binary',type=Path);p.add_argument('member',type=Path);p.add_argument('--output',type=Path);a=p.parse_args();a.binary=a.binary.resolve();a.member=a.member.resolve();run(a)
