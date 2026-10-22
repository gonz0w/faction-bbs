#!/usr/bin/env python3
"""Stopped/drained local backup of catalog AND file bytes; restore into fresh paths.

Run as root on NetBSD. The existing owner export/restore binary validates the DB
and strips transfer authority. This directory bundle is private, not encrypted.
Keep listeners disabled throughout. No live lock inode is removed or replaced.
"""
import argparse
import contextlib
import fcntl
import hashlib
import json
import os
from pathlib import Path
import pwd
import re
import sqlite3
import stat
import subprocess
import backup_status

MAX = 16 * 1024 * 1024
BOARD = 256 * 1024 * 1024
NAME = re.compile(r'[0-9a-f]{32}(?:\.part)?\Z')


def regular(path, uid, cap):
    fd = os.open(path, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK)
    f = os.fdopen(fd, 'rb')
    m = os.fstat(fd)
    if not stat.S_ISREG(m.st_mode) or m.st_nlink != 1 or m.st_uid != uid or m.st_mode & 0o077 or m.st_size > cap:
        f.close()
        raise ValueError(f'unsafe or oversized file: {path.name}')
    return f


def private(path, uid):
    m = path.lstat()
    if not stat.S_ISDIR(m.st_mode) or m.st_uid != uid or m.st_mode & 0o077:
        raise ValueError(f'unsafe private directory: {path}')


def mkdir(path, uid):
    path.mkdir(mode=0o700)
    if os.geteuid() == 0:
        os.chown(path, uid, pwd.getpwuid(uid).pw_gid)


def cli(binary, uid, args):
    kw = {}
    if os.geteuid() == 0:
        kw = dict(user=uid, group=pwd.getpwuid(uid).pw_gid, extra_groups=())
    subprocess.run([str(binary), *map(str, args)], check=True, cwd='/',
                   env={'PATH': '/usr/bin:/bin', 'HOME': pwd.getpwuid(uid).pw_dir},
                   stdout=subprocess.DEVNULL, **kw)


def rows(snapshot):
    # The Rust verifier has already checked exact schema, row caps and values.
    with sqlite3.connect(snapshot.as_uri()+'?mode=ro&immutable=1', uri=True) as db:
        return {r[0]: r[1:] for r in db.execute('SELECT id,bytes,sha256,state FROM file_objects')}


def digest_copy(source, destination, uid, cap):
    h = hashlib.sha256()
    size = 0
    with regular(source, uid, cap) as src:
        out = None
        if destination is not None:
            out = os.fdopen(os.open(destination, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW, 0o600), 'wb')
        try:
            while True:
                b = src.read(32768)
                if not b:
                    break
                size += len(b)
                if size > cap:
                    raise ValueError('file grew beyond its limit')
                h.update(b)
                if out:
                    out.write(b)
            if out:
                out.flush()
                os.fsync(out.fileno())
        finally:
            if out:
                out.close()
    return {'bytes': size, 'sha256': h.hexdigest()}


def sync_dir(path):
    fd = os.open(path, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW)
    try:
        os.fsync(fd)
    finally:
        os.close(fd)


@contextlib.contextmanager
def drained(pidfile, store, owner, files):
    private(store, files)
    with contextlib.ExitStack() as stack:
        # Match the real shared-owner pidfile inode; never unlink it.
        pid = stack.enter_context(regular(pidfile, owner, 64))
        fcntl.flock(pid, fcntl.LOCK_EX | fcntl.LOCK_NB)
        for i in range(2):
            path = store/f'slot{i}'
            try:
                fd = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
            except FileExistsError:
                pass
            else:
                os.fchown(fd, files, pwd.getpwuid(files).pw_gid)
                os.close(fd)
            lock = stack.enter_context(regular(path, files, 0))
            fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
        yield


