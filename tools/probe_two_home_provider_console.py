"""Explicit two-maintained-home console fixture, never a deployed XMPP service."""
import json,os,re,sys
from probe_device_provider_console import fingerprint,_TRANSFER_PROGRESS,_TRANSFER_OUTPUT,_transfer_marker


def table(values):
    return ''.join(k+'='+json.dumps(v)+'\n' for k,v in values.items())


def composed_config(home,peer):
    def endpoint(section,remote,credential):
        return '\n['+section+']\n'+table(dict(address=remote.address,http_host=remote.name,
            server_name=remote.name,leaf_pin=fingerprint(remote.server[0]).hex(),
            ca=str(home.ca if remote is home else home.root/'peer-ca.pem'),
            certificate=str(credential[0]),key=str(credential[1])))
    text=table(dict(version=2,home=home.name,endpoint=home.address,
        policy_root=str(home.root/'policy'),blob_root=str(home.root/'blobs'),
        ca=str(home.ca),certificate=str(home.server[0]),key=str(home.server[1]),
        client_cas=[str(home.ca),str(home.root/'peer-ca.pem')],publication_clients=['publisher']))
    text+=endpoint('local_relay',home,home.relay)
    text+='\n[[peers]]\n'+table(dict(home=peer.name))
    for role,credential in [('publication',home.publisher),('introspection',home.inspector),('resource',home.relay)]:
        text+=endpoint('peers.'+role,peer,credential)
    if len(text.encode())>65536:raise ValueError('composed config bound')
    return text


def composed_proposal(home,peer,*,generation,reset_epoch,remote_generation,remote_reset,
                      approved,peer_enabled,stamp):
    text=table(dict(version=2,generation=generation,reset_epoch=reset_epoch,
        homes=[home.name,peer.name],local_home=home.name,issuer=home.name))
    text+='\n[[bindings]]\n'+table(dict(certificate_sha256=fingerprint(home.device_pair[0]).hex(),
        account=home.account,device=home.device,identity=home.identity,
        valid_from=stamp-60,valid_until=stamp+3600,approved=approved,
        account_enabled=True,device_enabled=True,generation=generation))
    for owner,client,purpose,certificate in [
        (home,'relay','relay',home.relay[0]),(peer,'inspector','introspection',peer.inspector[0]),
        (peer,'relay','resource-client',peer.relay[0]),
        (home,'inspector','outbound-introspection',home.inspector[0]),
        (peer,'endpoint','issuer-endpoint',peer.server[0]),
        (home,'publisher','outbound-observation',home.publisher[0]),
        (peer,'publisher','observation',peer.publisher[0])]:
        text+='\n[[services]]\n'+table(dict(home=owner.name,client_id=client,purpose=purpose,
            dns_san=owner.name,certificate_sha256=fingerprint(certificate).hex(),
            enabled=True if owner is home else peer_enabled,valid_from=stamp-60,valid_until=stamp+3600))
    text+='\n[[issuer_routes]]\n'+table(dict(sender_home=peer.name,audience=peer.name+'/attachments',
        relay_client_id='relay',introspection_client_id='inspector'))
    text+='\n[[resources]]\n'+table(dict(recipient_home=peer.name,issuer=peer.name,
        audience=home.name+'/attachments',relay_client_id='relay',outbound_client_id='inspector',
        endpoint_id='endpoint',address=peer.address,http_host=peer.name,
        ca_sha256=fingerprint(peer.ca).hex(),remote_generation=remote_generation,remote_reset=remote_reset))
    text+='\n[[publications]]\n'+table(dict(recipient_home=peer.name,outbound_client_id='publisher',
        endpoint_id='endpoint',address=peer.address,http_host=peer.name,
        ca_sha256=fingerprint(peer.ca).hex(),remote_generation=remote_generation,remote_reset=remote_reset))
    if len(text.encode())>65536:raise ValueError('composed proposal bound')
    return text


