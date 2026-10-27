#!/usr/bin/env python3
"""Community wall through real local/worker PTYs, using private disposable boards."""
import argparse,json,secrets,sqlite3,tempfile
from pathlib import Path
from smoke_local_console import Console
from smoke_member_worker import Worker
from smoke_room_browser import MENU
SETTINGS='Wall settings> '
WALL='Wall> '
TEXT='Wall text> '
REVIEW='[S]end [E]dit [C]ancel: '

def settings(c,plain=False):
    c.send('s') if plain else c.raw(b's');c.expect('[C]reate room: ');c.send('w');c.expect(SETTINGS)
def setting(c,key,prompt,value):
    c.send(key);c.expect(prompt);c.send(value);c.expect(SETTINGS)
def leave_settings(c,plain=False,shared=False):
    c.send('q')
    if not plain or shared:c.expect('Press Enter to return to menu: ');c.send('')
    c.expect('Command (then Enter): ' if plain else MENU)
def wall(c,plain=False):
    c.send('w') if plain else c.raw(b'w');c.expect(WALL)
def draft(c,text):
    c.send('p');c.expect(TEXT)
    for line in text.split('\n'):c.send(line);c.expect(TEXT)
    c.send('.');c.expect('NOT PUBLISHED');c.expect(REVIEW)
def send(c):c.send('s');c.expect('Wall note published locally.');c.expect(WALL)
def leave_wall(c,plain=False,shared=False):
    c.send('q')
    if not plain:c.expect('Press Enter to return to menu: ');c.send('')
    c.expect('Command (then Enter): ' if plain else MENU)

def run(a):
    checks=[];children=[];password=secrets.token_hex(20)
    with tempfile.TemporaryDirectory(prefix='faction-wall-') as tmp:
        state=Path(tmp)/'board'
        def query(sql):
            with sqlite3.connect(f'file:{state}/faction.sqlite3?mode=ro',uri=True) as db:return db.execute(sql).fetchall()
        def start(plain=False,worker=False,columns=80,handle='sysop'):
            c=Worker(a.member,state,columns=columns,rows=18 if columns==40 else 24) if worker else Console(a.binary,state,'login',['--plain' if plain else '--ansi'],columns=columns,rows=18 if columns==40 else 24);children.append(c)
            if worker:c.expect('[L]ogin [R]egister [Q]uit: ');c.send('l')
            c.expect('Handle: ');c.send(handle);c.password(password)
            if not plain and (worker or columns>=80):c.expect('Press Enter to continue: ');c.send('')
            c.expect('Command (then Enter): ' if plain else MENU);return c
        def logout(c,plain=False):c.send('q') if plain else c.raw(b'q');c.expect('Logged out.');c.finish()
        try:
            c=Console(a.binary,state,'init');children.append(c);c.expect('Handle: ');c.send('sysop');c.password(password,'New password (12+ characters, hidden): ');c.password(password,'Confirm password (hidden): ');c.expect('Lobby created.');c.finish()
            c=start();assert query('select enabled from wall_settings')==[(0,)];settings(c);c.send('e');c.expect(SETTINGS);setting(c,'n','Wall name (1-24 ASCII): ','Basement signals');setting(c,'l','Limits: notes bytes days cooldown: ','16 240 7 3600');leave_settings(c);wall(c);draft(c,'First signal.\n /\\\n/  \\');assert query('select count(*) from wall_notes')==[(0,)];send(c);assert query('select body from wall_notes')==[('First signal.\n /\\\n/  \\',)];checks.append('ansi_sysop_enable_name_limits_explicit_ascii_review_send')
            draft(c,'Too soon.');c.send('s');c.expect('wall cooldown');c.expect(REVIEW);c.send('c');c.expect(WALL);assert query('select count(*) from wall_notes')==[(1,)];checks.append('cooldown_failure_keeps_review_without_duplicate');leave_wall(c);logout(c)
            # Edit through ordinary plain mode; age only this owned synthetic receipt.
            with sqlite3.connect(state/'faction.sqlite3') as db:db.execute('update wall_receipts set last_at=unixepoch()-3601')
            c=start(True);wall(c,True);c.send('e');c.expect(TEXT);c.send('/undo');c.expect(TEXT);c.send('Plain edit.');c.expect(TEXT);c.send('.');c.expect(REVIEW);send(c);assert query('select edited_at>0 from wall_notes')==[(1,)];c.send('x');c.expect('Note deleted');c.expect(WALL);leave_wall(c,True);logout(c,True);assert b'\x1b' not in c.transcript;checks.append('plain_edit_own_note_delete_and_no_ansi_output')
            with sqlite3.connect(state/'faction.sqlite3') as db:db.execute('update wall_receipts set last_at=unixepoch()-3601')
            c=start(worker=True);wall(c);draft(c,'Worker signal.');send(c);leave_wall(c);logout(c);checks.append('worker_uses_same_wall_workflow')
            c=start(columns=40);wall(c);c.send('b');c.expect('Worker signal.');c.expect(WALL);c.send('a');c.expect('Edition ID, or Enter for current: ');c.send('');c.expect(WALL);leave_wall(c);logout(c);checks.append('compact_40x18_wall_paging_and_archive_return')
            c=start();settings(c);c.send('w');c.expect('Weekly change CLEARS notes/archives. Type YES: ');c.send('no');c.expect(SETTINGS);assert query('select count(*) from wall_notes')==[(1,)];c.send('w');c.expect('Type YES: ');c.send('YES');c.expect(SETTINGS);setting(c,'a','Retain 0-12 weekly editions: ','1');leave_settings(c);assert query('select count(*) from wall_notes')==[(0,)];wall(c);draft(c,'This week.');c.send('c');c.expect(WALL);assert query('select count(*) from wall_notes')==[(0,)];leave_wall(c);logout(c);checks.append('weekly_change_requires_explicit_clear_confirmation_and_cancel_does_not_post')
            # A second private board keeps its own disabled/name settings.
            other=Path(tmp)/'other';c=Console(a.binary,other,'init');children.append(c);c.expect('Handle: ');c.send('sysop');c.password(password,'New password (12+ characters, hidden): ');c.password(password,'Confirm password (hidden): ');c.expect('Lobby created.');c.finish()
            with sqlite3.connect(other/'faction.sqlite3') as db:assert db.execute('select enabled,name from wall_settings').fetchall()==[(0,'Community wall')]
            checks.append('independent_board_configuration_is_local')
            assert query('pragma user_version')==[(20,)]
            assert all(password.encode() not in c.transcript for c in children)
        finally:
            for c in children:c.close()
    result=dict(result='pass',count=len(checks),checks=checks,schema_version=20,capacity_claim=False)
    if a.output:a.output.write_text(json.dumps(result,indent=2,sort_keys=True)+'\n')
    print('Local/worker community-wall checks passed:',len(checks))
if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('binary',type=Path);p.add_argument('member',type=Path);p.add_argument('--output',type=Path);a=p.parse_args();a.binary=a.binary.resolve();a.member=a.member.resolve();run(a)
