#!/usr/bin/env python3
"""Reset synthetic comparison inputs ONLY in the offline disposable NetBSD fixture."""
import json
import os
import pwd
import sqlite3
import sys
from pathlib import Path

assert sys.argv[1:]==['--offline-disposable-fixture']
assert Path('/etc/faction-disposable-test').is_file()
assert os.geteuid()==pwd.getpwnam('factiond').pw_uid
assert not Path('/var/run/faction/terminal.sock').exists(),'stop/reap backend first'
state=Path('/var/db/faction-profile')
assert state.is_dir() and not state.is_symlink() and (state/'.profile-fixture').is_file()
db=sqlite3.connect(state/'faction.sqlite3');db.execute('pragma foreign_keys=on')
assert db.execute("select count(*) from members where handle not glob 'profile*' and handle not glob 'pending_*' and handle not glob 'ui_*'").fetchone()[0]==0
assert db.execute('select count(*) from members').fetchone()[0]<=1040
assert db.execute('select count(*) from posts').fetchone()[0]<=100020
assert db.execute("select count(*) from posts where id>100000 and body not like 'shared ui durability %' and body not like 'UI load %' and body not in ('reply parity','context boundary','retry parity')").fetchone()[0]==0
assert db.execute("select count(*) from rooms where id>50 and (slug not like 'ui-%' or title not in ('UI parity room','UI test room'))").fetchone()[0]==0
assert db.execute('select count(*) from rooms').fetchone()[0]<=60
with db:
    db.execute('delete from chat_messages');db.execute('delete from chat_sessions')
    db.execute('delete from chat_grants');db.execute('delete from chat_channels where id!=1')
    db.execute('update chat_channels set sequence=0 where id=1')
    db.execute('delete from posts where id>100000')
    db.execute('delete from unread')
    db.execute("delete from members where handle glob 'pending_*' or handle glob 'ui_*'")
    db.execute('delete from rooms where id>50')
    for table in ['members','posts','rooms','chat_channels']:
        n=db.execute('select max(id) from '+table).fetchone()[0]
        db.execute('update sqlite_sequence set seq=? where name=?',(n,table))
counts={key:db.execute(query).fetchone()[0] for key,query in {
    'accounts':'select count(*) from members','unique_hashes':'select count(distinct password_hash) from members',
    'rooms':'select count(*) from rooms','posts':'select count(*) from posts'}.items()}
assert counts=={'accounts':1001,'unique_hashes':1001,'rooms':50,'posts':100000}
assert db.execute('pragma integrity_check').fetchone()[0]=='ok'
db.execute('pragma wal_checkpoint(truncate)');db.close()
print(json.dumps({'integrity':'ok',**counts}))
