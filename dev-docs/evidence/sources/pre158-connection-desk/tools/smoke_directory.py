#!/usr/bin/env python3
"""Curated directory local/worker PTYs with a loopback no-connect trap."""
import argparse,json,secrets,socket,sqlite3,subprocess,tempfile
from pathlib import Path
from smoke_local_console import Console
from smoke_member_worker import Worker

FOOTER='Ctrl-C exits; Esc: Enter to recover.'

def catalog(count=6,port=2222):
    source='version=1\n'
    for i in range(count):
        source+=f'''[[boards]]
slug='board-{i+1}'
name='Board {i+1}'
summary='Independent rooms, ANSI art and late-night talk.'
host='{ '127.0.0.1' if i==0 else f'board{i+1}.example' }'
port={port if i==0 else 2222}
login='bbs'
reviewed='2026-10-22'
valid_until='2099-10-22'
'''
    return source

def run(a):
    checks=[];children=[];password=secrets.token_hex(24)
    with tempfile.TemporaryDirectory(prefix='faction-directory-') as tmp,socket.socket() as trap:
        trap.bind(('127.0.0.1',0));trap.listen(1);trap.setblocking(False)
        root=Path(tmp);state=root/'board';pack=root/'pack';pack.mkdir(mode=0o700);source=pack/'boards.toml';source.write_text(catalog(port=trap.getsockname()[1]));source.chmod(0o600)
        def owner(action,handle='sysop',extra=(),columns=80,rows=24):
            c=Console(a.binary,state,action,extra,columns=columns,rows=rows);children.append(c);c.expect('Handle: ');c.send(handle);return c
        def login(mode='ansi',handle='sysop',enabled=True):
            c=owner('login',handle,['--'+('plain' if mode=='plain' else 'ansi'),*(['--directory',str(pack)] if enabled else [])],40 if mode=='compact' else 80,18 if mode=='compact' else 24);c.password(password)
            if mode=='plain':c.expect('Command (then Enter): ')
            elif mode=='compact':c.expect(FOOTER)
            else:c.expect('Press Enter to continue: ');c.send('');c.expect(FOOTER)
            return c
        def logout(c,plain=False):
            if plain:c.send('q')
            else:c.raw(b'q')
            c.expect('Logged out.');c.finish()
        def setup(action,handle):
            c=owner(action,handle);c.password(password,'New password (12+ characters, hidden): ');c.password(password,'Confirm password (hidden): ');c.expect('Lobby created.' if action=='init' else 'Registered pending sysop approval.');c.finish()
        def admin(verb):
            c=owner('login');c.password(password);c.expect('Command (then Enter): ');c.send('s');c.expect('[C]reate room: ');c.send(verb);c.expect('Member handle: ');c.send('member');c.expect('Status updated; previous sessions expired.');c.expect('Command (then Enter): ');logout(c,True)
        try:
            setup('init','sysop');setup('register','member')
            c=login(enabled=False);assert b'Other BBSes' not in c.transcript;logout(c);checks.append('default_directory_disabled')
            c=owner('login','member',['--ansi','--directory',str(pack)]);c.password(password);c.expect('No community access.');c.finish();assert b'OTHER BOARDS' not in c.transcript and b'Board 1' not in c.transcript;checks.append('pending_member_sees_no_directory')
            p=subprocess.run([str(a.binary),'directory-check',str(pack)],capture_output=True,timeout=5);assert p.returncode==0 and b'6 curated listings' in p.stdout;checks.append('offline_catalog_check_makes_no_connection')
            c=login();c.raw(b'o');c.expect('OTHER BOARDS');c.expect('Board 1');c.raw(b'a');c.expect('Connection card 1 / 6');c.expect('ssh -p ');c.expect('Independent login/approval required.');c.raw(b'l');c.expect('Page 1 / 2');c.raw(b'\x1b[B');c.raw(b'\r');c.expect('Connection card 2 / 6');c.raw(b'l');c.expect('Page 1 / 2');c.raw(b'n');c.expect('Page 2 / 2');c.expect('Board 5');c.raw(b'q');c.expect(FOOTER);logout(c);checks.append('highlight_hotkeys_cards_paging_and_main_return')
            c=login(mode='compact');c.raw(b'o');c.expect('OTHER BOARDS');c.raw(b'a');c.expect('Connection card 1 / 6');c.raw(b'n');c.expect('No account, password or device keys');c.raw(b'q');c.expect(FOOTER);logout(c);checks.append('compact_card_pagination_and_classic_menu')
            c=login(mode='plain');c.send('o');c.expect('Directory command: ');c.send('a');c.expect('Connection card 1 / 6');c.expect('Directory command: ');c.send('q');c.expect('Command (then Enter): ');logout(c,True);assert b'\x1b' not in c.transcript;checks.append('plain_cards_and_return_without_ansi')
            c=Worker(a.member,state,['--directory',str(pack)]);children.append(c);c.expect('[L]ogin [R]egister [Q]uit: ');c.send('l');c.expect('Handle: ');c.send('sysop');c.password(password);c.expect('Press Enter to continue: ');c.send('');c.expect(FOOTER);c.raw(b'o');c.expect('OTHER BOARDS');c.raw(b'a');c.expect('Connection card 1 / 6');c.raw(b'q');c.expect(FOOTER);logout(c);checks.append('inherited_terminal_worker_directory')
            c=login();c.raw(b'o');c.expect('OTHER BOARDS');c.resize(40,18);c.expect(FOOTER);logout(c);checks.append('resize_returns_to_working_main_menu')
            admin('a');c=login(handle='member');c.raw(b'o');c.expect('OTHER BOARDS');admin('v');c.raw(b'a');c.finish(1);assert b'Connection card' not in c.transcript;checks.append('revocation_prevents_next_connection_card')
            source.write_text(catalog(port=trap.getsockname()[1]).replace("login='bbs'","login='bbs;sh'"));p=subprocess.run([str(a.binary),'directory-check',str(pack)],capture_output=True,timeout=5);assert p.returncode!=0;checks.append('unsafe_endpoint_tokens_rejected')
            try:conn,_=trap.accept()
            except BlockingIOError:pass
            else:conn.close();raise AssertionError('directory unexpectedly connected')
            checks.append('no_network_connection_to_displayed_loopback_endpoint')
            assert all(password.encode() not in c.transcript for c in children)
        finally:
            for c in children:c.close()
    result=dict(result='pass',count=len(checks),checks=checks,capacity_claim=False,remote_login_claim=False,federation_claim=False)
    if a.output:a.output.write_text(json.dumps(result,indent=2)+'\n')
    print(f'Local/worker directory passed {len(checks)} controls; no automatic connection.')

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('binary',type=Path);p.add_argument('member',type=Path);p.add_argument('--output',type=Path)
    a=p.parse_args();a.binary=a.binary.resolve(strict=True);a.member=a.member.resolve(strict=True);run(a)
