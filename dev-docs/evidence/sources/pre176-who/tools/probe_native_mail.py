#!/usr/bin/env python3
"""Disposable trusted-mail PTY/security probe; never writes secrets or transcripts.

Run against a built native faction-client (on developer host or NetBSD). Stdlib
only; no network or real accounts. Output contains only named checks and platform.
"""
import argparse
import json
import platform
import re
import secrets
import tempfile
from pathlib import Path
from smoke_native_client_ssh import NativeConsole


def probe(binary):
    checks=[];children=[]
    password,new_password=secrets.token_hex(24),secrets.token_hex(24)
    body,subject='private-body-'+secrets.token_hex(16),'private-subject-'+secrets.token_hex(8)
    def passed(name):checks.append(name)
    with tempfile.TemporaryDirectory(prefix='faction-native-mail-') as tmp:
        root=Path(tmp).resolve();store=root/'vault';backup=root/'backup'
        def start(path,action):
            c=NativeConsole([str(binary),'--mail-store',str(path),action]);children.append(c);return c
        def menu(c):c.expect('Lock and exit')
        def pause(c):c.expect('Enter to return: ');c.send('')
        def unlock(path=store,secret=password):
            c=start(path,'open');c.password(secret,'LOCAL passphrase (hidden; not a BBS password): ');return c
        def recovery(c):
            c.expect('Save it privately, then Enter to clear this screen: ')
            matches=re.findall(rb'\r\n\r\n([a-f0-9]{64})\r\n',c.transcript)
            assert len(matches)==1,'missing explicit recovery display'
            c.send('');menu(c);return matches[0].decode()
        try:
            c=start(store,'init');c.expect('Your full home identity (local claim only): ');c.send('alice@home.test')
            c.password(password,'New LOCAL passphrase (12-128 characters, hidden): ')
            c.password(password,'Repeat LOCAL passphrase (hidden): ')
            key=recovery(c);assert password.encode() not in c.transcript;passed('hidden_local_unlock_and_recovery_display')
            c.raw(b'd');c.expect('To (full user@home): ');c.send('bob@away.test');c.expect('Subject: ');c.send(subject)
            c.expect('Body (one line, up to 4096 bytes): ');c.send(body)
            c.expect('NOT sent or queued for delivery.');pause(c);menu(c);passed('draft_is_explicitly_local_and_unsent')
            c.raw(b't');c.expect('Member (full user@home): ');c.send('bob@away.test');c.expect('Device ID (1-2147483647): ');c.send('42')
            c.expect('Verified fingerprint (64 lowercase hex): ');c.send('ab'*32)
            c.expect('After independent comparison, type verified: ');c.send('verified');pause(c);menu(c);passed('explicit_full_device_pin_confirmation')
            busy=start(store,'open');busy.expect('Faction client: ');busy.finish(1);passed('concurrent_vault_access_denied')
            c.raw(b'b');c.expect('NEW absolute backup directory (parent must exist): ');c.send(str(backup))
            c.expect('Encrypted backup created.');pause(c);menu(c);c.raw(b'l');c.finish()
            raw=(store/'mail.fkv').read_bytes();assert raw==(backup/'mail.fkv').read_bytes()
            for marker in [body,subject,'alice@home.test','bob@away.test',password,key]:
                assert marker.encode() not in raw,'private marker leaked to vault file'
            assert (store.stat().st_mode&0o777)==0o700 and ((store/'mail.fkv').stat().st_mode&0o777)==0o600
            passed('private_encrypted_backup_contains_no_known_plaintext_markers')
            wrong=unlock(secret='not the local secret');wrong.expect('Faction client: ');wrong.finish(1);passed('wrong_password_denied_without_draft_render')
            c=unlock();menu(c);c.raw(b'\x1b[B\r');c.expect(subject);c.expect(body);pause(c);pause(c);menu(c)
            c.raw(b'l');c.finish();assert password.encode() not in c.transcript;passed('reopen_and_arrow_selected_draft_view')
            c=start(store,'recover');c.password(key,'Offline recovery secret (64 hex, hidden): ')
            c.password(new_password,'New LOCAL passphrase (12-128 characters, hidden): ')
            c.password(new_password,'Repeat LOCAL passphrase (hidden): ');next_key=recovery(c);assert key!=next_key;c.raw(b'l');c.finish()
            assert key.encode() not in c.transcript and new_password.encode() not in c.transcript
            wrong=unlock(secret=password);wrong.expect('Faction client: ');wrong.finish(1)
            wrong=start(store,'recover');wrong.password(key,'Offline recovery secret (64 hex, hidden): ');wrong.expect('Faction client: ');wrong.finish(1)
            passed('member_recovery_rotates_current_password_key_and_recovery_secret')
            c=unlock(backup);menu(c);c.raw(b'l');c.finish();passed('old_backup_keeps_old_member_secret')
            c=unlock(secret=new_password);menu(c);c.raw(b'r');c.expect('bob@away.test / 42 / '+'ab'*32)
            c.expect('Pin number to remove locally: ');c.send('1');c.expect('does not revoke a remote device');pause(c);menu(c)
            c.raw(b'x');c.expect('Draft number to delete: ');c.send('1');c.expect('Backups may retain it.');pause(c);menu(c);c.raw(b'l');c.finish()
            c=unlock(secret=new_password);c.expect('Encrypted local drafts: 0/32 | Device pins: 0/64');menu(c);c.raw(b'l');c.finish();passed('draft_delete_and_local_pin_removal_persist')
            current=(store/'mail.fkv').read_bytes();changed=bytearray(current);changed[-1]^=1;(store/'mail.fkv').write_bytes(changed)
            c=unlock(secret=new_password);c.expect('Faction client: ');c.finish(1);assert body.encode() not in c.transcript
            (store/'mail.fkv').write_bytes(current);passed('tampered_payload_denied_before_render')
            c=unlock(secret=new_password);menu(c);c.raw(b'\x03');c.finish(1);passed('cancel_relocks_and_restores_terminal_echo')
            c=start(store,'open');c.password('x'*129,'LOCAL passphrase (hidden; not a BBS password): ');c.expect('Faction client: ');c.finish(1);passed('oversize_secret_refused_without_truncation')
            return {'result':'pass','platform':platform.system(),'machine':platform.machine(),'checks':checks,'count':len(checks),
                    'network_mail_implemented':False,'roaming_authentication_implemented':False,'encrypted_attachments_implemented':False,
                    'private_fixture_removed_on_exit':True}
        finally:
            for c in children:c.close()


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--client',type=Path,required=True);p.add_argument('--output',type=Path)
    a=p.parse_args();result=probe(a.client.resolve());raw=json.dumps(result,indent=2)+'\n'
    if a.output:a.output.write_text(raw)
    print('Native local-mail PTY probe passed:',result['count'],'checks;',result['platform'],result['machine'])
