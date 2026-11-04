#!/usr/bin/env python3
"""Bounded device-local private writing controls. No services or VM activation."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import secrets
import subprocess
import tempfile
import unicodedata

from smoke_native_client_ssh import NativeConsole
from probe_mail_reply import PASS, UNLOCK, MENU, INBOX, ACK, REPLY, SUBJECT, QUEUE, private_root, require_reply_commit
from probe_mail_search import save

WRITER = '[LOCAL MULTILINE WRITER]'
NEXT = '[Enter] Next page  [B] Return'
PROMPT = 'write> '
CHECKS = ['original_selected_body_pages_and_early_back_no_state_change',
    'complete_body_fresh_original_read_decision_and_scoped_return',
    'explicit_new_writer_preview_cancel_and_atomic_exact_queue',
    'received_writer_current_peer_cancel_queue_original_read_unchanged',
    'writer_refuses_unprepared_revoked_expired_history_only_full_pending_full',
    'classic_unicode_bounded_inert_private_output_tty_and_owned_teardown']


def seed_input(subject, peer, body):
    assert subject and '\n' not in subject and '\r' not in subject and len(subject.encode()) <= 128
    assert peer and '\n' not in peer and '\r' not in peer
    assert body and len(body.encode()) <= 1024
    return (subject + '\n' + peer + '\n' + body).encode()


def require_body_pages(pages, expected_rows):
    count = (len(expected_rows) + 9) // 10
    assert len(pages) == count, 'missing or duplicate body page'
    found = []
    for number, page in enumerate(pages, 1):
        assert f'[LOCAL BODY] Page {number}/{count}\r\n' in page, 'page order/count changed'
        assert not any(unicodedata.category(c) in {'Cc', 'Cf'} for c in page.replace('\r\n', '')), 'unsafe raw page scalar'
        lines = page.split('\r\n')
        rows = [line for line in lines if line.startswith('| ')]
        assert 1 <= len(rows) <= 10
        for line in rows:
            assert not any(unicodedata.category(c) in {'Cc', 'Cf'} for c in line), 'unsafe body scalar'
            cells = sum(0 if unicodedata.combining(c) else 2 if unicodedata.east_asian_width(c) in 'WF' else 1 for c in line)
            assert cells <= 39, 'whole body row exceeds39 cells'
        if number < count:
            assert NEXT in page
            trusted = '\n'.join(line for line in lines if not line.startswith('| '))
            assert 'Type read' not in trusted and 'Reply locally' not in trusted and 'Type queue' not in trusted, 'action before complete output'
        found.extend(line[2:] for line in rows)
    assert found == expected_rows, 'body scalar order/content changed'


def _escaped(text, unicode):
    # Probe inputs use only this explicit known scalar set; broader scalar contract is in Rust tests.
    return text if unicode else text.replace('λ', '\\u{3bb}').replace('界', '\\u{754c}')


def probe(client, fixture, owned_root, unicode=False):
    private_root(owned_root)
    root = Path(tempfile.mkdtemp(prefix='writing-local-OWNED-', dir=owned_root)); root.chmod(0o700)
    token = secrets.token_hex(6)
    subject, peer = 'writing-' + token, 'peer-' + secrets.token_hex(6) + '@away.test'
    # Every row fits both modes; explicit final LF requires the final empty row.
    source_body = '\n'.join('row%02d-' % n + token + ' λ界' for n in range(12)) + '\n'
    reply_body = 'first-' + token + '\n\n.done\nlast λ界'
    new_subject = 'new-' + token
    values = [subject, peer, source_body, reply_body, new_subject, token]
    inventory = json.dumps(dict(private_values=values, public_input_provenance=dict(values=[PASS], source_sha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest())), ensure_ascii=False)
    save(root, 'publication-inputs.private.json', inventory)
    encoded = seed_input(subject, peer, source_body); save(root, 'seed-input.private', encoded)
    binding = dict(source_sha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
        binary_sha256={n:hashlib.sha256(p.read_bytes()).hexdigest() for n,p in [('client',client),('fixture',fixture)]},
        seed_input_sha256=hashlib.sha256(encoded).hexdigest(), publication_input_sha256=hashlib.sha256(inventory.encode()).hexdigest())
    save(root, 'input-binding.private.json', json.dumps(binding))
    children = []
    ca = save(root,'unused-ca.pem','synthetic unused CA\n')
    config = save(root,'home.toml','version=1\naccount="alice@home.test"\nendpoint="127.0.0.1:9"\ndomain="home.test"\nca='+json.dumps(str(ca))+'\npin="'+'00'*32+'"\n')
    def call(action,store,mode=None,data=None):
        argv=[str(fixture),action,str(store)]+([mode] if mode else [])
        p=subprocess.run(argv,input=data,capture_output=True,timeout=90)
        save(root,action+'-'+store.name+'.stdout.private',p.stdout);save(root,action+'-'+store.name+'.stderr.private',p.stderr)
        assert p.returncode==0,'synthetic writing fixture failed'
        return json.loads(p.stdout) if action=='inspect' else None
    def seed(name,mode='ready'):
        store=root/name;call('seed',store,mode,encoded);return store
    def start(store,online=True):
        argv=[str(client),'--mail-online',str(store),str(config)] if online else [str(client),'--mail-checkpoints',str(store),'open']
        c=NativeConsole(argv+(['--unicode'] if unicode else []));children.append(c);c.resize(40,24);c.password(PASS,UNLOCK);c.expect(MENU);return c
    def inbox(c):c.raw(b'i');c.expect(INBOX);c.expect('[B] Back')
    def lock(c):c.raw(b'l');c.finish()
    def rows(text):return [_escaped(line,unicode) for line in text.split('\n')]
    def pages(c,expected):
        captured=[];count=(len(expected)+9)//10
        for number in range(1,count+1):
            prior=c.buffer;start=len(c.transcript)
            c.expect(f'[LOCAL BODY] Page {number}/{count}\r\n')
            if number<count:c.expect(NEXT)
            else:c.expect('| '+expected[-1]+'\r\n')
            captured.append((prior+c.transcript[start:]).decode())
            if number<count:c.send('')
        require_body_pages(captured,expected)
    def write(c,reply=False):
        c.raw(b'w');c.expect('CURRENT recipient: '+peer);c.expect('CURRENT device: 202 / ed25519:')
        c.expect(SUBJECT if reply else 'Subject (128 UTF-8 bytes): ');c.send(new_subject);c.expect(WRITER);c.expect(PROMPT)
    def draft(c):
        for line in ['first-'+token,'','..done','last λ界']:
            c.send(line);c.expect('Accepted:');c.expect(PROMPT)
    def finish_write(c,queue):
        c.send('.done');c.expect('Review local multiline intent:');c.expect(QUEUE);c.send('queue' if queue else '');c.expect('Queued durably;' if queue else 'Writing cancelled;')
    try:
        store=seed('reader');before=call('inspect',store);raw=(store/'mail.fkv').read_bytes()
        assert before['history'][0]['body']==source_body,'fixture failed exact multiline/trailingLF roundtrip'
        c=start(store,False);inbox(c);c.raw(b'o');c.expect('[LOCAL BODY] Page 1/2');c.expect(NEXT)
        assert ACK.encode() not in c.transcript;c.raw(b'b');c.expect(INBOX);c.expect('[B] Back');c.raw(b'b');c.expect(MENU);lock(c)
        assert raw==(store/'mail.fkv').read_bytes() and call('inspect',store)==before
        c=start(store,False);inbox(c);c.raw(b'o');pages(c,rows(source_body));c.expect(ACK);c.send('read');c.expect('Acknowledged on this device only');c.expect('Enter to return: ');c.send('');c.expect(INBOX);c.expect('[B] Back');c.raw(b'b');c.expect(MENU);lock(c)
        after=call('inspect',store);assert after['history'][0]['read'] is True and not after['pending']
        # New-message and received-reply cancellation/queue use separate canonical fixtures.
        for reply in [False,True]:
            for queue in [False,True]:
                store=seed('write-'+str(reply)+'-'+str(queue));before=call('inspect',store);raw=(store/'mail.fkv').read_bytes();c=start(store)
                if reply:
                    inbox(c);c.raw(b'o');pages(c,rows(source_body));c.expect(ACK);c.send('');c.expect(REPLY)
                write(c,reply);draft(c)
                c.send('.preview');c.expect('[LOCAL BODY] Page 1/1');c.expect('| '+_escaped('last λ界',unicode)+'\r\n');c.expect(PROMPT)
                finish_write(c,queue)
                if reply:c.expect(INBOX);c.expect('[B] Back');c.raw(b'b')
                c.expect(MENU);lock(c);after=call('inspect',store)
                if queue:require_reply_commit(before,after,peer,202,new_subject,reply_body)
                else:assert raw==(store/'mail.fkv').read_bytes() and after==before
                assert before['history'][0]['read'] is False and after['history'][0]['read'] is False
        for mode in ['unprepared','revoked','expired','history-only','full','pending-full']:
            store=seed('refuse-'+mode,mode);before=(store/'mail.fkv').read_bytes();c=start(store);c.raw(b'w');c.expect('Writing unavailable; check current prepared peer, expiry and retained-store limits.');c.expect(MENU);lock(c);assert before==(store/'mail.fkv').read_bytes()
        result=dict(result='pass',scope='device-local-private-writing',platform=platform.system(),unicode=unicode,count=len(CHECKS),checks=CHECKS,**binding,
            children_reaped=all(c.done for c in children),native_same_ssh='NOT RUN; separate operational approval required')
        save(root,'result.private.json',json.dumps(result,indent=2)+'\n');return result
    finally:
        for n,c in enumerate(children):
            save(root,'terminal-'+str(n)+'.private',c.transcript);c.close()


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--client',type=Path,required=True);p.add_argument('--fixture',type=Path,required=True);p.add_argument('--owned-root',type=Path,required=True);p.add_argument('--unicode',action='store_true');p.add_argument('--output',type=Path)
    a=p.parse_args();result=probe(a.client.resolve(),a.fixture.resolve(),a.owned_root.resolve(),a.unicode)
    if a.output:save(a.output.parent,a.output.name,json.dumps(result,indent=2)+'\n')
    print('Local private writing controls passed:',result['count'],result['platform'])
