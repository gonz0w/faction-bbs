"""Contract checks for the retained-correspondent PTY observation oracle."""
import copy
import unittest

import probe_mail_correspondents as proof


class CorrespondentOracle(unittest.TestCase):
    def setUp(self):
        self.before = {
            'history': [
                {'address': 'bob@away.test', 'device': 202, 'direction': 'received',
                 'read': False, 'received_id': '11' * 32},
                {'address': 'bob@away.test', 'device': 203, 'direction': 'received',
                 'read': False, 'received_id': '22' * 32},
                {'address': 'bob@away.test', 'device': 202, 'direction': 'sent',
                 'read': None, 'received_id': None},
            ],
            'pending': [{'operation': '33' * 16}],
            'archive': {'cursor': 'synthetic-retained-position'},
        }

    def test_exact_account_and_device_keep_original_canonical_indices(self):
        proof.require_group_records(self.before, 'bob@away.test', 202, [2, 0])
        for indices in [[2, 1, 0], [0, 2], [1], [], [2, 2]]:
            with self.subTest(indices=indices), self.assertRaises(AssertionError):
                proof.require_group_records(self.before, 'bob@away.test', 202, indices)

    def test_received_ack_changes_only_the_selected_original_record(self):
        after = copy.deepcopy(self.before)
        after['history'][0]['read'] = True
        proof.require_only_read_change(self.before, after, 0)
        for mutate in [
            lambda x: x['history'][1].update(read=True),
            lambda x: x['history'][0].update(device=203),
            lambda x: x['history'].reverse(),
            lambda x: x['pending'].clear(),
            lambda x: x['archive'].update(cursor='wrong'),
        ]:
            changed = copy.deepcopy(after)
            mutate(changed)
            with self.subTest(changed=changed), self.assertRaises(AssertionError):
                proof.require_only_read_change(self.before, changed, 0)
        with self.assertRaises(AssertionError):
            proof.require_only_read_change(self.before, self.before, 0)
        with self.assertRaises(AssertionError):
            proof.require_only_read_change(self.before, self.before, 2)

    def test_visible_indices_reject_group_leak_and_selection_drift(self):
        screen = b'> 03 PENDING current\r\n  01 UNREAD old\r\n'
        proof.require_visible_indices(screen, [3, 1], 3)
        for screen in [b'> 03 PENDING current\r\n  02 UNREAD other\r\n',
                       b'  03 PENDING current\r\n> 01 UNREAD old\r\n',
                       b'> 03 PENDING current\r\n> 01 UNREAD old\r\n']:
            with self.subTest(screen=screen), self.assertRaises(AssertionError):
                proof.require_visible_indices(screen, [3, 1], 3)


if __name__ == '__main__':
    unittest.main()
