"""Independent approval and bounded dedicated-forward qualification contracts."""
import copy
import importlib
import os
from pathlib import Path
import subprocess
import tempfile
import time
import unittest
from unittest.mock import patch


class EnrollmentTests(unittest.TestCase):
    def module(self):
        return importlib.import_module('probe_enrolled_provider')

    def claims(self):
        return dict(version=1, account='alice@home.test', device=101,
                    identity='01'*32, certificate_sha256='02'*32)

    def verified(self):
        return dict(self.claims(), account_control_verified=True,
                    verification_channel='owned independent member inspection')

    def test_card_alone_and_every_substituted_claim_refuse_independent_approval(self):
        p = self.module()
        card, verified = self.claims(), self.verified()
        self.assertEqual(p.validate_approval(card, '02'*32, verified), card)
        for field, value in [('account', 'bob@home.test'), ('device', 202),
                             ('identity', '03'*32), ('certificate_sha256', '04'*32),
                             ('account_control_verified', False),
                             ('verification_channel', '')]:
            changed = dict(verified, **{field: value})
            with self.subTest(field=field), self.assertRaises(ValueError):
                p.validate_approval(card, '02'*32, changed)
        with self.assertRaises(ValueError):
            p.validate_approval(card, '02'*32, card)
        with self.assertRaises(ValueError):
            p.validate_approval(card, '05'*32, verified)
        self.assertEqual(card, self.claims())

    def test_approval_rejects_noncanonical_or_unknown_claims(self):
        p = self.module()
        for field, value in [('version', True), ('device', True), ('device', 0),
                             ('identity', 'AA'*32), ('identity', '00'*32),
                             ('certificate_sha256', '00'*32),
                             ('account', 'alice@away.test'), ('extra', 'ignored')]:
            card = dict(self.claims(), **{field: value})
            verified = dict(card, account_control_verified=True,
                            verification_channel='independent')
            with self.subTest(field=field, value=value), self.assertRaises(ValueError):
                p.validate_approval(card, card['certificate_sha256'], verified)

    def test_complete_proposal_retains_exact_verified_values_and_policy_flags(self):
        import tomllib
        p = self.module()
        first = self.claims()
        second = dict(first, account='bob@home.test', device=202, identity='03'*32,
                      certificate_sha256='04'*32)
        before = copy.deepcopy([first, second])
        text = p.approved_proposal([first, second], generation=7, reset_epoch=3,
                                   approved=[False, True], stamp=1000)
        value = tomllib.loads(text)
        self.assertEqual(value['generation'], 7)
        self.assertEqual(value['reset_epoch'], 3)
        self.assertEqual(value['homes'], ['home.test'])
        self.assertEqual([b['identity'] for b in value['bindings']],
                         [first['identity'], second['identity']])
        self.assertEqual([b['approved'] for b in value['bindings']], [False, True])
        self.assertTrue(all(b['generation'] == 7 for b in value['bindings']))
        self.assertEqual([first, second], before)
        self.assertNotIn('key', value['bindings'][0])
        with self.assertRaises(ValueError):
            p.approved_proposal([first, first], generation=7, reset_epoch=3,
                                approved=[True, True], stamp=1000)

    def test_only_public_enrollment_artifacts_can_cross_to_operator(self):
        p = self.module()
        with tempfile.TemporaryDirectory(prefix='faction293-artifacts-OWNED-') as name:
            root = Path(name).resolve(); root.chmod(0o700)
            member, operator = root/'member', root/'operator'
            member.mkdir(mode=0o700); operator.mkdir(mode=0o700)
            for filename, data in [('request.csr', b'public signed request'),
                                   ('device.pem', b'public certificate'),
                                   ('card.toml', b'public claims'),
                                   ('device.key', b'PRIVATE-KEY-MARKER')]:
                (member/filename).write_bytes(data)
                (member/filename).chmod(0o600)
            p.copy_public(member/'request.csr', operator/'request.csr', 'csr')
            p.copy_public(member/'device.pem', operator/'device.pem', 'certificate')
            p.copy_public(member/'card.toml', operator/'card.toml', 'card')
            with self.assertRaises(ValueError):
                p.copy_public(member/'device.key', operator/'stolen.key', 'key')
            self.assertEqual(sorted(p.name for p in operator.iterdir()),
                             ['card.toml', 'device.pem', 'request.csr'])
            self.assertNotIn(b'PRIVATE-KEY-MARKER', b''.join(p.read_bytes() for p in operator.iterdir()))
            with self.assertRaises(FileExistsError):
                p.copy_public(member/'card.toml', operator/'card.toml', 'card')

    def test_csr_verification_uses_explicit_config_without_system_openssl_config(self):
        p = self.module()
        with tempfile.TemporaryDirectory(prefix='faction293-csr-OWNED-') as name:
            root = Path(name).resolve(); root.chmod(0o700)
            member, operator = root/'member', root/'operator'
            member.mkdir(mode=0o700); operator.mkdir(mode=0o700)
            (operator/'ca.pem').write_bytes(b'public CA certificate')
            (operator/'ca.pem').chmod(0o600)
            calls = []
            def openssl(argv):
                calls.append(argv)
                if argv[1] == 'req' and '-new' in argv:
                    Path(argv[argv.index('-keyout')+1]).write_bytes(b'PRIVATE KEY')
                    Path(argv[argv.index('-out')+1]).write_bytes(b'public signed CSR')
                elif argv[1] == 'req' and '-verify' in argv:
                    self.assertIn('-config', argv,
                                  'CSR verification must not depend on a system openssl.cnf')
                    self.assertEqual(Path(argv[argv.index('-config')+1]), member/'openssl.cnf')
                    self.assertTrue((member/'openssl.cnf').is_file())
                    self.assertEqual(Path(argv[argv.index('-in')+1]), operator/'alice-request.csr')
                elif argv[1] == 'x509':
                    Path(argv[argv.index('-out')+1]).write_bytes(b'public issued certificate')
                else:
                    self.fail('unexpected credential operation')
                return b''
            with patch.object(p, 'command', side_effect=openssl):
                certificate, key = p.member_credentials(member, operator, 'alice', '/openssl')
            self.assertEqual(certificate, member/'device.pem')
            self.assertEqual(key, member/'device.key')
            self.assertEqual(sum('-verify' in argv for argv in calls), 1)
            self.assertFalse(any(path.suffix == '.key' for path in operator.iterdir()))

    def test_replacement_requires_independent_reapproval_and_retires_exact_old_digest(self):
        import tomllib
        p = self.module()
        old = self.claims()
        replacement = dict(old, certificate_sha256='06'*32)
        verified = dict(self.verified(), certificate_sha256='06'*32)
        bob = dict(old, account='bob@home.test', device=202, identity='03'*32,
                   certificate_sha256='04'*32)
        text = p.rotation_proposal([old, bob], replacement, verified, generation=4,
                                  reset_epoch=2, stamp=1000)
        rows = tomllib.loads(text)['bindings']
        self.assertEqual([row['certificate_sha256'] for row in rows], ['06'*32, '04'*32])
        self.assertEqual(rows[0]['identity'], old['identity'])
        self.assertNotIn(old['certificate_sha256'], text)
        self.assertEqual(old, self.claims())
        with self.assertRaises(ValueError):
            p.rotation_proposal([old, bob], replacement, self.verified(), generation=4,
                                reset_epoch=2, stamp=1000)
        changed = dict(replacement, identity='07'*32)
        with self.assertRaises(ValueError):
            p.rotation_proposal([old, bob], changed, dict(verified, identity='07'*32),
                                generation=4, reset_epoch=2, stamp=1000)

    def test_bad_credentials_have_current_policy_and_only_expiry_case_has_short_interval(self):
        import tomllib
        p = self.module()
        cards = [self.claims(), dict(self.claims(), account='bob@home.test', device=202,
                                    identity='03'*32, certificate_sha256='04'*32)]
        current = p.scenario_policy(cards, 'current', generation=1, reset_epoch=1, stamp=1000)
        expired = p.scenario_policy(cards, 'expired', generation=2, reset_epoch=1, stamp=1000)
        self.assertTrue(all(row['valid_until'] == 4600 for row in tomllib.loads(current)['bindings']))
        self.assertEqual([row['valid_until'] for row in tomllib.loads(expired)['bindings']], [1002, 4600])
        p.require_policy_case(current, 'current', 1003)
        p.require_policy_case(expired, 'expired', 1003)
        for policy, scenario, stamp in [(expired, 'current', 1003), (current, 'expired', 1003),
                                         (expired, 'expired', 1001), (current, 'current', 4601)]:
            with self.assertRaises(ValueError):
                p.require_policy_case(policy, scenario, stamp)
        revoked = p.scenario_policy(cards, 'revoked', generation=4, reset_epoch=1, stamp=1000)
        p.require_policy_case(revoked, 'revoked', 1003)
        with self.assertRaises(ValueError): p.require_policy_case(revoked, 'current', 1003)


