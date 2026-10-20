#!/usr/bin/env python3
"""SQLite journal/recovery research with synthetic content and disposable databases.

Python is a research driver, not a proposed server language.
"""
import argparse
import hashlib
import json
import os
import platform
import sqlite3
import statistics
import subprocess
import sys
import tempfile
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
RAW = ROOT / 'dev-docs/research/results/009-local-storage.json'
SCHEMA = '''
CREATE TABLE members(id INTEGER PRIMARY KEY, approved INTEGER NOT NULL CHECK(approved IN(0,1)));
CREATE TABLE posts(id INTEGER PRIMARY KEY, room INTEGER NOT NULL, author INTEGER NOT NULL REFERENCES members(id),
 request TEXT NOT NULL, body TEXT NOT NULL, UNIQUE(author,request));
CREATE INDEX room_order ON posts(room,id);
CREATE VIRTUAL TABLE search USING fts5(body, content='posts', content_rowid='id');
CREATE TRIGGER post_insert AFTER INSERT ON posts BEGIN
 INSERT INTO search(rowid,body) VALUES(new.id,new.body); END;
CREATE TRIGGER post_delete AFTER DELETE ON posts BEGIN
 INSERT INTO search(search,rowid,body) VALUES('delete',old.id,old.body); END;
'''

def open_db(path, mode=None):
    db = sqlite3.connect(path, timeout=.1, isolation_level=None)
    db.execute('PRAGMA foreign_keys=ON')
    db.execute('PRAGMA cache_size=-2048')
    db.execute('PRAGMA mmap_size=0')
    db.execute('PRAGMA synchronous=FULL')
    db.execute('PRAGMA fullfsync=ON')
    if mode: assert db.execute(f'PRAGMA journal_mode={mode}').fetchone()[0].upper() == mode
    assert db.execute('PRAGMA foreign_keys').fetchone()[0] == 1
    assert db.execute('PRAGMA synchronous').fetchone()[0] == 2
    return db

def post(db, request, body):
    db.execute('BEGIN IMMEDIATE')
    try:
        if not db.execute('SELECT approved FROM members WHERE id=1').fetchone()[0]:
            raise PermissionError('local approval required')
        prior = db.execute('SELECT id,body FROM posts WHERE author=1 AND request=?',(request,)).fetchone()
        if prior:
            if prior[1] != body: raise ValueError('request reused with different content')
            identity = prior[0]
        else:
            identity = db.execute('INSERT INTO posts(room,author,request,body) VALUES(1,1,?,?)',(request,body)).lastrowid
        db.execute('COMMIT')
        return identity
    except Exception:
        db.execute('ROLLBACK'); raise

def digest(db):
    h=hashlib.sha256()
    for table in ('members','posts'):
        for row in db.execute(f'SELECT * FROM {table} ORDER BY id'):
            h.update(json.dumps(row,ensure_ascii=True,separators=(',',':')).encode())
    return h.hexdigest()

def kill_writer(path, committed):
    code = '''import sqlite3,sys,time
d=sqlite3.connect(sys.argv[1],isolation_level=None)
d.execute('PRAGMA synchronous=FULL'); d.execute('PRAGMA fullfsync=ON')
d.execute('BEGIN IMMEDIATE')
d.execute("INSERT INTO posts(room,author,request,body) VALUES(1,1,?,?)",(sys.argv[2],'crashsentinel'))
if sys.argv[3]=='yes': d.execute('COMMIT')
print('checkpoint',flush=True)
time.sleep(30)
'''
    key='crash-after' if committed else 'crash-before'
    child=subprocess.Popen([sys.executable,'-c',code,str(path),key,'yes' if committed else 'no'],stdout=subprocess.PIPE,text=True)
    try:
        if child.stdout.readline().strip() != 'checkpoint': raise RuntimeError('crash checkpoint missing')
    finally:
        child.kill(); child.wait(timeout=3); child.stdout.close()
    db=open_db(path)
    count=db.execute('SELECT count(*) FROM posts WHERE request=?',(key,)).fetchone()[0]
    assert count == int(committed)
    assert db.execute('PRAGMA integrity_check').fetchone()[0]=='ok'
    db.close()

