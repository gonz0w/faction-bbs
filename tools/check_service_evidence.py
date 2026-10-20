#!/usr/bin/env python3
"""Verify saved native lifecycle oracles/frozen provenance; does not run a VM."""
import hashlib
import json
from pathlib import Path
from evidence_sources import source_bytes

ROOT = Path(__file__).resolve().parents[1]
EXPECTED = {
    'rc_status_and_duplicate_start', 'rc_restart_idle',
    'eight_approved_ssh_members_chat', 'ninth_ssh_member_rejected_at_cap',
    '100_and_250_over_cap_offers_preserve_sessions_and_descriptors',
    'worker_crash_isolated_and_reaped', 'rc_stop_drains_then_reaps_active_members',
    'broker_sigkill_exits_member_without_host_shell',
    'stale_socket_recovery_and_fresh_authenticated_login',
    'sqlite_write_contention_bounded_retry_and_recovery',
    'absolute_session_deadline_reaps_observably_blocked_writer',
    'dedicated_sshd_rc_restart',
    'fresh_login_after_service_failures_and_ordered_stop',
}


def check():
    data = json.loads((ROOT/'dev-docs/research/results/026-netbsd-lifecycle.json').read_text())
    assert data['schema'] == 1 and data['target'] == 'aarch64-unknown-netbsd'
    assert data['guest']['vcpus'] == 2 and data['native_rust_tests'] == 58
    assert data['native_pty_workflows'] == 5
    assert len(data['native_local_lifecycle']['passed']) == 7
    assert data['native_local_lifecycle']['stalled_shutdown_seconds'] < 8
    campaign = data['lifecycle']
    assert len(campaign['results']) == len(EXPECTED)
    assert {r['oracle'] for r in campaign['results']} == EXPECTED
    assert all(r['result'] == 'pass' for r in campaign['results'])
    o = campaign['observations']
    assert o['active_stop_seconds'] < 10
    assert o['broker_loss_worker_exit_seconds'] < 3
    assert o['blocked_writer_completed_chunks'] < 8191
    assert [x['offered'] for x in o['over_cap_local_offers']] == [100, 250]
    assert all(x['offered'] == x['rejected'] for x in o['over_cap_local_offers'])
    fd = o['overload_broker_descriptor_rows']
    assert fd['before'] >= 10 and fd['before'] == fd['after']
    rows = o['eight_chat_processes']
    assert sum(x['role'] == 'faction-member' for x in rows) == 8
    assert sum(x['role'] == 'faction-broker' for x in rows) == 1
    assert sum(x['role'] == 'faction-ssh' for x in rows) == 8
    assert all(set(row) == {'role','rss_kib','vsz_kib'} for row in rows)
    assert all(row['rss_kib'] > 0 for row in rows)
    assert data['cleanup']['guest_stopped'] and data['cleanup']['private_inputs_removed']
    assert len(data['sources']) >= 40
    for name, expected in data['sources'].items():
        assert hashlib.sha256(source_bytes(name,expected)).hexdigest() == expected, 'Lifecycle evidence source changed: '+name
    print('Saved NetBSD lifecycle evidence passed: 7 local + 13 real service/load/fault oracles and preserved executed source hashes. Does not rerun the VM or prove 100/250-member capacity.')


if __name__ == '__main__':
    check()
