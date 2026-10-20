"""Small semantic regression checks for the storage research transaction."""
import importlib.util
import sqlite3
import unittest
from pathlib import Path

SOURCE=Path(__file__).resolve().parents[1]/'experiments/009-local-storage/probe.py'
spec=importlib.util.spec_from_file_location('storage_probe',SOURCE)
probe=importlib.util.module_from_spec(spec)
spec.loader.exec_module(probe)

class StorageProbeTests(unittest.TestCase):
    def setUp(self):
        self.db=sqlite3.connect(':memory:',isolation_level=None)
        self.db.execute('PRAGMA foreign_keys=ON')
        self.db.executescript('''CREATE TABLE members(id INTEGER PRIMARY KEY,approved INTEGER);
        CREATE TABLE posts(id INTEGER PRIMARY KEY,room INTEGER,author INTEGER REFERENCES members(id),
        request TEXT,body TEXT,UNIQUE(author,request)); INSERT INTO members VALUES(1,0);''')
    def tearDown(self): self.db.close()
    def test_pending_cannot_mutate_and_transaction_rolls_back(self):
        with self.assertRaises(PermissionError): probe.post(self.db,'one','body')
        self.assertEqual(self.db.execute('SELECT count(*) FROM posts').fetchone()[0],0)
        self.assertFalse(self.db.in_transaction)
    def test_retry_same_content_preserves_identity_and_rejects_mismatch(self):
        self.db.execute('UPDATE members SET approved=1')
        identity=probe.post(self.db,'one','body')
        self.assertEqual(probe.post(self.db,'one','body'),identity)
        with self.assertRaises(ValueError): probe.post(self.db,'one','different')
        self.assertFalse(self.db.in_transaction)
        self.assertEqual(self.db.execute('SELECT count(*) FROM posts').fetchone()[0],1)
    def test_revocation_precedes_retry_lookup(self):
        self.db.execute('UPDATE members SET approved=1')
        probe.post(self.db,'one','body')
        self.db.execute('UPDATE members SET approved=0')
        with self.assertRaises(PermissionError): probe.post(self.db,'one','body')
    def test_bound_parameter_content_round_trips(self):
        self.db.execute('UPDATE members SET approved=1')
        body="'); DROP TABLE members; -- synthetic"
        probe.post(self.db,'one',body)
        self.assertEqual(self.db.execute('SELECT body FROM posts').fetchone()[0],body)
        self.assertEqual(self.db.execute('SELECT count(*) FROM members').fetchone()[0],1)

if __name__=='__main__':unittest.main()
