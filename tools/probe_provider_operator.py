#!/usr/bin/env python3
"""Owned restart proof through the maintained private provider operator.

Only the old member seed/audit, production-console driver, transfer observer and
separate encrypted-mail relay are reused. The server process is faction-provider.
Private inputs, keys, endpoints and captures are retained only under owned root.
"""
from probe_device_provider_console import (
    PublicMail, LauncherConsole, HomePeer, certificates, device_certificate,
    fingerprint, profile, private, command, wait_fact, wait_transfer,
    retain_capture, parse_ready, UNLOCK, MENU, DESK, RECEIVED, HOME, ONLINE_HOME,
    TRANSFER,
)
from probe_attachment_transfer import _settings
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import secrets
import select
import shutil
import signal
import socket
import stat
import subprocess
import tempfile
import time


class Operator:
    """One owned foreground process, with bounded readiness and owned drain."""
    def __init__(self, binary, config, capture_root, *, ready_seconds=5):
        self.root=Path(capture_root)
        self.process=subprocess.Popen([str(binary),'serve',str(config)],
            stdin=subprocess.DEVNULL,stdout=subprocess.PIPE,stderr=subprocess.PIPE)
        self.closed=False
        deadline=time.monotonic()+ready_seconds
        raw=bytearray()
        try:
            while b'\n' not in raw:
                remaining=deadline-time.monotonic()
                if remaining<=0 or not select.select([self.process.stdout],[],[],remaining)[0]:
                    raise ValueError('operator readiness deadline')
                part=os.read(self.process.stdout.fileno(),65-len(raw))
                if not part:raise ValueError('operator readiness unavailable')
                raw.extend(part)
                if len(raw)>64:raise ValueError('operator readiness bound')
            self.port=parse_ready(bytes(raw))
        except BaseException:
            try:
                self.process.terminate()
                try:self.process.wait(timeout=1)
                except subprocess.TimeoutExpired:
                    self.process.kill();self.process.wait(timeout=4)
                retain_capture(self.root,'operator-startup-failed',bytes(raw)+self.process.stderr.read(4097))
            finally:
                self.closed=True
                self.process.stdout.close();self.process.stderr.close()
            raise

    def close(self, stop_signal=signal.SIGTERM):
        if self.closed:return
        start=time.monotonic()
        killed=False
        try:
            self.process.send_signal(stop_signal)
            try:self.process.wait(timeout=4)
            except subprocess.TimeoutExpired:
                killed=True;self.process.kill()
                self.process.wait(timeout=max(.001,5-(time.monotonic()-start)))
            out=self.process.stdout.read(129)
            error=self.process.stderr.read(4097)
            retain_capture(self.root,'operator-stop',out+error)
            if killed or self.process.returncode or out!=b'stopped\n' or error:
                raise ValueError('operator orderly drain failed')
            if time.monotonic()-start>=5:raise ValueError('operator drain exceeded bound')
        finally:
            self.closed=True
            if self.process.poll() is None:
                self.process.kill();self.process.wait(timeout=1)
            self.process.stdout.close();self.process.stderr.close()


def proposal(identities,digests,*,generation,reset_epoch,approved,stamp):
    if len(identities)!=2 or len(digests)!=2 or len(approved)!=2:
        raise ValueError('bounded synthetic public policy required')
    text=f'version=1\ngeneration={generation}\nreset_epoch={reset_epoch}\nhomes=["home.test"]\n'
    for name,device,identity,digest,allow in zip(['alice','bob'],[101,202],identities,digests,approved):
        if not re.fullmatch('[0-9a-f]{64}',identity) or len(digest)!=32:
            raise ValueError('invalid synthetic public identity')
        text+='\n[[bindings]]\n'+''.join(k+'='+json.dumps(v)+'\n' for k,v in dict(
            certificate_sha256=digest.hex(),account=name+'@home.test',device=device,
            identity=identity,valid_from=stamp-60,valid_until=stamp+3600,
            approved=allow,account_enabled=True,device_enabled=True,generation=generation).items())
    if len(text.encode())>65536:raise ValueError('public policy bound')
    return text


def require_root(path):
    path=Path(path);info=path.lstat()
    if not path.is_absolute() or path.is_symlink() or not stat.S_ISDIR(info.st_mode) or info.st_uid!=os.getuid() or stat.S_IMODE(info.st_mode)!=0o700 or (path/'OWNED').read_bytes()!=b'Owned maintained-provider operator proof\n':
        raise ValueError('marked owned private operator root required')
    return path


