#!/usr/bin/env python3
"""Unprivileged disposable broker lifecycle/admission tests; never starts SSH.

Uses no actual remote member UID/TTY; native SSH tests cover real handoffs.
Temporary paths and generated board credentials are removed, not printed.
"""
import argparse
import json
import os
import signal
import socket
import subprocess
import tempfile
import time
from pathlib import Path


def wait_for(predicate, timeout=10):
    end = time.monotonic() + timeout
    while time.monotonic() < end:
        if predicate():
            return
        time.sleep(.05)
    raise AssertionError('lifecycle condition timed out')


def run(binary):
    assert os.geteuid() != 0, 'run as unprivileged test user'
    results = []
    with tempfile.TemporaryDirectory(prefix='faction-lifecycle-') as tmp:
        root = Path(tmp)
        state = root / 'state'; state.mkdir(mode=0o700)
        # Broker only checks initialized state presence. No SQLite contents are
        # consumed: pending malformed handoffs never reach the fake worker.
        (state / 'faction.sqlite3').touch(mode=0o600)
        endpoint = root / 'terminal.sock'
        argv = [str(binary), '--socket', str(endpoint), '--terminal-uid', str(os.geteuid()+1),
                '--worker', '/usr/bin/false', '--state-dir', str(state)]
        processes, connections = [], []

        def start():
            p = subprocess.Popen(argv, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
            processes.append(p)
            wait_for(lambda: endpoint.exists() and root.joinpath('terminal.pid').exists()
                     and root.joinpath('terminal.pid').read_text().strip() == str(p.pid))
            assert p.poll() is None
            return p

        def stopped(p, signo=signal.SIGTERM):
            begin = time.monotonic(); p.send_signal(signo); p.wait(timeout=10)
            assert p.returncode == 0
            assert not endpoint.exists()
            assert root.joinpath('terminal.pid').read_text() == ''
            return round(time.monotonic()-begin, 3)

        try:
            p = start()
            other = subprocess.run(argv, capture_output=True, timeout=5)
            assert other.returncode != 0 and p.poll() is None
            results.append('duplicate_instance_refused')
            # Eight stalled streams fill the fixed supervisor cap. None sends a
            # descriptor; the ninth is promptly closed while the first is live.
            for _ in range(8):
                s = socket.socket(socket.AF_UNIX); s.settimeout(2); s.connect(str(endpoint))
                connections.append(s)
            time.sleep(.15)
            # Prove all stalled admissions actually remain live, rather than
            # confusing immediate failure/token depletion with a bounded wait.
            for s in connections:
                s.settimeout(.02)
                try:
                    s.recv(1)
                except TimeoutError:
                    pass
                else:
                    raise AssertionError('stalled handoff closed prematurely')
            extra = socket.socket(socket.AF_UNIX); extra.settimeout(1); extra.connect(str(endpoint))
            assert extra.recv(1) == b''; extra.close()
            results.append('eight_pending_handoffs_cap_admission')
            # Shutdown while one receive is blocked and others wait on spawn lock.
            elapsed = stopped(p)
            assert elapsed < 8
            results.append('term_drains_stalled_handoffs_and_releases_socket')
            for s in connections: s.close()
            connections.clear()
            p = start(); p.kill(); p.wait(timeout=5)
            assert endpoint.is_socket()
            p = start() # lock survived as file, kernel lock was released
            results.append('kill_then_restart_recovers_owned_stale_socket')
            stopped(p, signal.SIGHUP)
            results.append('hup_is_stop_not_configuration_reload')
            # Unexpected path must never be removed as stale socket.
            endpoint.write_text('preserve')
            assert subprocess.run(argv, capture_output=True, timeout=5).returncode != 0
            assert endpoint.read_text() == 'preserve'; endpoint.unlink()
            endpoint.symlink_to(state / 'faction.sqlite3')
            assert subprocess.run(argv, capture_output=True, timeout=5).returncode != 0
            assert endpoint.is_symlink(); endpoint.unlink()
            results.append('non_socket_and_symlink_paths_preserved')
            pidfile = root / 'terminal.pid'; pidfile.unlink(); pidfile.symlink_to(state/'faction.sqlite3')
            assert subprocess.run(argv, capture_output=True, timeout=5).returncode != 0
            assert pidfile.is_symlink()
            results.append('symlink_pidfile_refused')
        finally:
            for s in connections: s.close()
            for p in processes:
                if p.poll() is None: p.kill()
                p.wait(timeout=5)
    print(json.dumps({'passed': results, 'stalled_shutdown_seconds': elapsed}))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('broker', type=Path)
    run(parser.parse_args().broker.resolve())
