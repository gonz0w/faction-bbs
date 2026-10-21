#!/usr/bin/env python3
"""Real OpenSSH/SFTP file-area oracle; fresh, marked disposable NetBSD VM ONLY.

Trusted private argv JSONs contain destinations/keys. No credentials, endpoints,
member transcripts or file payloads are included in sanitized results.
"""
import argparse
import hashlib
import json
from pathlib import Path
import re
import secrets
import shlex
import struct
import subprocess
import tempfile
import time
from check_restricted_ssh import SSHConsole


def smoke(a):
    assert a.disposable_fixture
    admin = json.loads(a.admin_argv.read_text())
    member = json.loads(a.member_argv.read_text())
    sftp = json.loads(a.sftp_argv.read_text())
    ssh_files = json.loads(a.files_ssh_argv.read_text())
    key = a.public_key.read_text().strip()
    password = secrets.token_hex(24)
    children, transfers, checks = [], [], []
    footprint = {}

    def remote(cmd, data=None, ok=True):
        r = subprocess.run(admin+['set -e; PATH=/usr/sbin:/sbin:/usr/bin:/bin:/usr/pkg/bin; export PATH; '+cmd], input=data, capture_output=True, timeout=45)
        assert not ok or r.returncode == 0, 'private guest fixture action failed'
        return r

    def passed(name):
        checks.append(name)

    def menu(c):
        c.expect('Ctrl-C exits; Esc: Enter to recover.')

    def enter(choice, handle):
        c = SSHConsole(member[:-1]+['-tt', member[-1]])
        children.append(c)
        c.expect('[L]ogin [R]egister [Q]uit: '); c.send(choice)
        c.expect('Handle: '); c.send(handle)
        c.password(password)
        if choice == 'r':
            c.password(password, 'Confirm password (hidden): ')
        return c

    def login(handle):
        c = enter('l', handle)
        c.expect('Press Enter to continue: '); c.send(''); menu(c)
        return c

    def files(c):
        c.raw(b'f'); c.expect('files> ')

    def command(c, text):
        start = len(c.transcript)
        c.send(text); c.expect('files> ')
        return c.transcript[start:]

    def deny(c, text):
        c.send(text); c.expect('Press Enter to return to menu: '); c.send(''); menu(c); files(c)

    def upload(c, area, name, size):
        c.send(f'upload {area} {name} {size} Fixture description')
        c.expect('Transfer Ed25519 public key (one line): ')
        start = len(c.transcript); c.send(key); c.expect('files> ')
        return re.findall(rb'Upload grant ([a-f0-9]{32})', c.transcript[start:])[-1].decode()

    def download(c, object_id):
        c.send('download '+object_id); c.expect('Transfer Ed25519 public key (one line): ')
        c.send(key); c.expect('files> ')

    def batch(commands, ok=True):
        r = subprocess.run(sftp[:-1]+['-b', '-', sftp[-1]], input=commands.encode(), capture_output=True, timeout=45)
        if ok and r.returncode != 0:
            a.output.with_suffix('.sftp-private.log').write_bytes(r.stdout+r.stderr)
        assert not ok or r.returncode == 0, 'real SFTP transfer failed; inspect private SFTP log beside output'
        return r

    def object_for(grant):
        script = "import sqlite3; d=sqlite3.connect('/var/db/faction/faction.sqlite3'); print(d.execute('SELECT object_id FROM file_transfers WHERE id=?',("+repr(grant)+",)).fetchone()[0])"
        return remote('/usr/pkg/bin/python3.13 -c '+shlex.quote(script)).stdout.decode().strip()

    class Raw:
        def __init__(self):
            self.p = subprocess.Popen(ssh_files[:-1]+['-s', ssh_files[-1], 'sftp'], stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
            transfers.append(self.p)
            assert self.send(b'\x01'+struct.pack('>I', 3))[0] == 2
        def send(self, b):
            self.p.stdin.write(struct.pack('>I', len(b))+b); self.p.stdin.flush()
            header = self.p.stdout.read(4)
            assert len(header) == 4, 'SFTP helper closed unexpectedly'
            n = struct.unpack('>I', header)[0]
            assert n <= 33792
            out = self.p.stdout.read(n)
            assert len(out) == n
            return out
        def close(self):
            self.p.stdin.close(); self.p.wait(timeout=8)

    def string(b):
        return struct.pack('>I', len(b))+b

    try:
        remote('test -f /etc/faction-disposable-test; test ! -e /var/db/faction/faction.sqlite3; test ! -S /var/run/faction/terminal.sock')
        c = SSHConsole(admin[:-1]+['-tt', admin[-1], "su - factiond -c '/usr/local/libexec/faction --state-dir /var/db/faction init'"])
        children.append(c); c.expect('Handle: '); c.send('sysop')
        c.password(password, 'New password (12+ characters, hidden): '); c.password(password, 'Confirm password (hidden): ')
        c.expect('Lobby created.'); c.finish()
        if remote('test -f /etc/rc.d/faction_files_sshd', ok=False).returncode != 0:
            remote('cd '+shlex.quote(a.guest_source)+'; /usr/pkg/bin/python3.13 tools/install_file_areas.py --binary '+shlex.quote(a.guest_binaries+'/faction-files')+' --listen-address 10.0.2.15 --install')
        remote('test "$(cat /etc/rc.conf.d/faction_files_sshd)" = faction_files_sshd=NO; test ! -S /var/run/faction/file-policy.sock; '
               'su -m factionfiles -c "test ! -r /var/db/faction/faction.sqlite3"; '
               'su -m factionfileauth -c "test ! -r /var/db/faction-files"; '
               'test ! -u /usr/local/libexec/faction-files; '
               '/etc/rc.d/faction_broker onestart; /etc/rc.d/faction_sshd onestart; /etc/rc.d/faction_files_sshd onestart')
        passed('disabled_install_separate_uids_no_database_or_store_authority')
        c = enter('r', 'alice'); c.expect('Registration pending; sysop approval required.'); c.finish()
        c = enter('l', 'alice'); c.expect('Sysop approval required. No community access.'); c.finish()
        sysop = login('sysop')
        sysop.raw(b's'); sysop.expect('[C]reate room: '); sysop.send('a'); sysop.expect('Member handle: '); sysop.send('alice')
        sysop.expect('Status updated; previous sessions expired.'); sysop.expect('Press Enter to return to menu: '); sysop.send(''); menu(sysop)
        alice = login('alice'); files(alice); files(sysop)
        passed('pending_member_denied_and_approved_member_files_hotkey')
        shell = subprocess.run(member+['id'], capture_output=True, timeout=10)
        assert shell.returncode != 0 and b'uid=' not in shell.stdout
        subsystem = subprocess.run(member[:-1]+['-s', member[-1], 'sftp'], input=b'', capture_output=True, timeout=10)
        assert subsystem.returncode != 0
        forwarding = subprocess.run(member[:-1]+['-W', '127.0.0.1:22', member[-1]], input=b'', capture_output=True, timeout=10)
        assert forwarding.returncode != 0
        passed('ordinary_bbs_listener_still_denies_exec_sftp_and_forwarding')
        out = command(sysop, 'create general members 1048576 General uploads')
        area = int(re.findall(rb'Created file area (\d+)', out)[-1])
        out = command(sysop, 'create private private 1048576 Private exchange')
        private = int(re.findall(rb'Created file area (\d+)', out)[-1])
        assert b'General uploads' in command(alice, 'areas')
        assert b'Private exchange' not in command(alice, 'areas')
        deny(alice, f'list {private}')
        command(sysop, f'allow {private} alice')
        assert b'Private exchange' in command(alice, 'areas')
        passed('area_create_private_visibility_and_explicit_grant')
        with tempfile.TemporaryDirectory(prefix='faction-file-oracle-') as temp:
            local = Path(temp); payload = bytes(range(256))*513
            (local/'upload.bin').write_bytes(payload)
            grant = upload(alice, area, 'scene.bin', len(payload)); object_id = object_for(grant)
            # A ready key cannot run a shell; this does not claim/burn the transfer.
            r = subprocess.run(ssh_files+['id'], capture_output=True, timeout=10)
            assert r.returncode != 0 and b'uid=' not in r.stdout
            batch(f'put {local}/upload.bin scene.bin\n')
            assert b'scene.bin' not in command(alice, f'list {area}')
            assert b'pending' in command(sysop, f'review {area}')
            assert batch('ls\n', ok=False).returncode != 0
            passed('real_multichunk_sftp_upload_hidden_pending_and_one_use_replay_denied')
            download(sysop, object_id)
            batch(f'get scene.bin {local}/quarantine-review.bin\n')
            assert (local/'quarantine-review.bin').read_bytes() == payload
            deny(alice, 'publish '+object_id)
            command(sysop, 'publish '+object_id)
            assert b'scene.bin' in command(alice, f'list {area}')
            assert b'scene.bin' in command(alice, f'search {area} - Fixture')
            assert hashlib.sha256(payload).hexdigest().encode() in command(alice, 'info '+object_id)
            download(alice, object_id)
            batch(f'get scene.bin {local}/download.bin\n')
            assert (local/'download.bin').read_bytes() == payload
            passed('sysop_publication_search_metadata_and_real_sftp_download_hash')
            store = '/var/db/faction-files/'
            remote('ln '+store+object_id+' '+store+object_id+'.alias')
            download(alice, object_id)
            assert batch(f'get scene.bin {local}/hardlink-denied.bin\n', ok=False).returncode != 0
            remote('rm '+store+object_id+'.alias; mv '+store+object_id+' '+store+object_id+'.saved; ln -s /etc/passwd '+store+object_id)
            download(alice, object_id)
            assert batch(f'get scene.bin {local}/symlink-denied.bin\n', ok=False).returncode != 0
            assert not (local/'symlink-denied.bin').exists() or (local/'symlink-denied.bin').stat().st_size == 0
            remote('rm '+store+object_id+'; mv '+store+object_id+'.saved '+store+object_id)
            passed('native_sftp_refuses_hardlinked_and_symlinked_store_objects')
            # Path and offset probes use the actual SSH subsystem, never local stdio.
            grant = upload(alice, private, 'partial.bin', 65536); partial_id = object_for(grant)
            raw = Raw()
            footprint['owner_rss_kib_during_active_transfer'] = int(remote('ps -p "$(cat /var/run/faction/terminal.pid)" -o rss=').stdout)
            sample = remote('ps -ax -o rss= -o comm=').stdout.decode().splitlines()
            footprint['helper_rss_kib_samples'] = [int(line.split()[0]) for line in sample if line.split()[-1:] == ['/usr/local/libexec/faction-files']]
            # Observational sample only; process exit/scheduling may race ps.
            def open_packet(name):
                return b'\x03'+struct.pack('>I', 1)+string(name)+struct.pack('>II', 26, 0)
            for path in (b'../outside', b'/etc/passwd', b'a/b', b'.hidden'):
                response = raw.send(open_packet(path)); assert response[0] == 101 and struct.unpack('>I', response[5:9])[0] != 0
            response = raw.send(open_packet(b'partial.bin'))
            assert response[0] == 102
            handle = response[9:]
            response = raw.send(b'\x06'+struct.pack('>I', 2)+string(handle)+struct.pack('>Q', 65536)+string(b'x'))
            assert response[0] == 101 and struct.unpack('>I', response[5:9])[0] != 0
            response = raw.send(b'\x06'+struct.pack('>I', 3)+string(handle)+struct.pack('>Q', 0)+string(b'partial'))
            assert struct.unpack('>I', response[5:9])[0] == 0
            command(sysop, f'deny {private} alice')
            response = raw.p.communicate(struct.pack('>I', 9)+b'\x08'+struct.pack('>I', 4)+struct.pack('>I', 0), timeout=10)
            assert raw.p.returncode != 0
            assert b'partial.bin' not in command(sysop, f'list {private}')
            command(sysop, f'allow {private} alice')
            assert batch('ls\n', ok=False).returncode != 0
            passed('real_sftp_traversal_oversize_and_active_area_revocation')
            remote('ln /var/db/faction-files/'+partial_id+'.part /var/db/faction-files/'+partial_id)
            command(sysop, 'remove '+partial_id)
            remote("su -m factionfiles -c '/usr/local/libexec/faction-files gc'")
            passed('interrupted_upload_hidden_one_use_and_exact_crash_pair_gc')
            # Both owner and transfer slot locks must refuse live backups.
            uid = remote('id -u factiond').stdout.decode().strip()
            fid = remote('id -u factionfiles').stdout.decode().strip()
            backup = '/home/factiond/file-backup'
            base = '/usr/pkg/bin/python3.13 '+shlex.quote(a.guest_source+'/tools/file_area_backup.py')
            flags = f' --owner-uid {uid} --files-uid {fid} --bundle {backup}'
            assert remote(base+' backup'+flags, ok=False).returncode != 0
            passed('live_owner_prevents_inconsistent_file_backup')
            for c in (alice, sysop):
                c.send('back'); menu(c); c.raw(b'q'); c.expect('Logged out.'); c.finish()
            remote('/etc/rc.d/faction_files_sshd onestop; /etc/rc.d/faction_sshd onestop; /etc/rc.d/faction_broker onestop')
            remote(base+' backup'+flags)
            # Corrupt a private copied bundle: restore must reject it before destinations exist.
            remote('cp -Rp '+backup+' /home/factiond/file-backup-corrupt')
            corrupt = "from pathlib import Path; p=next(Path('/home/factiond/file-backup-corrupt/objects').iterdir()); b=bytearray(p.read_bytes()); b[0]^=1; p.write_bytes(b)"
            remote('/usr/pkg/bin/python3.13 -c '+shlex.quote(corrupt))
            assert remote(base+' restore'+flags.replace(backup, '/home/factiond/file-backup-corrupt')+' --board /home/factiond/rejected-board --store /var/db/rejected-files', ok=False).returncode != 0
            remote('test ! -e /home/factiond/rejected-board; test ! -e /var/db/rejected-files')
            passed('corrupt_file_bundle_rejected_before_creating_restore_paths')
            remote(base+' restore' +flags+' --board /home/factiond/restored-files-board --store /var/db/faction-files-restored')
            script = "from pathlib import Path; import hashlib,sqlite3; d=sqlite3.connect('/home/factiond/restored-files-board/faction.sqlite3'); r=d.execute(\"SELECT id,sha256 FROM file_objects WHERE state='published'\").fetchone(); assert r; assert hashlib.sha256(Path('/var/db/faction-files-restored',r[0]).read_bytes()).hexdigest()==r[1]; assert d.execute('SELECT count(*) FROM file_transfers').fetchone()[0]==0"
            remote('/usr/pkg/bin/python3.13 -c '+shlex.quote(script))
            passed('complete_file_bundle_restore_retains_bytes_and_discards_grants')
            remote('/etc/rc.d/faction_broker onestart; /etc/rc.d/faction_sshd onestart; /etc/rc.d/faction_files_sshd onestart')
            assert batch('ls\n', ok=False).returncode != 0
            alice = login('alice'); files(alice)
            assert b'scene.bin' in command(alice, f'list {area}')
            download(alice, object_id); batch(f'get scene.bin {local}/after-restart.bin\n')
            assert (local/'after-restart.bin').read_bytes() == payload
            passed('backend_listener_restart_preserves_catalog_and_requires_new_grant')
        return {'result': 'pass', 'checks': checks, 'count': len(checks), 'physical_hardware': False, 'capacity_claim': False, 'footprint_observation': footprint, 'footprint_is_peak': False}
    finally:
        for p in transfers:
            if p.poll() is None:
                p.kill(); p.wait()
        for c in children:
            c.close()


def main():
    p = argparse.ArgumentParser(description=__doc__)
    for name in ('admin-argv', 'member-argv', 'sftp-argv', 'files-ssh-argv', 'public-key', 'output'):
        p.add_argument('--'+name, type=Path, required=True)
    p.add_argument('--guest-source', default='/home/factionbuild/build')
    p.add_argument('--guest-binaries', default='/home/factionbuild/target/release')
    p.add_argument('--disposable-fixture', action='store_true', required=True)
    a = p.parse_args()
    result = smoke(a)
    a.output.write_text(json.dumps(result, indent=2)+'\n')
    print(f"Native real-SSH/SFTP file-area checks passed: {result['count']}.")


if __name__ == '__main__':
    main()