class RemoveAfterProgress:
    """Observe the unchanged read bytes; remove only the owned selected source."""
    def __init__(self,source,total,read):
        self.source,self.total,self.read=source,total,read
        self.buffer=b'';self.removed=False

    def __call__(self,fd,count):
        data=self.read(fd,count)
        if not self.removed:
            self.buffer+=data
            if len(self.buffer)<=_TRANSFER_OUTPUT:
                record=_TRANSFER_PROGRESS.search(self.buffer)
                marker=_transfer_marker(self.buffer)
                if record is not None and (marker is None or record.start()<marker[0]):
                    values=tuple(map(int,record.groups()));offset,call,writes=values
                    if (all(str(v).encode()==raw for v,raw in zip(values,record.groups()))
                        and 0<offset<=self.total and call==1 and writes in (0,1)
                        and (not writes or offset==self.total)):
                        self.source.unlink();self.removed=True;self.buffer=b''
        return data


def diagnostic_lines(raw):
    """Accept a complete bounded block of fixed numeric server observations."""
    rows=raw.splitlines(keepends=True)
    if len(rows)>8:raise ValueError('composed operator diagnostic count exceeded')
    lines=[]
    for row in rows:
        match=re.fullmatch(rb'carrier_rejection schema=1 phase=([12]) category=([1-9]) pending=([01]) shared_attempts=([0-8]) finished=([01]) consumed=([01])\n',row)
        if match is None:raise ValueError('composed operator diagnostic stderr invalid')
        phase,category,pending,attempted,finished,consumed=map(int,match.groups())
        lines.append(f'carrier_rejection schema=1 phase={phase} category={category} pending={pending} shared_attempts={attempted} finished={finished} consumed={consumed}')
    return tuple(lines)


class ComposedOperator:
    """One maintained foreground owner; no synthetic status or control worker."""
    def __init__(self,binary,config,capture_root):
        import select,subprocess,time
        from probe_device_provider_console import parse_ready,retain_capture
        self.root=capture_root;self.closed=False
        self.diagnostics_enabled=os.environ.get('FACTION_PROVIDER_DIAGNOSTICS')=='1'
        self.process=subprocess.Popen([str(binary),'serve-composed',str(config)],
            stdin=subprocess.DEVNULL,stdout=subprocess.PIPE,stderr=subprocess.PIPE)
        deadline=time.monotonic()+5;raw=bytearray()
        try:
            while b'\n' not in raw:
                remaining=deadline-time.monotonic()
                if remaining<=0 or not select.select([self.process.stdout],[],[],remaining)[0]:
                    raise ValueError('composed operator readiness deadline')
                part=os.read(self.process.stdout.fileno(),65-len(raw))
                if not part:raise ValueError('composed operator readiness unavailable')
                raw.extend(part)
                if len(raw)>64:raise ValueError('composed operator readiness bound')
            self.port=parse_ready(bytes(raw))
            retain_capture(self.root,'composed-ready',bytes(raw))
        except BaseException:
            try:
                self.process.terminate()
                try:self.process.wait(timeout=1)
                except subprocess.TimeoutExpired:self.process.kill();self.process.wait(timeout=4)
                error=self.process.stderr.read(4097)
                retain_capture(self.root,'composed-startup-failed',bytes(raw)+error)
                if self.diagnostics_enabled:
                    try:self.publish_diagnostics(error)
                    except ValueError:pass # The original startup failure remains fatal.
            finally:
                self.closed=True;self.process.stdout.close();self.process.stderr.close()
            raise

    def publish_diagnostics(self,error):
        # Validate the whole block before emitting anything. Never copy raw stderr
        # into the hosted log; private capture retains the original bytes.
        for line in diagnostic_lines(error):print(line,file=sys.stderr,flush=True)

    def close(self,stop_signal=None):
        import signal,subprocess,time
        from probe_device_provider_console import retain_capture
        if self.closed:return
        started=time.monotonic();killed=False
        try:
            self.process.send_signal(signal.SIGTERM if stop_signal is None else stop_signal)
            try:self.process.wait(timeout=4)
            except subprocess.TimeoutExpired:
                killed=True;self.process.kill()
                self.process.wait(timeout=max(.001,5-(time.monotonic()-started)))
            out=self.process.stdout.read(129);error=self.process.stderr.read(4097)
            retain_capture(self.root,'composed-stop',out+error)
            if self.diagnostics_enabled:self.publish_diagnostics(error)
            if killed or self.process.returncode or out!=b'stopped\n' or (error and not self.diagnostics_enabled):
                raise ValueError('composed operator orderly drain failed')
            if time.monotonic()-started>=5:raise ValueError('composed operator drain exceeded bound')
        finally:
            self.closed=True
            if self.process.poll() is None:self.process.kill();self.process.wait(timeout=1)
            self.process.stdout.close();self.process.stderr.close()


