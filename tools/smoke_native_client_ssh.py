#!/usr/bin/env python3
"""Real native-client/SSH/SFTP oracle; a marked disposable NetBSD board ONLY.

Private argv/profile paths, generated credentials and failed transcripts stay in
private fixture storage. Sanitized output has names/counts, never endpoints/keys.
"""
import argparse
import fcntl
import hashlib
import json
import os
from pathlib import Path
import pty
import re
import secrets
import shlex
import struct
import subprocess
import termios
from check_restricted_ssh import SSHConsole
from smoke_local_console import Console


class NativeConsole(Console):
    def __init__(self, argv):
        self.pid,self.fd=pty.fork()
        if self.pid==0:
            fcntl.ioctl(0,termios.TIOCSWINSZ,struct.pack('HHHH',24,80,0,0))
            os.environ['TERM']='xterm-256color'
            os.closerange(3,os.sysconf('SC_OPEN_MAX'))
            os.execv(argv[0],argv)
        self.buffer,self.transcript,self.done=b'',b'',False


def smoke(a):
    assert a.disposable_fixture
    admin=json.loads(a.admin_argv.read_text());member=json.loads(a.member_argv.read_text())
    native=json.loads(a.native_argv.read_text())
    wrong=json.loads(a.wrong_host_native_argv.read_text()) if a.wrong_host_native_argv else None
    guest=json.loads(a.guest_native_argv.read_text()) if a.guest_native_argv else None
    password=secrets.token_hex(24);children=[];checks=[]
    private=a.output.parent
    payload=bytes(range(256))*513;digest=hashlib.sha256(payload).hexdigest()
    source=private/'client.bin';source.write_bytes(payload);source.chmod(0o600)
    def remote(cmd,data=None,ok=True):
        r=subprocess.run(admin+['set -e; PATH=/usr/sbin:/sbin:/usr/bin:/bin:/usr/pkg/bin; export PATH; '+cmd],input=data,capture_output=True,timeout=45)
        if ok and r.returncode!=0:
            (private/'native-remote-private.log').write_bytes(r.stdout+r.stderr)
            raise AssertionError('private guest fixture operation failed')
        return r
    def passed(name):checks.append(name)
    def menu(c):c.expect('Ctrl-C exits; Esc: Enter to recover.')
    def files(c):c.raw(b'f');c.expect('files> ')
    def cmd(c,text):
        start=len(c.transcript);c.send(text);c.expect('files> ');return c.transcript[start:]
    def localmenu(c):c.expect('Back to the BBS')
    def done(c):c.expect('Enter to return to Files: ');c.send('');localmenu(c)
    def leavefiles(c):c.raw(b'b');menu(c)
    def enter(choice,handle,is_native=False,argv=None):
        c=NativeConsole(argv or native) if is_native else SSHConsole(member[:-1]+['-tt',member[-1]])
        children.append(c);c.expect('[L]ogin [R]egister [Q]uit: ');c.send(choice)
        c.expect('Handle: ');c.send(handle);c.password(password)
        if choice=='r':c.password(password,'Confirm password (hidden): ')
        return c
    def login(handle,is_native=False,argv=None):
        c=enter('l',handle,is_native,argv);c.expect('Press Enter to continue: ');c.send('');menu(c);return c
    def approve(c,handle):
        c.raw(b's');c.expect('[C]reate room: ');c.send('a');c.expect('Member handle: ');c.send(handle)
        c.expect('Status updated; previous sessions expired.');c.expect('Press Enter to return to menu: ');c.send('');menu(c)
    def nativefiles(c):c.raw(b'f');c.expect('Faction // Files on your device');localmenu(c)
    def upload(c,area,path,name):
        c.raw(b'u');c.expect('Local file to upload: ');c.send(str(path));c.expect('Area ID: ');c.send(str(area))
        c.expect('Remote filename [');c.expect(']: ');c.send(name)
        c.expect('Description (max 256 bytes): ');c.send('Native transfer proof')
        c.expect('[y/N]: ');c.send('y');c.expect('Transfer complete. SHA256 '+digest)
        c.expect('Upload awaits sysop review; it is not published yet.');done(c)
    def download(c,link,path,expected=True):
        c.raw(b'd');c.expect('File ID, faction-file link or recent #number: ');c.send(link)
        c.expect('Save as (new local filename): ');c.send(str(path));c.expect('Download to this local path? [y/N]: ');c.send('y')
        if expected:c.expect('Transfer complete. SHA256 '+digest)
        else:c.expect('Operation stopped: ')
        done(c)
    def object_named(name):
        code="import sqlite3; d=sqlite3.connect('/var/db/faction/faction.sqlite3'); print(d.execute('SELECT id FROM file_objects WHERE filename=?',("+repr(name)+",)).fetchone()[0])"
        return remote('/usr/pkg/bin/python3.13 -c '+shlex.quote(code)).stdout.decode().strip()
    def link(c,object_id):
        out=cmd(c,'link '+object_id)
        return re.findall(rb'faction-file:[0-9a-f]{32}/[0-9a-f]{32}',out)[-1].decode()
    def logout(c):c.raw(b'q');c.expect('Logged out.');c.finish()
    try:
        remote('test -f /etc/faction-disposable-test; test ! -e /var/db/faction/faction.sqlite3; test ! -S /var/run/faction/terminal.sock')
        c=SSHConsole(admin[:-1]+['-tt',admin[-1],"su - factiond -c '/usr/local/libexec/faction --state-dir /var/db/faction init'"])
        children.append(c);c.expect('Handle: ');c.send('sysop');c.password(password,'New password (12+ characters, hidden): ');c.password(password,'Confirm password (hidden): ');c.expect('Lobby created.');c.finish()
        if remote('test -f /etc/rc.d/faction_files_sshd',ok=False).returncode != 0:
            remote('cd '+shlex.quote(a.guest_source)+'; /usr/pkg/bin/python3.13 tools/install_file_areas.py --binary '+shlex.quote(a.guest_binaries+'/faction-files')+' --listen-address 10.0.2.15 --install')
        remote('/etc/rc.d/faction_broker onestart; /etc/rc.d/faction_sshd onestart; /etc/rc.d/faction_files_sshd onestart')
        if wrong:
            denied=NativeConsole(wrong);children.append(denied)
            denied.expect('HOST IDENTIFICATION HAS CHANGED')
            denied.finish(code=1)
            assert b'[L]ogin [R]egister' not in denied.transcript
            passed('native_changed_host_key_denied_before_bbs_with_terminal_restored')
        for handle in ['alice','bobby']:
            c=enter('r',handle,True);c.expect('Registration pending; sysop approval required.');c.finish()
        c=enter('l','alice',True);c.expect('Sysop approval required. No community access.');c.finish()
        assert b'Files on your device' not in c.transcript
        passed('native_login_registration_pending_gate_and_terminal_restoration')
        sysop=login('sysop');approve(sysop,'alice');approve(sysop,'bobby');files(sysop)
        general=int(re.findall(rb'Created file area (\d+)',cmd(sysop,'create general members 1048576 General uploads'))[-1])
        private_area=int(re.findall(rb'Created file area (\d+)',cmd(sysop,'create private private 1048576 Private uploads'))[-1])
        cmd(sysop,f'allow {private_area} alice')
        alice=login('alice',True);nativefiles(alice)
        alice.raw(b'\x1b[B');alice.expect('> [L]');alice.raw(b'\x1b[A');alice.expect('> [A]');alice.raw(b'\r');alice.expect('After area ID [0]: ');alice.send('');alice.expect('General uploads');done(alice)
        passed('native_files_automatic_negotiation_arrows_and_area_listing')
        upload(alice,general,source,'scene.bin');public_id=object_named('scene.bin')
        cmd(sysop,'publish '+public_id);public_link=link(sysop,public_id)
        download(alice,public_link,private/'download.bin');assert (private/'download.bin').read_bytes()==payload
        passed('native_upload_review_and_checksummed_download_without_key_paste_or_second_terminal')
        download(alice,public_link,private/'download.bin',False)
        assert (private/'download.bin').read_bytes()==payload
        download(alice,public_link,private/'download-again.bin');assert (private/'download-again.bin').read_bytes()==payload
        passed('native_no_overwrite_failure_abandons_grant_and_next_transfer_succeeds')
        upload(alice,private_area,source,'private.bin');private_id=object_named('private.bin');cmd(sysop,'publish '+private_id);private_link=link(sysop,private_id)
        leavefiles(alice)
        sysop.send('back');menu(sysop)
        # A room message contains a first-class reference. Core renders per reader.
        sysop.raw(b'p');sysop.expect('Room ID: ');sysop.send('1');sysop.expect('Reply to post ID (blank for new message): ');sysop.send('');sysop.expect('Subject (blank for generated/reply default): ');sysop.send('');sysop.expect('Message (one line, up to 1024 bytes): ');sysop.send(private_link);sysop.expect('Posted #');sysop.expect('Press Enter to return to menu: ');sysop.send('');menu(sysop)
        bob=login('bobby',True)
        for person,allowed in [(alice,True),(bob,False)]:
            start=len(person.transcript);person.raw(b'h');person.expect('Room ID: ');person.send('1');person.expect('More? [y/N]: ')
            output=person.transcript[start:]
            assert (b'private.bin' in output)==allowed
            assert (b'[file unavailable]' in output)!=allowed
            person.send('n');person.expect('Press Enter to return to menu: ');person.send('');menu(person)
        passed('room_file_links_show_metadata_only_to_currently_authorized_readers')
        for person in [sysop,alice,bob]:person.raw(b'c');person.expect('chat[#lobby]> ')
        sysop.send(private_link);alice.expect('private.bin');bob.expect('[file unavailable]')
        for person in [sysop,alice,bob]:person.send('/quit');menu(person)
        nativefiles(alice);download(alice,'#1',private/'recent-link.bin');assert (private/'recent-link.bin').read_bytes()==payload
        passed('chat_scoped_links_and_native_recent_link_download_without_copying_ids')
        files(sysop);cmd(sysop,f'deny {private_area} alice')
        alice.raw(b'd');alice.expect('File ID, faction-file link or recent #number: ');alice.send('#1');alice.expect('Operation stopped: ');done(alice)
        nativefiles(bob);bob.raw(b'i');bob.expect('File ID or faction-file link: ');bob.send(private_link);bob.expect('Operation stopped: ');done(bob)
        passed('native_link_open_rechecks_private_area_and_revoked_access')
        if guest:
            other=login('alice',True,guest);nativefiles(other)
            upload(other,general,Path('/home/factionbuild/native-upload.bin'),'guest-client.bin')
            download(other,public_link,Path('/home/factionbuild/native-download.bin'))
            check="from pathlib import Path; import hashlib; assert hashlib.sha256(Path('/home/factionbuild/native-download.bin').read_bytes()).hexdigest()=="+repr(digest)
            remote('/usr/pkg/bin/python3.13 -c '+shlex.quote(check));leavefiles(other);logout(other)
            passed('native_netbsd_client_upload_download_checksum_and_return_to_bbs')
        for person in [alice,bob]:leavefiles(person);logout(person)
        sysop.send('back');menu(sysop);logout(sysop)
        remote('/etc/rc.d/faction_files_sshd onestop; /etc/rc.d/faction_sshd onestop; /etc/rc.d/faction_broker onestop')
        uid=remote('id -u factiond').stdout.decode().strip()
        fid=remote('id -u factionfiles').stdout.decode().strip()
        base='/usr/pkg/bin/python3.13 '+shlex.quote(a.guest_source+'/tools/file_area_backup.py')
        flags=f' --owner-uid {uid} --files-uid {fid} --bundle /home/factiond/native-file-backup'
        remote(base+' backup'+flags)
        remote(base+' restore'+flags+' --board /home/factiond/native-restored-board --store /var/db/native-restored-files')
        restored="""from pathlib import Path
import sqlite3, hashlib
source=sqlite3.connect('/var/db/faction/faction.sqlite3')
target=sqlite3.connect('/home/factiond/native-restored-board/faction.sqlite3')
assert target.execute('PRAGMA user_version').fetchone()[0]==5
assert source.execute('SELECT id FROM file_namespace').fetchone()==target.execute('SELECT id FROM file_namespace').fetchone()
assert source.execute('SELECT identity FROM board').fetchone()!=target.execute('SELECT identity FROM board').fetchone()
assert source.execute('SELECT body FROM posts ORDER BY id').fetchall()==target.execute('SELECT body FROM posts ORDER BY id').fetchall()
assert source.execute('SELECT body FROM chat_messages ORDER BY channel,sequence').fetchall()==target.execute('SELECT body FROM chat_messages ORDER BY channel,sequence').fetchall()
assert target.execute('SELECT count(*) FROM file_transfers').fetchone()[0]==0
rows=target.execute("SELECT id,sha256 FROM file_objects WHERE state='published'").fetchall()
assert len(rows)>=2
for object_id,digest in rows:
 assert hashlib.sha256(Path('/var/db/native-restored-files',object_id).read_bytes()).hexdigest()==digest
"""
        remote('/usr/pkg/bin/python3.13 -c '+shlex.quote(restored))
        passed('schema5_complete_backup_restore_keeps_public_links_rotates_identity_and_strips_grants')
        return {'result':'pass','checks':checks,'count':len(checks),'payload_bytes':len(payload),'payload_sha256':digest,'wrong_host_key_test':bool(wrong),'guest_native_client':bool(guest),'physical_hardware':False,'capacity_claim':False}
    except BaseException:
        # Caller must keep output.parent private; these contain only synthetic data,
        # but credentials can be echoed by a broken candidate during development.
        for i,c in enumerate(children):(private/f'native-failure-{i}.private').write_bytes(c.transcript+c.buffer)
        raise
    finally:
        for c in children:c.close()


def main():
    p=argparse.ArgumentParser(description=__doc__)
    for n in ['admin-argv','member-argv','native-argv','output']:p.add_argument('--'+n,type=Path,required=True)
    p.add_argument('--guest-native-argv',type=Path)
    p.add_argument('--wrong-host-native-argv',type=Path)
    p.add_argument('--guest-source',default='/home/factionbuild/build');p.add_argument('--guest-binaries',default='/home/factionbuild/target/release')
    p.add_argument('--disposable-fixture',action='store_true',required=True)
    a=p.parse_args();result=smoke(a);a.output.write_text(json.dumps(result,indent=2)+'\n');print('Native-client workflow checks passed:',result['count'])


if __name__=='__main__':main()
