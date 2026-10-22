"""Historical backup attempt receipts must not turn a failure into current success."""
import os,tempfile,unittest
from pathlib import Path
from unittest.mock import patch
import backup_status as status


class BackupReceipts(unittest.TestCase):
    def test_failure_retains_last_verified_metadata_and_first_failure_has_no_success(self):
        with tempfile.TemporaryDirectory() as tmp:
            p=Path(tmp);uid=os.geteuid()
            with patch.object(status.time,'time',return_value=10):status.record(p,uid,None)
            self.assertEqual(status.read(p,uid)['verified'],0)
            summary=dict(catalog_bytes=4096,object_bytes=120,objects=1)
            with patch.object(status.time,'time',return_value=20):status.record(p,uid,summary)
            with patch.object(status.time,'time',return_value=30):status.record(p,uid,None)
            receipt=status.read(p,uid)
            self.assertEqual((receipt['attempted'],receipt['verified'],receipt['result']),(30,20,0))
            self.assertEqual(receipt['object_bytes'],120)
            raw=(p/'backup.status').read_bytes();self.assertLessEqual(len(raw),status.CAP)
            self.assertEqual((p/'backup.status').stat().st_mode & 0o777,0o600)
            for value in [b'path=',b'password',b'filename',b'handle']:self.assertNotIn(value,raw)

    def test_failed_or_busy_publication_keeps_previous_receipt(self):
        with tempfile.TemporaryDirectory() as tmp:
            p=Path(tmp);uid=os.geteuid();status.record(p,uid,dict(catalog_bytes=4096,object_bytes=0,objects=0))
            before=(p/'backup.status').read_bytes();busy=p/'.backup.status.next';busy.write_bytes(b'owned busy marker')
            with self.assertRaises(FileExistsError):status.record(p,uid,None)
            self.assertEqual(busy.read_bytes(),b'owned busy marker');busy.unlink()
            with self.assertRaises(ValueError):status.record(p,uid,dict(catalog_bytes=0,object_bytes=0,objects=0))
            self.assertEqual((p/'backup.status').read_bytes(),before)
            self.assertFalse(busy.exists())
            (p/'backup.status').chmod(0o644)
            with self.assertRaises(ValueError):status.read(p,uid)

    def test_unknown_or_inconsistent_protocol_and_aliases_are_refused(self):
        with tempfile.TemporaryDirectory() as tmp:
            p=Path(tmp);uid=os.geteuid();status.record(p,uid,dict(catalog_bytes=4096,object_bytes=0,objects=0))
            raw=(p/'backup.status').read_bytes()
            for bad in [raw+b'path=private\n',raw.replace(b'result=1',b'result=2'),raw.replace(b'objects=0',b'objects=2001'),b'x'*513]:
                with self.assertRaises(ValueError):status.parse(bad)
            (p/'backup.status').rename(p/'original');(p/'backup.status').symlink_to(p/'original')
            with self.assertRaises(OSError):status.read(p,uid)
            (p/'backup.status').unlink();os.link(p/'original',p/'backup.status')
            with self.assertRaises(ValueError):status.read(p,uid)