def owned_command(argv,capture,*,secret=None,timeout=30):
    """Retain this finite fixture's exact command and bounded raw outcome."""
    import hashlib,subprocess
    from probe_device_provider_console import retain_capture
    result=subprocess.run(list(map(str,argv)),input=secret,capture_output=True,timeout=timeout)
    retain_capture(capture,'command',json.dumps(dict(argv=list(map(str,argv)),exit=result.returncode,
        stdin_sha256=None if secret is None else hashlib.sha256(secret).hexdigest()),sort_keys=True).encode())
    retain_capture(capture,'command-stdout',result.stdout)
    retain_capture(capture,'command-stderr',result.stderr)
    if result.returncode:raise ValueError('owned composed fixture command failed')
    return result.stdout


def generate_home(root,name,account,device,identity,address,device_root,openssl):
    from types import SimpleNamespace
    from probe_device_provider_console import private
    root.mkdir(mode=0o700);device_root.mkdir(mode=0o700)
    for part in ['policy','blobs']:(root/part).mkdir(mode=0o700)
    private(root/'OWNED',b'Owned issue322 maintained composed home fixture\n')
    private(root/'openssl-fixture.cnf',b'[req]\ndistinguished_name=dn\n[dn]\n')
    ca=root/'ca.pem'
    owned_command([openssl,'req','-config',root/'openssl-fixture.cnf','-x509','-newkey','ec',
        '-pkeyopt','ec_paramgen_curve:P-256','-nodes','-sha256','-days','2','-subj','/CN='+name+' synthetic CA',
        '-addext','basicConstraints=critical,CA:TRUE','-addext','keyUsage=critical,keyCertSign,cRLSign',
        '-addext','subjectKeyIdentifier=hash','-keyout',root/'ca.key','-out',ca],root)
    for p in [ca,root/'ca.key']:p.chmod(0o600)
    def leaf(label,destination,server=False):
        import secrets
        pem,key,csr,ext=[destination/(label+suffix) for suffix in ['.pem','.key','.csr','.ext']]
        owned_command([openssl,'req','-config',root/'openssl-fixture.cnf','-new','-newkey','ec',
            '-pkeyopt','ec_paramgen_curve:P-256','-nodes','-sha256','-subj','/CN='+name+' '+label,
            '-keyout',key,'-out',csr],root)
        private(ext,('basicConstraints=critical,CA:FALSE\nsubjectAltName=DNS:'+name+
            '\nextendedKeyUsage='+('serverAuth' if server else 'clientAuth')+
            '\nkeyUsage=critical,digitalSignature\nsubjectKeyIdentifier=hash\nauthorityKeyIdentifier=keyid,issuer\n').encode())
        owned_command([openssl,'x509','-req','-in',csr,'-CA',ca,'-CAkey',root/'ca.key',
            '-set_serial',str(secrets.randbits(96) or 1),'-days','2','-sha256','-extfile',ext,'-out',pem],root)
        for p in [pem,key,csr]:p.chmod(0o600)
        return pem,key
    return SimpleNamespace(root=root,name=name,account=account,device=device,identity=identity,
        address=address,ca=ca,server=leaf('server',root,True),relay=leaf('relay',root),
        inspector=leaf('inspector',root),publisher=leaf('publisher',root),
        device_pair=leaf('device',device_root),generation=1,reset=1,approved=True,peer_enabled=True)


