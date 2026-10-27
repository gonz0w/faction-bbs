"""The combined outage proof must reject state mutation and private-data leakage."""
import copy
import unittest
from unittest.mock import patch, MagicMock
import probe_picker_mail_outage as proof


class OutageProofOracle(unittest.TestCase):
    def test_each_canonical_component_is_exact(self):
        before = dict(canonical=b'encrypted', history=[{'body':'private'}], pending=[{'operation':'01'*16}], archive={'enabled':True,'checkpoint':{'after':'private-id','epoch':'a'}})
        proof.assert_unchanged(before, copy.deepcopy(before))
        for field in before:
            changed=copy.deepcopy(before);changed[field]=None
            with self.assertRaisesRegex(AssertionError, 'mail state changed'):
                proof.assert_unchanged(before,changed)

    def test_remote_scan_requires_all_categories_and_zero_matches(self):
        good=dict(board_files=2,store_files=1,log_files=2,matches=0,symlinks=0)
        self.assertEqual(proof.validate_scan(good),good)
        for field,value in [('board_files',0),('store_files',0),('log_files',0),('matches',1),('symlinks',1)]:
            bad=dict(good);bad[field]=value
            with self.assertRaises(AssertionError):proof.validate_scan(bad)
        with self.assertRaises(AssertionError):proof.validate_scan(dict(good,private_marker='secret'))

    def test_closed_endpoint_requires_refusal_not_timeout_or_accept(self):
        with patch.object(proof.socket, 'create_connection', side_effect=ConnectionRefusedError):
            proof.require_closed_endpoint(('127.0.0.1', 9))
        with patch.object(proof.socket, 'create_connection', side_effect=TimeoutError), self.assertRaises(AssertionError):
            proof.require_closed_endpoint(('127.0.0.1', 9))
        with patch.object(proof.socket, 'create_connection', return_value=MagicMock()), self.assertRaises(AssertionError):
            proof.require_closed_endpoint(('127.0.0.1', 9))

    def test_outage_requires_actual_tcp_connect_failure(self):
        proof.require_connection_refused(1,b'',b'Transport(Connect)\n')
        for code,out,err in [(0,b'ok',b''),(1,b'',b'Transport(Config)\n'),(1,b'',b'Transport(Tls)\n'),(1,b'private',b'Transport(Connect)\n')]:
            with self.assertRaises(AssertionError):proof.require_connection_refused(code,out,err)

if __name__=='__main__':unittest.main()
