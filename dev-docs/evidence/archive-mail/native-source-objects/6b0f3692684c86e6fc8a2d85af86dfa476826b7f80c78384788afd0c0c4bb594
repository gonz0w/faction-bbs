#!/usr/bin/env python3
"""Rehearse classic-server doors on a fresh marked disposable NetBSD VM only.
Private admin/member argv files and synthetic credentials never enter the output.
The owned TCP fixture runs as the unprivileged build user, not a public service.
"""
import argparse
import json
from pathlib import Path
import secrets
import shlex
import subprocess
import time
from check_restricted_ssh import SSHConsole


def smoke(a):
    assert a.disposable_fixture
    admin=json.loads(a.admin_argv.read_text());member=json.loads(a.member_argv.read_text())
    password=secrets.token_hex(24);children=[];checks=[]
    def remote(cmd,ok=True):
        x=subprocess.run(admin+['set -e; PATH=/usr/sbin:/sbin:/usr/bin:/bin:/usr/pkg/bin; export PATH; '+cmd],capture_output=True,timeout=35)
        assert not ok or x.returncode==0,'private fixture action failed';return x
    def fixture():return json.loads(remote('cat /home/factionbuild/classic-info.json').stdout)
    def passed(name):checks.append(name)
    def menu(c):c.expect('Ctrl-C exits; Esc: Enter to recover.')
    def enter(choice,handle):
        c=SSHConsole(member[:-1]+['-tt',member[-1]]);children.append(c)
        c.expect('[L]ogin [R]egister [Q]uit: ');c.send(choice);c.expect('Handle: ');c.send(handle);c.password(password)
        if choice=='r':c.password(password,'Confirm password (hidden): ')
        return c
    def login(handle):
        c=enter('l',handle);c.expect('Press Enter to continue: ');c.send('');menu(c);return c
    def quit(c):c.raw(b'q');c.expect('Logged out.');c.finish()
    def status(c,action):
        c.raw(b's');c.expect('[C]reate room: ');c.send(action);c.expect('Member handle: ');c.send('alice')
        c.expect('Status updated; previous sessions expired.');c.expect('Press Enter to return to menu: ');c.send('');menu(c)
    def listing(c):c.raw(b'd');c.expect('Classic game servers');c.expect('Ctrl-C returns to the BBS.')
    def confirm(c):c.raw(b'a');c.expect('Never reuse your BBS password.');c.expect('Ctrl-C returns to the BBS.')
    def connect(c):
        listing(c);confirm(c);c.raw(b'c');c.expect('Owned classic link fixture');c.expect('Game> ')
    try:
        remote('test -f /etc/faction-disposable-test; test ! -e /var/db/faction/faction.sqlite3; test ! -S /var/run/faction/terminal.sock')
        remote('su - factionbuild -c '+shlex.quote('nohup /usr/pkg/bin/python3.13 '+a.guest_source+'/tools/fixtures/classic-doors/server.py --info /home/factionbuild/classic-info.json > /home/factionbuild/classic-fixture.log 2>&1 < /dev/null &'))
        deadline=time.monotonic()+5
        while remote('test -f /home/factionbuild/classic-info.json',ok=False).returncode:
            assert time.monotonic()<deadline;time.sleep(.05)
        f=fixture();base=remote('cat /etc/faction/service.toml').stdout.decode()
        remote('cp /etc/faction/service.toml /root/service.before-classics.toml')
        config=base+f'\n[[classic_games]]\nslug="owned-fixture"\ntitle="Owned classic fixture"\naddress="127.0.0.1:{f["port"]}"\n\n[[classic_games]]\nslug="offline"\ntitle="Offline fixture"\naddress="127.0.0.1:{f["refused_port"]}"\n'
        remote('printf %s '+shlex.quote(config)+' > /etc/faction/service.toml')
        c=SSHConsole(admin[:-1]+['-tt',admin[-1],"su - factiond -c '/usr/local/libexec/faction --state-dir /var/db/faction init'"])
        children.append(c);c.expect('Handle: ');c.send('sysop');c.password(password,'New password (12+ characters, hidden): ')
        c.password(password,'Confirm password (hidden): ');c.expect('Lobby created.');c.finish()
        remote('/etc/rc.d/faction_broker onestart; /etc/rc.d/faction_sshd onestart')
        c=enter('r','alice');c.expect('Registration pending; sysop approval required.');c.finish()
        c=enter('l','alice');c.expect('Sysop approval required. No community access.');c.finish()
        assert fixture()['accepted']==0
        sysop=login('sysop');status(sysop,'a');alice=login('alice')
        passed('classic_only_install_pending_members_denied')
        listing(alice);alice.raw(b'\x1b[B');alice.expect('Offline fixture');alice.raw(b'\x1b[A');alice.expect('Owned classic fixture')
        alice.raw(b'\r');alice.expect('Never reuse your BBS password.');alice.expect('Ctrl-C returns to the BBS.')
        assert fixture()['accepted']==0
        alice.raw(b'b');menu(alice);assert fixture()['accepted']==0
        passed('arrows_enter_confirmation_cancel_opens_no_network_connection')
        connect(alice);assert '░▒▓█'.encode() in alice.transcript
        assert b'do-not-copy' not in alice.transcript and b'hidden-payload' not in alice.transcript and b'\x1b[6n' not in alice.transcript
        alice.send('move');alice.expect('You made a move.');alice.expect('Game> ')
        f=fixture();assert f['terminal_ansi']>=1 and f['naws']>=1 and f['queries_received']==0
        alice.raw(b'\x1dignored-menu-typeahead');menu(alice)
        passed('real_telnet_ansi_cp437_keyboard_negotiation_and_local_escape')
        sysop.raw(b'c');sysop.expect('chat[#lobby]> ');alice.raw(b'c');alice.expect('chat[#lobby]> ')
        sysop.send('Welcome back from the classic door.');alice.expect('Welcome back from the classic door.')
        for c in (sysop,alice):c.send('/quit');menu(c)
        passed('local_chat_and_membership_remain_usable_after_remote_door')
        connect(alice);alice.send('q');alice.expect('Game goodbye.');menu(alice);alice.expect('Game server disconnected.')
        passed('remote_eof_returns_to_bbs')
        listing(alice);alice.raw(b'b');alice.expect('Never reuse your BBS password.');alice.expect('Ctrl-C returns to the BBS.')
        alice.raw(b'c');menu(alice);alice.expect('Classic door unavailable.')
        passed('refused_endpoint_returns_to_healthy_bbs')
        connect(alice);alice.send('z');menu(alice);alice.expect('Classic door unavailable.')
        passed('oversized_unterminated_terminal_control_closes_only_the_door')
        connect(alice);status(sysop,'v');alice.expect('Action failed:');alice.finish(1)
        status(sysop,'a');alice=login('alice')
        passed('idle_remote_connection_loses_current_approval')
        connect(alice);alice.resize(60,10);alice.expect('Terminal resized; game closed.');alice.expect('Command (then Enter): ')
        alice.send('d');alice.expect('Choice (then Enter): ');alice.send('a');alice.expect('Choice (then Enter): ')
        alice.send('c');alice.expect('Owned classic link fixture');alice.expect('Game> ')
        assert b'\x1b[' not in alice.buffer
        alice.raw(b'\x1d');alice.expect('Command (then Enter): ')
        alice.send('q');alice.expect('Logged out.');alice.finish();quit(sysop)
        assert fixture()['terminal_dumb']>=1
        passed('resize_closes_game_and_plain_terminal_connects_and_returns')
        # Same build can add Lua alongside classics without changing any limits.
        remote('/etc/rc.d/faction_sshd onestop; /etc/rc.d/faction_broker onestop; cd '+shlex.quote(a.guest_source)+'; /usr/pkg/bin/python3.13 tools/install_lua_doors.py --binary '+shlex.quote(a.guest_binaries+'/faction-doors'))
        remote('/etc/rc.d/faction_doors onestart; /etc/rc.d/faction_broker onestart; /etc/rc.d/faction_sshd onestart')
        alice=login('alice');alice.raw(b'd');alice.expect('After Hours Arcade');alice.expect('Ctrl-C returns to the BBS.')
        alice.raw(b'c');alice.expect('Classic game servers');alice.expect('Ctrl-C returns to the BBS.')
        alice.raw(b'l');alice.expect('After Hours Arcade');alice.expect('Ctrl-C returns to the BBS.')
        alice.raw(b'n');alice.expect('Neon Salvage //');alice.expect('Ctrl-C returns to the BBS.');alice.raw(b'l');alice.expect('Saved 0 points.');alice.expect('Ctrl-C returns to the BBS.')
        alice.raw(b'b');menu(alice);quit(alice)
        passed('lua_scores_and_classic_list_coexist_without_process_limit_changes')
        verified=remote("su - factiond -c '/usr/local/libexec/faction --state-dir /var/db/faction export /home/factiond/classic-check.sqlite3'")
        assert b'1 game score entries' in verified.stdout
        passed('remote_terminal_text_cannot_submit_local_arcade_scores')
        if a.reboot:
            remote("printf 'faction_broker=YES\\nfaction_broker_backend=shared\\n' > /etc/rc.conf.d/faction_broker; printf 'faction_sshd=YES\\n' > /etc/rc.conf.d/faction_sshd; printf 'faction_doors=YES\\n' > /etc/rc.conf.d/faction_doors")
            subprocess.run(admin+['/sbin/shutdown -r now'],capture_output=True,timeout=10);time.sleep(8);deadline=time.monotonic()+90
            while time.monotonic()<deadline:
                try:remote('/etc/rc.d/faction_broker onestatus; /etc/rc.d/faction_sshd onestatus; /etc/rc.d/faction_doors onestatus');break
                except (AssertionError,subprocess.TimeoutExpired):time.sleep(1)
            else:raise AssertionError('ordinary boot did not become ready')
            alice=login('alice');alice.raw(b'd');alice.expect('After Hours Arcade');alice.expect('Ctrl-C returns to the BBS.')
            alice.raw(b'c');alice.expect('Classic game servers');alice.expect('Ctrl-C returns to the BBS.')
            alice.raw(b'q');menu(alice);quit(alice)
            passed('ordinary_unassisted_boot_keeps_optional_classic_configuration')
        for c in children:assert password.encode() not in c.transcript
        passed('no_bbs_password_echoed')
        return {'schema':1,'result':'pass','checks':checks,'reboot_exercised':a.reboot,'actual_classic_title_tested':False}
    finally:
        for c in children:c.close()


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--disposable-fixture',action='store_true')
    p.add_argument('--admin-argv',type=Path,required=True);p.add_argument('--member-argv',type=Path,required=True)
    p.add_argument('--guest-source',required=True);p.add_argument('--guest-binaries',required=True)
    p.add_argument('--output',type=Path,required=True);p.add_argument('--reboot',action='store_true')
    a=p.parse_args();result=smoke(a);a.output.write_text(json.dumps(result,indent=2)+'\n')
    print(f"Classic connector passed {len(result['checks'])} real SSH/fixture checks. No original title support claimed.")