class Journey:
    """Two independent maintained attachment owners and a protocol-only mail queue.

    The two authenticated mail listeners share one explicit fixture delivery queue.
    They do not implement production XMPP S2S, enrollment or interoperability.
    Member vaults/keys/passwords remain outside both provider configuration roots.
    """
    def __init__(self,client,fixture,operator,owned_root,openssl,unicode):
        import tempfile,secrets,time
        from pathlib import Path
        from probe_device_provider_console import private
        self.root=Path(tempfile.mkdtemp(prefix='two-home-OWNED-',dir=owned_root));self.root.chmod(0o700)
        self.client,self.fixture,self.operator,self.openssl=client,fixture,operator,openssl
        self.unicode=unicode;self.children=[];self.peers=[];self.operators=[];self.reservations=[]
        self.local_password=secrets.token_hex(20);self.home_passwords=[secrets.token_hex(20) for _ in range(2)]
        self.secret=(self.local_password+'\n').encode();self.facts=[]
        private(self.root/'OWNED',b'Owned authenticated-provider member seed\n')
        private(self.root/'private-inputs.json',json.dumps(dict(local_password=self.local_password,
            home_passwords=self.home_passwords,scope='synthetic member/controller inputs; never home-worker configuration')).encode())
        self.stamp=int(time.time())

    def setup(self):
        import socket,re
        from probe_device_provider_console import private,PublicMail,HomePeer,mail_settings,profile
        owned_command([self.fixture,'seed',self.root,'foreign'],self.root,secret=self.secret)
        identities=(self.root/'public-identities.txt').read_text().splitlines()
        if len(identities)!=2 or any(not re.fullmatch('[0-9a-f]{64}',v) for v in identities):
            raise ValueError('actual seeded public identities unavailable')
        for _ in range(2):
            sock=socket.socket();sock.bind(('127.0.0.1',0));self.reservations.append(sock)
        (self.root/'providers').mkdir(mode=0o700);(self.root/'device-transport').mkdir(mode=0o700)
        self.homes=[]
        for n,(label,home,account,device) in enumerate([
            ('home','home.test','alice@home.test',101),('away','away.test','bob@away.test',202)]):
            address='127.0.0.1:'+str(self.reservations[n].getsockname()[1])
            self.homes.append(generate_home(self.root/'providers'/label,home,account,device,
                identities[n],address,self.root/'device-transport'/label,self.openssl))
        for n,home in enumerate(self.homes):
            peer=self.homes[1-n];private(home.root/'peer-ca.pem',peer.ca.read_bytes())
            home.config=home.root/'operator.toml'
            private(home.config,composed_config(home,peer).encode())
            proposal=home.root/'proposal-1.toml'
            private(proposal,composed_proposal(home,peer,generation=1,reset_epoch=1,
                remote_generation=1,remote_reset=1,approved=True,peer_enabled=True,stamp=self.stamp).encode())
            if owned_command([self.operator,'init-composed',home.config,proposal],home.root)!=b'initialized\n':
                raise ValueError('composed initialization reply unavailable')
            if any((home.root/'blobs').iterdir()):raise ValueError('fixture supplied unauthorized Store inputs')
        for n,home in enumerate(self.homes):
            self.reservations[n].close()
            server=ComposedOperator(self.operator,home.config,home.root);self.operators.append(server)
            if server.port!=int(home.address.rsplit(':',1)[1]):raise ValueError('fixed composed listener changed')
        if fingerprint(self.homes[0].ca)==fingerprint(self.homes[1].ca):raise ValueError('home roots are not independent')
        self.mail=PublicMail(self.root,accounts=tuple(h.account for h in self.homes),
            devices=tuple(h.device for h in self.homes),archive_owner=self.homes[1].account)
        self.mail_configs=[];self.profiles=[]
        for n,(member,home) in enumerate(zip(['alice','bob'],self.homes)):
            peer=HomePeer(self.root,home.server,home.account,home.device,self.home_passwords[n],self.mail,n)
            self.peers.append(peer)
            self.mail_configs.append(mail_settings(self.root,member,peer,home.server[0],home.ca))
            self.profiles.append(profile(self.root,member,home.identity,int(home.address.rsplit(':',1)[1]),
                home.server[0],home.device_pair,route='own-home',home=home.name,account=home.account,
                device=home.device,ca=home.ca))
        self.facts.append('independent_composed_operators_started_from_actual_seeded_public_identities_local_device_only_registries')

    def start(self,member):
        from probe_device_provider_console import LauncherConsole,UNLOCK,MENU
        n=0 if member=='alice' else 1
        args=[self.client,'--mail-online',self.root/member,self.mail_configs[n],
              '--device-provider',self.profiles[n]]
        if self.unicode:args.append('--unicode')
        console=LauncherConsole(args,columns=40,rows=18);self.children.append(console)
        console.password(self.local_password,UNLOCK);console.expect(MENU)
        return console

    def snapshots(self):
        return [(h.root/'blobs/private-blobs.snapshot').read_bytes() for h in self.homes]

    def close(self):
        from probe_device_provider_console import retain_capture
        failures=[]
        for console in self.children:
            try:retain_capture(self.root,'unicode-console' if self.unicode else 'classic-console',console.transcript)
            except BaseException as error:failures.append(error)
            try:console.close()
            except BaseException as error:failures.append(error)
        for peer in self.peers:
            try:peer.close()
            except BaseException as error:failures.append(error)
        for server in self.operators:
            try:server.close()
            except BaseException as error:failures.append(error)
        for reservation in self.reservations:reservation.close()
        if failures:raise ValueError('owned composed console cleanup failed') from failures[0]


