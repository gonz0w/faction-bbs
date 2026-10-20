#!/usr/bin/env python3
"""Check saved NetBSD observations/provenance; never runs the VM or benchmark."""
import hashlib
import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
EXPERIMENT = 'experiments/023-netbsd-dependencies/'
EXPECTED_FILES = {'Cargo.toml', 'Cargo.lock', 'probe.rs', 'check_gateway.py', 'gateway-match.conf', 'audit_sources.py'}
EXPECTED_ORACLES = {
    'key_auth_forced_probe_pending_denied', 'unknown_key_denied',
    'exec_request_denied_no_host_shell', 'sftp_subsystem_denied',
    'direct_tcp_forward_denied', 'remote_tcp_forward_denied', 'remote_unix_forward_denied',
    'oversized_command_disconnect', 'pty_live_approval_and_revocation_read_post',
}

def validate():
    e = json.loads((ROOT/'dev-docs/research/results/023-netbsd-dependencies.json').read_text())
    errors = []
    def check(ok, message):
        if not ok: errors.append(message)
    check(e['schema'] == 1, 'unknown evidence schema')
    check(set(e['sources']) == {EXPERIMENT+n for n in EXPECTED_FILES}, 'source list changed')
    for name, digest in e['sources'].items():
        check(hashlib.sha256((ROOT/name).read_bytes()).hexdigest() == digest, f'source hash changed: {name}')
    lock = (ROOT/EXPERIMENT/'Cargo.lock').read_text()
    packages = {}
    for block in lock.split('[[package]]')[1:]:
        name = re.search(r'^name = "([^"]+)"$',block,re.M).group(1)
        version = re.search(r'^version = "([^"]+)"$',block,re.M).group(1)
        packages[name] = version
        if 'source = ' in block:
            check(bool(re.search(r'^checksum = "[0-9a-f]{64}"$',block,re.M)), f'missing archive hash: {name}')
    check(len(e['dependencies']) == 14, 'selected graph count changed')
    check(len({p['name'] for p in e['dependencies']}) == 14, 'duplicate dependency')
    for p in e['dependencies']:
        check(packages.get(p['name']) == p['version'], f'unlocked dependency: {p["name"]}')
        check(p['license'] and not p['forbidden_files'], f'license/source policy: {p["name"]}')
    check({b['variant'] for b in e['builds']} == {'bundled','pkgsrc-rpath'} and len(e['builds']) == 2,
          'native build variants changed')
    for b in e['builds']:
        check(b['selftest']['exit_code'] == 0 and b['binary_bytes'] > 0, 'build/runtime failure')
        check(b['selftest']['stdout'].startswith('PASS tcp_readiness unix_readiness stalled_peer bounded_input\n'),
              'missing networking observation')
        check('PASS sqlite=3.53.2 wal_full foreign_keys parameters fts5 busy_timeout_ms=' in b['selftest']['stdout'],
              'missing intended SQLite observation')
        check(b['selftest']['stdout'].endswith('approval backup record_roundtrip'), 'missing DB oracles')
        libraries = b['dynamic_dependencies']['stdout']
        check(('/usr/pkg/lib/libsqlite3.so' in libraries) if b['variant'] == 'pkgsrc-rpath'
              else ('libsqlite3.so' not in libraries), 'SQLite linking evidence mismatch')
    rows = e['gateway']['results']
    check(len(rows) == 9 and {o['oracle'] for o in rows} == EXPECTED_ORACLES, 'SSH oracle set changed')
    check(all(o['result'] == 'pass' for o in rows), 'failed SSH oracle')
    settings = dict(line.split(' ',1) for line in e['gateway_effective_settings']['stdout'].splitlines())
    for name,value in {'disableforwarding':'yes','allowtcpforwarding':'no','allowagentforwarding':'no',
                       'allowstreamlocalforwarding':'no','passwordauthentication':'no',
                       'kbdinteractiveauthentication':'no','authenticationmethods':'publickey',
                       'permittty':'yes','permituserrc':'no','permittunnel':'no','x11forwarding':'no'}.items():
        check(settings.get(name) == value, f'gateway effective setting: {name}')
    for kind in ['image','kernel']:
        check(bool(re.fullmatch('[0-9a-f]{128}',e[kind]['sha512'])), f'invalid {kind} SHA512')
    return errors

if __name__ == '__main__':
    errors = validate()
    if errors:
        raise SystemExit('\n'.join(errors))
    print('Saved NetBSD evidence/provenance valid; VM/runtime/security tests were not rerun')
