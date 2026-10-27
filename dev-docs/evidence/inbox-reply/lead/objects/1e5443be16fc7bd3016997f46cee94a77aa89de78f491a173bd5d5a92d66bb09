#!/usr/bin/env python3
"""Publisher-lifetime UI controls on owned local PTYs; no live-service claim."""
import argparse, fcntl, json, os, secrets, subprocess, tempfile, termios, time
from pathlib import Path
from smoke_operator import Operator, status


def run(a):
    checks = []
    children = []
    with tempfile.TemporaryDirectory(prefix='faction-operator-health-') as tmp:
        directory = Path(tmp)
        snapshot = directory / 'operator.status'
        lease_path = directory / 'operator.lease'
        instance = secrets.token_hex(16)
        def sample(stamp=None, running=True, legacy=False, queued=0):
            status(snapshot, stamp=stamp, running=running)
            raw = snapshot.read_text().replace('output_bytes=0', 'output_bytes='+str(queued))
            if not legacy:
                raw = raw.replace('version=1', 'version=2') + 'instance='+instance+'\n'
            snapshot.write_text(raw)
        def hold():
            file = lease_path.open('a+b')
            lease_path.chmod(0o600)
            fcntl.flock(file, fcntl.LOCK_EX | fcntl.LOCK_NB)
            file.seek(0); file.truncate()
            file.write(('pid=1\ninstance='+instance+'\n').encode()); file.flush()
            return file
        argv = [str(a.binary), '--state-dir', str(directory)]
        def once(*args):
            r = subprocess.run([*argv, '--plain', '--once', *args], capture_output=True, timeout=4)
            assert r.returncode == 0 and b'\x1b' not in r.stdout and r.stdout.isascii()
            assert instance.encode() not in r.stdout
            return r.stdout
        sample(legacy=True)
        assert b'Owner: UNVERIFIED / legacy sample' in once()
        assert not lease_path.exists()
        checks.append('legacy_sample_unverified_and_reader_creates_no_lease')
        file = hold()
        try:
            sample()
            out = once()
            assert b'Owner: LEASE HELD / current instance' in out and b'FRESH sample' in out
            checks.append('matching_kernel_lease_observed_separately_from_sample_age')
            sample(stamp=int(time.time())-60)
            assert b'STALE' in once() and b'LEASE HELD' in once()
            sample(stamp=int(time.time())+60)
            assert b'CLOCK UNKNOWN' in once()
            sample(running=False)
            assert b'STOPPED' in once()
            checks.append('held_lease_never_refreshes_stale_future_or_stopped_snapshot')
            sample(queued=1)
            assert b'output: buffered' in once()
            assert b'Federation: not configured.' in once('--view', 'help')
            assert b'Federation: not configured.' in once('--view', 'data')
            checks.append('bounded_queue_occupancy_and_unconfigured_federation_are_explicit')
            sample()
            c = Operator([*argv, '--ansi', '--refresh', '2']); children.append(c)
            c.expect('Owner: LEASE HELD / current instance')
            file.close()
            c.expect('NO LEASE / publisher inactive')
            checks.append('interactive_release_is_observed_without_waiting_for_sample_expiry')
            instance = secrets.token_hex(16)
            file = hold()
            c.expect('DIFFERENT / last-known sample')
            sample()
            c.expect('LEASE HELD / current instance')
            checks.append('same_pid_new_instance_rejects_old_sample_then_recovers')
            snapshot.unlink()
            c.expect('UNAVAILABLE / last-known only')
            c.expect('Owner: UNAVAILABLE')
            checks.append('failed_refresh_clears_previous_live_owner_observation')
            c.resize(40,18); c.expect('40x18'); c.expect('Q:quit P:pause R:redraw B/H/D/I:views')
            c.raw(b'q'); c.finish()
            flags = termios.tcgetattr(c.fd)[3]
            assert flags & termios.ECHO and flags & termios.ICANON
            checks.append('classic_compact_resize_exit_and_tty_restore')
            sample(); lease_path.chmod(0o644)
            assert b'Owner: UNAVAILABLE' in once()
            lease_path.chmod(0o600)
            checks.append('unsafe_lease_is_unavailable_without_exposing_instance')
        finally:
            file.close()
            for child in children: child.close()
    result = dict(result='pass', count=len(checks), checks=checks,
                  live_service_claim=False, capacity_claim=False)
    if a.output: a.output.write_text(json.dumps(result, indent=2)+'\n')
    print('Local operator lifetime UI passed',len(checks),'controls.')


if __name__ == '__main__':
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('binary',type=Path); p.add_argument('--output',type=Path)
    a = p.parse_args(); a.binary=a.binary.resolve(strict=True); run(a)