def checkpoint_bytes(root):
    return {p.relative_to(root).as_posix():p.read_bytes() for p in root.rglob('*') if p.is_file()}



def expect_prepare(journey,console,before,bob_before):
    """The same positive prompt oracle is used before and after native support."""
    from probe_device_provider_console import private
    try:
        console.expect('Absolute LOCAL source path (blank cancels): ')
    except BaseException:
        current=console.transcript
        refusal=('v2-profile-before-prepare' if b'Selected attachment profile refused.' in current
                 else 'foreign-route-before-prepare' if b'Attachment action unavailable.' in current else None)
        if refusal is not None:
            assert journey.snapshots()==before and not (journey.root/'delivery.xml').exists()
            assert not (journey.root/'alice/attachments').exists()
            assert checkpoint_bytes(journey.root/'bob')==bob_before and journey.peers[1].authenticated==0
            private(journey.root/'current-refusal.private.json',json.dumps(dict(
                stage=refusal,actual_maintained_composed_owners=2,stores_unchanged=True,
                recipient_checkpoint_unchanged=True,no_recipient_process=True,no_spool=True,
                no_mail_release=True,scope='actual current native refusal after verified composed setup')).encode())
            raise AssertionError('current native path refused the positive two-home preparation') from None
        raise


def preparation_gate(client,fixture,operator,owned_root,openssl,unicode=False):
    """Positive preparation prompt oracle, independent of the later byte journey."""
    from probe_device_provider_console import DESK,MENU,private
    journey=Journey(client,fixture,operator,owned_root,openssl,unicode)
    try:
        journey.setup();before=journey.snapshots();bob_before=checkpoint_bytes(journey.root/'bob')
        console=journey.start('alice');console.raw(b'm');console.expect(DESK);console.raw(b'c')
        expect_prepare(journey,console,before,bob_before)
        console.send('');console.expect('Attachment compose cancelled.');console.expect(DESK)
        console.raw(b'b');console.expect(MENU);console.raw(b'l');console.finish()
        assert journey.snapshots()==before and checkpoint_bytes(journey.root/'bob')==bob_before
        return dict(result='pass',scope='two-maintained-home-v2-preparation-gate',private_fixture_retained=True)
    finally:journey.close()


