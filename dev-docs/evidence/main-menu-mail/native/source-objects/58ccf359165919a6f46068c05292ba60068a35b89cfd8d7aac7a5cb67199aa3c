#!/usr/bin/env python3
"""Real cross-process credential-budget regression; generated private PTY fixture."""
import fcntl
import secrets
import sqlite3
import tempfile
from pathlib import Path
from smoke_local_console import Console


def smoke(binary):
    secret=secrets.token_hex(24)
    children=[]
    with tempfile.TemporaryDirectory(prefix='faction-kdf-budget-') as root:
        state=Path(root)/'board'
        def start(command, name):
            c=Console(binary,state,command);children.append(c)
            c.expect('Handle: ');c.send(name);return c
        try:
            c=start('init','sysop');c.password(secret,'New password (12+ characters, hidden): ')
            c.password(secret,'Confirm password (hidden): ');c.expect('Lobby created.');c.finish()
            locks=[]
            try:
                for n in range(2):
                    p=state/f'credential-{n}.lock'
                    if not p.exists():p.touch(mode=0o600)
                    f=p.open('r+');fcntl.flock(f,fcntl.LOCK_EX|fcntl.LOCK_NB);locks.append(f)
                c=start('login','sysop');c.password(secret)
                c.expect('credential work busy; reconnect later');c.finish(1)
                # Closing a Python-held lock must release the actual OS budget.
                locks.pop().close()
                c=start('register','newmember');c.password(secret,'New password (12+ characters, hidden): ')
                c.password(secret,'Confirm password (hidden): ');c.expect('Registered pending sysop approval.');c.finish()
            finally:
                for f in locks:f.close()
            db=sqlite3.connect(state/'faction.sqlite3')
            stored=db.execute('SELECT password_hash FROM members').fetchall();db.close()
            assert len(stored)==2 and len({v[0] for v in stored})==2
            assert all(v[0].startswith('$argon2id$v=19$m=19456,t=2,p=1$') for v in stored)
            for c in children:assert secret.encode() not in c.transcript
        finally:
            for c in children:c.close()
    print('Credential budget PTY passed: cross-process saturation/denial, slot release, salted unchanged Argon2 and hidden input.')


if __name__=='__main__':
    import argparse
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('binary',type=Path)
    smoke(parser.parse_args().binary.resolve())
