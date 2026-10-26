#!/usr/bin/env python3
"""Authenticated editorial journeys on fresh owned local/worker boards."""
import argparse,json,secrets,sqlite3,subprocess,tempfile
from pathlib import Path
from smoke_local_console import Console
from smoke_member_worker import Worker
from smoke_room_browser import MENU
from smoke_library import enter
HOME='Workshop> '
ISSUE='Issue review> '
SOURCE='Submission> '
CONFIRM='Confirm YES or Q: '
FIELDS=['Title (1-64; blank keeps/cancels): ','Section (1-24; blank keeps): ','Distribution rights (1-96; blank keeps): ','Provenance (1-128; blank keeps): ']
HEADERS=['Issue number (1-9999; blank keeps/cancels): ','Edition revision (1-9999; blank keeps): ','Issue title (1-64; blank keeps): ','Edition label (1-32; blank keeps): ','Schedule label (1-64; blank keeps): ']
def send(c,text,prompt):
    mark=len(c.transcript);c.send(text);c.expect(prompt);return c.transcript[mark:]
def complete(c,frame,prompt):
    for _ in range(200):
        if b'[End of review]' in frame:return frame
        frame=send(c,'n',prompt)
    raise AssertionError('review did not reach its bounded end')
def manuscript(c,title,body,proposal=False,private=False):
    c.expect(FIELDS[0])
    for i,v in enumerate([title,'Reports','BSD-3-Clause','Original synthetic author']):send(c,v,FIELDS[i+1] if i<3 else 'Manuscript> ')
    if proposal:send(c,'.clear','Manuscript> ')
    for line in body.splitlines():send(c,line,'Manuscript> ')
    frame=send(c,'.review','Manuscript review> ')
    complete(c,frame,'Manuscript review> ')
    send(c,'s' if proposal or private else 't',CONFIRM)
    return send(c,'yes',ISSUE if proposal else SOURCE)
def open_issue(c,id=1):
    send(c,'i','Editions> ');send(c,'v','Issue ID (blank cancels): ');return send(c,str(id),ISSUE)
def create_issue(c):
    send(c,'c',HEADERS[0])
    for i,v in enumerate(['1','1','FIRST-ISSUE','Autumn','When ready']):frame=send(c,v,HEADERS[i+1] if i<4 else 'Save draft issue? YES or Q: ')
    complete(c,frame,'Save draft issue? YES or Q: ');return send(c,'yes',ISSUE)
def select(c,id=1):
    send(c,'s','Slot 1-8 to select/replace (blank cancels): ');send(c,'1','Submitted manuscript ID (0 removes slot): ');return send(c,str(id),ISSUE)
def approve(c,frame,yes=True):
    complete(c,frame,ISSUE);send(c,'a' if yes else 'u',CONFIRM);return send(c,'yes',ISSUE)
def publish(c,frame):
    complete(c,frame,ISSUE);send(c,'publish',CONFIRM);return send(c,'yes',ISSUE)