def audit_selected(journey,payload):
    """Byte-bound reuse of the unchanged member-only descriptor audit."""
    import base64,hashlib,subprocess
    from probe_device_provider_console import private,retain_capture
    root=journey.root;mirror=root/'server';mirror.mkdir(mode=0o700)
    (mirror/'serverstore').mkdir(mode=0o700)
    private(mirror/'SCOPE',b'Audit-only exact copies; no live provider, Store opening, Session or seed authority.\n')
    selected=[]
    for n,home in enumerate(journey.homes):
        for p in sorted((home.root/'blobs').iterdir()):selected.append(('home-'+str(n)+'-store',p))
        for p in sorted(home.root.glob('*.private')):
            if p.name.startswith(('composed-','command-stdout-','command-stderr-')):
                selected.append(('home-'+str(n)+'-log',p))
    selected += [('encrypted-mail',root/'delivery.xml'),('encrypted-mail',root/'mail.private')]
    needles=[payload,b'private-selected-name.bin\xe2\x80\xae',b'private-selected-subject',b'private-selected-body',
             journey.local_password.encode(),*(s.encode() for s in journey.home_passwords)]
    needles += [base64.b64encode(v) for v in list(needles)] + [v.hex().encode() for v in list(needles)]
    fields=[b'authorization: bearer ',b'"access_token"',b'"token":']
    bindings=[]
    for index,(role,source) in enumerate(selected):
        if source.is_symlink() or not source.is_file():raise ValueError('actual audit source must be regular')
        data=source.read_bytes()
        if len(data)>16*1024*1024 or any(v in data for v in needles) or any(v in data.lower() for v in fields):
            raise ValueError('selected actual store log or mail privacy check failed')
        copy=mirror/'serverstore'/('selected-'+str(index)+'.private')
        private(copy,data)
        bindings.append(dict(role=role,source=str(source),copy=str(copy),sha256=hashlib.sha256(data).hexdigest(),bytes=len(data)))
    before=checkpoint_bytes(root/'alice')
    argv=[str(journey.fixture),'audit',str(root)]
    result=subprocess.run(argv,input=journey.secret,capture_output=True,timeout=30)
    retain_capture(root,'member-audit-command',json.dumps(dict(argv=argv,stdin_sha256=hashlib.sha256(journey.secret).hexdigest(),exit=result.returncode)).encode())
    retain_capture(root,'member-audit-stdout',result.stdout);retain_capture(root,'member-audit-stderr',result.stderr)
    if result.returncode or result.stdout!=b'audited\n' or result.stderr:
        raise ValueError('unchanged member-side descriptor audit failed')
    if checkpoint_bytes(root/'alice')!=before:raise ValueError('member audit changed checkpoint')
    for binding in bindings:
        from pathlib import Path
        source,copy=Path(binding['source']),Path(binding['copy'])
        if source.read_bytes()!=copy.read_bytes() or hashlib.sha256(source.read_bytes()).hexdigest()!=binding['sha256']:
            raise ValueError('audit copy no longer matches actual selected input')
    proof=dict(sources=bindings,direct_actual_path_scan=True,member_descriptor_audit=True,
        private_name_key_iv_hash_subject_body_scan=True,plaintext_password_raw_base64_hex_scan=True,
        bounded_token_field_pattern_scan=True,hidden_exact_token_values_available=False,
        public_identity_routing_and_server_owned_tls_keys_are_distinct=True,
        no_store_session_policy_authority_from_copies=True,member_audit_checkpoint_unchanged=True)
    private(root/'privacy-audit.private.json',json.dumps(proof,indent=2).encode())
    return dict(actual_stores=2,actual_source_files=len(bindings),descriptor_and_content_key_audit=True,
        direct_plaintext_password_scan=True,token_field_patterns_only=True,hidden_exact_tokens_unavailable=True)