class ForwardTests(unittest.TestCase):
    def module(self):
        return importlib.import_module('probe_enrolled_provider')

    def test_dedicated_client_is_pinned_local_only_and_has_no_remote_command(self):
        p = self.module()
        args = p.forward_argv('/usr/bin/ssh', '/private/key', '/private/known_hosts',
                              '127.0.0.1', 2222, 'forwardonly', 41001, 41002,
                              '/private/control')
        self.assertEqual(args[0], '/usr/bin/ssh')
        self.assertEqual(args[-1], 'forwardonly@127.0.0.1')
        self.assertIn('-N', args); self.assertIn('-T', args)
        self.assertEqual(args[args.index('-F')+1], '/dev/null')
        self.assertEqual(args[args.index('-L')+1], '127.0.0.1:41001:127.0.0.1:41002')
        options = [args[i+1] for i, arg in enumerate(args) if arg == '-o']
        for option in ['BatchMode=yes', 'StrictHostKeyChecking=yes', 'IdentitiesOnly=yes',
                       'ForwardAgent=no', 'ForwardX11=no', 'PermitLocalCommand=no',
                       'ControlMaster=yes', 'ControlPersist=no', 'ExitOnForwardFailure=yes']:
            self.assertIn(option, options)
        for address, port, user in [('192.0.2.1', 2222, 'forwardonly'),
                                    ('127.0.0.1', 0, 'forwardonly'),
                                    ('127.0.0.1', 2222, '-option')]:
            with self.assertRaises(ValueError):
                p.forward_argv('/usr/bin/ssh', '/private/key', '/private/known_hosts',
                               address, port, user, 41001, 41002, '/private/control')

    def test_setup_failure_reaps_owned_child_without_connecting_or_claiming_provider_ready(self):
        p = self.module()
        with tempfile.TemporaryDirectory(prefix='faction293-forward-OWNED-') as name:
            root = Path(name).resolve(); root.chmod(0o700)
            executable = root/'unready'
            executable.write_text('#!/usr/bin/env python3\nimport time\ntime.sleep(30)\n')
            executable.chmod(0o700)
            children = []; original = subprocess.Popen
            def record(*args, **kwargs):
                child = original(*args, **kwargs); children.append(child); return child
            began = time.monotonic()
            with patch.object(p.subprocess, 'Popen', side_effect=record):
                with self.assertRaises(ValueError):
                    p.Forward([str(executable)], root, setup_seconds=.1,
                              check=lambda: False)
            self.assertLess(time.monotonic()-began, 5)
            self.assertEqual(len(children), 1)
            self.assertIsNotNone(children[0].poll())
            self.assertTrue(children[0].stderr.closed)

    def test_both_display_modes_use_supplied_dedicated_ssh_fixture(self):
        p = self.module()
        args = p.make_parser().parse_args([
            '--client', '/client', '--fixture', '/member-seed', '--operator', '/operator',
            '--owned-root', '/owned', '--output', '/owned/result.json', '--ssh', '/ssh',
            '--sshd-log', '/sshd-log',
            '--ssh-key', '/key', '--known-hosts', '/known', '--ssh-user', 'forwardonly',
            '--ssh-port', '2222', '--provider-port', '41002', '--forward-port', '41001',
            '--denied-ssh-key', '/bbs-key', '--denied-ssh-key', '/sftp-key',
            '--openssl', '/openssl'])
        with patch.object(p, 'require_root', return_value=Path('/owned')):
            with patch.object(p, 'one_journey', return_value={'checks': ['observed']}) as journey:
                result = p.probe(args)
        self.assertEqual([call.args[-1] for call in journey.call_args_list], [False, True])
        self.assertTrue(all(call.args[0] is args for call in journey.call_args_list))
        self.assertEqual(result['scope'], 'same-os-loopback-dedicated-ssh-software-proof')
        self.assertFalse(result['provider_deployment'])

    def test_denied_roles_request_the_destination_channel_and_reject_timeout_as_evidence(self):
        p = self.module()
        fixture = dict(ssh='/usr/bin/ssh', key='/private/role-key', known_hosts='/private/known',
                       address='127.0.0.1', ssh_port=2222, user='forwardonly')
        forward = p.denial_argv(**fixture, request='forward', target_port=41002)
        self.assertEqual(forward[forward.index('-W')+1], '127.0.0.1:41002')
        self.assertNotIn('-L', forward)
        self.assertNotIn('-N', forward)
        subsystem = p.denial_argv(**fixture, request='subsystem')
        self.assertEqual(subsystem[-2:], ['forwardonly@127.0.0.1', 'sftp'])
        self.assertIn('-s', subsystem)
        self.assertTrue(p.require_ssh_denial(subprocess.CompletedProcess(forward, 255, b'',
            b'channel 0: open failed: administratively prohibited: open failed\n'), 'forward'))
        for result in [subprocess.CompletedProcess(forward, 0, b'', b''),
                       subprocess.CompletedProcess(forward, 255, b'', b'Permission denied (publickey).'),
                       subprocess.CompletedProcess(forward, 255, b'', b'connect failed: Connection refused'),
                       subprocess.CompletedProcess(forward, 255, b'', b'Connection timed out')]:
            with self.assertRaises(ValueError):
                p.require_ssh_denial(result, 'forward')

    def test_session_channel_refusal_requires_fresh_authenticated_same_attempt_server_evidence(self):
        p = self.module()
        result = subprocess.CompletedProcess([], 255, b'',
            b'channel 0: open failed: connect failed: open failed\r\n')
        log = (b'Accepted publickey for forwardonly from 127.0.0.1 port 12345 ssh2: ED25519 SHA256:fixture\n'
               b'User child is on pid 4321\nno more sessions\n')
        self.assertTrue(p.require_ssh_denial(result, 'shell', server_log=log,
                                           expected_user='forwardonly'))
        self.assertTrue(p.require_ssh_denial(result, 'subsystem', server_log=log,
                                           expected_user='forwardonly'))
        for fragment in [b'', b'no more sessions\n', log.replace(b'forwardonly', b'other'),
                         log.replace(b'no more sessions', b'normal close'), log+log]:
            with self.assertRaises(ValueError):
                p.require_ssh_denial(result, 'shell', server_log=fragment,
                                     expected_user='forwardonly')
        with self.assertRaises(ValueError):
            p.require_ssh_denial(result, 'forward', server_log=log,
                                 expected_user='forwardonly')

    def test_log_interval_allows_later_growth_but_rejects_truncation_or_identity_change(self):
        p = self.module()
        with tempfile.TemporaryDirectory(prefix='faction293-log-OWNED-') as name:
            log = Path(name)/'sshd.log'; log.write_bytes(b'older\nfresh\nlater\n')
            before = (1, 2, 6)
            with patch.object(p, 'log_position', side_effect=[(1, 2, 12), (1, 2, 18)]):
                self.assertEqual(p.appended_log(log, before), b'fresh\n')
            for after in [(1, 2, 5), (1, 3, 12), (1, 2, 6+65537)]:
                with patch.object(p, 'log_position', return_value=after):
                    with self.assertRaises(ValueError): p.appended_log(log, before)


if __name__ == '__main__':
    unittest.main()