def backup(a):
    private(a.board, a.owner_uid)
    with drained(a.pidfile, a.store, a.owner_uid, a.files_uid):
        mkdir(a.bundle, a.owner_uid)
        # Failure leaves an explicitly incomplete private directory; never overwrite it.
        cli(a.faction, a.owner_uid, ['--state-dir', a.board, 'export', a.bundle/'catalog.sqlite3'])
        mkdir(a.bundle/'objects', a.owner_uid)
        catalog = rows(a.bundle/'catalog.sqlite3')
        entries = {}
        for object_id, (size, sha, state) in catalog.items():
            present = []
            for suffix in ('', '.part'):
                name = object_id+suffix
                path = a.store/name
                if not path.exists() and not path.is_symlink():
                    continue
                d = digest_copy(path, a.bundle/'objects'/name, a.files_uid, size)
                os.chown(a.bundle/'objects'/name, a.owner_uid, pwd.getpwuid(a.owner_uid).pw_gid)
                if suffix == '' and state in ('pending', 'published') and d != {'bytes': size, 'sha256': sha}:
                    raise ValueError('published/pending bytes do not match catalog')
                entries[name] = d
                present.append(suffix)
            if state in ('pending', 'published') and present != ['']:
                raise ValueError('missing or ambiguous published/pending file data')
        actual = {x.name for x in a.store.iterdir()} - {'slot0', 'slot1'}
        if actual != set(entries):
            raise ValueError('unknown file-store entries; reconcile before backup')
        if len(entries) > 2000 or sum(v['bytes'] for v in entries.values()) > BOARD:
            raise ValueError('file-data limits exceeded')
        manifest = {'version': 1, 'catalog': digest_copy(a.bundle/'catalog.sqlite3', None, a.owner_uid, 512*1024*1024), 'objects': entries}
        fd = os.open(a.bundle/'manifest.json', os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
        with os.fdopen(fd, 'w') as out:
            json.dump(manifest, out, indent=2, sort_keys=True)
            out.write('\n')
            out.flush()
            os.fsync(out.fileno())
        os.chown(a.bundle/'manifest.json', a.owner_uid, pwd.getpwuid(a.owner_uid).pw_gid)
        sync_dir(a.bundle/'objects')
        sync_dir(a.bundle)


def verify(a):
    """Check every byte without restoring, reserving destinations or starting services."""
    private(a.bundle, a.owner_uid)
    private(a.bundle/'objects', a.owner_uid)
    with regular(a.bundle/'manifest.json', a.owner_uid, 512*1024) as f:
        m = json.load(f)
    if not isinstance(m,dict) or set(m) != {'version', 'catalog', 'objects'} or type(m['version']) is not int or m['version'] != 1 or not isinstance(m['objects'], dict):
        raise ValueError('invalid bundle manifest')
    objects = m['objects']
    def valid_digest(data,cap):
        return isinstance(data,dict) and set(data)=={'bytes','sha256'} and type(data['bytes']) is int and 0<=data['bytes']<=cap and isinstance(data['sha256'],str) and re.fullmatch('[a-f0-9]{64}',data['sha256'])
    if not valid_digest(m['catalog'],512*1024*1024) or any(not valid_digest(v,MAX) for v in objects.values()):
        raise ValueError('invalid bundle digest metadata')
    if len(objects) > 2000 or any(not NAME.fullmatch(n) for n in objects):
        raise ValueError('invalid object names/count')
    if set(x.name for x in (a.bundle/'objects').iterdir()) != set(objects):
        raise ValueError('unexpected or missing bundle objects')
    if set(x.name for x in a.bundle.iterdir()) != {'manifest.json', 'catalog.sqlite3', 'objects'}:
        raise ValueError('unexpected bundle content')
    if digest_copy(a.bundle/'catalog.sqlite3', None, a.owner_uid, 512*1024*1024) != m['catalog']:
        raise ValueError('catalog hash mismatch')
    cli(a.faction, a.owner_uid, ['verify-export', a.bundle/'catalog.sqlite3'])
    catalog = rows(a.bundle/'catalog.sqlite3')
    total = 0
    for name, data in objects.items():
        object_id = name.removesuffix('.part')
        if object_id not in catalog:
            raise ValueError('object missing from catalog')
        size, sha, state = catalog[object_id]
        got = digest_copy(a.bundle/'objects'/name, None, a.owner_uid, size)
        if got != data:
            raise ValueError('object hash mismatch')
        total += got['bytes']
        if state in ('pending', 'published') and (name.endswith('.part') or got != {'bytes': size, 'sha256': sha}):
            raise ValueError('catalog/file mismatch')
    if total > BOARD or any(state in ('pending', 'published') and object_id not in objects for object_id, (_, _, state) in catalog.items()):
        raise ValueError('incomplete or oversized file data')
    return m


def restore(a):
    if a.board.exists() or a.board.is_symlink() or a.store.exists() or a.store.is_symlink():
        raise ValueError('restore requires fresh board AND store paths')
    objects=verify(a)['objects']
    mkdir(a.store, a.files_uid)
    # Rust restore creates a fresh board identity and invalidates old session authority.
    cli(a.faction, a.owner_uid, ['--state-dir', a.board, 'restore', a.bundle/'catalog.sqlite3'])
    for name in objects:
        digest_copy(a.bundle/'objects'/name, a.store/name, a.owner_uid, MAX)
        os.chown(a.store/name, a.files_uid, pwd.getpwuid(a.files_uid).pw_gid)
    sync_dir(a.store)


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('operation', choices=('backup', 'verify', 'restore'))
    p.add_argument('--board', type=Path, default=Path('/var/db/faction'))
    p.add_argument('--store', type=Path, default=Path('/var/db/faction-files'))
    p.add_argument('--pidfile', type=Path, default=Path('/var/run/faction/terminal.pid'))
    p.add_argument('--bundle', type=Path, required=True)
    p.add_argument('--owner-uid', type=int, required=True)
    p.add_argument('--files-uid', type=int)
    p.add_argument('--record-status', action='store_true', help='record backup/check attempt in the private --board directory (not for restore)')
    p.add_argument('--faction', type=Path, default=Path('/usr/local/libexec/faction'))
    a = p.parse_args()
    record_enabled=False
    try:
        if os.geteuid() != 0 or a.owner_uid<=0 or (a.operation!='verify' and (a.files_uid is None or a.files_uid<=0 or a.owner_uid==a.files_uid)):
            raise ValueError('use root and distinct non-root owner/file UIDs')
        os.umask(0o077)
        for field in ('board', 'store', 'pidfile', 'bundle', 'faction'):
            path = getattr(a, field)
            if not path.is_absolute() or '..' in path.parts:
                raise ValueError('use absolute paths without parent traversal')
        if a.record_status:
            if a.operation=='restore':raise ValueError('record-status is for backup/verify, not restore')
            backup_status.read(a.board,a.owner_uid)
            record_enabled=True
        if a.operation=='restore':restore(a)
        else:
            if a.operation=='backup':backup(a)
            m=verify(a)
            if record_enabled:backup_status.record(a.board,a.owner_uid,dict(catalog_bytes=m['catalog']['bytes'],object_bytes=sum(v['bytes'] for v in m['objects'].values()),objects=len(m['objects'])))
        print('Complete private file-area '+a.operation+'. No services started.')
    except (OSError, ValueError, KeyError, sqlite3.Error, subprocess.CalledProcessError) as exc:
        if record_enabled:
            try:backup_status.record(a.board,a.owner_uid,None)
            except (OSError,ValueError):print('Backup check status unavailable; earlier receipt may remain.',file=__import__('sys').stderr)
        p.exit(1, f'File-area {a.operation} failed: {exc}. Inspect partial fresh paths before retrying.\n')


if __name__ == '__main__':
    main()
