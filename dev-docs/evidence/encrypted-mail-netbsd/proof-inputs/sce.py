"""Bounded SCE reference adapter for the synthetic OMEMO-2 experiment only.

No cryptography here: encryption/ratchets remain in the unchanged MIT Twomemo
implementation. This is not a supported Faction mail wire profile.
"""
import base64
import secrets
from datetime import datetime, timezone
from xml.etree import ElementTree as ET

NS = 'urn:xmpp:sce:1'
CLIENT = 'jabber:client'
LIMIT = 8192


def pack(sender, recipient, subject, body):
    if not subject or len(subject.encode()) > 128 or not body or len(body.encode()) > 1024:
        raise ValueError('bounded subject/body required')
    e = ET.Element('{'+NS+'}envelope')
    content = ET.SubElement(e, '{'+NS+'}content')
    ET.SubElement(content, '{'+CLIENT+'}subject').text = subject
    ET.SubElement(content, '{'+CLIENT+'}body').text = body
    ET.SubElement(e, '{'+NS+'}from', jid=sender)
    ET.SubElement(e, '{'+NS+'}to', jid=recipient)
    ET.SubElement(e, '{'+NS+'}time', stamp=datetime.now(timezone.utc).isoformat())
    pad = ET.SubElement(e, '{'+NS+'}rpad')
    baseline = len(ET.tostring(e, short_empty_elements=False))
    length = max(1, 2048-baseline)+secrets.randbelow(201)
    pad.text = base64.b64encode(secrets.token_bytes((length+2)//3*3))[:length].decode()
    raw = ET.tostring(e, short_empty_elements=False)
    if len(raw) > LIMIT:
        raise ValueError('envelope too large')
    return raw


def unpack(raw, sender, recipient):
    if len(raw) > LIMIT or b'<!' in raw:
        raise ValueError('oversized/unsafe envelope')
    e = ET.fromstring(raw)
    allowed = {'{'+NS+'}'+name for name in ['content', 'from', 'to', 'time', 'rpad']}
    if e.tag != '{'+NS+'}envelope' or len(e) != 5 or {x.tag for x in e} != allowed:
        raise ValueError('unexpected envelope shape')
    if e.find('{'+NS+'}from').get('jid') != sender or e.find('{'+NS+'}to').get('jid') != recipient:
        raise ValueError('wrong authenticated sender/recipient')
    if not e.findtext('{'+NS+'}rpad') or not e.find('{'+NS+'}time').get('stamp'):
        raise ValueError('missing affix')
    content = e.find('{'+NS+'}content')
    tags = {'{'+CLIENT+'}subject', '{'+CLIENT+'}body'}
    if len(content) != 2 or {x.tag for x in content} != tags or any(len(x) for x in content):
        raise ValueError('unexpected/private extension content')
    subject, body = content.findtext('{'+CLIENT+'}subject'), content.findtext('{'+CLIENT+'}body')
    if not subject or len(subject.encode()) > 128 or not body or len(body.encode()) > 1024:
        raise ValueError('bounded subject/body required')
    return subject, body
