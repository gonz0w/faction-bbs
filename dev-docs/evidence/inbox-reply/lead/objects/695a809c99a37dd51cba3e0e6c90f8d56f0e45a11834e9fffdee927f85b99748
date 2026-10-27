#!/usr/bin/env python3
"""Curated local/worker library journeys using owned synthetic metadata only."""
import argparse,json,secrets,sqlite3,subprocess,tempfile
from pathlib import Path
from smoke_local_console import Console
from smoke_member_worker import Worker
from smoke_room_browser import MENU
LIST='Library> '
INFO='Library entry> '
LINK='Library link> '
def pack(namespace):
    source='version=1\nenabled=true\nname="Nightwire library"\ncompact=false\n[[shelf]]\nid="manuals"\ntitle="Public field notes"\n[[shelf]]\nid="hidden"\ntitle="Hidden collection"\n'
    for i in range(1,11):
        ns=namespace if i<10 else 'f'*32
        source+='\n[[book]]\nid="book-'+str(i)+'"\nshelf="'+('hidden' if i==7 else 'manuals')+'"\nreference="faction-file:'+ns+'/'+f'{i:032x}'+'"\nnote='+json.dumps('PRIVATE-LIBRARY-NOTE' if i==7 else 'Original field notes, reviewed by the curator.\nRead and discuss on this board.')+'\nrights="BSD-3-Clause synthetic fixture only"\nprovenance="Original synthetic fixture; curator reviewed."\nroom=1\n'
    return source

def seed(db):
    db.execute("INSERT INTO file_areas VALUES(1,'public','Public manuals',0,10000)");db.execute("INSERT INTO file_areas VALUES(2,'private','Private manuals',1,10000)")
    for i in range(1,10):db.execute('INSERT INTO file_objects VALUES(?,?,1,?,?,1,?,?)',(f'{i:032x}',2 if i==7 else 1,f'manual-{i}.txt','Original useful manual','a'*64,'pending' if i==8 else 'removed' if i==9 else 'published'))
