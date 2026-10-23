#!/usr/bin/env python3
"""Actual shared named-room journeys on an owned marked NetBSD fixture."""
import argparse,json,secrets,shlex,subprocess,time,tomllib
from pathlib import Path
from check_restricted_ssh import SSHConsole
from smoke_native_client_ssh import NativeConsole
from smoke_room_browser import MENU,BROWSE,CARD

def run(a):
    assert a.disposable_fixture
    admin=json.loads(a.admin_argv.read_text());member=json.loads(a.member_argv.read_text());password=a.password_file.read_text().strip();guest=json.loads(a.guest_native_argv.read_text())
    children=[];checks=[];observations={};calls=0;pending='reader-'+secrets.token_hex(3)
    def remote(cmd,data=None):
        p=subprocess.run(admin+['set -e; PATH=/usr/sbin:/sbin:/usr/bin:/bin:/usr/pkg/bin; export PATH; test -f /etc/faction-disposable-test; '+cmd],input=data,capture_output=True,timeout=30)
        if p.returncode:(a.output.parent/'browser-remote-private.log').write_bytes(p.stdout+p.stderr)
        assert p.returncode==0,'owned room-browser fixture operation failed';return p.stdout.decode()
    original=remote('cat /etc/faction/service.toml');assert tomllib.loads(original)['state_dir']=='/var/db/faction-profile-fixture'
    def connect(route='ssh',columns=80,handle='sysop',choice='l',plain=False):
        nonlocal calls
        if calls:time.sleep(5.1)
        calls+=1
        c=SSHConsole(member,columns=columns,rows=24 if columns==80 else 18) if route=='ssh' else NativeConsole(guest if route=='guest' else [str(a.native_binary.resolve()),'--profile',str(a.native_profile.resolve())])
        children.append(c);c.expect('[L]ogin [R]egister [Q]uit: ');c.send(choice);c.expect('Handle: ');c.send(handle);c.password(password)
        if handle=='sysop':c.expect('Press Enter to continue: ');c.send('');c.expect('Command (then Enter): ' if plain else MENU)
        return c
    def pause(c):c.expect('Press Enter to return to menu: ');c.send('');c.expect(MENU)
    def logout(c,plain=False):
        c.send('q') if plain else c.raw(b'q');c.expect('Logged out.');c.finish()
    def card(c):c.raw(b'r');c.expect(BROWSE);c.raw(b'b');c.expect('ROOM #2');c.expect(CARD)
    def post(c,body,subject='Night frequency'):
        mark=len(c.transcript);card(c);c.raw(b'p');c.expect('Post in room #2');c.expect('Reply to post ID (blank for new message): ');assert b'Room ID: ' not in c.transcript[mark:];c.send('');c.expect('Subject (blank for generated/reply default): ');c.send(subject);c.expect('Message (one line, up to 1024 bytes): ');c.send(body);c.expect('Posted #');pause(c)
    def config(text):
        remote('/etc/rc.d/faction_broker onestop');remote('cat > /etc/faction/service.toml; chown root:factionipc /etc/faction/service.toml; chmod 640 /etc/faction/service.toml',text.encode());remote('/etc/rc.d/faction_broker onestart')
    try:
        c=connect()
        for i in range(2,101):
            c.raw(b's');c.expect('[C]reate room: ');c.send('c');c.expect('Room slug: ');c.send('radio' if i==2 else 'room-'+str(i));c.expect('Room title: ');c.send('Night Radio' if i==2 else 'Room '+str(i));c.expect(f'Created room #{i}.');pause(c)
        c.raw(b'r');c.expect('Page 1 / 25');c.expect(BROWSE);c.raw(b'\x1b[B\r');c.expect('ROOM #2');c.expect(CARD);c.raw(b'l');c.expect(BROWSE);c.raw(b'p');c.expect('Page 25 / 25');c.expect('#100');c.expect(BROWSE);c.raw(b'd');c.expect('ROOM #100');c.expect(CARD);c.raw(b'q');c.expect(MENU);checks.append('actual_ssh_hundred_rooms_arrows_hotkeys_and_last_page')
        post(c,'A shared-room signal.');checks.append('actual_selected_post_uses_room_without_numeric_prompt')
        c.raw(b'r');c.expect('#2 (1 unread)');c.expect(BROWSE);c.raw(b'u');c.expect('> [B] #2');c.raw(b'\r');c.expect(CARD);c.raw(b'n');c.expect('A shared-room signal.');c.expect('More? [y/N]: ');c.send('n');pause(c);c.raw(b'r');c.expect('#2 (0 unread)');c.expect(BROWSE);c.raw(b'q');c.expect(MENU);logout(c);checks.append('actual_next_unread_selected_reader_and_drained_acknowledgement')
        c=connect('native');c.raw(b't');c.expect('[D]one: ');c.send('u');c.expect('[D]one: ');c.send('d');c.expect(MENU);post(c,'Unicode subject, local room.',subject='Échos 界');card(c);c.raw(b'n');c.expect('Échos 界');c.expect('More? [y/N]: ');c.send('n');pause(c);logout(c);checks.append('actual_host_native_selected_unicode_composer_and_reader')
        c=connect('guest');card(c);c.raw(b'h');c.expect('Room #2 | chronological messages');c.expect('A shared-room signal.');c.expect('More? [y/N]: ');c.send('n');pause(c);logout(c);checks.append('actual_netbsd_native_client_selected_history_and_main_return')
        c=connect();post(c,' '.join('word%03d'%i for i in range(100))+' END-MARKER');logout(c)
        c=connect(columns=40);card(c);c.raw(b'n');c.expect('More? [y/N]: ');c.send('n');pause(c);c.raw(b'r');c.expect('#2 (1 unread)');c.expect(BROWSE);c.raw(b'q');c.expect(MENU);logout(c);checks.append('actual_compact_partial_message_stays_unread')
        config(original.replace('ansi=true','ansi=false'));c=connect(plain=True);c.send('r');c.expect('lobby - Lobby');c.expect('More rooms? [y/N]: ');c.send('n');c.expect('Press Enter to return to menu: ');c.send('');c.expect('Command (then Enter): ');logout(c,True);assert b'\x1b' not in c.transcript;checks.append('actual_plain_ssh_keeps_numeric_listing_and_no_ansi');config(original)
        c=connect();c.raw(b'r');c.expect(BROWSE);c.resize(40,18);c.expect(MENU);logout(c);checks.append('actual_browser_resize_returns_to_main')
        c=connect();c.raw(b'r');c.expect(BROWSE);samples=[]
        for _ in range(8):
            lines=remote("ps -o rss,vsz,nlwp -p $(cat /var/run/faction/terminal.pid) | awk 'NR==2 {print}'; vmstat -s | awk '/pages free$/{print}'; /sbin/sysctl -n hw.pagesize").splitlines();rss,vsz,lwps=map(int,lines[0].split());assert lwps==4;samples.append(dict(owner_rss_kib=rss,owner_vsz_kib=vsz,owner_lwps=lwps,vm_free_bytes=int(lines[1].split()[0])*int(lines[2].split()[0])));time.sleep(.5)
        observations['hundred_room_samples']=samples;c.raw(b'q');c.expect(MENU);logout(c);checks.append('maximum_room_list_one_member_resource_samples')
        c=connect(handle=pending,choice='r');c.password(password,'Confirm password (hidden): ');c.expect('Registration pending; sysop approval required.');c.finish();c=connect(handle=pending);c.expect('No community access.');c.finish();assert b'ROOMS /' not in c.transcript;checks.append('actual_pending_member_cannot_open_browser')
        def status(verb):
            op=connect();op.raw(b's');op.expect('[C]reate room: ');op.send(verb);op.expect('Member handle: ');op.send(pending);op.expect('Status updated; previous sessions expired.');pause(op);logout(op)
        status('a');c=connect(handle=pending);c.expect('Press Enter to continue: ');c.send('');c.expect(MENU);card(c);status('v');mark=len(c.transcript);c.raw(b'p');c.expect('session expired; log in again');c.finish(1);assert b'Reply to post ID' not in c.transcript[mark:];checks.append('actual_revocation_denies_selected_room_composer')
        assert all(password.encode() not in c.transcript for c in children)
        result=dict(result='pass',count=len(checks),checks=checks,observations=observations,room_limit=100,normal_sessions=8,admission_refill_ms=5000,capacity_claim=False,federation_claim=False)
        a.output.write_text(json.dumps(result,indent=2,sort_keys=True)+'\n');a.output.chmod(0o600);print(f'Actual shared SSH/native room browser passed {len(checks)} controls.')
    finally:
        for c in children:c.close()
        remote('/etc/rc.d/faction_broker onestop || true')

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--disposable-fixture',action='store_true',required=True)
    for name in ['admin-argv','member-argv','password-file','native-binary','native-profile','guest-native-argv','output']:p.add_argument('--'+name,type=Path,required=True)
    run(p.parse_args())