def one_journey(client,fixture,operator,owned_root,openssl,unicode=False):
    """Actual maximum-file positive, with original observer and SDK limits."""
    import re,time,stat,hashlib
    from xml.etree import ElementTree as ET
    from probe_device_provider_console import (DESK,MENU,RECEIVED,HOME,ONLINE_HOME,TRANSFER,
        private,wait_transfer,wait_fact,retain_capture)
    journey=Journey(client,fixture,operator,owned_root,openssl,unicode)
    result=None
    def desk(c):c.raw(b'm');c.expect(DESK)
    def lock(c,received=False):
        if received:c.raw(b'b');c.expect(DESK)
        c.raw(b'b');c.expect(MENU);c.raw(b'l');c.finish()
    def upload(c,operation):
        c.raw(b'u');c.expect('Full operation ID (blank cancels): ');c.send(operation)
        c.expect(TRANSFER);c.send('transfer');c.password(journey.home_passwords[0],HOME)
        return time.monotonic()
    def save(c,destination):
        c.raw(b's');c.expect('Absolute save path (blank cancels): ');c.send(str(destination))
        c.expect('Type save to verify and publish: ');c.send('save');c.expect(RECEIVED)
    try:
        journey.setup();before=journey.snapshots();bob_before=checkpoint_bytes(journey.root/'bob')
        source=journey.root/'selected-source.bin'
        payload=bytes((i*73+19)%251 for i in range(1048576));private(source,payload)
        c=journey.start('alice');desk(c);c.raw(b'c');expect_prepare(journey,c,before,bob_before)
        c.send(str(source))
        for prompt,answer in [
            ('Private display name (255 UTF-8 bytes): ','private-selected-name.bin\u202e'),
            ('Media type (127 ASCII bytes): ','application/octet-stream'),
            ('Subject (128 UTF-8 bytes): ','private-selected-subject'),
            ('Body (1024 UTF-8 bytes, one line): ','private-selected-body')]:
            c.expect(prompt);c.send(answer)
        c.expect('Type queue to encrypt and persist attachment: ');c.send('queue');c.expect(DESK)
        operations=re.findall(rb'Operation: ([0-9a-f]{32})',c.transcript);assert len(operations)==1
        operation=operations[0].decode();lock(c)
        assert source.exists() and not (journey.root/'delivery.xml').exists()
        assert checkpoint_bytes(journey.root/'bob')==bob_before and journey.peers[1].authenticated==0
        journey.facts.append('actual_foreign_peer_native_queue_is_device_local_with_recipient_absent_and_unchanged')
        c=journey.start('alice');desk(c);started=upload(c,operation)
        remove=RemoveAfterProgress(source,1048592,os.read)
        upload_observation=wait_transfer(c,1048592,started=started,read=remove,unicode=unicode);c.expect(DESK)
        assert remove.removed and not source.exists()
        assert upload_observation['progress'][-1]['writes']==1
        wait_fact(lambda:journey.peers[0].messages==1)
        lock(c)
        assert journey.mail.blob_requests==0 and journey.peers[1].authenticated==0
        assert checkpoint_bytes(journey.root/'bob')==bob_before
        delivery=(journey.root/'delivery.xml').read_bytes();message=ET.fromstring(delivery)
        assert message.get('from')==journey.homes[0].account and message.get('to')==journey.homes[1].account
        assert message.find('{urn:xmpp:omemo:2}encrypted') is not None
        assert message.find('{jabber:client}body') is None and message.find('{jabber:client}subject') is None
        assert payload not in delivery
        committed=journey.snapshots();assert committed[0]!=before[0] and committed[1]==before[1]
        private(journey.root/'committed-sender-store.snapshot',committed[0])
        journey.facts.extend(['source_removed_only_after_canonical_confirmed_progress_and_upload_completed_under_original_limits',
            'authenticated_committed_sender_store_precedes_actual_encrypted_mail_while_recipient_process_absent',
            'recipient_checkpoint_byte_identical_before_archive_catchup_and_no_foreign_object_seed'])
        # Alice has exited and joined before Bob becomes the sole active client.
        c=journey.start('bob');c.raw(b'a');c.password(journey.home_passwords[1],ONLINE_HOME)
        c.expect('Archive page saved. Entries: 1; accepted: 1;');c.expect('Enter to return: ');c.send('');c.expect(MENU)
        assert checkpoint_bytes(journey.root/'bob')!=bob_before
        archive=(journey.root/'mail.private').read_bytes()
        assert b'urn:xmpp:mam:2' in archive and b'urn:xmpp:omemo:2' in archive
        desk(c);c.raw(b'r');c.expect('Original record number (blank cancels): ');c.send('1');c.expect(RECEIVED)
        assert not (journey.root/'bob/attachments').exists()
        before_password=c.transcript.count(HOME.encode())
        c.raw(b'f');c.expect(TRANSFER);c.send('transfer');started=time.monotonic()
        fetch_observation=wait_transfer(c,1048592,started=started,unicode=unicode);c.expect(RECEIVED)
        assert c.transcript.count(HOME.encode())==before_password
        c.raw(b'v');c.expect('Verified locally: 1048576 plaintext bytes.');c.expect(RECEIVED)
        destination=journey.root/'saved'/'explicit.bin';destination.parent.mkdir(mode=0o700)
        save(c,destination)
        assert destination.read_bytes()==payload and stat.S_IMODE(destination.stat().st_mode)==0o600
        save(c,destination)
        assert b'Save failed before publication.' in c.transcript and destination.read_bytes()==payload
        assert b'\\u{202e}' in c.transcript and '\u202e'.encode() not in c.transcript
        lock(c,True)
        assert journey.peers[0].messages==1 and journey.mail.archive_pages>=1 and journey.mail.blob_requests==0
        assert journey.snapshots()[1]==before[1]
        journey.facts.extend(['actual_encrypted_archive_catchup_persists_receipt_after_offline_checkpoint',
            'recipient_uses_only_own_configured_home_device_tls_without_home_password_or_iq_fallback',
            'exact_maximum_plaintext_verification_and_explicit_0600_save_refuse_overwrite'])
        for console in journey.children:
            assert journey.local_password.encode() not in console.transcript
            assert all(p.encode() not in console.transcript for p in journey.home_passwords)
        result=dict(result='pass',mode='unicode' if unicode else 'classic',checks=journey.facts,
            plaintext_bytes=len(payload),ciphertext_bytes=1048592,saved_sha256=hashlib.sha256(payload).hexdigest(),
            maintained_composed_owners=2,one_active_client_maximum=True,recipient_initially_offline=True,
            transfer_observations=dict(upload=upload_observation,fetch=fetch_observation),
            topology='two separately authenticated mail endpoints; one fixture-only opaque delivery/archive queue',
            private_fixture_retained=True,provider_deployment=False,production_s2s=False,
            independent_interoperability=False,physical_hardware=False,capacity_claim=False)
    finally:journey.close()
    result['privacy']=audit_selected(journey,payload)
    result['children_reaped']=True
    private(journey.root/'console-result.private.json',json.dumps(result,indent=2).encode())
    return result


def probe(client,fixture,operator,owned_root,openssl):
    from probe_device_provider_console import require_root
    owned_root=require_root(owned_root)
    modes=[one_journey(client,fixture,operator,owned_root,openssl,unicode) for unicode in [False,True]]
    return dict(result='pass',count=sum(len(m['checks']) for m in modes),modes=modes,
        private_fixture_retained=True,children_reaped=True,provider_deployment=False,
        production_s2s=False,independent_interoperability=False,physical_hardware=False,capacity_claim=False)