def config(root):
    values=dict(version=1,home='home.test',endpoint='127.0.0.1:0',
        policy_root=str(root/'policy'),blob_root=str(root/'serverstore'),
        ca=str(root/'ca.pem'),certificate=str(root/'server.pem'),key=str(root/'server.key'))
    path=root/'operator.toml'
    private(path, ''.join(k+'='+json.dumps(v)+'\n' for k,v in values.items()).encode())
    return path


def make_parser():
    p=argparse.ArgumentParser(description=__doc__)
    for name in ['client','fixture','operator','owned-root','output']:
        p.add_argument('--'+name,type=Path,required=True)
    p.add_argument('--openssl',default='openssl')
    return p


def one_journey(client, fixture, operator, owned_root, openssl, unicode):
    root = Path(tempfile.mkdtemp(prefix='journey-OWNED-', dir=owned_root))
    root.chmod(0o700)
    private(root/'OWNED', b'Owned authenticated-provider member seed\n')
    children, peers = [], []
    provider = None
    facts = []
    local_password, home_password = [secrets.token_hex(20) for _ in range(2)]
    secret = (local_password+'\n').encode()
    try:
        command([fixture, 'seed', root], secret=secret)
        pairs = certificates(root, openssl)
        device_pairs = [device_certificate(root, name, openssl) for name in ['alice', 'bob']]
        identities = (root/'public-identities.txt').read_text().splitlines()
        if len(identities)!=2 or any(not re.fullmatch('[0-9a-f]{64}', value) for value in identities):
            raise ValueError('public identity exports invalid')
        server = root/'server'
        server.mkdir(mode=0o700)
        private(server/'OWNED', b'Owned public-policy provider fixture\n')
        for source, name in [(root/'public-principals.bin','public-principals.bin'),
                             (pairs['good'][0],'server.pem'),(pairs['good'][1],'server.key'),
                             (root/'ca.pem','ca.pem')]:
            private(server/name, source.read_bytes())
        private(server/'certificate-digests.bin', b''.join(fingerprint(pair[0]) for pair in device_pairs))
        for name in ['policy','serverstore']:(server/name).mkdir(mode=0o700)
        configuration=config(server)
        proposed=server/'proposal.toml'
        stamp=int(time.time())
        digests=[fingerprint(pair[0]) for pair in device_pairs]
        private(proposed,proposal(identities,digests,generation=1,reset_epoch=1,approved=[True,True],stamp=stamp).encode())
        assert command([operator,'init',configuration,proposed])==b'initialized\n'
        assert command([operator,'inspect',configuration])==b'policy generation=1 reset=1 available=1 bindings=2 history=2\n'
        provider=Operator(operator,configuration,server)
        initial_port=provider.port
        # Explicit offline operations refuse the permanent live serving lock.
        before_policy=(server/'policy/provider-policy.snapshot').read_bytes()
        for action,extra in [('inspect',[]),('apply',[proposed]),('reset',['2'])]:
            rejected=subprocess.run(list(map(str,[operator,action,configuration,*extra])),capture_output=True,timeout=5)
            assert rejected.returncode and rejected.stdout==b''
        assert (server/'policy/provider-policy.snapshot').read_bytes()==before_policy
        facts.append('offline_policy_commands_refuse_live_owner_without_policy_mutation')
        public_mail = PublicMail(root)
        alice = HomePeer(root, pairs['good'], 'alice@home.test', 101, home_password, public_mail, 0)
        peers.append(alice)
        bob = HomePeer(root, pairs['good'], 'bob@home.test', 202, home_password, public_mail, 1)
        peers.append(bob)
        configs = [_settings(root,name,peer,pairs['good'][0]) for name,peer in [('alice',alice),('bob',bob)]]
        profiles = [profile(root,name,identity,provider.port,pairs['good'][0],pair)
                    for name,identity,pair in zip(['alice','bob'],identities,device_pairs)]
        def start(member, selected=None, storage=None):
            args = [client,'--mail-online',(root if storage is None else storage)/member,configs[0 if member=='alice' else 1]]
            if unicode: args.append('--unicode')
            if selected is not None: args.extend(['--device-provider',selected])
            c = LauncherConsole(args, columns=40, rows=18)
            children.append(c)
            c.password(local_password, UNLOCK)
            c.expect(MENU)
            return c
        def desk(c): c.raw(b'm');c.expect(DESK)
        def lock(c, received=False):
            if received: c.raw(b'b');c.expect(DESK)
            c.raw(b'b');c.expect(MENU);c.raw(b'l');c.finish()
        def upload(c, operation):
            c.raw(b'u');c.expect('Full operation ID (blank cancels): ');c.send(operation)
            c.expect(TRANSFER);c.send('transfer');c.password(home_password,HOME)
            return time.monotonic()
        def select_received(c):
            c.raw(b'r');c.expect('Original record number (blank cancels): ');c.send('1');c.expect(RECEIVED)
        def save(c, destination):
            c.raw(b's');c.expect('Absolute save path (blank cancels): ');c.send(str(destination))
            c.expect('Type save to verify and publish: ');c.send('save');c.expect(RECEIVED)
        # Invalid profile is lazy and refuses preparation before a spool exists.
        invalid = profiles[0].parent/'invalid.toml'
        private(invalid, profiles[0].read_bytes().replace(b'device=101',b'device=202'))
        c=start('alice',invalid);desk(c);c.raw(b'c');c.expect('Selected attachment profile refused.');c.expect(DESK)
        assert not (root/'alice/attachments').exists()
        lock(c)
        facts.append('lazy_invalid_profile_refuses_before_prepare_spool_or_connection')
        # A foreign peer is only a separate member-side negative fixture. It is
        # never installed in the provider's registry or used for the main journey.
        foreign=root/'foreign-member';foreign.mkdir(mode=0o700)
        private(foreign/'OWNED',b'Owned authenticated-provider member seed\n')
        command([fixture,'seed',foreign,'foreign'],secret=secret)
        private(foreign/'ca.pem',(root/'ca.pem').read_bytes())
        foreign_identity=(foreign/'public-identities.txt').read_text().splitlines()[0]
        foreign_profile=profile(foreign,'alice',foreign_identity,provider.port,pairs['good'][0],device_pairs[0])
        c=start('alice',foreign_profile,foreign);desk(c);c.raw(b'c');c.expect('Attachment action unavailable.');c.expect(DESK)
        assert not (foreign/'alice/attachments').exists()
        lock(c)
        facts.append('foreign_peer_is_refused_before_prepare_spool_or_provider_connection')
        # Production prepare selects a snapshot, then source deletion proves retry
        # uses the durable ciphertext spool rather than reopening the plaintext.
        source=root/'selected-source.bin'
        payload=bytes((i*73+19)%251 for i in range(1_048_576))
        private(source,payload);source.chmod(0o644)
        c=start('alice',profiles[0]);desk(c);c.raw(b'c')
        for prompt,answer in [
            ('Absolute LOCAL source path (blank cancels): ',str(source)),
            ('Private display name (255 UTF-8 bytes): ','private-selected-name.bin\u202e'),
            ('Media type (127 ASCII bytes): ','application/octet-stream'),
            ('Subject (128 UTF-8 bytes): ','private-selected-subject'),
            ('Body (1024 UTF-8 bytes, one line): ','private-selected-body')]:
            c.expect(prompt);c.send(answer)
        c.expect('Type queue to encrypt and persist attachment: ');c.send('queue');c.expect(DESK)
        operations=re.findall(rb'Operation: ([0-9a-f]{32})',c.transcript)
        assert len(operations)==1
        operation=operations[0].decode();source.unlink();lock(c)
        assert not (root/'delivery.xml').exists()
        facts.append('maximum_size_prepare_is_device_local_and_source_independent')
        # Selected provider outage leaves actual ordinary XMPP publication usable.
        with socket.socket() as unavailable:
            unavailable.bind(('127.0.0.1',0))
            bad=profiles[0].parent/'outage.toml'
            private(bad,profiles[0].read_bytes().replace(('127.0.0.1:'+str(provider.port)).encode(),('127.0.0.1:'+str(unavailable.getsockname()[1])).encode()))
            c=start('alice',bad);desk(c);upload(c,operation)
            c.expect('Transfer stopped; no automatic retry.');c.expect(DESK)
            assert not (root/'delivery.xml').exists()
            c.raw(b'b');c.expect(MENU);c.raw(b'u');c.password(home_password,ONLINE_HOME)
            c.expect('Manual cycle complete.');c.expect('Enter to return: ');c.send('');c.expect(MENU)
            c.raw(b'l');c.finish()
        assert alice.authenticated>=1 and public_mail.blob_requests==0
        facts.append('provider_outage_preserves_ordinary_mail_without_attachment_fallback')
        # A wrong server pin and a different approved device's actual key must
        # not authorize Alice's pending object or open account-only attachment IQ.
        snapshot=server/'serverstore/private-blobs.snapshot'
        pristine=snapshot.read_bytes()
        wrong_pin=profiles[0].parent/'wrong-pin.toml'
        private(wrong_pin,profiles[0].read_bytes().replace(fingerprint(pairs['good'][0]).hex().encode(),b'01'*32))
        c=start('alice',wrong_pin);desk(c);upload(c,operation)
        c.expect('Transfer stopped; no automatic retry.');c.expect(DESK);lock(c)
        assert snapshot.read_bytes()==pristine and not (root/'delivery.xml').exists()
        directory=profiles[0].parent
        private(directory/'other-device.pem',device_pairs[1][0].read_bytes())
        private(directory/'other-device.key',device_pairs[1][1].read_bytes())
        wrong_device=directory/'wrong-device.toml'
        private(wrong_device,profiles[0].read_bytes().replace(b'device.pem',b'other-device.pem').replace(b'device.key',b'other-device.key'))
        c=start('alice',wrong_device);desk(c);upload(c,operation)
        c.expect('Transfer stopped; no automatic retry.');c.expect(DESK);lock(c)
        assert snapshot.read_bytes()==pristine and not (root/'delivery.xml').exists() and public_mail.blob_requests==0
        facts.append('wrong_server_pin_and_actual_wrong_device_key_cannot_store_or_release_mail')
        # Stop the maintained server after actual production-console progress.
        # This is an incomplete upload and possible durable progress observation;
        # no claim is made that a Commit reply was definitely lost.
        c=start('alice',profiles[0]);desk(c);upload(c,operation)
        c.expect('SDK call: 1 / 65; writes: 0')
        provider.close(signal.SIGTERM)
        c.expect('Transfer stopped; no automatic retry.');c.expect(DESK);lock(c)
        incomplete=snapshot.read_bytes()
        assert pristine!=incomplete and len(incomplete)<1_048_592 and not (root/'delivery.xml').exists()
        with socket.socket() as stopped_socket:
            assert stopped_socket.connect_ex(('127.0.0.1',initial_port))!=0
        facts.append('maintained_process_stop_after_incomplete_upload_preserves_possible_durable_progress_without_mail')
        provider=Operator(operator,configuration,server)
        # Profiles retain each original local principal; only the observed port changes.
        for path in profiles:
            private(path,path.read_bytes().replace(('127.0.0.1:'+str(initial_port)).encode(),('127.0.0.1:'+str(provider.port)).encode()))
        facts.append('explicit_reopen_uses_retained_policy_and_actual_new_listener_port')
        c=start('alice',profiles[0]);desk(c);submitted=upload(c,operation)
        upload_observation=wait_transfer(c,1_048_592,started=submitted);c.expect(DESK)
        wait_fact(lambda:alice.messages==1)
        assert re.search(rb'SDK call: (?:[1-9]|[1-5][0-9]|6[0-5]) / 65',c.transcript)
        lock(c)
        assert public_mail.blob_requests==0
        facts.append('reopened_exact_retry_commits_before_actual_encrypted_mail_release')
        # Bob's actual production archive action authenticates and persists receipt.
        c=start('bob',profiles[1]);c.raw(b'a');c.password(home_password,ONLINE_HOME)
        c.expect('Archive page saved. Entries: 1; accepted: 1;');c.expect('Enter to return: ');c.send('');c.expect(MENU)
        desk(c);select_received(c)
        assert not (root/'bob/attachments').exists()
        before=c.transcript.count(HOME.encode())
        c.raw(b'f');c.expect(TRANSFER);c.send('transfer');submitted=time.monotonic()
        fetch_observation=wait_transfer(c,1_048_592,started=submitted);c.expect(RECEIVED)
        assert c.transcript.count(HOME.encode())==before
        c.raw(b'v');c.expect('Verified locally: 1048576 plaintext bytes.');c.expect(RECEIVED)
        destination=root/'saved'/'explicit.bin';destination.parent.mkdir(mode=0o700)
        save(c,destination)
        assert destination.read_bytes()==payload and stat.S_IMODE(destination.stat().st_mode)==0o600
        save(c,destination)
        assert b'Save failed before publication.' in c.transcript and destination.read_bytes()==payload
        assert b'\\u{202e}' in c.transcript and '\u202e'.encode() not in c.transcript
        lock(c,True)
        facts.append('actual_authenticated_mail_receipt_fetch_without_password_verify_private_save_no_overwrite')
        before_revocation=snapshot.read_bytes()
        delivered=(root/'delivery.xml').read_bytes()
        old_port=provider.port
        provider.close(signal.SIGINT)
        private(proposed,proposal(identities,digests,generation=2,reset_epoch=1,approved=[False,True],stamp=stamp).encode())
        assert command([operator,'apply',configuration,proposed])==b'applied\n'
        provider=Operator(operator,configuration,server)
        for path in profiles:
            private(path,path.read_bytes().replace(('127.0.0.1:'+str(old_port)).encode(),('127.0.0.1:'+str(provider.port)).encode()))
        c=start('alice',profiles[0]);desk(c);upload(c,operation)
        c.expect('Transfer stopped; no automatic retry.');c.expect(DESK);lock(c)
        assert snapshot.read_bytes()==before_revocation and (root/'delivery.xml').read_bytes()==delivered
        assert alice.messages==1 and public_mail.blob_requests==0
        facts.append('offline_revocation_survives_restart_and_denies_old_key_without_store_or_mail_mutation')
        provider.close()
        assert command([operator,'reset',configuration,'2'])==b'reset\n'
        assert command([operator,'inspect',configuration])==b'policy generation=3 reset=2 available=1 bindings=0 history=2\n'
        provider=Operator(operator,configuration,server)
        facts.append('explicit_reset_retains_retired_certificate_history_across_command_exit_and_reopen')
        assert public_mail.archive_pages>=1 and public_mail.blob_requests==0
        command([fixture,'audit',root],secret=secret)
        assert all(local_password.encode() not in c.transcript and home_password.encode() not in c.transcript for c in children)
        facts.append('server_ciphertext_and_public_mail_exclude_member_private_metadata_keys_plaintext')
        return dict(mode='unicode' if unicode else 'classic',checks=facts,plaintext_bytes=len(payload),
                    maintained_operator=True,provider_restarts=3,
                    ordinary_mail_connections=alice.connections+bob.connections,mail_received=True,
                    transfer_observations=dict(upload=upload_observation,fetch=fetch_observation))
    finally:
        failures=[]
        for index,c in enumerate(children):
            try:
                retain_capture(owned_root,'unicode-console' if unicode else 'classic-console',c.transcript)
                if hasattr(c,'transfer_observation'):
                    retain_capture(owned_root,'transfer-observation',
                                   (json.dumps(c.transfer_observation,sort_keys=True)+'\n').encode())
            except BaseException as error: failures.append(error)
            try:c.close()
            except BaseException as error:failures.append(error)
        for peer in peers:
            try:peer.close()
            except BaseException as error:failures.append(error)
        if provider is not None:
            try:provider.close()
            except BaseException as error:failures.append(error)
        for capture in [root/'mail.private',root/'delivery.xml']:
            if capture.exists():
                try:retain_capture(owned_root,'private-mail-capture',capture.read_bytes())
                except BaseException as error:failures.append(error)
        # Keep failed fixture bytes privately for diagnosis; successful fixtures
        # are removed after every process/socket was closed and joined.
        if not failures and __import__('sys').exc_info()[0] is None:
            shutil.rmtree(root)
            assert not root.exists()
        if failures:
            raise ValueError('owned console fixture cleanup failed') from failures[0]



def probe(client,fixture,operator,owned_root,openssl):
    owned_root=require_root(owned_root)
    modes=[one_journey(client,fixture,operator,owned_root,openssl,unicode) for unicode in [False,True]]
    return dict(result='pass',count=sum(len(mode['checks']) for mode in modes),modes=modes,
        private_fixture_removed=True,children_reaped=True,provider_deployment=False,
        independent_interoperability=False,physical_hardware=False,capacity_claim=False)

if __name__=='__main__':
    args=make_parser().parse_args()
    result=probe(args.client.resolve(),args.fixture.resolve(),args.operator.resolve(),args.owned_root,args.openssl)
    private(args.output,(json.dumps(result,indent=2,sort_keys=True)+'\n').encode())
    print('Maintained provider operator console passed',result['count'],'controls.')
