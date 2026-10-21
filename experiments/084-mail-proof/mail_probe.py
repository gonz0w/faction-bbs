#!/usr/bin/env python3
"""Synthetic two-home OMEMO-2 reference proof; never production or real member data.

Uses the unchanged MIT OMEMO/Twomemo APIs and a separate bounded SCE adapter.
Client keys stay in memory on the developer device; homes receive only public
bundles and encrypted envelopes. No dependency of the product workspace.
"""
import argparse
import asyncio
from copy import deepcopy
import json
import logging
import secrets
from pathlib import Path
import ssl
from xml.etree import ElementTree as ET

import omemo
from omemo.session_manager import SessionManager
from omemo.storage import Storage, Just, Nothing
from omemo.types import TrustLevel
import slixmpp
from slixmpp.exceptions import IqError
from slixmpp.xmlstream.handler import Callback
from slixmpp.xmlstream.matcher import MatchXPath
import twomemo
import twomemo.etree
from sce import pack, unpack

NS = 'urn:xmpp:omemo:2'
PS = 'http://jabber.org/protocol/pubsub'
DATA = 'jabber:x:data'


class Memory(Storage):
    def __init__(self):
        super().__init__()
        self.values = {}
    async def _load(self, key):
        return Just(deepcopy(self.values[key])) if key in self.values else Nothing()
    async def _store(self, key, value):
        self.values[key] = deepcopy(value)
    async def _delete(self, key):
        self.values.pop(key, None)


class Peer(slixmpp.ClientXMPP):
    def __init__(self, account, ca, storage=None, pins=None):
        super().__init__(account['jid']+'/probe-'+secrets.token_hex(4), account['password'])
        self.account, self.store, self.pins = account, storage or Memory(), pins or {}
        self.ssl_context = ssl.create_default_context(cafile=ca)
        self.ready, self.inbox = asyncio.Event(), asyncio.Queue(maxsize=32)
        self.sent, self.failures, self.manager = [], [], None
        for plugin in ['xep_0030', 'xep_0060', 'xep_0313']:
            self.register_plugin(plugin)
        self.add_event_handler('session_start', self.start)
        self.add_event_handler('message', self.receive)
        self.register_handler(Callback('ciphertext', MatchXPath('{jabber:client}message/{'+NS+'}encrypted'), lambda stanza: asyncio.create_task(self.receive(stanza))))
        self.add_event_handler('failed_auth', lambda _: self.failures.append('auth'))

    async def iq(self, kind, peer, child):
        iq = self.Iq()
        iq['type'], iq['to'] = kind, peer
        iq.append(child)
        return await iq.send(timeout=8)

    async def item(self, peer, node, item_id):
        ps = ET.Element('{'+PS+'}pubsub')
        items = ET.SubElement(ps, '{'+PS+'}items', node=node)
        ET.SubElement(items, '{'+PS+'}item', id=item_id)
        try:
            result = await self.iq('get', peer, ps)
        except IqError as e:
            if e.iq['error']['condition'] == 'item-not-found':
                return None
            raise
        rows = result.xml.findall('.//{'+PS+'}item')
        return rows[0][0] if rows and len(rows[0]) else None

    async def publish(self, node, item_id, payload):
        ps = ET.Element('{'+PS+'}pubsub')
        pub = ET.SubElement(ps, '{'+PS+'}publish', node=node)
        ET.SubElement(pub, '{'+PS+'}item', id=item_id).append(payload)
        form = ET.SubElement(ET.SubElement(ps, '{'+PS+'}publish-options'), '{'+DATA+'}x', type='submit')
        for key, value in [('FORM_TYPE', PS+'#publish-options'), ('pubsub#persist_items', 'true'), ('pubsub#max_items', 'max' if node.endswith(':bundles') else '1'), ('pubsub#access_model', 'open')]:
            ET.SubElement(ET.SubElement(form, '{'+DATA+'}field', var=key), '{'+DATA+'}value').text = value
        await self.iq('set', self.boundjid.bare, ps)

    async def start(self, _):
        peer = self
        class Manager(SessionManager):
            @staticmethod
            async def _download_device_list(namespace, bare_jid):
                assert namespace == NS
                e = await peer.item(bare_jid, NS+':devices', 'current')
                return {} if e is None else twomemo.etree.parse_device_list(e)
            @staticmethod
            async def _upload_device_list(namespace, device_list):
                assert namespace == NS
                await peer.publish(NS+':devices', 'current', twomemo.etree.serialize_device_list(device_list))
            @staticmethod
            async def _upload_bundle(bundle):
                await peer.publish(NS+':bundles', str(bundle.device_id), twomemo.etree.serialize_bundle(bundle))
            @staticmethod
            async def _download_bundle(namespace, bare_jid, device_id):
                assert namespace == NS
                e = await peer.item(bare_jid, NS+':bundles', str(device_id))
                if e is None:
                    raise RuntimeError('missing public bundle')
                return twomemo.etree.parse_bundle(e, bare_jid, device_id)
            @staticmethod
            async def _delete_bundle(namespace, device_id):
                assert namespace == NS
                ps = ET.Element('{'+PS+'}pubsub')
                ET.SubElement(ET.SubElement(ps, '{'+PS+'}retract', node=NS+':bundles'), '{'+PS+'}item', id=str(device_id))
                await peer.iq('set', peer.boundjid.bare, ps)
            @staticmethod
            async def _send_message(message, bare_jid):
                peer.send_encrypted(message, bare_jid)
            async def _evaluate_custom_trust_level(self, device):
                return TrustLevel.TRUSTED if peer.pins.get(device.bare_jid) == device.identity_key else TrustLevel.UNDECIDED
            async def _make_trust_decision(self, undecided, identifier):
                if all(peer.pins.get(d.bare_jid) == d.identity_key for d in undecided):
                    for d in undecided:
                        await self.set_trust(d.bare_jid, d.identity_key, 'PINNED')
                    return
                raise RuntimeError('unverified device: explicit fingerprint pin required')
        try:
            self.manager = await Manager.create([twomemo.Twomemo(self.store, max_num_per_session_skipped_keys=64)], self.store, self.boundjid.bare, 'synthetic-reference', 'UNDECIDED')
            own, _ = await self.manager.get_own_device_information()
            self.pins[self.boundjid.bare] = own.identity_key
            self.send_presence()
            self.ready.set()
        except Exception as e:
            self.failures.append(type(e).__name__)
            self.ready.set()
            raise

    def send_encrypted(self, encrypted, recipient):
        message = self.make_message(mto=recipient, mtype='chat')
        message.append(twomemo.etree.serialize_message(encrypted))
        message.append(ET.Element('{urn:xmpp:hints}store'))
        message.append(ET.Element('{urn:xmpp:eme:0}encryption', namespace=NS, name='OMEMO'))
        assert message.xml.find('{jabber:client}body') is None
        self.sent.append(deepcopy(message.xml))
        message.send()

    async def encrypt_mail(self, recipient, subject, body):
        device_list = await self.manager._download_device_list(NS, recipient)
        await self.manager.update_device_list(NS, recipient, device_list)
        messages, errors = await self.manager.encrypt(frozenset({recipient}), {NS: pack(self.boundjid.bare, recipient, subject, body)}, backend_priority_order=[NS])
        if errors or not messages:
            raise RuntimeError('no fully verified encrypted delivery')
        return messages

    async def send_mail(self, recipient, subject, body):
        messages = await self.encrypt_mail(recipient, subject, body)
        for message in messages:
            assert message.namespace == NS
            self.send_encrypted(message, recipient)
        return len(messages)

    async def receive(self, stanza):
        e = stanza.xml.find('{'+NS+'}encrypted')
        if e is None:
            if stanza['body']:
                self.failures.append('plaintext-refused')
            return
        try:
            message = twomemo.etree.parse_message(e, stanza['from'].bare)
            raw, device, _ = await self.manager.decrypt(message)
            if self.pins.get(device.bare_jid) != device.identity_key:
                raise RuntimeError('unverified sender')
            if raw is not None:
                subject, body = unpack(raw, device.bare_jid, self.boundjid.bare)
                await self.inbox.put((device.bare_jid, subject, body))
        except Exception as error:
            self.failures.append(type(error).__name__)

    async def connect_ready(self):
        await self.connect(self.account['host'], self.account['port'])
        await asyncio.wait_for(self.ready.wait(), 20)
        if self.manager is None or self.failures:
            raise RuntimeError('reference login/initialization failed')

    async def close_peer(self):
        if self.manager:
            await self.manager.shutdown()
        await self.disconnect(wait=2)


