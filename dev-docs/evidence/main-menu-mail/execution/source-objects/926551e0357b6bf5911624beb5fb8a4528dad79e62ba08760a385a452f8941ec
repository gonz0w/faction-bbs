"""Lifecycle boundaries: no VM or SSH is started by these tests."""
import os
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch
import playground as preview


class PlaygroundTests(unittest.TestCase):
    def test_private_state_rejects_shared_directory_and_symlink(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self.assertEqual(preview.private_directory(root), root.resolve())
            root.chmod(0o755)
            with self.assertRaises(ValueError):
                preview.private_directory(root)
            root.chmod(0o700)
            link = root / 'alias'
            link.symlink_to(root, target_is_directory=True)
            with self.assertRaises(ValueError):
                preview.private_directory(link)

    def test_private_records_reject_symlink_hardlink_and_shared_modes(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            path = root / 'record.json'
            preview.save(path, {'version': 1})
            self.assertEqual(preview.private_json(path), {'version': 1})
            link = root / 'alias'
            link.symlink_to(path)
            with self.assertRaises(ValueError):
                preview.private_json(link)
            link.unlink()
            os.link(path, link)
            with self.assertRaises(ValueError):
                preview.save(path, {'version': 2})
            link.unlink()
            path.chmod(0o644)
            with self.assertRaises(ValueError):
                preview.private_json(path)

    def test_reused_pid_never_receives_shutdown_or_signal(self):
        recorded = dict(pid=123, birth='old', command='qemu old')
        other = dict(pid=123, birth='new', command='unrelated process')
        with patch.object(preview, 'records', return_value={'qemu': recorded}), \
                patch.object(preview, 'process', return_value=other), \
                patch.object(preview.subprocess, 'run') as run, \
                patch.object(preview.os, 'kill') as kill:
            with self.assertRaises(ValueError):
                preview.stop(Path('/unused'), {})
            run.assert_not_called()
            kill.assert_not_called()

    def test_missing_pid_is_stopped_not_authority_to_signal(self):
        with patch.object(preview, 'process', return_value=None):
            self.assertFalse(preview.same_process(dict(pid=123)))
        with patch.object(preview.subprocess, 'run') as run:
            for pid in (0, 1, -12, True, '123'):
                self.assertIsNone(preview.process(pid))
            run.assert_not_called()

    def test_forwarding_is_literal_loopback_without_shell(self):
        state = Path('/private/tmp/preview with spaces')
        command = preview.qemu_command(state, dict(kernel='/private/tmp/kernel image',
                                                   admin_port=23456, member_port=23457))
        self.assertEqual(command[command.index('-kernel') + 1], '/private/tmp/kernel image')
        network = command[command.index('-netdev') + 1]
        self.assertEqual(network, 'user,id=net0,hostfwd=tcp:127.0.0.1:23456-:22,'
                                  'hostfwd=tcp:127.0.0.1:23457-:2222')
        self.assertIn(f'file={state}/disk.qcow2,if=none,format=qcow2,id=hd0', command)
        self.assertNotIn('shell', command)

    def test_invalid_ports_and_config_versions_fail(self):
        with tempfile.TemporaryDirectory() as directory:
            state = Path(directory)
            kernel = state / 'kernel'
            kernel.touch()
            base = dict(version=1, kernel=str(kernel), admin_port=22022, member_port=22023)
            for update in ({'version': 2}, {'member_port': 22}, {'member_port': True},
                           {'member_port': 22022}, {'admin_port': 65536}, {'extra': 'ignored?'}):
                preview.save(state / 'playground.json', dict(base, **update))
                with self.assertRaises(ValueError):
                    preview.read_config(state)

    def test_ssh_rejects_remote_target_wrong_port_and_unpinned_hosts(self):
        with tempfile.TemporaryDirectory() as directory:
            state = Path(directory)
            base = ['/usr/bin/ssh', '-o', 'StrictHostKeyChecking=yes', '-o', 'BatchMode=yes',
                    '-p', '22023', '-tt', 'factionterm@127.0.0.1']
            config = dict(member_port=22023)
            preview.save(state / 'member-argv.json', base)
            self.assertEqual(preview.ssh_command(state, 'member', config), base)
            for args in (base[:-1] + ['factionterm@remote.example'],
                         [v.replace('22023', '22024') for v in base],
                         [v.replace('Checking=yes', 'Checking=no') for v in base], []):
                preview.save(state / 'member-argv.json', args)
                with self.assertRaises((ValueError, IndexError)):
                    preview.ssh_command(state, 'member', config)

    def test_running_preview_start_is_idempotent(self):
        with patch.object(preview, 'records', return_value={'qemu': {}, 'controller': {}}), \
                patch.object(preview, 'same_process', return_value=True), \
                patch.object(preview.subprocess, 'Popen') as spawn:
            preview.start(Path('/unused'), {})
            spawn.assert_not_called()

    def test_partial_running_preview_refuses_second_disk_opener(self):
        with patch.object(preview, 'records', return_value={'qemu': {}, 'controller': {}}), \
                patch.object(preview, 'same_process', side_effect=[True, False]), \
                patch.object(preview.subprocess, 'Popen') as spawn:
            with self.assertRaises(ValueError):
                preview.start(Path('/unused'), {})
            spawn.assert_not_called()

    def test_open_disk_refuses_start_even_without_recorded_processes(self):
        from types import SimpleNamespace
        with patch.object(preview, 'records', return_value={'qemu': {}, 'controller': {}}), \
                patch.object(preview, 'same_process', return_value=False), \
                patch.object(preview.subprocess, 'run', return_value=SimpleNamespace(returncode=0, stdout=b'999\n')), \
                patch.object(preview.subprocess, 'Popen') as spawn:
            with self.assertRaises(ValueError):
                preview.start(Path('/unused'), {})
            spawn.assert_not_called()

    def test_live_loopback_listener_refuses_start(self):
        import socket
        from types import SimpleNamespace
        with socket.socket() as listener:
            listener.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
            listener.bind(('127.0.0.1', 0))
            listener.listen(1)
            port = listener.getsockname()[1]
            with patch.object(preview, 'records', return_value={'qemu': {}, 'controller': {}}), \
                    patch.object(preview, 'same_process', return_value=False), \
                    patch.object(preview.subprocess, 'run', return_value=SimpleNamespace(returncode=1, stdout=b'')), \
                    patch.object(preview.subprocess, 'Popen') as spawn:
                with self.assertRaises(OSError):
                    preview.start(Path('/unused'), dict(admin_port=port, member_port=port + 1))
                spawn.assert_not_called()


if __name__ == '__main__':
    unittest.main()
