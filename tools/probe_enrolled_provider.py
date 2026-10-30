#!/usr/bin/env python3
"""Owned #293 manual enrollment and real loopback SSH software qualification.

The lead supplies an already authorized dedicated sshd fixture. This driver never
starts sshd, creates an OS account, or changes a host service. The home provider
is a separate maintained process on this OS; this is not cross-machine deployment.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import secrets
import shutil
import stat
import subprocess
import tempfile
import time
import tomllib

from probe_provider_operator import (
    Operator, prepare_member, checkpoint_bytes, invalid_device_profile,
)
from probe_device_provider_console import (
    PublicMail, LauncherConsole, HomePeer, certificates, fingerprint, private,
    command, wait_fact, wait_transfer, retain_capture, UNLOCK, MENU, DESK,
    RECEIVED, HOME, ONLINE_HOME, TRANSFER,
)
from probe_attachment_transfer import _settings

MARKER = b'Owned deliberate enrollment and dedicated-forward proof\n'
CARD_FIELDS = {'version', 'account', 'device', 'identity', 'certificate_sha256'}


def _card(card):
    if (not isinstance(card, dict) or set(card) != CARD_FIELDS
            or type(card['version']) is not int or card['version'] != 1
            or type(card['device']) is not int or not 1 <= card['device'] <= 2147483647
            or not isinstance(card['account'], str)
            or re.fullmatch(r'[a-z][a-z0-9._-]{0,63}@home\.test', card['account']) is None):
        raise ValueError('invalid bounded same-home public claim')
    for field in ['identity', 'certificate_sha256']:
        value = card[field]
        if not isinstance(value, str) or re.fullmatch('[0-9a-f]{64}', value) is None or value == '00'*32:
            raise ValueError('invalid complete public digest')
    return dict(card)


def validate_approval(card, certificate_digest, verified):
    """Compare against separately observed operator input, never infer approval.

    The fixture's record is operator judgment through its trusted channel. This
    comparison and the later mTLS operation do not prove OMEMO-key possession.
    """
    claimed = _card(card)
    if (not isinstance(verified, dict)
            or set(verified) != CARD_FIELDS | {'account_control_verified', 'verification_channel'}
            or verified['account_control_verified'] is not True
            or not isinstance(verified['verification_channel'], str)
            or not 1 <= len(verified['verification_channel']) <= 256
            or any(ord(c) < 32 or ord(c) > 126 for c in verified['verification_channel'])
            or certificate_digest != claimed['certificate_sha256']
            or _card({key: verified[key] for key in CARD_FIELDS}) != claimed):
        raise ValueError('independent operator approval mismatch')
    return claimed


def approved_proposal(cards, *, generation, reset_epoch, approved, stamp):
    """Complete test-operator proposal from already independently checked claims."""
    if (not 1 <= len(cards) <= 32 or len(cards) != len(approved)
            or type(generation) is not int or not 1 <= generation < 2**63
            or type(reset_epoch) is not int or not 0 <= reset_epoch < 2**63
            or type(stamp) is not int or not 60 <= stamp < 2**63-3600
            or any(type(flag) is not bool for flag in approved)):
        raise ValueError('invalid complete policy bounds')
    checked = [_card(card) for card in cards]
    if (len({(c['account'], c['device']) for c in checked}) != len(cards)
            or len({c['certificate_sha256'] for c in checked}) != len(cards)):
        raise ValueError('duplicate complete policy target')
    text = f'version=1\ngeneration={generation}\nreset_epoch={reset_epoch}\nhomes=["home.test"]\n'
    for card, flag in zip(checked, approved):
        fields = {key: card[key] for key in ['certificate_sha256', 'account', 'device', 'identity']}
        fields.update(valid_from=stamp-60, valid_until=stamp+3600, approved=flag,
                      account_enabled=True, device_enabled=True, generation=generation)
        text += '\n[[bindings]]\n'+''.join(key+'='+json.dumps(value)+'\n' for key, value in fields.items())
    if len(text.encode()) > 65536:
        raise ValueError('complete policy byte bound')
    return text


def rotation_proposal(cards, replacement, verified, *, generation, reset_epoch, stamp):
    """Narrow fixture transport rotation; identity-loss recovery needs peer trust."""
    replacement = validate_approval(replacement, replacement['certificate_sha256'], verified)
    checked = [_card(card) for card in cards]
    targets = [index for index, card in enumerate(checked)
               if (card['account'], card['device']) == (replacement['account'], replacement['device'])]
    if len(targets) != 1:
        raise ValueError('rotation requires one already approved exact device')
    index = targets[0]
    old = checked[index]
    if old['identity'] != replacement['identity'] or old['certificate_sha256'] == replacement['certificate_sha256']:
        raise ValueError('fresh transport key and unchanged independently verified identity required')
    checked[index] = replacement
    return approved_proposal(checked, generation=generation, reset_epoch=reset_epoch,
                             approved=[True]*len(checked), stamp=stamp)


def scenario_policy(cards, scenario, *, generation, reset_epoch, stamp):
    if len(cards) != 2 or [card['account'] for card in cards] != ['alice@home.test', 'bob@home.test']:
        raise ValueError('exact owned two-member policy scenario required')
    if scenario not in {'current', 'expired', 'revoked'}:
        raise ValueError('unknown policy scenario')
    text = approved_proposal(cards, generation=generation, reset_epoch=reset_epoch,
                             approved=[scenario != 'revoked', True], stamp=stamp)
    if scenario == 'expired':
        text = text.replace('valid_until='+str(stamp+3600), 'valid_until='+str(stamp+2), 1)
    return text


def require_policy_case(text, scenario, stamp):
    value = tomllib.loads(text)
    rows = value.get('bindings', [])
    if scenario == 'reset':
        if rows: raise ValueError('reset scenario still has active authority')
        return
    if (scenario not in {'current', 'expired', 'revoked'} or len(rows) != 2
            or [row['account'] for row in rows] != ['alice@home.test', 'bob@home.test']):
        raise ValueError('incoherent active policy scenario')
    for index, row in enumerate(rows):
        if (row['generation'] != value['generation'] or not row['account_enabled']
                or not row['device_enabled'] or row['valid_from'] > stamp
                or row['approved'] != (scenario != 'revoked' or index != 0)
                or ((row['valid_until'] < stamp) != (scenario == 'expired' and index == 0))):
            raise ValueError('policy does not isolate the intended refusal cause')


def new_private(path, data):
    fd = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
    with os.fdopen(fd, 'wb') as output:
        output.write(data)


def copy_public(source, destination, kind):
    source, destination = Path(source), Path(destination)
    caps = {'csr': 65536, 'certificate': 65536, 'card': 8192}
    if kind not in caps or source.suffix == '.key':
        raise ValueError('only selected public enrollment artifacts may transfer')
    info = source.lstat()
    if not stat.S_ISREG(info.st_mode) or info.st_nlink != 1 or info.st_uid != os.getuid() or stat.S_IMODE(info.st_mode) != 0o600:
        raise ValueError('private regular public-artifact file required')
    data = source.read_bytes()
    if not 0 < len(data) <= caps[kind] or b'PRIVATE KEY' in data:
        raise ValueError('invalid public enrollment artifact')
    new_private(destination, data)


def forward_argv(ssh, key, known_hosts, address, ssh_port, user, local_port,
                 provider_port, control):
    if (address != '127.0.0.1' or re.fullmatch('[a-z_][a-z0-9_-]{0,31}', user) is None
            or any(type(port) is not int or not 1 <= port <= 65535
                   for port in [ssh_port, local_port, provider_port])
            or local_port == provider_port
            or any(not Path(path).is_absolute() for path in [ssh, key, known_hosts, control])):
        raise ValueError('explicit owned loopback SSH fixture required')
    options = ['BatchMode=yes', 'StrictHostKeyChecking=yes', 'IdentitiesOnly=yes',
               'GlobalKnownHostsFile=/dev/null', 'UserKnownHostsFile='+str(known_hosts),
               'ForwardAgent=no', 'ForwardX11=no', 'PermitLocalCommand=no',
               'ControlMaster=yes', 'ControlPersist=no', 'ExitOnForwardFailure=yes',
               'ConnectTimeout=5', 'ConnectionAttempts=1', 'ServerAliveInterval=2',
               'ServerAliveCountMax=1']
    args = [str(ssh), '-F', '/dev/null', '-N', '-T', '-p', str(ssh_port), '-i', str(key),
            '-S', str(control)]
    for option in options:
        args += ['-o', option]
    return args+['-L', f'127.0.0.1:{local_port}:127.0.0.1:{provider_port}', user+'@'+address]


def denial_argv(ssh, key, known_hosts, address, ssh_port, user, *, request, target_port=None):
    if (address != '127.0.0.1' or re.fullmatch('[a-z_][a-z0-9_-]{0,31}', user) is None
            or type(ssh_port) is not int or not 1 <= ssh_port <= 65535
            or any(not Path(path).is_absolute() for path in [ssh, key, known_hosts])
            or request not in {'forward', 'shell', 'subsystem'}):
        raise ValueError('invalid bounded SSH denial fixture')
    args = list(map(str, [ssh, '-F', '/dev/null', '-T', '-p', ssh_port, '-i', key]))
    for option in ['BatchMode=yes', 'StrictHostKeyChecking=yes', 'IdentitiesOnly=yes',
                   'GlobalKnownHostsFile=/dev/null', 'UserKnownHostsFile='+str(known_hosts),
                   'ForwardAgent=no', 'ForwardX11=no', 'PermitLocalCommand=no',
                   'ControlMaster=no', 'ControlPath=none', 'ConnectTimeout=5', 'ConnectionAttempts=1']:
        args += ['-o', option]
    if request == 'forward':
        if type(target_port) is not int or not 1 <= target_port <= 65535:
            raise ValueError('invalid requested denied destination')
        args += ['-W', '127.0.0.1:'+str(target_port)]
    elif request == 'subsystem':
        args += ['-s']
    args += [user+'@'+address]
    if request == 'subsystem':
        args += ['sftp']
    return args


def require_ssh_denial(result, request, *, server_log=b'', expected_user=None):
    if (result.returncode == 0 or result.stdout or len(result.stderr) > 65536
            or b'Permission denied' in result.stderr
            or b'Connection refused' in result.stderr or b'timed out' in result.stderr):
        raise ValueError('SSH failure did not establish channel-policy denial')
    expected = [b'administratively prohibited', b'open failed: administratively prohibited']
    if request == 'shell': expected.append(b'shell request failed on channel')
    if request == 'subsystem': expected.append(b'subsystem request failed on channel')
    if any(marker in result.stderr for marker in expected):
        return True
    # Some actual sshd binaries expose MaxSessions0 as generic session-channel
    # open failure. Accept only this precise client outcome plus a newly appended
    # exclusive-attempt server fragment proving authentication and session denial.
    if (request in {'shell', 'subsystem'}
            and b'channel 0: open failed: connect failed: open failed' in result.stderr
            and isinstance(server_log, bytes) and len(server_log) <= 65536
            and isinstance(expected_user, str)
            and re.fullmatch('[a-z_][a-z0-9_-]{0,31}', expected_user)):
        lines = server_log.replace(b'\r\n', b'\n').splitlines()
        accepted = [line for line in lines if line.startswith(b'Accepted publickey for ')]
        prefix = b'Accepted publickey for '+expected_user.encode()+b' from 127.0.0.1 port '
        if (len(accepted) == 1 and accepted[0].startswith(prefix)
                and re.fullmatch(rb'[0-9]{1,5} ssh2: [^\r\n]+', accepted[0][len(prefix):])
                and lines.count(b'no more sessions') == 1):
            return True
    raise ValueError('expected authenticated SSH channel-policy denial unavailable')


def log_position(path):
    info = Path(path).lstat()
    if not stat.S_ISREG(info.st_mode) or info.st_uid != os.getuid() or info.st_nlink != 1:
        raise ValueError('owned regular sshd fixture log required')
    return info.st_dev, info.st_ino, info.st_size


def appended_log(path, before):
    after = log_position(path)
    if before[:2] != after[:2] or not 0 <= after[2]-before[2] <= 65536:
        raise ValueError('sshd fixture log changed identity, truncated, or exceeded attempt bound')
    with Path(path).open('rb') as source:
        source.seek(before[2])
        fragment = source.read(after[2]-before[2])
    final = log_position(path)
    if len(fragment) != after[2]-before[2] or final[:2] != after[:2] or final[2] < after[2]:
        raise ValueError('sshd fixture log changed during bounded attempt read')
    return fragment


class Forward:
    """Bounded ownership of one SSH child. Setup never claims provider readiness."""
    def __init__(self, argv, root, *, setup_seconds=5, check=None):
        self.root = Path(root)
        self.process = subprocess.Popen(list(map(str, argv)), stdin=subprocess.DEVNULL,
                                        stdout=subprocess.DEVNULL, stderr=subprocess.PIPE)
        self.closed = False
        self.setup_only = True
        self.argv = argv
        if check is None:
            control = argv[argv.index('-S')+1]
            def check():
                result = subprocess.run([argv[0], '-F', '/dev/null', '-S', control,
                                         '-O', 'check', argv[-1]], stdin=subprocess.DEVNULL,
                                        capture_output=True, timeout=.5)
                return result.returncode == 0
        deadline = time.monotonic()+setup_seconds
        try:
            while True:
                if self.process.poll() is not None:
                    raise ValueError('owned SSH forward exited during setup')
                try:
                    ready = check()
                except subprocess.TimeoutExpired:
                    ready = False
                if ready:
                    break
                if time.monotonic() >= deadline:
                    raise ValueError('owned SSH forward setup deadline')
                time.sleep(min(.02, max(0, deadline-time.monotonic())))
        except BaseException:
            self.close()
            raise

    def close(self):
        if self.closed:
            return
        began = time.monotonic()
        try:
            if self.process.poll() is None:
                self.process.terminate()
                try:
                    self.process.wait(timeout=4)
                except subprocess.TimeoutExpired:
                    self.process.kill()
                    self.process.wait(timeout=max(.001, 5-(time.monotonic()-began)))
            error = self.process.stderr.read(65537)
            retain_capture(self.root, 'dedicated-forward', error)
            if len(error) > 65536 or time.monotonic()-began >= 5:
                raise ValueError('owned SSH forward drain bound')
        finally:
            self.closed = True
            if self.process.poll() is None:
                self.process.kill(); self.process.wait(timeout=1)
            self.process.stderr.close()


def member_credentials(member, server, name, openssl):
    """Member generates its key/CSR; the offline test CA receives only the CSR."""
    key, csr = member/'device.key', member/'request.csr'
    new_private(member/'openssl.cnf', b'[req]\ndistinguished_name=dn\n[dn]\n')
    command([openssl, 'req', '-config', member/'openssl.cnf', '-new', '-newkey', 'ec',
             '-pkeyopt', 'ec_paramgen_curve:P-256', '-nodes', '-sha256',
             '-subj', '/CN=member-generated-transport', '-keyout', key, '-out', csr])
    key.chmod(0o600); csr.chmod(0o600)
    request = server/(name+'-request.csr')
    copy_public(csr, request, 'csr')
    command([openssl, 'req', '-config', member/'openssl.cnf', '-in', request, '-verify', '-noout'])
    extension = server/(name+'-client.ext')
    new_private(extension, b'basicConstraints=critical,CA:FALSE\nextendedKeyUsage=clientAuth\nkeyUsage=critical,digitalSignature\n')
    certificate = server/(name+'-device.pem')
    command([openssl, 'x509', '-req', '-in', request, '-CA', server/'ca.pem',
             '-CAkey', server/'ca.key', '-set_serial', str(secrets.randbits(64) or 1),
             '-days', '2', '-sha256', '-extfile', extension, '-out', certificate])
    certificate.chmod(0o600)
    copy_public(certificate, member/'device.pem', 'certificate')
    copy_public(server/'ca.pem', member/'ca.pem', 'certificate')
    return member/'device.pem', key


def require_root(path):
    path = Path(path)
    info = path.lstat()
    if (not path.is_absolute() or not stat.S_ISDIR(info.st_mode) or info.st_uid != os.getuid()
            or stat.S_IMODE(info.st_mode) != 0o700 or (path/'OWNED').read_bytes() != MARKER):
        raise ValueError('marked owned private enrollment root required')
    return path


def make_parser():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ['client', 'fixture', 'operator', 'owned-root', 'output', 'ssh', 'ssh-key', 'known-hosts', 'sshd-log']:
        parser.add_argument('--'+name, type=Path, required=True)
    for name in ['ssh-port', 'provider-port', 'forward-port']:
        parser.add_argument('--'+name, type=int, required=True)
    parser.add_argument('--ssh-address', default='127.0.0.1')
    parser.add_argument('--ssh-user', required=True)
    parser.add_argument('--openssl', required=True)
    parser.add_argument('--denied-ssh-key', type=Path, action='append', default=[],
                        help='separate BBS/SFTP role keys expected to deny direct forwarding')
    return parser


def one_journey(args, owned_root, unicode):
    root = Path(tempfile.mkdtemp(prefix='enroll-', dir=owned_root))
    root.chmod(0o700)
    new_private(root/'OWNED', b'Owned authenticated-provider member seed\n')
    children, peers, forwards = [], [], []
    provider = None
    facts = []
    local_password, home_password = [secrets.token_hex(20) for _ in range(2)]
    private_password_values = [local_password.encode(), home_password.encode()]
    secret = (local_password+'\n').encode()
    try:
        # This helper creates only member vaults/public bundles and later audits
        # device-held descriptor secrets. Its synthetic serve path is never used.
        command([args.fixture, 'seed', root], secret=secret)
        server = root/'server'; server.mkdir(mode=0o700)
        pairs = certificates(server, args.openssl)
        for path in server.iterdir():
            if path.is_file(): path.chmod(0o600)
        copy_public(server/'ca.pem', root/'ca.pem', 'certificate')
        for name in ['policy', 'serverstore']: (server/name).mkdir(mode=0o700)
        configuration = server/'operator.toml'
        values = dict(version=1, home='home.test', endpoint=f'127.0.0.1:{args.provider_port}',
                      policy_root=str(server/'policy'), blob_root=str(server/'serverstore'),
                      ca=str(server/'ca.pem'), certificate=str(pairs['good'][0]), key=str(pairs['good'][1]))
        new_private(configuration, ''.join(k+'='+json.dumps(v)+'\n' for k, v in values.items()).encode())
        public_mail = PublicMail(root)
        for actor, name, device in [(0, 'alice', 101), (1, 'bob', 202)]:
            peers.append(HomePeer(root, pairs['good'], name+'@home.test', device,
                                  home_password, public_mail, actor))
        configs = [_settings(root, name, peer, pairs['good'][0])
                   for name, peer in zip(['alice', 'bob'], peers)]

        def start(member, selected=None):
            argv = [args.client, '--mail-online', root/member, configs[0 if member == 'alice' else 1]]
            if unicode: argv.append('--unicode')
            if selected is not None: argv += ['--device-provider', selected]
            console = LauncherConsole(argv, columns=40, rows=18); children.append(console)
            console.password(local_password, UNLOCK); console.expect(MENU)
            return console

        def lock(console, received=False):
            if received: console.raw(b'b'); console.expect(DESK)
            console.raw(b'b'); console.expect(MENU); console.raw(b'l'); console.finish()

        def desk(console):
            console.raw(b'm'); console.expect(DESK)

        def upload(console, operation):
            console.raw(b'u'); console.expect('Full operation ID (blank cancels): '); console.send(operation)
            console.expect(TRANSFER); console.send('transfer'); console.password(home_password, HOME)
            return time.monotonic()

        # Independent account verification uses a real maintained ordinary XMPP
        # action. Public identity inspection uses the separate local P screen.
        verified_records, profiles, cards, templates, member_pairs = [], [], [], [], []
        for name, device, peer in zip(['alice', 'bob'], [101, 202], peers):
            console = start(name)
            console.raw(b'p'); console.expect('Enter to return: ')
            identities = re.findall(rb'Local public device: ([0-9]+)\r\ned25519:([0-9a-f]{64})', console.transcript)
            assert len(identities) == 1 and int(identities[0][0]) == device
            identity = identities[0][1].decode()
            console.send(''); console.expect(MENU)
            before_auth = peer.authenticated
            console.raw(b'u'); console.password(home_password, ONLINE_HOME)
            console.expect('Manual cycle complete.'); console.expect('Enter to return: ')
            console.send(''); console.expect(MENU); console.raw(b'l'); console.finish()
            wait_fact(lambda: peer.authenticated > before_auth)
            member = root/(name+'-provider'); member.mkdir(mode=0o700)
            pair = member_credentials(member, server, name, args.openssl); member_pairs.append(pair)
            transport = dict(version=1, home='home.test', endpoint=f'127.0.0.1:{args.forward_port}',
                             name='home.test', pin=fingerprint(pairs['good'][0]).hex(),
                             ca=str(member/'ca.pem'), certificate=str(pair[0]), key=str(pair[1]))
            template = member/'transport.toml'
            new_private(template, ''.join(k+'='+json.dumps(v)+'\n' for k, v in transport.items()).encode())
            profile, card_path = member/'profile.toml', member/'card.toml'
            before = checkpoint_bytes(root/name)
            prepare_member(args.client, root/name, template, profile, card_path,
                           local_password, unicode, children)
            assert checkpoint_bytes(root/name) == before
            card = tomllib.loads(card_path.read_text())
            verified = dict(version=1, account=peer.account, device=device, identity=identity,
                            certificate_sha256=fingerprint(pair[0]).hex(),
                            account_control_verified=True,
                            verification_channel='owned independent XMPP account authentication and member P inspection')
            checked = validate_approval(card, fingerprint(pair[0]).hex(), verified)
            new_private(server/(name+'-verification.json'), (json.dumps(verified, sort_keys=True)+'\n').encode())
            copy_public(card_path, server/(name+'-card.toml'), 'card')
            verified_records.append(verified); cards.append(checked)
            profiles.append(profile); templates.append(template)
        assert not (server/'public-principals.bin').exists()
        assert not list((server/'policy').iterdir()) and not list((server/'serverstore').iterdir())
        facts.append('member_generated_signed_csrs_verified_by_offline_ca_without_member_private_key_transfer')
        facts.append('independent_account_authentication_and_full_public_device_inspection_precede_deliberate_policy_approval')

        stamp = int(time.time())
        proposed = server/'approved.toml'
        initial_policy = scenario_policy(cards, 'current', generation=1, reset_epoch=1, stamp=stamp)
        active_policy = initial_policy
        new_private(proposed, initial_policy.encode())
        candidate = None
        for name in ['alice', 'bob']:
            candidate = server/(name+'-checked.toml')
            assert command([args.operator, 'prepare', configuration, server/(name+'-card.toml'),
                            server/(name+'-device.pem'), proposed, candidate]) == b'prepared candidate; not applied or approved\n'
            assert tomllib.loads(candidate.read_text()) == tomllib.loads(proposed.read_text())
        assert not list((server/'policy').iterdir())
        assert command([args.operator, 'init', configuration, candidate]) == b'initialized\n'
        assert command([args.operator, 'inspect', configuration]) == b'policy generation=1 reset=1 available=1 bindings=2 history=2\n'
        facts.append('maintained_prepare_checks_complete_proposal_before_separate_explicit_init')

        provider = Operator(args.operator, configuration, server)
        assert provider.port == args.provider_port

        def open_forward():
            control = root/('mux-'+secrets.token_hex(3))
            if len(os.fsencode(control)) > 100:
                raise ValueError('owned SSH control path exceeds portable socket bound')
            argv = forward_argv(args.ssh, args.ssh_key, args.known_hosts, args.ssh_address,
                                args.ssh_port, args.ssh_user, args.forward_port,
                                args.provider_port, control)
            forward = Forward(argv, owned_root); forwards.append(forward)
            return forward

        forward = open_forward()
        assert forward.setup_only
        facts.append('dedicated_real_ssh_local_forward_setup_is_separate_from_provider_tls_readiness')
        before_denials = checkpoint_bytes(server/'serverstore')
        denial_cases = [(key, 'forward', args.provider_port) for key in args.denied_ssh_key]
        denial_cases += [(args.ssh_key, 'forward', 1 if args.provider_port != 1 else 2),
                         (args.ssh_key, 'shell', None), (args.ssh_key, 'subsystem', None)]
        for key, request, target in denial_cases:
            argv = denial_argv(args.ssh, key, args.known_hosts, args.ssh_address,
                               args.ssh_port, args.ssh_user, request=request, target_port=target)
            offset = log_position(args.sshd_log)
            result = subprocess.run(argv, stdin=subprocess.DEVNULL, capture_output=True, timeout=6)
            fragment = appended_log(args.sshd_log, offset)
            retain_capture(owned_root, 'ssh-denied-channel', result.stdout+result.stderr)
            retain_capture(owned_root, 'sshd-denied-attempt', fragment)
            require_ssh_denial(result, request, server_log=fragment, expected_user=args.ssh_user)
        assert checkpoint_bytes(server/'serverstore') == before_denials
        facts.append('actual_ordinary_role_forwarding_wrong_dedicated_destination_shell_and_subsystem_channels_are_administratively_denied')
        source = root/'selected-source.bin'
        payload = bytes((i*73+19) % 251 for i in range(1_048_576))
        new_private(source, payload)
        console = start('alice', profiles[0]); desk(console); console.raw(b'c')
        for prompt, answer in [
            ('Absolute LOCAL source path (blank cancels): ', str(source)),
            ('Private display name (255 UTF-8 bytes): ', 'private-selected-name.bin\u202e'),
            ('Media type (127 ASCII bytes): ', 'application/octet-stream'),
            ('Subject (128 UTF-8 bytes): ', 'private-selected-subject'),
            ('Body (1024 UTF-8 bytes, one line): ', 'private-selected-body')]:
            console.expect(prompt); console.send(answer)
        console.expect('Type queue to encrypt and persist attachment: '); console.send('queue'); console.expect(DESK)
        operations = re.findall(rb'Operation: ([0-9a-f]{32})', console.transcript)
        assert len(operations) == 1
        operation = operations[0].decode(); source.unlink(); lock(console)
        assert not (root/'delivery.xml').exists()
        snapshot = server/'serverstore/private-blobs.snapshot'
        pristine = snapshot.read_bytes()
        facts.append('production_console_encrypts_maximum_file_locally_and_persists_exact_retry_after_source_removal')

        # Establish real authorization under the same long-valid generation
        # before credential negatives. Interrupt after confirmed durable progress
        # so this positive control releases no encrypted attachment mail yet.
        require_policy_case(active_policy, 'current', int(time.time()))
        console = start('alice', profiles[0]); desk(console); upload(console, operation)
        console.expect('SDK call: 1 / 65; writes: 0'); forward.close()
        console.expect('Transfer stopped; no automatic retry.'); console.expect(DESK); lock(console)
        require_policy_case(active_policy, 'current', int(time.time()))
        assert snapshot.read_bytes() != pristine and not (root/'delivery.xml').exists()
        pristine = snapshot.read_bytes()
        forward = open_forward()
        facts.append('known_good_approved_original_key_authorizes_real_progress_under_current_policy_before_bad_credential_controls')

        for field, value in [('account', 'other@home.test'), ('device', 999),
                             ('identity', '07'*32), ('key', str(profiles[0].parent/'missing.key'))]:
            changed = tomllib.loads(profiles[0].read_text()); changed[field] = value
            invalid = profiles[0].parent/('refused-'+field+'.toml')
            new_private(invalid, ''.join(k+'='+json.dumps(v)+'\n' for k, v in changed.items()).encode())
            require_policy_case(active_policy, 'current', int(time.time()))
            before = checkpoint_bytes(root/'alice')
            console = start('alice', invalid); desk(console); console.raw(b'u')
            console.expect('Selected attachment profile refused.'); console.expect(DESK); lock(console)
            assert checkpoint_bytes(root/'alice') == before and snapshot.read_bytes() == pristine
            assert not (root/'delivery.xml').exists()
            require_policy_case(active_policy, 'current', int(time.time()))
        facts.append('wrong_account_device_full_identity_and_copied_public_card_without_private_key_refuse_locally_before_network_or_mutation')

        def refuse(profile, *, policy_case='current'):
            require_policy_case(active_policy, policy_case, int(time.time()))
            before = snapshot.read_bytes()
            delivery = (root/'delivery.xml').read_bytes() if (root/'delivery.xml').exists() else None
            console = start('alice', profile); desk(console); upload(console, operation)
            console.expect('Transfer stopped; no automatic retry.'); console.expect(DESK); lock(console)
            assert snapshot.read_bytes() == before
            assert ((root/'delivery.xml').read_bytes() if (root/'delivery.xml').exists() else None) == delivery
            assert public_mail.blob_requests == 0
            require_policy_case(active_policy, policy_case, int(time.time()))

        wrong_pin = profiles[0].parent/'wrong-pin.toml'
        new_private(wrong_pin, profiles[0].read_bytes().replace(fingerprint(pairs['good'][0]).hex().encode(), b'01'*32))
        refuse(wrong_pin)
        wrong_device = profiles[0].parent/'wrong-key.toml'
        copy_public(member_pairs[1][0], profiles[0].parent/'other.pem', 'certificate')
        # This stays in the member fixture: a deliberately selected different
        # member-owned key tests the real provider's actor binding.
        new_private(profiles[0].parent/'other.key', member_pairs[1][1].read_bytes())
        new_private(wrong_device, profiles[0].read_bytes().replace(b'device.pem', b'other.pem').replace(b'device.key', b'other.key'))
        refuse(wrong_device)
        assert snapshot.read_bytes() == pristine
        facts.append('wrong_home_leaf_pin_and_actual_different_device_key_deny_before_store_or_mail_mutation')

        unknown = profiles[0].parent/'unapproved'; unknown.mkdir(mode=0o700)
        unknown_pair = member_credentials(unknown, server, 'unapproved', args.openssl)
        member_pairs.append(unknown_pair)
        unknown_values = tomllib.loads(profiles[0].read_text())
        unknown_values.update(ca=str(unknown/'ca.pem'), certificate=str(unknown_pair[0]), key=str(unknown_pair[1]))
        unknown_profile = unknown/'profile.toml'
        new_private(unknown_profile, ''.join(k+'='+json.dumps(v)+'\n' for k, v in unknown_values.items()).encode())
        refuse(unknown_profile)
        facts.append('valid_ca_signed_but_unapproved_member_generated_transport_key_cannot_create_provider_authority')
        provider.close(); provider = None
        expiry_stamp = int(time.time())
        active_policy = scenario_policy(cards, 'expired', generation=2, reset_epoch=1, stamp=expiry_stamp)
        expired_proposed = server/'expiry-approved.toml'
        new_private(expired_proposed, active_policy.encode())
        expired_candidate = server/'expiry-checked.toml'
        assert command([args.operator, 'prepare', configuration, server/'alice-card.toml',
                        server/'alice-device.pem', expired_proposed, expired_candidate]) == b'prepared candidate; not applied or approved\n'
        assert command([args.operator, 'apply', configuration, expired_candidate]) == b'applied\n'
        provider = Operator(args.operator, configuration, server)
        wait_fact(lambda: int(time.time()) > expiry_stamp+2, seconds=5)
        refuse(profiles[0], policy_case='expired')
        facts.append('current_credential_policy_expiry_refuses_the_previously_approved_key_without_mutation')

        provider.close(); provider = None
        replacement = profiles[0].parent/'replacement'; replacement.mkdir(mode=0o700)
        replacement_pair = member_credentials(replacement, server, 'alice-replacement', args.openssl)
        member_pairs.append(replacement_pair)
        rotated_values = tomllib.loads(templates[0].read_text())
        rotated_values.update(ca=str(replacement/'ca.pem'), certificate=str(replacement_pair[0]), key=str(replacement_pair[1]))
        rotated_template = replacement/'transport.toml'
        new_private(rotated_template, ''.join(k+'='+json.dumps(v)+'\n' for k, v in rotated_values.items()).encode())
        rotated_profile, rotated_card_path = replacement/'profile.toml', replacement/'card.toml'
        prepare_member(args.client, root/'alice', rotated_template, rotated_profile, rotated_card_path,
                       local_password, unicode, children)
        rotated_card = tomllib.loads(rotated_card_path.read_text())
        rotated_verification = dict(verified_records[0], certificate_sha256=fingerprint(replacement_pair[0]).hex())
        replacement_policy = rotation_proposal(cards, rotated_card, rotated_verification,
                                              generation=3, reset_epoch=1, stamp=int(time.time()))
        rotated_proposed = server/'rotated-approved.toml'
        new_private(rotated_proposed, replacement_policy.encode())
        copy_public(rotated_card_path, server/'alice-replacement-card.toml', 'card')
        rotated_candidate = server/'rotated-checked.toml'
        assert command([args.operator, 'prepare', configuration, server/'alice-replacement-card.toml',
                        server/'alice-replacement-device.pem', rotated_proposed, rotated_candidate]) == b'prepared candidate; not applied or approved\n'
        assert command([args.operator, 'apply', configuration, rotated_candidate]) == b'applied\n'
        assert command([args.operator, 'inspect', configuration]) == b'policy generation=3 reset=1 available=1 bindings=2 history=3\n'
        active_policy = replacement_policy
        provider = Operator(args.operator, configuration, server)
        refuse(profiles[0])
        profiles[0] = rotated_profile; cards[0] = rotated_card
        verified_records.append(rotated_verification)
        new_private(server/'alice-replacement-verification.json', (json.dumps(rotated_verification, sort_keys=True)+'\n').encode())
        facts.append('deliberate_same_identity_transport_rotation_rechecks_new_csr_card_leaf_and_complete_proposal_retires_old_key_preserving_history')

        # A live SSH interruption leaves the encrypted durable operation intact.
        console = start('alice', profiles[0]); desk(console); upload(console, operation)
        console.expect('SDK call: 1 / 65; writes: 0')
        forward.close()
        console.expect('Transfer stopped; no automatic retry.'); console.expect(DESK); lock(console)
        assert snapshot.read_bytes() != pristine and not (root/'delivery.xml').exists()
        facts.append('real_ssh_interruption_after_confirmed_progress_retains_pending_ciphertext_without_mail_release')
        provider.close(); provider = None
        assert command([args.operator, 'inspect', configuration]) == b'policy generation=3 reset=1 available=1 bindings=2 history=3\n'
        provider = Operator(args.operator, configuration, server)
        forward = open_forward()
        console = start('alice', profiles[0]); desk(console); submitted = upload(console, operation)
        upload_observation = wait_transfer(console, 1_048_592, started=submitted); console.expect(DESK)
        wait_fact(lambda: peers[0].messages == 1); lock(console)
        facts.append('restarted_maintained_provider_and_new_ssh_forward_resolve_exact_persisted_retry_before_encrypted_mail_release')

        console = start('bob', profiles[1]); console.raw(b'a'); console.password(home_password, ONLINE_HOME)
        console.expect('Archive page saved. Entries: 1; accepted: 1;'); console.expect('Enter to return: ')
        console.send(''); console.expect(MENU); desk(console)
        console.raw(b'r'); console.expect('Original record number (blank cancels): '); console.send('1'); console.expect(RECEIVED)
        before = console.transcript.count(HOME.encode())
        console.raw(b'f'); console.expect(TRANSFER); console.send('transfer'); submitted = time.monotonic()
        fetch_observation = wait_transfer(console, 1_048_592, started=submitted); console.expect(RECEIVED)
        assert console.transcript.count(HOME.encode()) == before
        console.raw(b'v'); console.expect('Verified locally: 1048576 plaintext bytes.'); console.expect(RECEIVED)
        destination = root/'saved'/'explicit.bin'; destination.parent.mkdir(mode=0o700)
        def save():
            console.raw(b's'); console.expect('Absolute save path (blank cancels): '); console.send(str(destination))
            console.expect('Type save to verify and publish: '); console.send('save'); console.expect(RECEIVED)
        save(); assert destination.read_bytes() == payload and stat.S_IMODE(destination.stat().st_mode) == 0o600
        save(); assert b'Save failed before publication.' in console.transcript and destination.read_bytes() == payload
        assert b'\\u{202e}' in console.transcript and '\u202e'.encode() not in console.transcript
        lock(console, True)
        facts.append('separate_actual_encrypted_xmpp_receipt_authenticated_forwarded_fetch_verify_and_explicit_private_no_overwrite_save')

        # Production operator policy changes deliberately stop admission first;
        # Runtime live-replacement behavior is qualified separately, not invented
        # as an unimplemented online administration interface.
        delivered = (root/'delivery.xml').read_bytes(); completed = snapshot.read_bytes()
        provider.close(); provider = None
        revoked = server/'revoked.toml'
        active_policy = scenario_policy(cards, 'revoked', generation=4, reset_epoch=1, stamp=int(time.time()))
        new_private(revoked, active_policy.encode())
        assert command([args.operator, 'apply', configuration, revoked]) == b'applied\n'
        provider = Operator(args.operator, configuration, server)
        refuse(profiles[0], policy_case='revoked')
        assert snapshot.read_bytes() == completed and (root/'delivery.xml').read_bytes() == delivered
        facts.append('stop_apply_restart_revocation_denies_old_device_without_attachment_or_mail_mutation')
        provider.close(); provider = None
        assert command([args.operator, 'reset', configuration, '2']) == b'reset\n'
        assert command([args.operator, 'inspect', configuration]) == b'policy generation=5 reset=2 available=1 bindings=0 history=3\n'
        active_policy = 'version=1\ngeneration=5\nreset_epoch=2\nhomes=["home.test"]\nbindings=[]\n'
        provider = Operator(args.operator, configuration, server)
        refuse(profiles[0], policy_case='reset')
        facts.append('explicit_provider_reset_retains_history_and_requires_fresh_approval_for_old_transport_key')

        before_password_reset = checkpoint_bytes(server/'policy')
        home_password = secrets.token_hex(20)
        private_password_values.append(home_password.encode())
        for peer in peers: peer.password = home_password
        console = start('alice', profiles[0]); console.raw(b'u'); console.password(home_password, ONLINE_HOME)
        console.expect('Manual cycle complete.'); console.expect('Enter to return: ')
        console.send(''); console.expect(MENU); console.raw(b'l'); console.finish()
        assert checkpoint_bytes(server/'policy') == before_password_reset
        wrong_unlock = subprocess.run([str(args.fixture), 'inspect', str(root), 'alice'],
                                      input=(home_password+'\n').encode(), capture_output=True, timeout=5)
        assert wrong_unlock.returncode != 0 and wrong_unlock.stdout == b''
        assert command([args.fixture, 'inspect', root, 'alice'], secret=secret)
        refuse(unknown_profile, policy_case='reset')
        facts.append('independent_home_account_password_reset_authenticates_mail_but_neither_approves_a_new_transport_key_nor_unlocks_member_keys')

        provider.close(); provider = None
        # A successfully established SSH forward can outlive an unavailable home
        # provider. The attachment fails while real ordinary mail still works.
        refuse(profiles[0], policy_case='reset')
        console = start('alice', profiles[0]); console.raw(b'u'); console.password(home_password, ONLINE_HOME)
        console.expect('Manual cycle complete.'); console.expect('Enter to return: ')
        console.send(''); console.expect(MENU); console.raw(b'l'); console.finish()
        assert public_mail.blob_requests == 0 and peers[0].messages == 1
        facts.append('provider_outage_with_live_ssh_setup_preserves_independent_ordinary_mail_without_attachment_fallback')
        command([args.fixture, 'audit', root], secret=secret)
        excluded = private_password_values+[b'private-selected-subject',
                    b'private-selected-body', 'private-selected-name.bin\u202e'.encode(), payload[:128]]
        excluded += [pair[1].read_bytes() for pair in member_pairs]
        scanned = []
        for path in server.rglob('*'):
            if path.is_symlink(): raise ValueError('unexpected provider scan symlink')
            if path.is_file():
                data = path.read_bytes()
                assert len(data) <= 16*1024*1024 and not any(value in data for value in excluded)
                scanned.append(dict(path=str(path.relative_to(server)), bytes=len(data),
                                    sha256=hashlib.sha256(data).hexdigest()))
        assert all(not any(password in c.transcript for password in private_password_values) for c in children)
        facts.append('recorded_provider_file_scan_and_device_descriptor_audit_exclude_selected_plaintext_names_passwords_attachment_and_transport_keys')
        return dict(mode='unicode' if unicode else 'classic', checks=facts, plaintext_bytes=len(payload),
                    verification_record_sha256=[hashlib.sha256((json.dumps(v, sort_keys=True)+'\n').encode()).hexdigest() for v in verified_records],
                    provider_scan_files=scanned, provider_scan_count=len(scanned),
                    metadata_visible=['public account/device/full identity', 'leaf digest', 'ciphertext hash/size', 'policy generations', 'timing'],
                    transfer_observations=dict(upload=upload_observation, fetch=fetch_observation),
                    real_ssh_forward=True, maintained_operator=True, mail_received=True)
    finally:
        failures = []
        for console in children:
            try:
                retain_capture(owned_root, 'unicode-console' if unicode else 'classic-console', console.transcript)
                if hasattr(console, 'transfer_observation'):
                    retain_capture(owned_root, 'transfer-observation',
                                   (json.dumps(console.transfer_observation, sort_keys=True)+'\n').encode())
                console.close()
            except BaseException as error: failures.append(error)
        for forward in forwards:
            try: forward.close()
            except BaseException as error: failures.append(error)
        if provider is not None:
            try: provider.close()
            except BaseException as error: failures.append(error)
        for peer in peers:
            try: peer.close()
            except BaseException as error: failures.append(error)
        for name in ['mail.private', 'delivery.xml']:
            if (root/name).exists():
                try: retain_capture(owned_root, 'private-mail', (root/name).read_bytes())
                except BaseException as error: failures.append(error)
        if not failures and __import__('sys').exc_info()[0] is None:
            shutil.rmtree(root)
            assert not root.exists()
        if failures:
            raise ValueError('owned enrolled-provider cleanup failed') from failures[0]


def probe(args):
    if len(args.denied_ssh_key) != 2 or len(set(args.denied_ssh_key)) != 2 or args.ssh_key in args.denied_ssh_key:
        raise ValueError('two separate denied BBS and SFTP role keys required')
    root = require_root(args.owned_root)
    modes = [one_journey(args, root, unicode) for unicode in [False, True]]
    return dict(result='pass', count=sum(len(mode['checks']) for mode in modes), modes=modes,
                scope='same-os-loopback-dedicated-ssh-software-proof',
                private_fixture_removed=True, children_reaped=True, provider_deployment=False,
                independent_interoperability=False, physical_hardware=False, capacity_claim=False)


if __name__ == '__main__':
    arguments = make_parser().parse_args()
    result = probe(arguments)
    new_private(arguments.output, (json.dumps(result, indent=2, sort_keys=True)+'\n').encode())
    print('Deliberate enrollment and dedicated-forward software proof passed', result['count'], 'controls.')
