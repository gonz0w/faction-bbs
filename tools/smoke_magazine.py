#!/usr/bin/env python3
"""File-request desk journeys on fresh owned local/worker boards."""
import argparse,json,secrets,sqlite3,subprocess,tempfile
from pathlib import Path
from smoke_local_console import Console
from smoke_member_worker import Worker
from smoke_room_browser import MENU
LIST='Library> '
INFO='Library entry> '
LINK='Library link> '
from magazine_fixture import pack
from smoke_library import seed,enter
def run(a):
    checks=[];children=[];password=secrets.token_hex(20)
    with tempfile.TemporaryDirectory(prefix='faction-magazine-pty-') as directory:
        root=Path(directory);state=root/'board';catalog=root/'library';catalog.mkdir()
        def query(sql):
            with sqlite3.connect(f'file:{state}/faction.sqlite3?mode=ro',uri=True) as db:return db.execute(sql).fetchall()
        def write(sql):
            with sqlite3.connect(state/'faction.sqlite3') as db:db.execute(sql)
        def start(plain=False,worker=False,handle='reader',has_library=True):
            flags=['--library',str(catalog)]
            c=Worker(a.member,state,flags,columns=40,rows=18) if worker else Console(a.binary,state,'login',['--plain' if plain else '--ansi',*flags],columns=40,rows=18);children.append(c)
            if worker:c.expect('[L]ogin [R]egister [Q]uit: ');c.send('l')
            c.expect('Handle: ');c.send(handle);c.password(password)
            if handle=='pending':return c
            c.expect('Command (then Enter): ' if plain else MENU)
            if has_library:enter(c,plain);c.send('m');c.expect('Magazine> ')
            return c
        def issue(c):c.send('v');c.expect('Issue ID');c.send('signal-1');c.expect('Contents> ')
        def article(c):c.send('r');c.expect('Article ID');c.send('article-1');c.expect('Magazine page> ')
        def back(c,plain=False):c.send('q');c.expect('Magazine> ');c.send('q');c.expect(LIST);c.send('q');c.expect('Command (then Enter): ' if plain else MENU);c.send('q') if plain else c.raw(b'q');c.expect('Logged out.');c.finish()
        try:
            c=Console(a.binary,state,'init');children.append(c);c.expect('Handle: ');c.send('sysop');c.password(password,'New password (12+ characters, hidden): ');c.password(password,'Confirm password (hidden): ');c.expect('Lobby created.');c.finish()
            with sqlite3.connect(state/'faction.sqlite3') as db:
                for name,status in [('reader','approved'),('pending','pending')]:db.execute("INSERT INTO members(handle,password_hash,role,status) SELECT ?,password_hash,'member',? FROM members WHERE id=1",(name,status))
                seed(db);ns=db.execute('SELECT id FROM file_namespace').fetchone()[0]
            text=pack(ns);(catalog/'library.toml').write_text(text)
            c=start();assert b'HIDDEN-EDITION' not in c.transcript;c.send('v');c.expect('Issue ID');c.send('signal-2');c.expect('not found');c.expect('Magazine> ');assert b'HIDDEN-ARTICLE-BODY' not in c.transcript;issue(c);c.send('n');c.expect('Contents> ');assert b'article-5' in c.transcript;article(c)
            for _ in range(25):
                if b'ORIGINAL-ARTICLE-END' in c.transcript:break
                c.send('n');c.expect('Magazine page> ')
            assert b'ORIGINAL-ARTICLE-END' in c.transcript;c.send('p');c.expect('Magazine page> ');c.resize(20,18);c.send('n');c.expect('Magazine page> ');c.resize(40,18);c.send('c');c.expect('Magazine page> ')
            for _ in range(10):
                if b'Declared consent: board-local' in c.transcript:break
                c.send('n');c.expect('Magazine page> ')
            assert b'Declared consent: board-local' in c.transcript;back(c);checks.append('named_published_editions_only_toc_paging_complete_articles_credits_and_resize')
            c=start(True);issue(c);article(c);c.send('t');c.expect('Contents> ');c.send('l');c.expect('Edition link> ');assert ('faction-file:'+ns+'/').encode() in c.transcript;back(c,True);assert b'\x1b' not in c.transcript;checks.append('plain_narrow_article_and_current_edition_link_emit_no_escape_bytes')
            c=start(worker=True);issue(c);article(c);assert b'F:Files' not in c.transcript;back(c);checks.append('worker_reuses_common_magazine_without_transfer_adapter')
            (catalog/'library.toml').write_text(pack(ns,True));c=start();issue(c);assert b'L:edition link' not in c.transcript;write('INSERT INTO file_area_grants VALUES(2,2)');c.send('i');c.expect('Magazine page> ');c.send('t');c.expect('Contents> ');c.send('l');c.expect('Edition link> ');mark=len(c.transcript);write('DELETE FROM file_area_grants');c.send('l');c.expect('not found');c.expect('Magazine> ');assert b'faction-file:' not in c.transcript[mark:];c.send('q');c.expect(LIST);c.send('q');c.expect(MENU);c.raw(b'q');c.expect('Logged out.');c.finish();checks.append('optional_edition_file_grants_rechecked_without_hiding_deliberately_published_articles')
            (catalog/'library.toml').write_text(text.replace('state="published"','state="withdrawn"'));c=start(has_library=False);assert b'Library shelves' not in c.transcript;c.raw(b'q');c.expect('Logged out.');c.finish();(catalog/'library.toml').write_text(text.replace('consent="board-local"','consent="pending"'));x=subprocess.run([str(a.binary),'library-check',str(catalog)],capture_output=True);assert x.returncode!=0;x=subprocess.run([str(a.binary),'--state-dir',str(state),'login','--library',str(catalog)],input=b'',capture_output=True);assert x.returncode!=0 and b'Handle:' not in x.stdout;(catalog/'library.toml').write_text(text);checks.append('withdrawal_disables_magazine_only_pack_and_unconsented_release_rejects_before_login')
            out=root/'edition.txt';x=subprocess.run([str(a.binary),'library-preview',str(catalog),'signal-2'],capture_output=True);assert x.returncode==0 and b'[draft]' in x.stdout and b'HIDDEN-ARTICLE-BODY' in x.stdout
            for id,ok in [('signal-2',False),('signal-1',True),('signal-1',False)]:
                x=subprocess.run([str(a.binary),'library-edition',str(catalog),id,str(out)],capture_output=True);assert (x.returncode==0)==ok
            assert out.stat().st_mode&0o777==0o600 and b'ORIGINAL-ARTICLE-END' in out.read_bytes();out.unlink();out.symlink_to(root/'missing');x=subprocess.run([str(a.binary),'library-edition',str(catalog),'signal-1',str(out)],capture_output=True);assert x.returncode!=0 and not (root/'missing').exists();checks.append('owner_plain_draft_preview_published_text_export_private_no_overwrite_or_symlink_follow')
            c=start(handle='pending');c.expect('No community access.');c.finish();assert query('PRAGMA user_version')==[(18,)] and query('SELECT count(*) FROM unread')==[(0,)] and query('SELECT count(*) FROM file_transfers')==[(0,)] and all(password.encode() not in c.transcript for c in children);checks.append('pending_denied_no_schema_unread_transfer_or_secret_echo_side_effects')
        finally:
            for c in children:c.close()
    result=dict(result='pass',count=len(checks),checks=checks,schema_version=18,capacity_claim=False,contributor_authentication=False,member_submission_queue=False,new_transfer_authority=False,public_publication=False)
    if a.output:a.output.write_text(json.dumps(result,indent=2,sort_keys=True)+'\n')
    print('Local/worker magazine passed:',len(checks),'controls.')
if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('binary',type=Path);p.add_argument('member',type=Path);p.add_argument('--output',type=Path);a=p.parse_args();a.binary=a.binary.resolve();a.member=a.member.resolve();run(a)