def one_run(directory, mode):
    path=directory/'board.db'; db=open_db(path,mode)
    db.executescript(SCHEMA)
    db.execute('BEGIN')
    db.executemany('INSERT INTO members VALUES(?,?)', ((i,int(i%10!=0)) for i in range(1,1001)))
    body='synthetic local message '+ 'x'*1000
    assert len(body)==1024
    db.executemany('INSERT INTO posts(room,author,request,body) VALUES(?,?,?,?)',
        ((i%50+1,1,f'seed-{i}',body) for i in range(100000)))
    db.execute('COMMIT')
    db.execute('PRAGMA wal_checkpoint(TRUNCATE)')
    latency={'read_ms':[], 'commit_ms':[]}
    for i in range(500):
        start=time.perf_counter()
        if i%5:
            rows=db.execute('SELECT id,body FROM posts WHERE room=? ORDER BY id DESC LIMIT 2',(i%50+1,)).fetchall()
            assert len(rows)==2
            metric='read_ms'
        else:
            post(db,f'bench-{i}',body); metric='commit_ms'
        latency[metric].append((time.perf_counter()-start)*1000)
    first=post(db,'retry','idempotency sentinel')
    assert post(db,'retry','idempotency sentinel')==first
    try: post(db,'retry','different body')
    except ValueError: pass
    else: raise AssertionError('mismatched retry accepted')
    db.execute('UPDATE members SET approved=0 WHERE id=1')
    try: post(db,'denied','private')
    except PermissionError: pass
    else: raise AssertionError('revocation bypass')
    db.execute('UPDATE members SET approved=1 WHERE id=1')
    competitor=open_db(path)
    db.execute('BEGIN IMMEDIATE')
    lock_start=time.perf_counter()
    try: competitor.execute('BEGIN IMMEDIATE')
    except sqlite3.OperationalError as e:
        assert 'locked' in str(e)
    else: raise AssertionError('second writer unexpectedly acquired lock')
    lock_ms=(time.perf_counter()-lock_start)*1000
    db.execute('ROLLBACK'); competitor.close()
    reader=open_db(path); reader.execute('BEGIN')
    before=reader.execute('SELECT count(*) FROM posts').fetchone()[0]
    db.execute('BEGIN IMMEDIATE')
    db.execute("INSERT INTO posts(room,author,request,body) VALUES(1,1,'snapshot','snapshot sentinel')")
    if mode=='WAL':
        db.execute('COMMIT')
        assert reader.execute('SELECT count(*) FROM posts').fetchone()[0]==before
        reader.execute('COMMIT')
    else:
        try: db.execute('COMMIT')
        except sqlite3.OperationalError as e: assert 'locked' in str(e)
        else: raise AssertionError('rollback writer committed through held reader')
        reader.execute('COMMIT'); db.execute('COMMIT')
    assert reader.execute('SELECT count(*) FROM posts').fetchone()[0]==before+1
    reader.close()
    db.close()
    kill_writer(path,False); kill_writer(path,True)
    db=open_db(path)
    post(db,'retention','retentionsentinel')
    assert db.execute("SELECT count(*) FROM search WHERE search MATCH 'retentionsentinel'").fetchone()[0]==1
    db.execute("DELETE FROM posts WHERE request='retention'")
    assert db.execute("SELECT count(*) FROM search WHERE search MATCH 'retentionsentinel'").fetchone()[0]==0
    # Retention/deletion is logically correct; not forensic erasure or backup deletion.
    original=digest(db)
    backup_path=directory/'backup.db'; backup=sqlite3.connect(backup_path)
    db.backup(backup)
    assert digest(backup)==original
    assert backup.execute('PRAGMA integrity_check').fetchone()[0]=='ok'
    restored=sqlite3.connect(directory/'restored.db')
    # Explicit domain export: derived FTS structures are rebuilt, never serialized.
    export_path=directory/'export.jsonl'
    with export_path.open('w') as exported:
        exported.write(json.dumps({'format':'faction-research','version':1})+'\n')
        for table in ('members','posts'):
            for row in db.execute(f'SELECT * FROM {table} ORDER BY id'):
                exported.write(json.dumps({'table':table,'row':row})+'\n')
    restored.executescript(SCHEMA)
    with export_path.open() as exported:
        assert json.loads(next(exported))=={'format':'faction-research','version':1}
        for line in exported:
            item=json.loads(line); table=item['table']
            assert table in ('members','posts')
            placeholders=','.join('?' for _ in item['row'])
            restored.execute(f'INSERT INTO {table} VALUES({placeholders})',item['row'])
    restored.commit()
    assert restored.execute("SELECT count(*) FROM search WHERE search MATCH 'crashsentinel'").fetchone()[0]==1
    assert digest(restored)==original
    assert restored.execute('PRAGMA integrity_check').fetchone()[0]=='ok'
    backup.close(); restored.close()
    sizes={p.name.replace('board','database'):p.stat().st_size for p in directory.glob('board.db*')}
    allocated=sum(p.stat().st_blocks*512 for p in directory.glob('board.db*'))
    row_count=db.execute('SELECT count(*) FROM posts').fetchone()[0]
    db.execute('PRAGMA wal_checkpoint(TRUNCATE)'); db.close()
    return {'mode':mode, 'seed_posts':100000, 'final_posts':row_count, 'latency':latency,
            'writer_busy_ms':lock_ms, 'logical_file_bytes':sizes, 'allocated_bytes':allocated,
            'oracles':['same_request_same_id','mismatched_request_rejected','revoked_denied',
                       'bounded_writer_contention','reader_snapshot','killed_before_commit',
                       'killed_after_commit','fts_retention','backup_restore','domain_export_restore','integrity_check']}

