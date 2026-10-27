#!/usr/bin/env python3
"""Real trusted-local checkpoint PTY acceptance. Synthetic owned fixtures only.

No listener, provider, environment/argument secret, or saved playground. The optional
lab-only fixture executable builds authenticated history/outbox with the real backend.
Only sanitized check names/counts are emitted; terminal transcripts stay in memory.
"""
import argparse
import json
import os
import platform
import re
import secrets
import select
import time
import subprocess
import stat
import sys
import tempfile
from pathlib import Path
from smoke_native_client_ssh import NativeConsole


def probe(binary, fixture, owner):
    owner.mkdir(mode=0o700, parents=True, exist_ok=True)
    info = owner.lstat()
    assert stat.S_ISDIR(info.st_mode) and info.st_uid == os.getuid()
    assert info.st_mode & 0o777 == 0o700
    checks, children = [], []
    password = secrets.token_hex(24)
    with tempfile.TemporaryDirectory(prefix='checkpoint-ui-', dir=owner) as tmp:
        root = Path(tmp).resolve()
        store = root / 'created'
        def passed(name): checks.append(name)
        def start(path=store, action='open', unicode=False):
            args = [str(binary), '--mail-checkpoints', str(path), action]
            if unicode: args.append('--unicode')
            c = NativeConsole(args); children.append(c); return c
        def menu(c): c.expect('[L] Lock and exit')
        def pause(c): c.expect('Enter to return: '); c.send(''); menu(c)
        def unlock(path=store, secret=password, unicode=False):
            c = start(path, unicode=unicode)
            c.password(secret, 'LOCAL checkpoint passphrase (hidden): ')
            menu(c); assert secret.encode() not in c.transcript
            return c
        def recover_display(c):
            c.expect('Save it privately, then Enter to clear: ')
            # A trailing paste/newline must not dismiss a just-created recovery.
            deadline = time.monotonic()+0.2
            while time.monotonic()<deadline and select.select([c.fd],[],[],0.02)[0]:
                data = os.read(c.fd,8192); c.buffer += data; c.transcript += data
            assert b'[L] Lock and exit' not in c.buffer, 'recovery was dismissed by typeahead'
            keys = re.findall(rb'\r\n\r\n([a-f0-9]{64})\r\n', c.transcript)
            assert len(keys) == 1, 'one new recovery secret must be displayed'
            c.send(''); menu(c); return keys[0].decode()
        def lock(c): c.raw(b'l'); c.finish()
        def seed(name, mode='pending'):
            path = root / name
            result = subprocess.run([str(fixture), str(path), mode], capture_output=True)
            assert result.returncode == 0, 'synthetic checkpoint fixture failed'
            return path
        try:
            # This first actual PTY control fails on the old client: the CLI entry
            # is absent and Usage exits before the LOCAL checkpoint screen appears.
            c = start(action='init')
            c.expect('[LOCAL / OFFLINE] Native mail checkpoints')
            c.expect('Your full home identity (local claim only): '); c.send('alice@home.test')
            c.expect('Local device ID (1-2147483647): '); c.send('101')
            c.password(password, 'New LOCAL passphrase (12-128 ASCII characters, hidden): ')
            c.expect('Repeat LOCAL passphrase (hidden): '); c.send(password+'\n')
            recovery = recover_display(c)
            assert password.encode() not in c.transcript and b'\x1b' not in c.transcript
            passed('explicit_local_create_hidden_secret_and_new_recovery')
            c.raw(b'p'); c.expect('Local public device: 101'); c.expect('ed25519:'); pause(c)
            busy = start(); busy.expect('Checkpoint locked: '); busy.finish(1)
            passed('public_device_inspection_and_exclusive_lock')
            c.raw(b'v'); c.expect('Member (full user@home): '); c.send('bob@away.test')
            c.expect('Peer device ID (1-2147483647): '); c.send('202')
            c.expect('Full independently verified identity (ed25519: + 64 lowercase hex): '); c.send('ed25519:' + 'ab'*32)
            c.expect('Type verified after independent comparison: '); c.send('verified'); c.expect('Peer enrolled locally.'); pause(c)
            c.raw(b'p'); c.expect('Peer: bob@away.test / 202 / ed25519:'+'ab'*32); pause(c)
            lock(c); passed('typed_ed25519_peer_enrollment_survives_reopen')
            c = unlock(); c.raw(b'v'); c.expect('Peer already enrolled;'); pause(c); lock(c)
            passed('changed_or_server_added_peer_is_never_silently_accepted')
            c = start(); c.password('wrong checkpoint password', 'LOCAL checkpoint passphrase (hidden): ')
            c.expect('Checkpoint locked: '); c.finish(1)
            assert b'Peer:' not in c.transcript; passed('authentication_failure_never_renders_private_state')
            c = unlock(); c.raw(b'\x03'); c.finish(1); passed('cancel_relocks_and_restores_terminal')
            wrapper = root/'return-probe'
            wrapper.write_text('#!/usr/bin/env python3\nimport os, select, subprocess, sys\nr=subprocess.run(sys.argv[1:])\nprint("TYPEAHEAD_REMAINED" if select.select([0],[],[],0.2)[0] else "NO_TYPEAHEAD", flush=True)\nsys.exit(r.returncode)\n')
            wrapper.chmod(0o700)
            c = NativeConsole([sys.executable, str(wrapper), str(binary), '--mail-checkpoints', str(store), 'open']); children.append(c)
            c.password('x'*129+'trailing-secret-marker', 'LOCAL checkpoint passphrase (hidden): ')
            c.expect('Checkpoint locked: '); c.expect('NO_TYPEAHEAD'); c.finish(1)
            passed('oversize_hidden_secret_rejected_and_queued_secret_flushed_before_shell')
            # Pasted typeahead must be flushed before a hidden prompt. If it were
            # consumed as the password this would exit before the next input.
            c = start(action='init', path=root/'typeahead')
            c.expect('Your full home identity (local claim only): '); c.send('alice@home.test')
            c.expect('Local device ID (1-2147483647): ')
            c.raw(b'303\nnot-the-unlock-secret\n')
            c.password(password, 'New LOCAL passphrase (12-128 ASCII characters, hidden): ')
            c.password(password, 'Repeat LOCAL passphrase (hidden): '); recover_display(c); lock(c)
            passed('typeahead_before_hidden_passphrase_is_discarded')
            reject = root/'typeahead'
            before = (reject/'mail.fkv').read_bytes()
            for identity in ['ab'*32, 'ed25519:'+'ab'*31]:
                c = unlock(reject); c.raw(b'v')
                c.expect('Member (full user@home): '); c.send('bob@away.test')
                c.expect('Peer device ID (1-2147483647): '); c.send('202')
                c.expect('Full independently verified identity (ed25519: + 64 lowercase hex): '); c.send(identity)
                c.expect('Type verified after independent comparison: '); c.send('verified')
                c.expect('Checkpoint locked: '); c.finish(1)
                assert (reject/'mail.fkv').read_bytes() == before
            passed('legacy_curve_or_partial_ed_identity_cannot_enroll')
            c = unlock(reject); c.raw(b'v')
            c.expect('Member (full user@home): '); c.send('bob@away.test')
            c.expect('Peer device ID (1-2147483647): '); c.send('202')
            c.expect('Full independently verified identity (ed25519: + 64 lowercase hex): ')
            c.raw(('ed25519:'+'ab'*32).encode()+b'\nverified\n')
            c.expect('Type verified after independent comparison: '); c.send('no')
            c.expect('Checkpoint locked: '); c.finish(1)
            assert (reject/'mail.fkv').read_bytes() == before
            c = unlock(reject); c.raw(b'v')
            c.expect('Member (full user@home): '); c.raw(b'bob@away.test\x1b[31m\n')
            c.expect('Checkpoint locked: '); c.finish(1)
            assert (reject/'mail.fkv').read_bytes() == before
            passed('confirmation_typeahead_and_control_splicing_cannot_change_trust')
            if fixture is None:
                return checks
            pending = seed('pending')
            c = unlock(pending, 'checkpoint synthetic fixture password')
            c.raw(b'h'); c.expect('Authenticated history: 2/64'); c.expect('PENDING / local ciphertext; delivery unproven')
            c.expect('Operation: '+'01'*16); c.expect('Subject: synthetic subject \\u{3bb}')
            c.expect('Body: inert \\u{a}line \\u{202e}safe'); c.expect('Enter for next record: '); c.send('')
            c.expect('RECEIVED / authenticated local history'); c.expect('Enter for next record: '); c.send(''); pause(c)
            passed('bounded_authenticated_history_ids_and_classic_inert_unicode')
            c.raw(b'o'); c.expect('Durable outbox: 1/8'); c.expect('OFFLINE: no delivery attempted by this UI. Remote delivery is unknown.')
            c.expect('Operation: '+'01'*16); pause(c)
            c.raw(b'f'); c.expect('Retire pending ciphertext explicitly before changing devices.'); pause(c)
            passed('reopened_outbox_persisted_ids_pending_and_fresh_device_guard')
            c.raw(b't'); c.expect('Operation ID to retire (32 lowercase hex, or all): '); c.send('01'*16)
            c.expect('Type retire to record this LOCAL decision: '); c.send('retire'); c.expect('Retired locally.'); pause(c)
            c.raw(b'h'); c.expect('RETIRED / local decision; delivery unproven'); c.expect('Enter for next record: '); c.send('')
            c.expect('Enter for next record: '); c.send(''); pause(c); lock(c)
            passed('explicit_retirement_preserves_history_without_fake_delivery_claim')
            c = unlock(pending, 'checkpoint synthetic fixture password', unicode=True)
            c.raw(b'h'); c.expect('Subject: synthetic subject λ'); c.expect('Body: inert \\u{a}line \\u{202e}safe')
            c.expect('Enter for next record: '); c.send(''); c.expect('Enter for next record: '); c.send(''); pause(c); lock(c)
            passed('explicit_unicode_override_keeps_control_and_bidi_text_inert')
            c = unlock(pending, 'checkpoint synthetic fixture password')
            c.raw(b'b'); c.expect('NEW absolute history-only backup directory: '); c.send(str(root/'backup'))
            c.expect('Encrypted HISTORY-ONLY backup created.'); pause(c); lock(c)
            c = unlock(root/'backup', 'checkpoint synthetic fixture password')
            assert b'HISTORY ONLY' in c.transcript
            c.raw(b'f'); c.expect('New local device ID (different from current): '); c.send('909')
            c.expect('Type fresh device to retire the old protocol identity: '); c.send('fresh device')
            c.expect('Fresh local device created. Independently verify and enroll the peer again.'); pause(c); lock(c)
            passed('official_backup_is_history_only_and_fresh_device_needs_reverification')
            for mode, label in [('expired','EXPIRED'), ('revoked','REVOKED'), ('full','HISTORY FULL')]:
                path = seed(mode, mode); c = unlock(path, 'checkpoint synthetic fixture password')
                assert label.encode() in c.transcript
                c.raw(b'o'); c.expect('Durable outbox:'); pause(c); lock(c)
                passed(mode+'_state_is_visible_without_silent_restart')
            # A real filesystem write failure, not a product fault-injection hook:
            # replacing the canonical file with a directory poisons revoke's publish.
            poison = seed('poison'); c = unlock(poison, 'checkpoint synthetic fixture password')
            canonical = poison/'mail.fkv'; saved = canonical.read_bytes(); canonical.unlink(); canonical.mkdir()
            c.raw(b'r'); c.expect('Type revoke to block this peer locally: '); c.send('revoke')
            c.expect('Checkpoint locked: '); c.expect('Drop and reopen the canonical store'); c.finish(1)
            canonical.rmdir(); canonical.write_bytes(saved); canonical.chmod(0o600)
            c = unlock(poison, 'checkpoint synthetic fixture password'); lock(c)
            passed('poisoned_write_exits_and_canonical_reopen_resolves_handle')
            # Recovery quarantines before reset; changing password cannot resume the
            # old ratchet. Old data and operation retirement remain available.
            c = start(action='recover'); c.password(recovery, 'Offline recovery secret (64 hex, hidden): ')
            c.password(password, 'New LOCAL passphrase (12-128 ASCII characters, hidden): ')
            c.password(password, 'Repeat LOCAL passphrase (hidden): '); recover_display(c)
            assert b'HISTORY ONLY' in c.transcript; lock(c)
            c = unlock(); assert b'HISTORY ONLY' in c.transcript; lock(c)
            passed('recovery_and_password_reset_cannot_unquarantine_old_ratchets')
            # v1 upgrade is explicit, cancellable, key-rotating and preserves legacy
            # draft/Curve records without enrolling them as native identities.
            legacy = seed('legacy','legacy'); before = (legacy/'mail.fkv').read_bytes()
            untouched = {p: (p/'mail.fkv').read_bytes() for p in (legacy, pending)}
            hostile = (b'\x1b[31mremote scene\x1b[0m'
                       b'\x1b]52;c;clipboard-secret\x07'
                       b'\x1b]7770;CHECKPOINTS;upgrade;verified\x07'
                       b'\x1b_Ga=T,t=f;remote-path\x1b\\'
                       b' --mail-checkpoints '+str(pending).encode()+b' recover verified')
            c = NativeConsole([str(fixture),str(root/'unopened'),'hostile-stream']); children.append(c)
            c.expect('REMOTE FILTER PTY READY'); c.raw(hostile+b'\n')
            c.expect('REMOTE FILTER TEXT ONLY; no local store API called'); c.finish()
            assert b'clipboard-secret' not in c.transcript and b'CHECKPOINTS;upgrade' not in c.transcript
            assert b'LOCAL checkpoint passphrase' not in c.transcript and b'synthetic subject' not in c.transcript
            assert all((p/'mail.fkv').read_bytes()==raw for p,raw in untouched.items())
            assert not (root/'unopened').exists()
            passed('hostile_remote_filter_pty_has_no_unlock_upgrade_or_store_authority')
            # Unknown profile fields are rejected before transport startup.
            profile = root/'hostile-profile.toml'
            profile.write_text('version=1\nhost="127.0.0.1"\nport=1\nfile_host="127.0.0.1"\nfile_port=1\nidentity="/no-fixture-identity"\npublic_key="/no-fixture-public"\nknown_hosts="/no-fixture-hosts"\nmail_checkpoints='+json.dumps(str(pending))+'\n')
            profile.chmod(0o600)
            c = NativeConsole([str(binary),'--profile',str(profile)]); children.append(c)
            c.expect('Faction client: '); c.finish(1)
            assert b'LOCAL checkpoint passphrase' not in c.transcript
            assert all((p/'mail.fkv').read_bytes()==raw for p,raw in untouched.items())
            passed('profile_metadata_cannot_select_or_open_checkpoint_store')
            c = start(legacy,'upgrade'); c.expect('Type upgrade to replace v1 with v2: '); c.send('no')
            c.expect('Checkpoint locked: '); c.finish(1); assert (legacy/'mail.fkv').read_bytes() == before
            c = start(legacy,'upgrade'); c.expect('Type upgrade to replace v1 with v2: '); c.send('upgrade')
            c.password('checkpoint synthetic fixture password', 'Existing v1 LOCAL passphrase (hidden): ')
            c.expect('Local device ID (1-2147483647): '); c.send('707')
            c.password(password, 'New LOCAL passphrase (12-128 ASCII characters, hidden): ')
            c.password(password, 'Repeat LOCAL passphrase (hidden): '); recover_display(c)
            c.raw(b'd'); c.expect('Legacy Curve25519 pins: 1 (NOT Ed25519 enrollment)'); c.expect('legacy draft'); c.expect('Enter for next draft: '); c.send(''); pause(c); lock(c)
            assert (legacy/'mail.fkv').read_bytes().startswith(b'FACTVLT2')
            passed('deliberate_upgrade_preserves_legacy_records_and_rotates_recovery')
            return checks
        finally:
            for c in children: c.close()


if __name__ == '__main__':
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--client', type=Path, required=True)
    p.add_argument('--fixture', type=Path)
    p.add_argument('--owned-root', type=Path, default=None)
    p.add_argument('--output', type=Path)
    a = p.parse_args()
    root_temporary = tempfile.TemporaryDirectory(prefix="faction205-ui-owned-") if a.owned_root is None else None
    if a.owned_root: assert not a.owned_root.is_symlink(), "owned root must not be a symlink"
    owner = a.owned_root.resolve() if a.owned_root else Path(root_temporary.name).resolve()
    checks = probe(a.client.resolve(), a.fixture.resolve() if a.fixture else None, owner)
    result = dict(result='pass', platform=platform.system(), checks=checks, count=len(checks),
                  fixture_children_removed=not list(owner.glob('checkpoint-ui-*')),
                  network_mail=False, hardware_claim=False)
    if a.output: a.output.write_text(json.dumps(result, indent=2)+'\n')
    if root_temporary: root_temporary.cleanup()
    print('Checkpoint UI PTY probe passed:', len(checks), 'checks;', platform.system())
