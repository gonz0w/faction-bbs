#!/usr/bin/env python3
"""Owned synthetic PTY controls for explicit manual home mail. No real home."""
import argparse,json,os,platform,secrets,subprocess,tempfile,termios,time,select
from pathlib import Path
from smoke_native_client_ssh import NativeConsole
PASS='checkpoint synthetic fixture password' # disposable fixture only

def probe(client,checkpoint_fixture,mail_fixture):
    checks=[];children=[]
    with tempfile.TemporaryDirectory(prefix='faction209-ui-') as tmp:
        root=Path(tmp).resolve();root.chmod(0o700);store=root/'store'
        assert subprocess.run([str(checkpoint_fixture),str(store),'pending'],capture_output=True).returncode==0
        ca=root/'ca.pem';ca.write_text('public placeholder; no connection is possible');ca.chmod(0o600)
        config=root/'home.toml';config.write_text("version=1\naccount='alice@home.test'\nendpoint='127.0.0.1:9'\ndomain='home.test'\nca="+repr(str(ca))+"\npin='"+'12'*32+"'\n");config.chmod(0o600)
        def inspect():
            p=subprocess.run([str(mail_fixture),'inspect',str(store)],input=(PASS+'\n').encode(),capture_output=True)
            assert p.returncode==0 and not p.stderr
            return json.loads(p.stdout)
        before=inspect()
        try:
            c=NativeConsole([str(client),'--mail-online',str(store),str(config)]);children.append(c)
            c.expect('[LOCAL / MANUAL ONLINE]')
            c.password(PASS,'LOCAL checkpoint passphrase (hidden): ');c.expect('[L] Lock and exit')
            assert PASS.encode() not in c.transcript
            checks.append('explicit_local_online_entry_and_hidden_unlock')
            c.raw(b'h')
            for _ in before['history']: c.expect('Enter for next record: ');c.send('')
            c.expect('Enter to return: ');c.send('');c.expect('[L] Lock and exit')
            assert b'synthetic subject \\u{3bb}' in c.transcript
            checks.append('authenticated_history_has_classic_inert_unicode')
            c.raw(b'c');c.expect('Subject (128 UTF-8 bytes): ');c.send('new λ')
            c.expect('Body (1024 UTF-8 bytes, one line): ');c.send('private <&> body')
            c.expect('Type queue to persist encrypted ciphertext: ');c.send('queue')
            c.expect('Queued durably; no delivery or read receipt.');c.expect('Enter to return: ');c.send('');c.expect('[L] Lock and exit')
            checks.append('unicode_compose_queues_encrypted_state_without_network')
            c.raw(b'u');secret=secrets.token_hex(24);c.password(secret,'HOME account password (hidden; never sent to a BBS): ')
            c.expect('Home cycle stopped; pending ciphertext retained.');c.expect('Enter to return: ');c.send('');c.expect('[L] Lock and exit')
            assert secret.encode() not in c.transcript
            checks.append('unavailable_home_and_hidden_credential_keep_pending_state')
            c.raw(b'l');c.finish()
            after=inspect();assert len(after['pending'])==len(before['pending'])+1
            assert all(p in after['pending'] for p in before['pending'])
            assert any(r['subject']=='new λ' and r['body']=='private <&> body' for r in after['history'])
            checks.append('reopen_preserves_exact_existing_ids_and_new_ciphertext')
            c=NativeConsole([str(client),'--mail-online',str(store),str(config),'--unicode']);children.append(c)
            c.password(PASS,'LOCAL checkpoint passphrase (hidden): ');c.expect('[L] Lock and exit');c.raw(b'h')
            for _ in after['history']: c.expect('Enter for next record: ');c.send('')
            c.expect('Enter to return: ');c.send('');c.expect('[L] Lock and exit');assert 'new λ'.encode() in c.transcript;c.raw(b'\x03');c.finish(code=1)
            checks.append('unicode_override_and_local_cancel_restore_terminal')
            c=NativeConsole([str(client),'--mail-online',str(store),str(config)]);children.append(c)
            c.password('deliberately wrong local password','LOCAL checkpoint passphrase (hidden): ');c.finish(code=1)
            assert b'synthetic subject' not in c.transcript and b'private <&> body' not in c.transcript
            checks.append('wrong_local_unlock_never_discloses_history')
            c=NativeConsole([str(client),'--mail-online',str(store),str(config)]);children.append(c)
            c.password(PASS,'LOCAL checkpoint passphrase (hidden): ');c.expect('[L] Lock and exit');c.raw(b'u')
            oversized=secrets.token_hex(65);c.password(oversized,'HOME account password (hidden; never sent to a BBS): ');c.finish(code=1)
            assert oversized.encode() not in c.transcript
            checks.append('oversized_hidden_home_credential_locks_and_flushes_terminal')
            c=NativeConsole([str(client),'--mail-online',str(store),str(config)]);children.append(c)
            c.password(PASS,'LOCAL checkpoint passphrase (hidden): ');c.expect('[L] Lock and exit')
            queued=secrets.token_hex(24);c.raw(b'u'+queued.encode());c.expect('HOME account password (hidden; never sent to a BBS): ')
            assert not termios.tcgetattr(c.fd)[3]&termios.ECHO
            until=time.monotonic()+0.2
            while time.monotonic()<until and select.select([c.fd],[],[],0.02)[0]:
                chunk=os.read(c.fd,8192);c.buffer+=chunk;c.transcript+=chunk
            assert b'Connecting directly' not in c.buffer and queued.encode() not in c.transcript
            c.raw(b'\x03');c.finish(code=1)
            checks.append('typeahead_cannot_supply_hidden_home_credential')

        finally:
            for child in children:child.close()
    return dict(result='pass',platform=platform.system(),count=len(checks),checks=checks,private_fixture_removed=True,provider_contact=False)
if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('--client',type=Path,required=True);p.add_argument('--checkpoint-fixture',type=Path,required=True);p.add_argument('--mail-fixture',type=Path,required=True);p.add_argument('--output',type=Path);a=p.parse_args()
    result=probe(a.client.resolve(),a.checkpoint_fixture.resolve(),a.mail_fixture.resolve());raw=json.dumps(result,indent=2)+'\n'
    if a.output:a.output.write_text(raw)
    print('Manual home-mail PTY fixture passed:',result['count'],'controls;',result['platform'])