def run(a):
    checks=[];children=[];password=secrets.token_hex(20)
    with tempfile.TemporaryDirectory(prefix='faction-editorial-pty-') as directory:
        root=Path(directory);state=root/'board';catalog=root/'library';catalog.mkdir();(catalog/'library.toml').write_text('version=1\nenabled=true\nname="Workshop"\ncompact=false\nshelf=[]\nbook=[]\neditorial=true\nmagazine_name="Local zine"\n')
        def query(sql):
            with sqlite3.connect(f'file:{state}/faction.sqlite3?mode=ro',uri=True) as db:return db.execute(sql).fetchall()
        def start(handle='author',plain=False,worker=False):
            flags=['--library',str(catalog)]
            c=Worker(a.member,state,flags,columns=40,rows=18) if worker else Console(a.binary,state,'login',['--plain' if plain else '--ansi',*flags],columns=40,rows=18);children.append(c)
            if worker:c.expect('[L]ogin [R]egister [Q]uit: ');c.send('l')
            c.expect('Handle: ');c.send(handle);c.password(password)
            if handle=='pending':return c
            c.expect('Command (then Enter): ' if plain else MENU);enter(c,plain);send(c,'w',HOME);return c
        def end(c,plain=False):
            send(c,'q','Library> ');send(c,'q','Command (then Enter): ' if plain else MENU);c.send('q') if plain else c.raw(b'q');c.expect('Logged out.');c.finish()
        try:
            c=Console(a.binary,state,'init');children.append(c);c.expect('Handle: ');c.send('sysop');c.password(password,'New password (12+ characters, hidden): ');c.password(password,'Confirm password (hidden): ');c.expect('Lobby created.');c.finish()
            with sqlite3.connect(state/'faction.sqlite3') as db:
                for name,status in [('author','approved'),('reader','approved'),('pending','pending')]:db.execute("INSERT INTO members(handle,password_hash,role,status) SELECT ?,password_hash,'member',? FROM members WHERE id=1",(name,status))
            c=start(plain=True);send(c,'s','Submissions> ');c.send('c');manuscript(c,'PRIVATE-MANUSCRIPT','Original manuscript body.\nORIGINAL-END',private=True);send(c,'q',HOME);end(c,True)
            c=start('reader');send(c,'s','Submissions> ');assert b'PRIVATE-MANUSCRIPT' not in c.transcript;send(c,'q',HOME);send(c,'i','Editions> ');send(c,'v','Issue ID (blank cancels): ');send(c,'1',HOME);assert b'Original manuscript body.' not in c.transcript;end(c);checks.append('private_source_and_unselected_review_hidden_from_ordinary_members')
            c=start();send(c,'s','Submissions> ');send(c,'v','Submission ID (blank cancels): ');send(c,'1',SOURCE);c.send('e');manuscript(c,'SUBMITTED-MANUSCRIPT','Added author line.');send(c,'q',HOME);end(c);assert query("SELECT state FROM magazine_submissions")==[('submitted',)];checks.append('save_private_reconnect_revise_and_explicitly_submit_through_authenticated_ui')
            c=start('sysop');frame=create_issue(c);frame=select(c);complete(c,frame,ISSUE);send(c,'publish',CONFIRM);frame=send(c,'yes',CONFIRM);assert query('SELECT state FROM magazine_issues')==[('draft',)];send(c,'q',HOME);end(c);checks.append('editor_selects_proposal_but_cannot_publish_without_author_approval')
            c=start(worker=True);frame=open_issue(c);approve(c,frame);send(c,'q',HOME);end(c);assert query('SELECT approved IS NOT NULL FROM magazine_entries')==[(1,)];checks.append('worker_author_complete_exact_issue_review_and_explicit_approval')
            c=start('sysop');open_issue(c);send(c,'e','Slot 1-8 to revise proposal (blank cancels): ');c.send('1');frame=manuscript(c,'EDITOR-PROPOSAL','Revised by editor for approval.\nFINAL-APPROVED-END',proposal=True);assert query('SELECT approved=(SELECT token FROM magazine_issues WHERE id=1) FROM magazine_entries')==[(0,)];send(c,'q',HOME);end(c);checks.append('editor_copy_preserves_author_and_resets_exact_issue_approval')
            c=start(plain=True);frame=open_issue(c);approve(c,frame);send(c,'q',HOME);end(c,True);assert b'FINAL-APPROVED-END' in c.transcript and b'\x1b' not in c.transcript
            c=start('sysop');frame=open_issue(c);publish(c,frame);send(c,'q',HOME);end(c);checks.append('reapprove_changed_complete_issue_and_publish_without_changing_approved_text')
            c=start('reader',True);frame=open_issue(c);complete(c,frame,ISSUE);assert b'FINAL-APPROVED-END' in c.transcript and b'By author' in c.transcript;send(c,'q',HOME);end(c,True);assert b'\x1b' not in c.transcript and query('SELECT count(*) FROM unread')==[(0,)];checks.append('published_paged_plain_reading_preserves_credits_and_room_unread')
            output=root/'edition.txt';argv=[str(a.binary),'--state-dir',str(state),'magazine-edition','1',str(output)];x=subprocess.run(argv,capture_output=True);assert x.returncode==0 and b'FINAL-APPROVED-END' in output.read_bytes() and output.stat().st_mode&0o777==0o600;assert subprocess.run(argv,capture_output=True).returncode!=0;checks.append('filesystem_owner_currently_consented_text_export_is_private_and_no_overwrite')
            c=start();frame=open_issue(c);approve(c,frame,False);send(c,'q',HOME);end(c);assert query('SELECT state FROM magazine_issues')==[('withdrawn',)];c=start('reader');send(c,'i','Editions> ');assert b'FIRST-ISSUE' not in c.transcript;send(c,'q',HOME);end(c);checks.append('contributor_revocation_withdraws_issue_without_erasing_existing_export')
            c=start('pending');c.expect('No community access.');c.finish();assert all(password.encode() not in x.transcript for x in children) and query('PRAGMA user_version')==[(20,)];checks.append('pending_denied_schema_17_and_passwords_never_echoed')
        finally:
            for c in children:c.close()
    result=dict(result='pass',count=len(checks),checks=checks,schema_version=20,capacity_claim=False,public_publication=False,new_transfer_authority=False)
    if a.output:a.output.write_text(json.dumps(result,indent=2,sort_keys=True)+'\n')
    print('Local/worker editorial passed:',len(checks),'controls.')
if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('binary',type=Path);p.add_argument('member',type=Path);p.add_argument('--output',type=Path);a=p.parse_args();a.binary=a.binary.resolve();a.member=a.member.resolve();run(a)
