#!/usr/bin/env python3
"""Owned marked NetBSD directory fixture, actual member SSH/native client routes."""
import argparse,json,secrets,shlex,subprocess,time,tomllib
from pathlib import Path
from check_restricted_ssh import SSHConsole
from smoke_native_client_ssh import NativeConsole
from smoke_directory import catalog,FOOTER

PACK='/var/db/faction-directory-fixture'

def run(a):
    assert a.disposable_fixture
    admin=json.loads(a.admin_argv.read_text());member=json.loads(a.member_argv.read_text());password=a.password_file.read_text().strip()
    children=[];checks=[];observations={};calls=0;pending="dir-"+secrets.token_hex(4)
    def remote(command,data=None,ok=True):
        p=subprocess.run(admin+['set -e; PATH=/usr/sbin:/sbin:/usr/bin:/bin:/usr/pkg/bin; export PATH; test -f /etc/faction-disposable-test; '+command],input=data,capture_output=True,timeout=25)
        assert not ok or p.returncode==0,'owned directory fixture action failed (private diagnostics only)'
        return p
    def write(path,data,owner='factiond:factionipc',mode='600'):
        remote('cat > '+shlex.quote(path)+'; chown '+owner+' '+shlex.quote(path)+'; chmod '+mode+' '+shlex.quote(path),data.encode())
    def start(config):
        write('/etc/faction/service.toml',config,'root:factionipc','640');remote('/etc/rc.d/faction_broker onestart')
    def connect(route='ssh',columns=80,choice='l',handle='sysop',plain=False):
        nonlocal calls
        if calls:time.sleep(5.1)
        calls+=1
        c=SSHConsole(member,columns=columns,rows=24 if columns==80 else 18) if route=='ssh' else NativeConsole([str(a.native_binary.resolve()),'--profile',str(a.native_profile.resolve())])
        children.append(c);c.expect('[L]ogin [R]egister [Q]uit: ');c.send(choice);c.expect('Handle: ');c.send(handle)
        if choice=='l':c.password(password)
        if handle=='sysop':
            c.expect('Press Enter to continue: ');c.send('')
            c.expect('Command (then Enter): ' if plain else FOOTER)
        return c
    def quit(c,plain=False):
        c.send('q') if plain else c.raw(b'q');c.expect('Logged out.');c.finish()
    def check_config():return remote("su - factiond -c '/usr/local/libexec/faction-service --check-config /etc/faction/service.toml'",ok=False)
    try:
        original=tomllib.loads(remote('cat /etc/faction/service.toml').stdout.decode());assert original['state_dir']=='/var/db/faction-profile-fixture'
        remote('/etc/rc.d/faction_broker onestop; test ! -e '+PACK+'; install -d -o factiond -g factionipc -m 700 '+PACK)
        base='version=1\nstate_dir="/var/db/faction-profile-fixture"\nsocket="/var/run/faction/terminal.sock"\nterminal_uid='+str(original['terminal_uid'])+'\nansi=true\n'
        start(base);c=connect();assert b'Other BBSes' not in c.transcript;quit(c);checks.append('actual_ssh_default_disabled')
        remote('/etc/rc.d/faction_broker onestop');write(PACK+'/boards.toml',catalog(32));config=base+'directory="'+PACK+'"\n';start(config)
        c=connect();c.raw(b'o');c.expect('Page 1 / 8');c.raw(b'a');c.expect('Connection card 1 / 32');c.expect('ssh -p ');c.raw(b'l');c.expect('Page 1 / 8');c.raw(b'\x1b[B\r');c.expect('Connection card 2 / 32');c.raw(b'l');c.expect('Page 1 / 8');c.raw(b'n');c.expect('Page 2 / 8');c.expect('Board 5');c.raw(b'q');c.expect(FOOTER);quit(c);checks.append('actual_ssh_maximum_catalog_arrows_hotkeys_pages_cards')
        c=connect('native');c.raw(b'o');c.expect('OTHER BOARDS');c.raw(b'a');c.expect('Connection card 1 / 32');c.expect('Independent login/approval required.');c.raw(b'q');c.expect(FOOTER);quit(c);checks.append('actual_native_client_directory_and_return')
        c=connect(columns=40);c.raw(b'o');c.expect('OTHER BOARDS');c.raw(b'a');c.expect('Connection card 1 / 32');c.raw(b'n');c.expect('No account, password or device keys');c.raw(b'q');c.expect(FOOTER);quit(c);checks.append('actual_compact_ssh_paginated_card')
        c=connect();c.raw(b'o');c.expect('OTHER BOARDS');c.resize(40,18);c.expect(FOOTER);quit(c);checks.append('actual_ssh_resize_returns_to_main')
        # Sample the owner with the maximum catalog enabled and one approved member.
        c=connect();c.raw(b'o');c.expect('OTHER BOARDS');samples=[]
        for _ in range(12):
            lines=remote("ps -o rss,vsz,nlwp -p $(cat /var/run/faction/terminal.pid) | awk 'NR==2 {print}'; vmstat -s | awk '/pages free$/{print}'; /sbin/sysctl -n hw.pagesize").stdout.decode().splitlines()
            assert len(lines)==3;rss,vsz,lwps=map(int,lines[0].split());assert lwps==4
            samples.append(dict(owner_rss_kib=rss,owner_vsz_kib=vsz,owner_lwps=lwps,vm_free_bytes=int(lines[1].split()[0])*int(lines[2].split()[0])));time.sleep(.5)
        observations['max_catalog_resource_samples']=samples;c.raw(b'q');c.expect(FOOTER);quit(c);checks.append('maximum_catalog_shared_owner_cost_sampled')
        c=connect(choice='r',handle=pending);c.password(password);c.password(password,'Confirm password (hidden): ');c.expect('Registration pending; sysop approval required.');c.finish()
        c=connect(handle=pending);c.expect('No community access.');c.finish();assert b'Board 1' not in c.transcript and b'OTHER BOARDS' not in c.transcript;checks.append('actual_pending_registration_cannot_open_directory')
        def change_member(verb):
            op=connect();op.raw(b's');op.expect('[C]reate room: ');op.send(verb);op.expect('Member handle: ');op.send(pending);op.expect('Status updated; previous sessions expired.');op.expect('Press Enter to return to menu: ');op.send('');op.expect(FOOTER);quit(op)
        change_member('a');c=connect(handle=pending);c.expect('Press Enter to continue: ');c.send('');c.expect(FOOTER);c.raw(b'o');c.expect('OTHER BOARDS');change_member('v');c.raw(b'a');c.expect('session expired; log in again');c.finish(1);assert b'Connection card' not in c.transcript;checks.append('actual_revocation_prevents_next_card')
        remote('/etc/rc.d/faction_broker onestop');start(config.replace('ansi=true','ansi=false'));c=connect(plain=True);c.send('o');c.expect('Directory command: ');c.send('a');c.expect('Connection card 1 / 32');c.expect('Directory command: ');c.send('q');c.expect('Command (then Enter): ');quit(c,True);assert b'\x1b' not in c.transcript;checks.append('actual_plain_ssh_directory_without_ansi')
        remote('/etc/rc.d/faction_broker onestop');write(PACK+'/boards.toml',catalog(1).replace('2026-10-22','2010-10-22').replace('2099-10-22','2011-10-22'));start(config);c=connect();c.raw(b'o');c.expect('[OLD]');c.raw(b'a');c.expect('Connection card 1 / 1');c.raw(b'q');c.expect(FOOTER);quit(c);checks.append('editorially_expired_listing_marked_old')
        remote('/etc/rc.d/faction_broker onestop');write(PACK+'/boards.toml',catalog(1).replace("login='bbs'","login='bbs;sh'"));assert check_config().returncode!=0
        write(PACK+'/boards.toml',catalog(1));remote('chmod 666 '+PACK+'/boards.toml');assert check_config().returncode!=0;checks.append('bad_endpoint_and_writable_pack_fail_preflight')
        write(PACK+'/boards.toml','version=1\nboards=[]\n');start(config);c=connect();c.raw(b'o');c.expect('No boards listed yet. Ask the sysop.');c.raw(b'q');c.expect(FOOTER);quit(c);checks.append('restart_removal_and_empty_directory_are_usable')
        assert all(password.encode() not in c.transcript for c in children)
        result=dict(result='pass',count=len(checks),checks=checks,observations=observations,normal_sessions=8,admission_refill_ms=5000,catalog_entries=32,capacity_claim=False,automatic_connection_claim=False,remote_login_claim=False,federation_claim=False)
        a.output.write_text(json.dumps(result,indent=2,sort_keys=True)+'\n');a.output.chmod(0o600)
        print(f'Shared SSH/native directory passed {len(checks)} controls.')
    finally:
        for c in children:c.close()
        remote('/etc/rc.d/faction_broker onestop || true')

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--disposable-fixture',action='store_true',required=True)
    for name in ['admin-argv','member-argv','password-file','native-binary','native-profile','output']:p.add_argument('--'+name,type=Path,required=True)
    run(p.parse_args())
