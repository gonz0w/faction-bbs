#!/usr/bin/env python3
"""Local/worker member-page journeys on private disposable PTYs."""
import argparse,json,secrets,sqlite3,tempfile
from pathlib import Path
from smoke_local_console import Console
from smoke_member_worker import Worker
from smoke_room_browser import MENU
SETTINGS='Page settings> '
PAGES='Member pages> '
EDIT='My page> '
CARD='Member page> '
TEXT='Page text> '
REVIEW='[S]ave [E]dit [H]ide/share plan [C]ancel: '

def settings(c,plain=False):
    c.send('s') if plain else c.raw(b's');c.expect('[C]reate room: ');c.send('m');c.expect(SETTINGS)
def setting(c,key,prompt,value):c.send(key);c.expect(prompt);c.send(value);c.expect(SETTINGS)
def enter(c,plain=False):c.send('m') if plain else c.raw(b'm');c.expect(PAGES)
def review_field(c,index,text):
    c.send(str(index));c.expect(TEXT)
    for line in text.split('\n'):c.send(line);c.expect(TEXT)
    c.send('.');c.expect('PREVIEW ONLY');c.expect(REVIEW)
def save(c):c.send('s');c.expect('Member page saved locally.');c.expect(EDIT)
def done(c,plain=False):
    c.send('q')
    if not plain:c.expect('Press Enter to return to menu: ');c.send('')
    c.expect('Command (then Enter): ' if plain else MENU)

def run(a):
    checks=[];children=[];password=secrets.token_hex(20)
    with tempfile.TemporaryDirectory(prefix='faction-members-') as tmp:
        state=Path(tmp)/'board'
        def query(sql):
            with sqlite3.connect(f'file:{state}/faction.sqlite3?mode=ro',uri=True) as db:return db.execute(sql).fetchall()
        def start(plain=False,worker=False,columns=80,themed=False):
            extra=['--theme',str(a.theme.resolve())] if themed else []
            c=Worker(a.member,state,extra=extra,columns=columns,rows=18 if columns==40 else 24) if worker else Console(a.binary,state,'login',['--plain' if plain else '--ansi',*extra],columns=columns,rows=18 if columns==40 else 24);children.append(c)
            if worker:c.expect('[L]ogin [R]egister [Q]uit: ');c.send('l')
            c.expect('Handle: ');c.send('sysop');c.password(password)
            if not plain and (worker or columns>=80):c.expect('Press Enter to continue: ');c.send('')
            c.expect('Command (then Enter): ' if plain else MENU);return c
        def logout(c,plain=False):c.send('q') if plain else c.raw(b'q');c.expect('Logged out.');c.finish()
        try:
            c=Console(a.binary,state,'init');children.append(c);c.expect('Handle: ');c.send('sysop');c.password(password,'New password (12+ characters, hidden): ');c.password(password,'Confirm password (hidden): ');c.expect('Lobby created.');c.finish()
            c=start();assert query('select enabled from page_settings')==[(0,)];settings(c);c.send('e');c.expect(SETTINGS);setting(c,'n','Page label (1-24 ASCII): ','Basement people');setting(c,'h','Plan history (1-16) and days (7-365): ','2 7');done(c);enter(c);c.send('e');c.expect(EDIT);review_field(c,1,'A tiny board builder.');assert query('select count(*) from member_pages')==[(0,)];save(c);assert query('select bio from member_pages')==[('A tiny board builder.',)];checks.append('ansi_enable_name_retention_and_explicit_field_preview_save')
            review_field(c,3,' /\\\n/  \\');save(c);c.send('v');c.expect('Shared fields (1-4, all/none)');c.expect(': ');c.send('1 3');c.expect(REVIEW);save(c);assert query('select visible from member_pages')==[(5,)];checks.append('original_ascii_signature_and_explicit_field_visibility')
            c.send('p');c.expect(TEXT);c.send('Project signal one.');c.expect(TEXT);c.send('.');c.expect(REVIEW);c.send('h');c.expect('hidden');c.expect(REVIEW);save(c);assert query('select body,visible from member_plans')==[('Project signal one.',0)];c.send('q');c.expect(PAGES);done(c);logout(c);checks.append('dated_plan_visibility_is_chosen_before_explicit_save')
            c=start(True);enter(c,True);c.send('v');c.expect('A tiny board builder.');c.expect(CARD)
            for _ in range(4):c.send('n');c.expect(CARD)
            assert b'Project signal one.' in c.transcript;c.send('x');c.expect('Plan update removed.');c.expect(CARD);c.send('q');c.expect(PAGES);c.send('e');c.expect(EDIT);review_field(c,4,'Plain terminal project.');save(c);c.send('q');c.expect(PAGES);done(c,True);logout(c,True);assert b'\x1b' not in c.transcript;checks.append('plain_view_dated_plan_delete_and_project_edit_without_escapes')
            c=start(worker=True);enter(c);c.send('e');c.expect(EDIT);review_field(c,2,'Hobby radio');save(c);c.send('q');c.expect(PAGES);done(c);logout(c);checks.append('inherited_worker_uses_shared_profile_workflow')
            c=start(columns=40);settings(c);c.send('l');c.expect(SETTINGS);done(c);enter(c);c.send('v');c.expect('sysop | local page');c.expect(CARD);c.send('q');c.expect(PAGES);c.send('e');c.expect(EDIT);c.send('h');c.expect(REVIEW);save(c);assert query('select published from member_pages')==[(0,)];c.send('q');c.expect(PAGES);done(c);logout(c);checks.append('compact_40x18_layout_and_explicit_page_hide')
            c=start(True);enter(c,True);c.send('e');c.expect(EDIT);c.send('d');c.expect('Type DELETE: ');c.send('no');c.expect(EDIT);assert query('select count(*) from member_pages')==[(1,)];c.send('d');c.expect('Type DELETE: ');c.send('DELETE');c.expect('Page and retained plans removed.');c.expect(PAGES);assert query('select count(*) from member_pages')==[(0,)] and query('select count(*) from member_plans')==[(0,)];done(c,True);logout(c,True);checks.append('page_delete_requires_exact_confirmation_and_clears_retained_history')
            other=Path(tmp)/'other';c=Console(a.binary,other,'init');children.append(c);c.expect('Handle: ');c.send('sysop');c.password(password,'New password (12+ characters, hidden): ');c.password(password,'Confirm password (hidden): ');c.expect('Lobby created.');c.finish()
            with sqlite3.connect(other/'faction.sqlite3') as db:assert db.execute('select enabled,name,compact from page_settings').fetchall()==[(0,'Member pages',0)]
            checks.append('independent_board_policy_remains_disabled_and_local');assert query('pragma user_version')==[(10,)] and query('select count(*) from unread')==[(0,)];assert all(password.encode() not in c.transcript for c in children)
        finally:
            for c in children:c.close()
    d=dict(result='pass',count=len(checks),checks=checks,schema_version=10,capacity_claim=False)
    if a.output:a.output.write_text(json.dumps(d,indent=2,sort_keys=True)+'\n')
    print('Local/worker member-page checks passed:',len(checks))
if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('binary',type=Path);p.add_argument('member',type=Path);p.add_argument('--theme',type=Path,default=Path('themes/base'));p.add_argument('--output',type=Path);a=p.parse_args();a.binary=a.binary.resolve();a.member=a.member.resolve();run(a)
