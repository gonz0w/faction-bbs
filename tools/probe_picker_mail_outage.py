#!/usr/bin/env python3
"""Same-process local mail outage -> approved BBS -> restricted SFTP proof.

Requires an explicitly marked disposable board and owner-private settings. All
secrets and raw transcripts remain in a generated private fixture root. The caller
owns final teardown; only bounded checks/counts are exported. No real mail home.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import re
import secrets
import shlex
import socket
import stat
import subprocess
import tempfile
import termios

from check_restricted_ssh import SSHConsole
from smoke_board_launcher import LauncherConsole, PICK, CONFIRM, RETURN

CHECKS = [
    'independent_board_registration_pending_and_explicit_approval',
    'real_encrypted_nonempty_cursor_history_and_pending_seed',
    'actual_closed_home_tcp_connection_failure',
    'same_picker_send_and_archive_outage_preserve_exact_state',
    'locked_picker_releases_store_and_requires_hidden_unlock',
    'same_picker_approved_board_restricted_sftp_upload_review_download',
    'same_picker_board_return_and_terminal_restoration',
    'private_mail_absent_from_visited_board_transcript_database_store_and_logs',
]
PASS = 'checkpoint synthetic fixture password'
UNLOCK = 'LOCAL checkpoint passphrase (hidden): '
HOME = 'HOME account password (hidden; never sent to a BBS): '
MAIL = '[L] Lock and exit'


def private_json(path):
    info=path.lstat()
    assert stat.S_ISREG(info.st_mode) and info.st_uid==os.getuid() and not info.st_mode & 0o077 and info.st_nlink==1
    return json.loads(path.read_text())


def assert_unchanged(before, after):
    assert before == after, 'mail state changed across unavailable home or board/files journey'


def require_connection_refused(code, out, err):
    assert code==1 and out==b'' and err==b'Transport(Connect)\n', 'expected actual unavailable TCP home, not local configuration/TLS refusal'


def require_closed_endpoint(endpoint):
    try:
        connection=socket.create_connection(endpoint,timeout=1)
    except ConnectionRefusedError:
        return
    except OSError as error:
        raise AssertionError('closed home endpoint must refuse TCP, not time out') from error
    else:
        connection.close()
        raise AssertionError('closed home endpoint unexpectedly accepted a connection')


def validate_scan(scan):
    assert set(scan)=={'board_files','store_files','log_files','matches','symlinks'}
    assert all(type(v) is int for v in scan.values())
    assert scan['board_files']>0 and scan['store_files']>0 and scan['log_files']>0
    assert scan['matches']==0 and scan['symlinks']==0, 'private mail reached board state/logs or unsafe scan'
    return scan


SCAN = r'''import json,sys,stat
from pathlib import Path
s=json.load(sys.stdin)
assert Path('/etc/faction-disposable-test').is_file()
needles=[x.encode() for x in s['markers']]
result=dict(board_files=0,store_files=0,log_files=0,matches=0,symlinks=0)
for category in ['board','store','log']:
 for name in s[category]:
  root=Path(name)
  assert root.exists()
  paths=[root]+(list(root.rglob('*')) if root.is_dir() else [])
  for p in paths:
   mode=p.lstat().st_mode
   if stat.S_ISLNK(mode):result['symlinks']+=1
   elif stat.S_ISREG(mode):
    result[category+'_files']+=1
    data=p.read_bytes()
    result['matches']+=sum(v in data for v in needles)
print(json.dumps(result))
'''


def client_arguments(args, catalog):
    """Keep provider selection local and exercise the same production picker."""
    command = [args.client, '--boards', catalog]
    selected = getattr(args, 'device_provider', None)
    if selected is not None:
        command.extend(['--device-provider', selected])
    return command


def probe(a):
    root_info=a.private_root.lstat()
    assert stat.S_ISDIR(root_info.st_mode) and root_info.st_uid==os.getuid() and not root_info.st_mode & 0o077
    assert (a.private_root/'OWNED').is_file()
    settings=private_json(a.settings)
    assert settings['disposable_fixture'] is True
    root=Path(tempfile.mkdtemp(prefix='outage-',dir=a.private_root)).resolve();root.chmod(0o700)
    children=[];checks=[];home_password=secrets.token_hex(24);password=secrets.token_hex(24)
    subject='private-mail-subject-'+secrets.token_hex(12);body='private-mail-body-'+secrets.token_hex(16)
    marker_values=[PASS,home_password,subject,body,'synthetic private archive id','synthetic-private-archive-id']
    def private(path,data):
        path.write_text(data);path.chmod(0o600)
    def command(action,*args,extra=(),ok=True):
        r=subprocess.run([str(a.mail_fixture),action,str(store),*map(str,args)],input=('\n'.join((PASS,*extra))+'\n').encode(),capture_output=True,timeout=30)
        if ok:assert r.returncode==0,'private mail fixture action failed'
        return r
    def snapshot():
        inspection=json.loads(command('inspect').stdout)
        return dict(canonical=(store/'mail.fkv').read_bytes(),history=inspection['history'],pending=inspection['pending'],archive=json.loads(command('archive-status').stdout))
    def remote(code,data=None):
        r=subprocess.run(settings['admin_argv']+['/usr/pkg/bin/python3.13 -c '+shlex.quote(code)],input=data,capture_output=True,timeout=45)
        if r.returncode:(root/'admin-failure.private').write_bytes(r.stdout+r.stderr)
        assert r.returncode==0,'owned board operation failed'
        return r.stdout
    def board_menu(c):c.expect('Ctrl-C exits; Esc: Enter to recover.')
    def login(c,handle,secret,choice='l'):
        c.expect('[L]ogin [R]egister [Q]uit: ');c.send(choice);c.expect('Handle: ');c.send(handle);c.password(secret)
        if choice=='r':c.password(secret,'Confirm password (hidden): ')
        else:c.expect('Press Enter to continue: ');c.send('');board_menu(c)
    def board():
        c=SSHConsole(settings['member_argv']);children.append(c);return c
    def files_command(c,text):
        mark=len(c.transcript);c.send(text);c.expect('files> ');return c.transcript[mark:]
    def file_done(c):c.expect('Enter to return to Files: ');c.send('');c.expect('Back to the BBS')
    def pause_mail(c):c.expect('Enter to return: ');c.send('');c.expect(MAIL)
    def unlock(c):c.raw(b'e');c.password(PASS,UNLOCK);c.expect(MAIL)
    def state_check():assert_unchanged(before,snapshot())
    try:
        # Select an ephemeral port, then close it: NetBSD can time out while
        # a non-listening TCP socket remains bound. Verify actual refusal.
        with socket.socket() as outage:
            outage.bind(('127.0.0.1',0));closed_endpoint=outage.getsockname();outage.close()
            require_closed_endpoint(closed_endpoint)
            endpoint='127.0.0.1:'+str(closed_endpoint[1])
            store=root/'mail';r=subprocess.run([str(a.checkpoint_fixture),str(store),'archive-pending'],capture_output=True,timeout=30)
            assert r.returncode==0,'archive seed failed'
            command('queue',extra=(subject,body))
            ca=root/'public-ca.pem';ca.write_bytes(a.public_ca.read_bytes());ca.chmod(0o600)
            config=root/'home.toml';private(config,'version=1\naccount="alice@home.test"\nendpoint='+json.dumps(endpoint)+'\ndomain="home.test"\nca='+json.dumps(str(ca))+'\npin="'+'ab'*32+'"\n')
            catalog=root/'boards.toml';private(catalog,'version=1\nhome_mail={store='+json.dumps(str(store))+',config='+json.dumps(str(config))+'}\n[[boards]]\nid="approved"\nlabel="Owned approved board"\nprofile='+json.dumps(settings['profile'])+'\n')
            before=snapshot();assert before['archive']['enabled'] and before['archive']['ready'] and before['archive']['cursor_present']
            assert before['archive']['checkpoint']['after'] and len(before['history'])==3 and len(before['pending'])==2
            assert before['pending'][0]['operation']=='01'*16
            assert all(v.encode() not in before['canonical'] for v in marker_values)
            # Classify the real adapter failure before the UI proof, preserving state.
            failure=command('sync',config,extra=(home_password,),ok=False)
            require_connection_refused(failure.returncode,failure.stdout,failure.stderr);state_check()
            handle=settings['member_handle'];c=board();login(c,handle,password,'r');c.expect('Registration pending; sysop approval required.');c.finish()
            c=board();c.expect('[L]ogin [R]egister [Q]uit: ');c.send('l');c.expect('Handle: ');c.send(handle);c.password(password);c.expect('Sysop approval required. No community access.');c.finish()
            sysop=board();login(sysop,'sysop',settings['sysop_password']);sysop.raw(b's');sysop.expect('[C]reate room: ');sysop.send('a');sysop.expect('Member handle: ');sysop.send(handle);sysop.expect('Status updated; previous sessions expired.');sysop.expect('Press Enter to return to menu: ');sysop.send('');board_menu(sysop)
            sysop.raw(b'f');sysop.expect('files> ')
            output=files_command(sysop,'create '+handle+' members 1048576 Public synthetic transfer payload')
            area=int(re.findall(rb'Created file area (\d+)',output)[-1])
            checks.extend(CHECKS[:3])
            c=LauncherConsole(client_arguments(a,catalog));children.append(c);picker_pid=c.pid;c.expect(PICK);unlock(c)
            for action,message in [(b'n','Home cycle stopped; pending ciphertext retained.'),(b'a','Catch-up stopped; earlier accepted entries may already be saved.')]:
                c.raw(action);c.password(home_password,HOME);c.expect(message);pause_mail(c)
            c.raw(b'h');c.expect('Authenticated history: 3/64')
            for _ in range(3):c.expect('Enter for next record: ');c.send('')
            pause_mail(c);assert subject.encode() in c.transcript and body.encode() in c.transcript
            c.raw(b'o');c.expect('Durable pending ciphertext; delivery remains unproven.')
            for pending in before['pending']:c.expect('Operation: '+pending['operation'])
            pause_mail(c)
            c.raw(b'l');c.expect(PICK);state_check();assert os.waitpid(picker_pid,os.WNOHANG)==(0,0)
            checks.append(CHECKS[3])
            # Snapshot reopening succeeds only once the native vault handle is gone.
            unlock(c);c.raw(b'l');c.expect(PICK);state_check();checks.append(CHECKS[4])
            board_start=len(c.transcript);c.raw(b'a');c.expect(CONFIRM);c.send('y');login(c,handle,password)
            c.raw(b'f');c.expect('Faction // Files on your device');c.expect('Back to the BBS')
            payload=bytes(range(256))*513;digest=hashlib.sha256(payload).hexdigest();source=root/'public-transfer.bin';source.write_bytes(payload);source.chmod(0o600)
            filename=handle+'-public.bin'
            c.raw(b'u');c.expect('Local file to upload: ');c.send(str(source));c.expect('Area ID: ');c.send(str(area));c.expect('Remote filename [');c.expect(']: ');c.send(filename);c.expect('Description (max 256 bytes): ');c.send('Public synthetic board file, separate from private mail');c.expect('[y/N]: ');c.send('y');c.expect('Transfer complete. SHA256 '+digest);c.expect('Upload awaits sysop review; it is not published yet.');file_done(c)
            code="import json,sqlite3,sys; s=json.load(sys.stdin); d=sqlite3.connect(s['database']); r=d.execute('SELECT id,state FROM file_objects WHERE filename=?',(s['filename'],)).fetchone(); print(json.dumps(r))"
            row=json.loads(remote(code,json.dumps(dict(database=settings['database'],filename=filename)).encode()));assert row[1]=='pending'
            files_command(sysop,'publish '+row[0]);link_output=files_command(sysop,'link '+row[0]);link=re.findall(rb'faction-file:[0-9a-f]{32}/[0-9a-f]{32}',link_output)[-1].decode()
            target=root/'download.bin';c.raw(b'd');c.expect('File ID, faction-file link or recent #number: ');c.send(link);c.expect('Save as (new local filename): ');c.send(str(target));c.expect('Download to this local path? [y/N]: ');c.send('y');c.expect('Transfer complete. SHA256 '+digest);file_done(c);assert target.read_bytes()==payload
            c.raw(b'b');board_menu(c);c.raw(b'q');c.expect('Logged out.');c.expect(RETURN);c.send('');c.expect(PICK);assert c.pid==picker_pid and os.waitpid(picker_pid,os.WNOHANG)==(0,0);state_check()
            checks.append(CHECKS[5]);board_segment=c.transcript[board_start:]
            c.raw(b'q');c.finish();assert termios.tcgetattr(c.fd)[3] & termios.ECHO;checks.append(CHECKS[6])
            sysop.send('back');board_menu(sysop);sysop.raw(b'q');sysop.expect('Logged out.');sysop.finish()
            assert all(v.encode() not in board_segment for v in marker_values)
            assert all(home_password.encode() not in child.transcript and PASS.encode() not in child.transcript for child in children)
            scan=validate_scan(json.loads(remote(SCAN,json.dumps(dict(markers=marker_values,board=settings['scan_board'],store=settings['scan_store'],log=settings['scan_logs'])).encode())))
            checks.append(CHECKS[7]);state_check();require_closed_endpoint(closed_endpoint)
            result=dict(result='pass',platform=platform.system(),scope='same-picker-unavailable-home-approved-board-restricted-sftp',count=len(checks),checks=checks,same_picker_pid=True,canonical_unchanged=True,history_unchanged=True,pending_ids_unchanged=True,archive_token_unchanged=True,nonempty_cursor=True,approved_member=True,restricted_sftp=True,payload_bytes=len(payload),payload_sha256=digest,privacy_scan=scan,home_transport_error='Connect',physical_hardware=False,capacity_claim=False,private_fixture_retained_for_lead=True)
            private(root/'private-inputs.json',json.dumps(dict(home_password=home_password,subject=subject,body=body,store=str(store))))
            return result
    except BaseException:
        for i,c in enumerate(children):(root/('failure-'+str(i)+'.private')).write_bytes(c.transcript+c.buffer)
        raise
    finally:
        for c in children:c.close()


def main():
    p=argparse.ArgumentParser(description=__doc__)
    for name in ['client','checkpoint-fixture','mail-fixture','public-ca','settings','private-root','output']:p.add_argument('--'+name,type=Path,required=True)
    p.add_argument('--device-provider', type=Path, help='optional local selection passed unchanged to the production picker')
    a=p.parse_args()
    for name in ['client','checkpoint_fixture','mail_fixture','public_ca','settings','private_root','output']:setattr(a,name,getattr(a,name).resolve())
    result=probe(a);a.output.write_text(json.dumps(result,indent=2)+'\n');print('Combined picker outage journey passed:',result['count'])

if __name__=='__main__':main()