async def opened(peer):
    try:
        return await asyncio.wait_for(peer.inbox.get(), 15)
    except TimeoutError:
        raise RuntimeError('encrypted receive failed: '+str(peer.failures)) from None


async def run(profile):
    checks = []
    a, b = Peer(profile['alice'], profile['ca']), Peer(profile['bob'], profile['ca'])
    peers = [a, b]
    try:
        await asyncio.gather(a.connect_ready(), b.connect_ready())
        ao, _ = await a.manager.get_own_device_information()
        bo, _ = await b.manager.get_own_device_information()
        # In this fixture the trusted harness compares keys directly on the devices.
        # Product enrollment must supply an actual member verification UX.
        a.pins[b.boundjid.bare], b.pins[a.boundjid.bare] = bo.identity_key, ao.identity_key
        checks.append('two_separate_homes_tls_login_and_device_publication')
        try:
            await a.item('mallory@uninvited.test', NS+':devices', 'current')
        except IqError as error:
            assert error.iq['error']['condition'] == 'service-unavailable'
        else:
            raise AssertionError('uninvited home route accepted')
        checks.append('uninvited_home_route_denied')
        await a.send_mail(b.boundjid.bare, 'private-subject', profile['marker'])
        assert await opened(b) == (a.boundjid.bare, 'private-subject', profile['marker'])
        checks.append('omemo2_device_encryption_sce_subject_body_and_cross_home_decryption')
        await b.send_mail(a.boundjid.bare, 'reply-subject', 'encrypted reply')
        assert (await asyncio.wait_for(a.inbox.get(), 15))[1:] == ('reply-subject', 'encrypted reply')
        checks.append('bidirectional_ratcheted_encrypted_reply')
        # New/unverified receiver keys fail before a message is sent.
        a.pins.pop(b.boundjid.bare)
        count = len(a.sent)
        try:
            await a.send_mail(b.boundjid.bare, 'blocked', 'unverified')
        except Exception:
            pass
        else:
            raise AssertionError('unknown device was accepted')
        assert len(a.sent) == count
        a.pins[b.boundjid.bare] = bo.identity_key
        checks.append('unknown_device_fails_closed_before_ciphertext_send')
        # Corrupt ciphertext through the real XMPP connection, not a crypto mock.
        fresh = await a.encrypt_mail(b.boundjid.bare, 'tamper-subject', 'tamper-body')
        bad = ET.Element('{jabber:client}message')
        bad.append(twomemo.etree.serialize_message(next(iter(fresh))))
        payload = bad.find('{'+NS+'}encrypted/{'+NS+'}payload')
        text = payload.text
        payload.text = ('A' if text[0] != 'A' else 'B')+text[1:]
        before = len(b.failures)
        msg = a.make_message(mto=b.boundjid.bare, mtype='chat')
        for e in bad: msg.append(e)
        msg.send()
        await asyncio.sleep(1)
        assert len(b.failures) > before and b.inbox.empty()
        checks.append('fresh_modified_ciphertext_does_not_open')
        await a.send_mail(b.boundjid.bare, 'after-tamper', 'verified after failure')
        assert (await opened(b))[1:] == ('after-tamper', 'verified after failure')
        checks.append('valid_delivery_recovers_after_ciphertext_rejection')
        a.send_message(mto=b.boundjid.bare, mbody='plaintext downgrade', mtype='chat')
        await asyncio.sleep(1)
        assert 'plaintext-refused' in b.failures and b.inbox.empty()
        checks.append('plaintext_message_refused_no_downgrade')
        # Keep keys only on the member device while the home queues ciphertext.
        await b.close_peer()
        await a.send_mail(b.boundjid.bare, 'offline-subject', profile['marker']+' offline')
        await asyncio.sleep(1)
        b2 = Peer(profile['bob'], profile['ca'], b.store, b.pins)
        peers.append(b2)
        await b2.connect_ready()
        assert (await asyncio.wait_for(b2.inbox.get(), 15))[1:] == ('offline-subject', profile['marker']+' offline')
        checks.append('offline_ciphertext_queue_and_same_device_reconnect')
        # Archive reads remain bounded and contain opaque encrypted stanzas.
        from slixmpp.jid import JID
        archive_ids = []
        async for result in b2['xep_0313'].iterate(with_jid=JID(a.boundjid.bare), rsm={'max': 20}, total=20):
            raw = ET.tostring(result.xml)
            assert profile['marker'].encode() not in raw and b'private-subject' not in raw
            archive_ids.append(result['mam_result']['id'])
        assert archive_ids and len(archive_ids) <= 20 and len(set(archive_ids)) == len(archive_ids)
        again = []
        async for result in b2['xep_0313'].iterate(with_jid=JID(a.boundjid.bare), rsm={'max': 20}, total=20):
            again.append(result['mam_result']['id'])
        assert archive_ids == again[:len(archive_ids)]
        checks.append('bounded_home_archive_fetch_has_stable_ids_and_no_private_subject_body')
        # Simulate a home administrator admitting a new device with an account reset.
        ghost = Peer(profile['bob'], profile['ca'])
        peers.append(ghost)
        await ghost.connect_ready()
        count = len(a.sent)
        try:
            await a.send_mail(b2.boundjid.bare, 'blocked-new-device', 'must not reach a server-added device')
        except Exception:
            pass
        else:
            raise AssertionError('server-added unverified device was trusted')
        assert len(a.sent) == count
        checks.append('server_added_unverified_device_blocks_delivery_without_plaintext_fallback')
        for stanza in a.sent+b.sent:
            raw = ET.tostring(stanza)
            assert profile['marker'].encode() not in raw and b'private-subject' not in raw
        checks.append('outgoing_wire_envelopes_hide_subject_body')
        return {'result': 'pass', 'checks': checks, 'count': len(checks), 'namespace': NS, 'crypto_library': 'Twomemo 2.1.0', 'product_integration': False, 'independent_client_interop': False}
    finally:
        for p in peers:
            try: await p.close_peer()
            except Exception: pass


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--private-fixture', type=Path, required=True)
    p.add_argument('--output', type=Path, required=True)
    p.add_argument('--disposable-synthetic-fixture', action='store_true', required=True)
    args = p.parse_args()
    logging.basicConfig(level=logging.ERROR)
    result = asyncio.run(run(json.loads(args.private_fixture.read_text())))
    args.output.write_text(json.dumps(result, indent=2)+'\n')
    print('Synthetic OMEMO-2 home checks:', result['count'])


if __name__ == '__main__': main()
