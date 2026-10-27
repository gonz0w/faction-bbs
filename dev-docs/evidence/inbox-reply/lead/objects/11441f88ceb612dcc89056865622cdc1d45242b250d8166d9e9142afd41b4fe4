"""Guard invitation/exposure policy without mutating a host."""
import argparse
import base64
from pathlib import Path
import struct
import tempfile
import unittest
from install_netbsd import listen_address, read_public_keys, render


class SmallBoardInstallTests(unittest.TestCase):
    def key(self):
        kind = b'ssh-ed25519'
        blob = struct.pack('>I', len(kind)) + kind + struct.pack('>I', 32) + bytes(range(32))
        return 'ssh-ed25519 ' + base64.b64encode(blob).decode()

    def read(self, content):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'invited.pub'
            path.write_text(content)
            return read_public_keys(path)

    def test_restricted_keys_strip_comments_and_deduplicate(self):
        key = self.key()
        self.assertEqual(self.read(f'# invitations\n{key} user\n{key} other\n'), f'restrict,pty {key}\n')

    def test_member_key_options_private_keys_and_empty_input_fail(self):
        for value in ('', '# empty\n', '-----BEGIN OPENSSH PRIVATE KEY-----',
                      'command="sh" ' + self.key(), 'restrict ' + self.key(), 'ssh-ed25519 !!!!!'):
            with self.subTest(value=value), self.assertRaises(ValueError):
                self.read(value)

    def test_mismatched_key_type_fails(self):
        with self.assertRaises(ValueError):
            self.read(self.key().replace('ssh-ed25519 ', 'ssh-rsa ', 1))

    def test_wildcard_multicast_and_hostnames_fail(self):
        for address in ('0.0.0.0', '::', '224.0.0.1', 'ff02::1', 'bbs.example', '10.0.0.2\nPort 22'):
            with self.subTest(address=address), self.assertRaises(argparse.ArgumentTypeError):
                listen_address(address)
        self.assertEqual(listen_address('127.0.0.1'), '127.0.0.1')
        self.assertEqual(listen_address('fd00::1'), 'fd00::1')

    def test_render_keeps_fixed_authority_and_normal_defaults(self):
        ssh, service = render(1001, 1002, listen_address('10.0.0.4'), 2223)
        self.assertIn('ListenAddress 10.0.0.4\n', ssh)
        self.assertIn('Port 2223\n', ssh)
        self.assertIn('ForceCommand /usr/local/libexec/faction-tty --socket /var/run/faction/terminal.sock --owner-uid 1001\n', ssh)
        for directive in ('AllowUsers factionterm', 'DisableForwarding yes', 'PasswordAuthentication no', 'PermitUserRC no'):
            self.assertIn(directive + '\n', ssh)
        self.assertIn('terminal_uid = 1002\n', service)
        self.assertIn('max_sessions = 8\n', service)
        self.assertIn('experimental_capacity = false\n', service)
        self.assertIn('theme = "/var/db/faction/themes/base"\n', service)
        self.assertNotIn('@FACTION_', ssh + service)


if __name__ == '__main__':
    unittest.main()
