#!/usr/bin/env python3
"""Real local archive-upgrade/catch-up PTYs; generated private state only."""
import argparse,json,platform,secrets,subprocess,tempfile
from pathlib import Path
from smoke_native_client_ssh import NativeConsole


def probe(client, fixture):
    checks=[];children=[]
    with tempfile.TemporaryDirectory(prefix='faction211-archive-ui-') as temp:
        root=Path(temp).resolve();root.chmod(0o700);store=root/'store'
        subprocess.run([str(fixture),str(store),'pending'],check=True,capture_output=True)
        ca=root/'ca.pem';ca.write_text('synthetic public CA placeholder\n');ca.chmod(0o600)
        config=root/'home.toml';config.write_text('version=1\naccount="alice@home.test"\nendpoint="127.0.0.1:9"\ndomain="home.test"\nca='+json.dumps(str(ca))+'\npin="'+'ab'*32+'"\n');config.chmod(0o600)
        password='checkpoint synthetic fixture password';home_password=secrets.token_hex(24)
        def start():
            c=NativeConsole([str(client),'--mail-online',str(store),str(config)]);children.append(c)
            c.expect('Manual archive pages; server completeness is not guaranteed.');c.password(password,'LOCAL checkpoint passphrase (hidden): ');c.expect('[G] Enable archive checkpoints');c.expect('[A] Catch up one archive page');c.expect('[L] Lock and exit');return c
        def pause(c):c.expect('Enter to return: ');c.send('');c.expect('[L] Lock and exit')
        def lock(c):c.raw(b'l');c.finish()
        try:
            original=(store/'mail.fkv').read_bytes();c=start()
            c.raw(b'a');c.expect('Enable archive checkpoints with G before catch-up.');pause(c);lock(c)
            assert (store/'mail.fkv').read_bytes()==original;checks.append('old_v2_catchup_requires_deliberate_local_upgrade_without_network')
            c=start();c.raw(b'g');c.expect('Type upgrade to enable encrypted archive checkpoints: ');c.send('no');c.finish(1)
            assert (store/'mail.fkv').read_bytes()==original;checks.append('declined_schema_upgrade_preserves_canonical_v2_and_restores_terminal')
            c=start();c.raw(b'g');c.expect('Type upgrade to enable encrypted archive checkpoints: ');c.send('upgrade');c.expect('Archive checkpoints enabled; existing messages, keys and pending ciphertext kept.');pause(c);lock(c)
            upgraded=(store/'mail.fkv').read_bytes();assert upgraded!=original;checks.append('explicit_schema_upgrade_is_local_and_preserves_member_unlock')
            c=start();assert b'Archive checkpoints: enabled' in c.transcript;assert b'[N] Send pending' in c.transcript;c.raw(b'g');c.expect('Archive checkpoints already enabled; existing cursor kept.');pause(c);lock(c)
            assert (store/'mail.fkv').read_bytes()==upgraded;checks.append('reopen_and_repeated_enable_keep_existing_epoch_and_cursor')
            c=start();c.raw(b'a');c.password(home_password,'HOME account password (hidden; never sent to a BBS): ')
            c.expect('Catch-up stopped; earlier accepted entries may already be saved.');pause(c)
            c.raw(b'h');c.expect('Subject: synthetic subject \\u{3bb}');c.expect('Enter for next record: ');c.send('');c.expect('Enter for next record: ');c.send('');pause(c);lock(c)
            assert (store/'mail.fkv').read_bytes()==upgraded and home_password.encode() not in c.transcript
            checks.append('unavailable_home_preserves_cursor_pending_ids_and_readable_classic_history')
            c=start();c.raw(b'a');c.expect('HOME account password (hidden; never sent to a BBS): ');c.raw(b'\x03');c.finish(1)
            assert (store/'mail.fkv').read_bytes()==upgraded;checks.append('cancelled_hidden_home_prompt_locks_without_cursor_mutation')
        finally:
            for c in children:c.close()
    return dict(result='pass',platform=platform.system(),count=len(checks),checks=checks,private_fixture_removed=True,provider_contact=False)

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--client',type=Path,required=True);p.add_argument('--fixture',type=Path,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args()
    result=probe(a.client.resolve(),a.fixture.resolve());a.output.write_text(json.dumps(result,indent=2)+'\n');print('Archive local terminal controls passed:',result['count'])
