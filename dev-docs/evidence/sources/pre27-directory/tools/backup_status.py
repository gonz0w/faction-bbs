"""Private numeric receipts for explicitly requested operator bundle checks.

A receipt is a historical observation, not authentication or a recovery promise.
No artifact paths, names, contents or hashes enter the dashboard protocol.
"""
import os,stat,time
from pathlib import Path

CAP=512
FIELDS=('attempted','verified','result','catalog_bytes','object_bytes','objects')


def parse(raw):
    text=raw.decode('ascii');lines=text.splitlines()
    if len(raw)>CAP or len(lines)!=7 or lines[0]!='version=1':raise ValueError('invalid backup receipt')
    values={}
    for key,line in zip(FIELDS,lines[1:]):
        prefix=key+'='
        if not line.startswith(prefix) or not line[len(prefix):].isascii() or not line[len(prefix):].isdigit():raise ValueError('invalid backup receipt')
        values[key]=int(line[len(prefix):])
    v=values
    if not 0<v['attempted']<2**64 or not 0<=v['verified']<2**64 or v['result'] not in (0,1):raise ValueError('invalid backup receipt')
    if not 0<=v['catalog_bytes']<=512*1024*1024 or not 0<=v['object_bytes']<=256*1024*1024 or not 0<=v['objects']<=2000:raise ValueError('invalid backup receipt')
    if v['result']==1 and (v['verified']!=v['attempted'] or not v['catalog_bytes']):raise ValueError('invalid backup receipt')
    if not v['verified'] and any(v[k] for k in ('catalog_bytes','object_bytes','objects')):raise ValueError('invalid backup receipt')
    if v['verified'] and not v['catalog_bytes']:raise ValueError('invalid backup receipt')
    return v


def context(directory,uid):
    m=directory.lstat()
    if not stat.S_ISDIR(m.st_mode) or m.st_uid!=uid or m.st_mode & 0o077:raise ValueError('unsafe backup receipt directory')


def read(directory,uid):
    context(directory,uid)
    try:fd=os.open(directory/'backup.status',os.O_RDONLY|os.O_NOFOLLOW|os.O_NONBLOCK)
    except FileNotFoundError:return None
    with os.fdopen(fd,'rb') as src:
        m=os.fstat(src.fileno())
        if not stat.S_ISREG(m.st_mode) or m.st_uid!=uid or m.st_nlink!=1 or m.st_mode & 0o077 or m.st_size>CAP:raise ValueError('unsafe backup receipt')
        return parse(src.read(CAP+1))


def record(directory,uid,summary):
    """summary=None records a failed attempt while retaining earlier verified data."""
    directory=Path(directory);context(directory,uid)
    temp=directory/'.backup.status.next'
    fd=os.open(temp,os.O_WRONLY|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW,0o600)
    try:
        with os.fdopen(fd,'wb') as dst:
            previous=read(directory,uid)
            values=previous or dict.fromkeys(FIELDS,0)
            values=dict(values,attempted=int(time.time()),result=int(summary is not None))
            if summary is not None:values.update(summary,verified=values['attempted'])
            raw=('version=1\n'+''.join(f'{k}={values[k]}\n' for k in FIELDS)).encode('ascii');parse(raw)
            os.fchown(dst.fileno(),uid,-1);dst.write(raw);dst.flush();os.fsync(dst.fileno())
        os.replace(temp,directory/'backup.status')
        d=os.open(directory,os.O_RDONLY|os.O_DIRECTORY|os.O_NOFOLLOW)
        try:os.fsync(d)
        finally:os.close(d)
    finally:
        try:temp.unlink()
        except FileNotFoundError:pass
