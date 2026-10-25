"""Contract tests for the product-transport fixture's public result boundary."""
import unittest
import probe_home_transport as fixture


class ResultBoundary(unittest.TestCase):
    def test_only_known_status_and_one_attempt_are_accepted(self):
        self.assertEqual(fixture.validate_result(b'{"status":"ok","attempts":1}\n', b'', 0, {'ok'}, 'private-password', 'private-payload')['status'], 'ok')
        for stdout, stderr, code in [
            (b'{"status":"remote error text","attempts":1}', b'', 1),
            (b'{"status":"ok","attempts":2}', b'', 0),
            (b'{"status":"ok","attempts":1,"unreviewed":"field"}', b'', 0),
            (b'{"status":"ok","attempts":1}', b'private diagnostic', 0),
            (b'{"status":"ok","attempts":1}', b'', 1),
            (b'private-password', b'', 1),
            (b'private-payload', b'', 1),
        ]:
            with self.subTest(stdout=stdout):
                with self.assertRaises((AssertionError, ValueError)):
                    fixture.validate_result(stdout, stderr, code, {'ok'}, 'private-password', 'private-payload')

    def test_source_matrix_covers_qualified_failures_and_binding(self):
        names = {name for name, _, _ in fixture.SCENARIOS}
        self.assertTrue({'wrong_server_name', 'untrusted_certificate', 'changed_certificate_pin', 'failed_account_authentication', 'explicit_cancellation_bounds_stalled_peer', 'wrong_bound_identity', 'wrong_bind_identifier'} <= names)
        self.assertEqual(len(names), len(fixture.SCENARIOS))


if __name__ == '__main__':
    unittest.main()