def main():
    p=argparse.ArgumentParser(description=__doc__)
    g=p.add_mutually_exclusive_group(required=True)
    g.add_argument('--run',action='store_true'); g.add_argument('--check',action='store_true')
    args=p.parse_args()
    source_hash=hashlib.sha256(Path(__file__).read_bytes()).hexdigest()
    if args.run:
        raw={'schema':1,'date':'2026-10-19','source_sha256':source_hash,
             'environment':f'{platform.system()} / {platform.machine()} / Python {platform.python_version()}',
             'sqlite_version':sqlite3.sqlite_version,'runs':[]}
        for mode in ('DELETE','WAL'):
            for _ in range(5):
                with tempfile.TemporaryDirectory(prefix='faction-storage-') as directory:
                    raw['runs'].append(one_run(Path(directory),mode))
                print(f'{mode}: independent database complete',flush=True)
        RAW.write_text(json.dumps(raw,indent=2)+'\n')
    else: raw=json.loads(RAW.read_text())
    assert raw['source_sha256']==source_hash and len(raw['runs'])==10
    for mode in ('DELETE','WAL'):
        runs=[r for r in raw['runs'] if r['mode']==mode]
        assert len(runs)==5
        for r in runs:
            assert r['seed_posts']==100000 and len(r['oracles'])==11
            assert len(r['latency']['read_ms'])==400 and len(r['latency']['commit_ms'])==100
        commits=[x for r in runs for x in r['latency']['commit_ms']]
        reads=[x for r in runs for x in r['latency']['read_ms']]
        print(f'{mode}: read median {statistics.median(reads):.3f} ms; FULL commit median {statistics.median(commits):.3f} ms; all 11 oracles passed in five runs')
    print('Synthetic local process-crash experiment; no physical power-failure, SBC or encryption proof.')

if __name__=='__main__':main()