def enter(c,plain=False):c.send('k') if plain else c.raw(b'k');c.expect(LIST)
def view(c,book='book-1'):c.send('v');c.expect('Entry ID (blank cancels): ');c.send(book);c.expect(INFO)
def run(a):
    checks=[];children=[];password=secrets.token_hex(20)
    with tempfile.TemporaryDirectory(prefix='faction-library-pty-') as directory:
        root=Path(directory);state=root/'board';catalog=root/'library';catalog.mkdir()
        def query(sql):
            with sqlite3.connect(f'file:{state}/faction.sqlite3?mode=ro',uri=True) as db:return db.execute(sql).fetchall()
        def write(sql):
            with sqlite3.connect(state/'faction.sqlite3') as db:db.execute(sql)
        def start(plain=False,worker=False,narrow=False,enabled=True,handle='reader'):
            flags=['--library',str(catalog)] if enabled else []
            c=Worker(a.member,state,flags,columns=40 if narrow else 80,rows=18 if narrow else 24) if worker else Console(a.binary,state,'login',['--plain' if plain else '--ansi',*flags],columns=40 if narrow else 80,rows=18 if narrow else 24);children.append(c)
            if worker:c.expect('[L]ogin [R]egister [Q]uit: ');c.send('l')
            c.expect('Handle: ');c.send(handle);c.password(password)
            if handle=='pending':return c
            if not plain and not narrow:c.expect('Press Enter to continue: ');c.send('')
            c.expect('Command (then Enter): ' if plain else MENU);return c
        def quit(c,plain=False):c.send('q') if plain else c.raw(b'q');c.expect('Logged out.');c.finish()
        def back(c,plain=False):c.send('q');c.expect('Command (then Enter): ' if plain else MENU)
        try:
            c=Console(a.binary,state,'init');children.append(c);c.expect('Handle: ');c.send('sysop');c.password(password,'New password (12+ characters, hidden): ');c.password(password,'Confirm password (hidden): ');c.expect('Lobby created.');c.finish()
            with sqlite3.connect(state/'faction.sqlite3') as db:
                db.execute("INSERT INTO members(handle,password_hash,role,status) SELECT 'reader',password_hash,'member','approved' FROM members WHERE id=1");db.execute("INSERT INTO members(handle,password_hash,role,status) SELECT 'pending',password_hash,'member','pending' FROM members WHERE id=1");seed(db);ns=db.execute('SELECT id FROM file_namespace').fetchone()[0]
            text=pack(ns);(catalog/'library.toml').write_text(text);result=subprocess.run([str(a.binary),'library-check',str(catalog)],capture_output=True);assert result.returncode==0 and b'10 curated entries' in result.stdout
            c=start();enter(c);assert b'manual-1.txt' in c.transcript;c.send('n');c.expect(LIST);assert b'manual-6.txt' in c.transcript
            for hidden in ['manual-7.txt','manual-8.txt','manual-9.txt','book-10','PRIVATE-LIBRARY-NOTE']:assert hidden.encode() not in c.transcript
            c.send('e');c.expect('Shelf ID (blank cancels): ');assert b'Hidden collection' not in c.transcript;c.send('manuals');c.expect(LIST);checks.append('optional_named_library_paging_filters_private_pending_removed_and_foreign_entries')
            view(c);seen=b'Provenance:' in c.transcript
            for _ in range(8):
                if seen:break
                c.send('n');c.expect(INFO);seen=b'Provenance:' in c.transcript
            assert seen;c.send('l');c.expect(LINK);assert ('faction-file:'+ns+'/').encode() in c.transcript;c.send('i');c.expect(INFO);c.send('q');c.expect(LIST);back(c);quit(c);checks.append('artist_rights_provenance_paged_notes_and_copyable_current_file_link')
            c=start();enter(c);write('INSERT INTO file_area_grants VALUES(2,2)');c.send('e');c.expect('Shelf ID (blank cancels): ');assert b'Hidden collection' in c.transcript;c.send('hidden');c.expect(LIST);view(c,'book-7');assert b'PRIVATE-LIBRARY-NOTE' in c.transcript;mark=len(c.transcript);write('DELETE FROM file_area_grants');c.send('n');c.expect('not found');c.expect(LIST);assert b'PRIVATE-LIBRARY-NOTE' not in c.transcript[mark:] and b'Hidden collection' not in c.transcript[mark:];back(c);quit(c);checks.append('held_entry_rechecks_private_grant_revocation_and_hides_shelf_metadata')
            c=start(True,narrow=True);enter(c,True);view(c);c.send('l');c.expect(LINK);c.send('q');c.expect(LIST);back(c,True);quit(c,True);assert b'\x1b' not in c.transcript;checks.append('plain_narrow_notes_and_links_emit_no_escape_sequences')
            c=start(worker=True);enter(c);view(c);assert b'F:Files' not in c.transcript;c.send('d');c.expect('No more messages.');c.expect('Press Enter to return to menu: ');c.send('');c.expect(MENU);quit(c);checks.append('worker_reuses_library_and_deliberate_existing_room_reader_without_transfer_authority')
            c=start(enabled=False);assert b'Library shelves' not in c.transcript;quit(c);(catalog/'library.toml').write_text(text.replace('enabled=true','enabled=false'));c=start();assert b'Library shelves' not in c.transcript;quit(c);(catalog/'library.toml').write_text(text);checks.append('absent_and_disabled_pack_do_not_add_main_menu_capability')
            (catalog/'library.toml').write_text(text.replace('rights="BSD-3-Clause synthetic fixture only"','rights=""',1));result=subprocess.run([str(a.binary),'library-check',str(catalog)],capture_output=True);assert result.returncode!=0;result=subprocess.run([str(a.binary),'--state-dir',str(state),'login','--library',str(catalog)],input=b'',capture_output=True);assert result.returncode!=0 and b'Handle:' not in result.stdout;(catalog/'library.toml').write_text(text);checks.append('invalid_curator_metadata_rejected_before_member_login')
            c=start(handle='pending');c.expect('No community access.');c.finish();assert b'Library>' not in c.transcript and query('SELECT count(*) FROM unread')==[(0,)] and query('PRAGMA user_version')==[(20,)] and all(password.encode() not in c.transcript for c in children);checks.append('pending_member_denied_unchanged_schema_and_no_preview_read_ack_or_secret_echo')
        finally:
            for c in children:c.close()
    result=dict(result='pass',count=len(checks),checks=checks,schema_version=20,capacity_claim=False,request_desk=False,new_transfer_authority=False,public_library=False)
    if a.output:a.output.write_text(json.dumps(result,indent=2,sort_keys=True)+'\n')
    print('Local/worker library passed:',len(checks),'controls.')
if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('binary',type=Path);p.add_argument('member',type=Path);p.add_argument('--output',type=Path);a=p.parse_args();a.binary=a.binary.resolve();a.member=a.member.resolve();run(a)
