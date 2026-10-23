#!/usr/bin/env python3
"""Validate new-only demo construction and refusal to overwrite existing board data."""
import argparse
import hashlib
from pathlib import Path
import secrets
import sqlite3
import subprocess
import tempfile


def run(binary):
    password = secrets.token_hex(20)
    with tempfile.TemporaryDirectory(prefix='faction-preview-seed-') as directory:
        parent = Path(directory)
        state = parent / 'board'
        command = [str(binary), '--new-private-state', str(state)]
        def seed(args=command, value=password):
            return subprocess.run(args, input=value + '\n', capture_output=True, text=True)
        result = seed()
        assert result.returncode == 0, 'fresh private seed failed'
        assert password not in result.stdout + result.stderr
        db_path = state / 'faction.sqlite3'
        with sqlite3.connect(f'file:{db_path}?mode=ro', uri=True) as db:
            assert db.execute('pragma user_version').fetchone() == (15,)
            assert db.execute('select count(*) from rooms').fetchone() == (5,)
            assert db.execute('select count(*) from posts').fetchone() == (5,)
            assert db.execute("select handle,role,status from members order by handle").fetchall() == [
                ('cam', 'member', 'approved'), ('nightowl', 'member', 'approved'),
                ('sysop', 'sysop', 'approved')]
            for table in ('wall_notes', 'member_pages', 'member_plans'):
                assert db.execute('select count(*) from ' + table).fetchone() == (3,)
            assert db.execute('select count(*) from post_metadata').fetchone() == (5,)
            assert db.execute('select count(*) from unread').fetchone() == (0,)
        before = hashlib.sha256(db_path.read_bytes()).digest()
        assert seed().returncode != 0
        assert hashlib.sha256(db_path.read_bytes()).digest() == before
        link = parent / 'symlink-board'
        link.symlink_to(state)
        assert seed([str(binary), '--new-private-state', str(link)]).returncode != 0
        assert hashlib.sha256(db_path.read_bytes()).digest() == before
        short = parent / 'short-password'
        assert seed([str(binary), '--new-private-state', str(short)], 'short').returncode != 0
        assert not short.exists()
        invalid = parent / 'multiline-password'
        assert seed([str(binary), '--new-private-state', str(invalid)], password + '\nother').returncode != 0
        assert not invalid.exists()
        parent.chmod(0o755)
        shared = parent / 'shared-parent'
        assert seed([str(binary), '--new-private-state', str(shared)]).returncode != 0
        assert not shared.exists()
        parent.chmod(0o700)
    print('Preview seed passed: contents/approval, no password output, existing-data/symlink '
          'preservation, short/multiline secret and shared-parent refusal.')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('binary', type=Path)
    run(parser.parse_args().binary.resolve())
