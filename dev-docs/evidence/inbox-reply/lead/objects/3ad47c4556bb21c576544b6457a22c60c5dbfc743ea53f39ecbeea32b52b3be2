#!/usr/bin/env python3
"""Play the Lua arcade over SSH on a marked fresh disposable NetBSD board only.

Uses private trusted argv JSONs like smoke_small_board_ssh.py. Optional door
installer must be available at --guest-source; built binaries at --guest-binaries.
Creates synthetic credentials in memory only. No transcript/endpoints published.
"""
import argparse
import json
from pathlib import Path
import secrets
import re
import shlex
import subprocess
import time
from check_restricted_ssh import SSHConsole


def smoke(a):
    assert a.disposable_fixture
    admin=json.loads(a.admin_argv.read_text());member=json.loads(a.member_argv.read_text())
    password=secrets.token_hex(24);children=[];checks=[]
    def remote(cmd,ok=True):
        r=subprocess.run(admin+['set -e; PATH=/usr/sbin:/sbin:/usr/bin:/bin:/usr/pkg/bin; export PATH; '+cmd],capture_output=True,timeout=35)
        assert not ok or r.returncode==0,'private guest action failed'
        return r
    def passed(name):checks.append(name)
    def owner_init():
        c=SSHConsole(admin[:-1]+['-tt',admin[-1],"su - factiond -c '/usr/local/libexec/faction --state-dir /var/db/faction init'"])
        children.append(c);c.expect('Handle: ');c.send('sysop')
        c.password(password,'New password (12+ characters, hidden): ');c.password(password,'Confirm password (hidden): ')
        c.expect('Lobby created.');c.finish()
    def enter(choice,handle):
        c=SSHConsole(member[:-1]+['-tt',member[-1]]);children.append(c)
        c.expect('[L]ogin [R]egister [Q]uit: ');c.send(choice);c.expect('Handle: ');c.send(handle);c.password(password)
        if choice=='r':c.password(password,'Confirm password (hidden): ')
        return c
    def menu(c):c.expect('Ctrl-C exits; Esc: Enter to recover.')
    def login(handle):
        c=enter('l',handle);c.expect('Press Enter to continue: ');c.send('');menu(c);return c
    def pause(c):c.expect('Press Enter to return to menu: ');c.send('');menu(c)
    def status(c,action):
        c.raw(b's');c.expect('[C]reate room: ');c.send(action);c.expect('Member handle: ');c.send('alice')
        c.expect('Status updated; previous sessions expired.');pause(c)
    def arcade(c):c.raw(b'd');c.expect('After Hours Arcade');c.expect('Ctrl-C returns to the BBS.')
    def quit(c):c.raw(b'q');c.expect('Logged out.');c.finish()
    try:
        remote('test -f /etc/faction-disposable-test; test ! -e /var/db/faction/faction.sqlite3; test ! -S /var/run/faction/terminal.sock')
        owner_init()
        if remote('test -f /etc/rc.d/faction_doors',ok=False).returncode != 0:
            remote('cd '+shlex.quote(a.guest_source)+'; /usr/pkg/bin/python3.13 tools/install_lua_doors.py --binary '+shlex.quote(a.guest_binaries+'/faction-doors'))
        remote('test "$(cat /etc/rc.conf.d/faction_doors)" = faction_doors=NO; test ! -S /var/run/faction-doors/door.sock; '
               'su -m factiongames -c "test ! -r /var/db/faction/faction.sqlite3"; '
               'test ! -u /usr/local/libexec/faction-doors; /etc/rc.d/faction_doors onestart; '
               '/etc/rc.d/faction_broker onestart; /etc/rc.d/faction_sshd onestart')
        passed('optional_install_disabled_separate_uid_cannot_read_board')
        c=enter('r','alice');c.expect('Registration pending; sysop approval required.');c.finish()
        c=enter('l','alice');c.expect('Sysop approval required. No community access.');c.finish()
        assert b'Arcade' not in c.transcript
        sysop=login('sysop');status(sysop,'a');alice=login('alice')
        passed('pending_member_cannot_enter_games_approved_member_can')
        arcade(alice)
        alice.raw(b'i');alice.expect('After Hours // arcade guide');alice.expect('Ctrl-C returns to the BBS.')
        assert b'seven guesses' in alice.transcript and b'Five sectors' in alice.transcript
        alice.raw(b'l');alice.expect('After Hours Arcade');alice.expect('Ctrl-C returns to the BBS.')
        passed('installed_arcade_guide_and_return_to_lobby')
        alice.raw(b'\x1b[B');alice.expect('Neon Salvage')
        alice.raw(b'\x1b[A');alice.expect('Signal Hunt')
        alice.raw(b'\r');alice.expect('Frequency (1-99), or Q to leave: ')
        alice.send('0');alice.expect('Tune a whole frequency from 1 to 99.');alice.expect('Frequency (1-99), or Q to leave: ')
        alice.send('q');alice.expect('The signal is still out there.');alice.expect('Ctrl-C returns to the BBS.')
        passed('lua_lobby_arrows_enter_signal_hunt_numeric_input_and_exit')
        alice.raw(b'l');alice.expect('After Hours Arcade');alice.expect('Ctrl-C returns to the BBS.')
        alice.raw(b'n');alice.expect('Neon Salvage //');alice.expect('Ctrl-C returns to the BBS.')
        for _ in range(5):
            alice.raw(b's');alice.expect('Recovered ');alice.expect('Ctrl-C returns to the BBS.')
        assert b'You dock before dawn' in alice.transcript
        score=int(re.findall(rb'Saved (\d+) points',alice.transcript)[-1]);assert 5<=score<=100
        assert b'Completed runs: 1.' in alice.transcript
        for _ in range(2):
            alice.raw(b't');alice.expect('High scores // neon-salvage')
            alice.expect(f'Your best: {score} points from 1 completed runs.');alice.expect('Ctrl-C returns to the BBS.')
            alice.raw(b'r');alice.expect('You dock before dawn');alice.expect('Ctrl-C returns to the BBS.')
        passed('persistent_best_and_repeated_score_browsing_does_not_resubmit')
        alice.raw(b'p');alice.expect('Neon Salvage //');alice.expect('Ctrl-C returns to the BBS.')
        alice.raw(b'l');alice.expect('Saved 0 points.');alice.expect('Ctrl-C returns to the BBS.')
        assert b'Completed runs: 2.' in alice.transcript
        alice.raw(b't');alice.expect(f'Your best: {score} points from 2 completed runs.');alice.expect('Ctrl-C returns to the BBS.')
        alice.raw(b'b');menu(alice)
        passed('play_again_early_landing_retains_best_and_counts_one_new_run')
        quit(alice);alice=login('alice');arcade(alice)
        alice.raw(b't');alice.expect(f'Your best: {score} points from 2 completed runs.');alice.expect('Ctrl-C returns to the BBS.')
        alice.raw(b'b');menu(alice)
        passed('leaderboard_survives_disconnect_and_new_login')
        passed('neon_salvage_five_turn_run_score_and_return_to_bbs')
        for c in (alice,sysop):c.raw(b'c');c.expect('chat[#lobby]> ')
        sysop.send('Games over; welcome back to the board.');alice.expect('Games over; welcome back to the board.')
        for c in (alice,sysop):c.send('/quit');menu(c)
        passed('peer_chat_works_after_playing_both_games')
        remote('/etc/rc.d/faction_doors onestop')
        alice.raw(b'd');menu(alice);alice.expect('That door is unavailable.')
        sysop.raw(b'r');sysop.expect('lobby - Lobby');sysop.expect('N/P:page U:next unread Q:main');sysop.raw(b'q');menu(sysop)
        remote('/etc/rc.d/faction_doors onestart');arcade(alice)
        passed('door_service_loss_returns_to_healthy_board_and_recovers')
        status(sysop,'v');alice.raw(b'h');alice.expect('Action failed:');alice.finish(1)
        status(sysop,'a');alice=login('alice');arcade(alice)
        alice.raw(b'\x03');menu(alice)
        passed('revocation_blocks_game_action_and_fresh_approval_resets_session')
        # A deliberately broken sysop script must discard output and leave the BBS.
        remote("cp /usr/local/share/faction/doors/doors.lua /root/doors-good.lua; printf 'bbs.title(\"partial\"); while true do end\\n' > /usr/local/share/faction/doors/doors.lua")
        alice.raw(b'd');menu(alice);alice.expect('That door is unavailable.')
        remote('install -o root -g wheel -m 644 /root/doors-good.lua /usr/local/share/faction/doors/doors.lua')
        arcade(alice);alice.raw(b'\x03');menu(alice)
        passed('broken_lua_output_is_discarded_and_next_game_menu_works')
        arcade(alice);alice.resize(60,10)
        alice.expect('Terminal resized; game closed.');alice.expect('Command (then Enter): ')
        alice.send('d');alice.expect('After Hours Arcade');alice.expect('Choice (then Enter): ')
        alice.send('');alice.expect('Choice (then Enter): ')
        alice.send('??');alice.expect('Choice (then Enter): ')
        alice.send('h');alice.expect('Frequency (1-99), or Q to leave: ')
        alice.send('q');alice.expect('Choice (then Enter): ');alice.send('b')
        alice.expect('Command (then Enter): ');alice.send('q');alice.expect('Logged out.');alice.finish()
        quit(sysop)
        export=remote("su - factiond -c '/usr/local/libexec/faction --state-dir /var/db/faction export /home/factiond/arcade-export.sqlite3'")
        assert b'1 game score entries' in export.stdout
        verified=remote("su - factiond -c '/usr/local/libexec/faction verify-export /home/factiond/arcade-export.sqlite3'")
        assert b'1 game score entries' in verified.stdout
        restored=remote("su - factiond -c '/usr/local/libexec/faction --state-dir /home/factiond/arcade-restored restore /home/factiond/arcade-export.sqlite3'")
        assert b'1 game score entries' in restored.stdout
        passed('owner_export_verify_restore_includes_local_scores')
        passed('resize_cancels_game_and_plain_terminal_can_play_and_return')
        if a.reboot:
            remote("printf 'faction_doors=YES\\n' > /etc/rc.conf.d/faction_doors; printf 'faction_broker=YES\\nfaction_broker_backend=shared\\n' > /etc/rc.conf.d/faction_broker; printf 'faction_sshd=YES\\n' > /etc/rc.conf.d/faction_sshd")
            subprocess.run(admin+['/sbin/shutdown -r now'],capture_output=True,timeout=10)
            time.sleep(8);deadline=time.monotonic()+90
            while time.monotonic()<deadline:
                try:
                    remote('/etc/rc.d/faction_doors onestatus; /etc/rc.d/faction_broker onestatus; /etc/rc.d/faction_sshd onestatus');break
                except (AssertionError,subprocess.TimeoutExpired):time.sleep(1)
            else:raise AssertionError('ordinary arcade boot did not remain ready')
            alice=login('alice');arcade(alice)
            alice.raw(b't');alice.expect(f'Your best: {score} points from 2 completed runs.');alice.expect('Ctrl-C returns to the BBS.')
            alice.raw(b'b');menu(alice);quit(alice)
            passed('score_and_play_count_survive_unassisted_reboot')
            passed('unassisted_boot_starts_optional_doors_and_approved_arcade')
        for c in children:assert password.encode() not in c.transcript
        passed('no_bbs_password_echoed')
        return {'schema':1,'result':'pass','checks':checks,'reboot_exercised':a.reboot}
    finally:
        for c in children:c.close()


def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--disposable-fixture',action='store_true')
    p.add_argument('--admin-argv',type=Path,required=True);p.add_argument('--member-argv',type=Path,required=True)
    p.add_argument('--guest-source',required=True);p.add_argument('--guest-binaries',required=True)
    p.add_argument('--output',type=Path,required=True);p.add_argument('--reboot',action='store_true')
    a=p.parse_args();result=smoke(a);a.output.write_text(json.dumps(result,indent=2)+'\n')
    print(f"Lua arcade passed {len(result['checks'])} real SSH/setup/play/recovery checks.")


if __name__=='__main__':main()
