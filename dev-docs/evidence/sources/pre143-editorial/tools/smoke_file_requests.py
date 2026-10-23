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
    with tempfile.TemporaryDirectory(prefix='faction-requests-pty-') as directory:
        root=Path(directory);state=root/'board';catalog=root/'library';catalog.mkdir()
        def query(sql):
            with sqlite3.connect(f'file:{state}/faction.sqlite3?mode=ro',uri=True) as db:return db.execute(sql).fetchall()
        def start(plain=False,worker=False,handle='reader'):
            flags=['--library',str(catalog)]
            c=Worker(a.member,state,flags,columns=40,rows=24) if worker else Console(a.binary,state,'login',['--plain' if plain else '--ansi',*flags],columns=40,rows=24);children.append(c)
            if worker:c.expect('[L]ogin [R]egister [Q]uit: ');c.send('l')
            c.expect('Handle: ');c.send(handle);c.password(password)
            if handle=='pending':return c
            if worker:c.expect(MENU)
            elif plain:c.expect('Command (then Enter): ')
            else:c.expect(MENU)
            c.send('k') if plain else c.raw(b'k');c.expect(LIST);c.send('r');c.expect('Requests> ');return c
        def create(c,yes=True):
            for value,prompt in [('c','Request title'),('Wanted original manual','What are you looking for?'),('NetBSD tutorial with distribution permission','Post request? YES or Q to cancel: ')]:c.send(value);c.expect(prompt)
            c.send('yes' if yes else 'q');c.expect('Request> ' if yes else 'Requests> ')
        def view_request(c,id):c.send('v');c.expect('Request ID');c.send(str(id));c.expect('Request> ')
        def back(c,plain=False):c.send('q');c.expect('Requests> ');c.send('q');c.expect(LIST);c.send('q');c.expect('Command (then Enter): ' if plain else MENU);c.send('q') if plain else c.raw(b'q');c.expect('Logged out.');c.finish()
        try:
            c=Console(a.binary,state,'init');children.append(c);c.expect('Handle: ');c.send('sysop');c.password(password,'New password (12+ characters, hidden): ');c.password(password,'Confirm password (hidden): ');c.expect('Lobby created.');c.finish()
            with sqlite3.connect(state/'faction.sqlite3') as db:
                for name,status in [('reader','approved'),('other','approved'),('pending','pending')]:db.execute("INSERT INTO members(handle,password_hash,role,status) SELECT ?,password_hash,'member',? FROM members WHERE id=1",(name,status))
                seed(db)
            text='version=1\nenabled=true\nname="Community library"\ncompact=false\nrequests=true\nrequests_name="Wanted files"\nshelf=[]\nbook=[]\n';(catalog/'library.toml').write_text(text)
            c=start();create(c,False);assert query('SELECT count(*) FROM file_requests')==[(0,)];create(c);id=query('SELECT id FROM file_requests')[0][0];assert b'Wanted files' in c.transcript;c.send('q');c.expect('Requests> ');view_request(c,id);back(c);checks.append('request_only_named_desk_explicit_review_cancel_create_view_and_reconnect')
            c=start(handle='other');view_request(c,id);c.send('u');c.expect('Published local file ID');c.send(f'{8:032x}');c.expect('not found');c.expect('Requests> ');view_request(c,id)
            for value,prompt in [('u','Published local file ID'),(f'{1:032x}','Declared distribution rights'),('BSD-3-Clause original fixture','Source/provenance'),('Written by fixture author','Fulfill request? YES or Q to cancel: ')]:c.send(value);c.expect(prompt)
            assert query('SELECT state FROM file_requests')==[('open',)];c.send('yes');c.expect('Requests> ');view_request(c,id);c.send('l');c.expect('Request link> ');assert b'faction-file:' in c.transcript;c.send('q');c.expect('Request> ');back(c);checks.append('other_member_fulfills_only_published_file_with_explicit_rights_source_confirmation')
            c=start(True);view_request(c,id);c.send('n');c.expect('Request> ');c.send('x');c.expect('Close request? YES or Q: ');c.send('yes');c.expect('Requests> ');view_request(c,id);assert query('SELECT state FROM file_requests')==[('closed',)];back(c,True);assert b'\x1b' not in c.transcript;checks.append('plain_narrow_detail_paging_and_author_close_preserve_escape_free_fallback')
            c=start(worker=True);view_request(c,id);assert b'F:Files' not in c.transcript;c.send('z');c.expect('Delete request permanently');c.send('yes');c.expect('Requests> ');assert query('SELECT count(*) FROM file_requests')==[(0,)];c.send('q');c.expect(LIST);c.send('q');c.expect(MENU);c.raw(b'q');c.expect('Logged out.');c.finish();checks.append('worker_same_desk_ownership_and_explicit_delete_without_transfer_adapter')
            c=start(True,handle='sysop');create(c);id=query('SELECT id FROM file_requests')[0][0];c.send('z');c.expect('Delete request permanently');c.send('q');c.expect('Requests> ');view_request(c,id);c.send('z');c.expect('Delete request permanently');c.send('yes');c.expect('Requests> ');c.send('q');c.expect(LIST);c.send('q');c.expect('Command (then Enter): ');c.send('q');c.expect('Logged out.');c.finish();checks.append('sysop_moderation_requires_deliberate_confirmation')
            c=start(handle='pending');c.expect('No community access.');c.finish();assert query('PRAGMA user_version')==[(15,)] and query('SELECT count(*) FROM unread')==[(0,)] and all(password.encode() not in c.transcript for c in children);checks.append('pending_member_denied_schema_fifteen_no_unread_mutation_or_password_echo')
        finally:
            for c in children:c.close()
    result=dict(result='pass',count=len(checks),checks=checks,schema_version=15,capacity_claim=False,new_transfer_authority=False,public_library=False)
    if a.output:a.output.write_text(json.dumps(result,indent=2,sort_keys=True)+'\n')
    print('Local/worker file requests passed:',len(checks),'controls.')
if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('binary',type=Path);p.add_argument('member',type=Path);p.add_argument('--output',type=Path);a=p.parse_args();a.binary=a.binary.resolve();a.member=a.member.resolve();run(a)
