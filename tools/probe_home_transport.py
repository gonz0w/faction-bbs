#!/usr/bin/env python3
"""Loopback real-TLS controls for the #209 product transport crate.

Reuses the unchanged qualified #208 peer/certificates. This exercises Home/Session,
not the product OMEMO codec or two-home encrypted-mail exchange. Keeps only public
named outcomes; generated credentials, TLS keys and listeners are disposable.
"""
import argparse
import base64
import hashlib
import json
import platform
import secrets
import ssl
import subprocess
import tempfile
import time
from pathlib import Path
import probe_xmpp_transport as reference


SCENARIOS = [
    ('verified_tls_plain_auth_and_ciphertext_exchange', {'ok'}, {}),
    ('verified_tls12_profile', {'ok'}, dict(mode='tls12')),
    ('fragmented_xml_exchange', {'ok'}, dict(mode='fragmented')),
    ('coalesced_sasl_restart_retains_unread_bytes', {'ok'}, dict(mode='coalesced-restart')),
    ('wrong_server_name', {'tls'}, dict(pair='wrong-name')),
    ('untrusted_certificate', {'tls'}, dict(ca='untrusted-ca')),
    ('changed_certificate_pin', {'pin'}, dict(pair='changed')),
    ('failed_account_authentication', {'auth'}, dict(mode='auth-fail')),
    ('no_plain_mechanism', {'mechanism'}, dict(mode='no-plain')),
    ('malformed_xml_rejected', {'xml_malformed'}, dict(mode='malformed')),
    ('excessive_depth_rejected_before_dom', {'xml_limit'}, dict(mode='depth')),
    ('excessive_events_rejected_before_dom', {'xml_limit'}, dict(mode='events')),
    ('oversized_unfinished_text_rejected', {'xml_limit'}, dict(mode='token')),
    ('oversized_stanza_rejected', {'xml_limit'}, dict(mode='stanza')),
    ('root_whitespace_budget_is_bounded', {'xml_limit'}, dict(mode='whitespace')),
    ('connection_loss_has_no_retry', {'lost'}, dict(mode='lost')),
    ('absolute_deadline_bounds_trickling_peer', {'timeout'}, dict(mode='trickle')),
    ('explicit_cancellation_bounds_stalled_peer', {'cancelled'}, dict(mode='stalled', extra=['--cancel-after-ms', '250'])),
    ('wrong_bound_identity', {'bind'}, dict(mode='wrong-bound')),
    ('wrong_bind_identifier', {'bind'}, dict(mode='wrong-bind-id')),
]


class ProductPeer(reference.Peer):
    def send(self, channel, data):
        if self.mode == 'wrong-bound':
            data = data.replace(b'<jid>alice@home.test/faction208</jid>', b'<jid>mallory@home.test/faction208</jid>')
        if self.mode == 'wrong-bind-id':
            data = data.replace(b"id='bind-209'", b"id='wrong-209'")
        return super().send(channel, data)


def validate_result(stdout, stderr, returncode, expected, password, payload):
    assert not stderr, 'product transport emitted diagnostic data'
    assert password.encode() not in stdout and payload.encode() not in stdout, 'private fixture data escaped'
    result = json.loads(stdout)
    assert set(result) == {'status', 'attempts'}, 'unreviewed result fields'
    assert result['status'] in expected
    assert returncode == (0 if result['status'] == 'ok' else 1)
    assert result['attempts'] == 1
    return result


def run_case(binary, root, pairs, name, expected, mode='good', extra=None, pair='good', ca='ca'):
    payload = base64.b64encode(secrets.token_bytes(64)).decode()
    password = secrets.token_hex(20)
    peer = ProductPeer(root, pairs[pair], password, payload, mode)
    pin_pair = pair if pair == 'wrong-name' else 'good'
    pin = hashlib.sha256(ssl.PEM_cert_to_DER_cert(pairs[pin_pair][0].read_text())).hexdigest()
    config = '\n'.join([str(peer.port), str(root / (ca + '.pem')), pin, password, payload]) + '\n'
    started = time.monotonic()
    try:
        process = subprocess.run([str(binary), 'probe', *(extra or [])], input=config.encode(), capture_output=True, timeout=15)
        elapsed = time.monotonic() - started
        result = validate_result(process.stdout, process.stderr, process.returncode, expected, password, payload)
        assert peer.connections == 1
        if result['status'] == 'ok':
            assert peer.auth_ok and peer.message_seen
        if name in {'wrong_server_name', 'untrusted_certificate', 'changed_certificate_pin', 'no_plain_mechanism'}:
            assert not peer.auth_seen, 'credentials sent before verified identity/mechanism'
        if name in {'wrong_bound_identity', 'wrong_bind_identifier'}:
            assert peer.auth_ok and not peer.message_seen
        if name == 'absolute_deadline_bounds_trickling_peer':
            assert elapsed < 13
        return reference.validated_outcome(name, result['status'], peer.tls_version)
    finally:
        peer.close()


def probe(binary, openssl='openssl'):
    outcomes = []
    with tempfile.TemporaryDirectory(prefix='faction209-transport-private-') as temporary:
        root = Path(temporary).resolve()
        root.chmod(0o700)
        (root / 'OWNED').write_text('faction209 disposable real-TLS transport fixture\n')
        pairs = reference.certificates(root, openssl)
        for name, expected, arguments in SCENARIOS:
            outcomes.append(run_case(binary, root, pairs, name, expected, **arguments))
    assert not root.exists(), 'private fixture retained'
    return dict(result='pass', platform=platform.system(), count=len(outcomes),
                outcomes=outcomes, private_fixture_removed=True,
                product_transport_exercised=True, encrypted_mail_proof=False,
                live_federation=False)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path, required=True)
    parser.add_argument('--openssl', default='openssl')
    parser.add_argument('--output', type=Path)
    arguments = parser.parse_args()
    result = probe(arguments.binary.resolve(), arguments.openssl)
    if arguments.output:
        arguments.output.write_text(json.dumps(result, indent=2) + '\n')
    print('Product XMPP transport fixture passed:', result['count'], 'controls;', result['platform'])
