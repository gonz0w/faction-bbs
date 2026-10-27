#!/usr/bin/env python3
"""Actual main-menu -> local encrypted mail -> same SSH acceptance.

Only an explicitly marked owned disposable board is accepted. Raw terminal and
trace files stay private; the output contains fixed control names and counts.
The caller owns fixture teardown and source capture before every execution.
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
import signal
import socket
import stat
import subprocess
import tempfile
import termios

from check_restricted_ssh import SSHConsole
from smoke_board_launcher import LauncherConsole, PICK, CONFIRM, RETURN
from probe_visit_mail import (assert_same_visit, visit_processes, private_json,
    require_closed_endpoint, require_connection_refused, assert_unchanged,
    drain_ready, TERMINALS, HOSTILE, SCAN, validate_scan, PASS, UNLOCK, HOME, MAIL)

TITLE = 'LOCAL / ENCRYPTED HOME MAIL'
OPEN = '[O] Open encrypted home mail'
BACK = '[B] Back to the BBS'
CONFIRM_MAIL = 'LOCAL: open encrypted home mail while this board stays parked? [y/N]: '
MENU = 'Ctrl-C exits; Esc: Enter to recover.'
OFFER = b'\x1b]7770;FACTION-MAIL1;offer\x07'
TRACE_START = r'''import json,subprocess,os
from pathlib import Path
assert Path('/etc/faction-disposable-test').is_file()
rows=[]
for line in subprocess.check_output(['ps','-axo','pid,user,comm'],text=True).splitlines():
 f=line.strip().split(None,2)
 if len(f)==3 and f[1]=='factiond' and f[2].strip()=='/usr/local/libexec/faction-service':rows.append(int(f[0]))
assert len(rows)==1
path='/root/issue230-board-'+os.urandom(8).hex()+'.ktrace'
subprocess.run(['ktrace','-t','i','-f',path,'-p',str(rows[0])],check=True)
print(json.dumps({'pid':rows[0],'path':path}))
'''
TRACE_STOP = r'''import json,subprocess,sys
from pathlib import Path
assert Path('/etc/faction-disposable-test').is_file()
s=json.load(sys.stdin);assert type(s['pid']) is int and s['path'].startswith('/root/issue230-board-') and s['path'].endswith('.ktrace')
subprocess.run(['ktrace','-c','-p',str(s['pid'])],check=True)
r=subprocess.run(['kdump','-X','1','-m','1048576','-f',s['path']],capture_output=True,check=True)
sys.stdout.buffer.write(r.stdout)
''' 
CHECKS = [
    'independent_approved_member_files_disabled_main_menu_entry',
    'ordinary_ssh_explanation_and_same_menu_return',
    'fixed_local_open_back_choice_without_unlock_or_home_contact',
    'queued_confirmation_discarded_and_fresh_local_confirmation_required',
    'wrong_unlock_and_canceled_unlock_lock_return_same_ssh',
    'hidden_prompt_and_private_console_do_not_render_hostile_board_output',
    'real_home_tcp_refusal_preserves_exact_checkpoint_history_pending_cursor',
    'existing_local_history_and_outbox_remain_device_private',
    'fresh_hidden_unlock_and_lock_return_to_same_ssh_main_menu',
    'private_view_disconnect_fails_closed_without_redial',
    'private_markers_absent_board_transcript_database_store_and_logs',
]


def require_menu_mail(menu, offer):
    menu=re.sub(rb'\x1b\[[0-9;?]*[ -/]*[@-~]',b'',menu)
    assert b'[M] Home mail' in menu, 'completed main menu lacks separately themed mail action'
    assert offer == OFFER, 'mail offer absent or transfer authority/control leaked'


def reconstruct_kdump_streams(data):
    """Strictly decode observed NetBSD kdump -X 1 GIO, never VIS or raw text.

    Threads share their process descriptors. Descriptor reuse is concatenated
    conservatively: it may produce a false positive, but cannot hide a marker.
    """
    assert data and data.endswith(b'\n'), 'empty or truncated hex capture'
    try:
        text=data.decode('ascii')
    except UnicodeDecodeError as error:
        raise AssertionError('non-ASCII kdump presentation') from error
    header=re.compile(r'\s*(\d+)(?:\s+(\d+))?\s+faction-service\s+GIO\s+fd\s+(\d+)\s+(read|wrote)\s+(\d+)\s+bytes')
    streams={};records=0;key=None;expected=0;payload=bytearray()
    repeat=False;previous=b''
    def finish():
        nonlocal records
        if key is not None:
            assert not repeat, 'unverified trailing repeated GIO rows'
            assert len(payload)==expected, 'declared GIO byte count differs or data truncated'
            streams.setdefault(key,bytearray()).extend(payload)
            records+=1
    for line in text.splitlines():
        match=header.fullmatch(line)
        if match:
            finish()
            key=(int(match[1]),int(match[3]),match[4])
            expected=int(match[5]);payload=bytearray();repeat=False;previous=b''
            assert key[0]>0 and key[1]>=0 and 0<=expected<=1048576, 'unsupported GIO bounds'
            continue
        if not line.strip():
            continue
        assert key is not None, 'unrecognized kdump header'
        if line=='\t***':
            assert not repeat and len(previous)==16 and len(payload)<expected, 'unsupported repeated GIO row'
            repeat=True
            continue
        row=re.fullmatch(r'\t([0-9a-f]{3,})   (.{48}) (.{1,16})',line)
        assert row is not None, 'unrecognized/truncated hex GIO row; no raw fallback'
        offset=int(row[1],16)
        octets=row[2].split()
        if not octets:
            # NetBSD hexdump_buf's dupl == 5 branch prints one repeated
            # complete row with a blank hex field and the prior ASCII column.
            assert not repeat and row[2]==' '*48 and offset==len(payload), 'invalid single repeated GIO offset'
            assert len(previous)==16 and len(payload)+16<expected, 'unverified single repeated GIO extent'
            assert ''.join(chr(v) if 33<=v<=126 else '.' for v in previous)==row[3], 'single repeated GIO ASCII mismatch'
            payload.extend(previous)
            # An explicit following hex row must precede another repeat.
            previous=b''
            continue
        if repeat:
            # kdump suppresses identical complete rows. Only the next explicit
            # offset verifies how many were omitted; end repeats stay unsupported.
            gap=offset-len(payload)
            assert gap>0 and gap%16==0 and offset<expected, 'invalid repeated GIO extent'
            payload.extend(previous*(gap//16))
            repeat=False
        assert offset==len(payload), 'wrong or repeated GIO offset'
        assert octets and all(re.fullmatch(r'[0-9a-f]{2}',v) for v in octets), 'invalid GIO hex byte'
        chunk=bytes(int(v,16) for v in octets)
        assert len(chunk)==min(16,expected-len(payload)) and len(chunk)>0, 'wrong GIO row byte count'
        canonical=' '.join(octets[:8])
        if len(octets)>8:
            canonical+='  '+' '.join(octets[8:])
        assert canonical.ljust(48)==row[2], 'noncanonical GIO hex layout'
        assert ''.join(chr(v) if 33<=v<=126 else '.' for v in chunk)==row[3], 'GIO ASCII column contradicts hex'
        payload.extend(chunk)
        previous=chunk
    finish()
    assert records and sum(map(len,streams.values())), 'no decoded GIO bytes'
    return {k:bytes(v) for k,v in streams.items()},records


def require_board_bound_private_absent(data, markers):
    assert markers and all(isinstance(v,bytes) and v for v in markers), 'empty privacy marker set'
    streams,records=reconstruct_kdump_streams(data)
    assert all(marker not in stream for stream in streams.values() for marker in markers), 'private data in reconstructed board I/O'
    return dict(records=records,streams=len(streams),decoded_bytes=sum(map(len,streams.values())),read_bytes=sum(len(v) for k,v in streams.items() if k[2]=='read'),write_bytes=sum(len(v) for k,v in streams.items() if k[2]=='wrote'))


def require_mail_control(data, action):
    assert action in {'park', 'back'}
    assert re.fullmatch(rb'@MAIL1 '+action.encode()+rb' [0-9a-f]{32}\r?\n', data), 'invalid mail control or extra fields'


def probe(a):
    info = a.private_root.lstat()
    assert stat.S_ISDIR(info.st_mode) and info.st_uid == os.getuid() and not info.st_mode & 0o077
    assert (a.private_root/'OWNED').is_file()
    settings = private_json(a.settings)
    assert settings['disposable_fixture'] is True and settings['files_disabled'] is True
    root = Path(tempfile.mkdtemp(prefix='menu-mail-', dir=a.private_root)).resolve()
    root.chmod(0o700)
    children = []; visits = []; checks = []; spans = []; trace = None
    password = secrets.token_hex(24); home_password = secrets.token_hex(24)
    subject = 'private-menu-subject-'+secrets.token_hex(12)
    body = 'private-menu-body-'+secrets.token_hex(16)
    markers = [PASS, home_password, subject, body, 'synthetic private archive id',
               'synthetic-private-archive-id', 'synthetic subject', 'synthetic incoming', 'synthetic received body']
    def private(path, data):
        path.write_text(data); path.chmod(0o600)
    private(root/'private-markers.json',json.dumps(markers+[password]))
    def remote(code, data=None):
        r = subprocess.run(settings['admin_argv']+['/usr/pkg/bin/python3.13 -c '+shlex.quote(code)], input=data, capture_output=True, timeout=45)
        if r.returncode: (root/'admin-failure.private').write_bytes(r.stdout+r.stderr)
        assert r.returncode == 0, 'owned board operation failed'
        return r.stdout
    def command(action, *args, extra=(), ok=True):
        r = subprocess.run([str(a.mail_fixture), action, str(store), *map(str,args)], input=('\n'.join((PASS,*extra))+'\n').encode(),capture_output=True,timeout=30)
        if ok: assert r.returncode == 0, 'private mail fixture action failed'
        return r
    def snapshot():
        inspection = json.loads(command('inspect').stdout)
        return dict(canonical=(store/'mail.fkv').read_bytes(), history=inspection['history'], pending=inspection['pending'], archive=json.loads(command('archive-status').stdout))
    def menu(c): c.expect(MENU)
    def login(c, handle, secret, register=False):
        c.expect('[L]ogin [R]egister [Q]uit: '); c.send('r' if register else 'l'); c.expect('Handle: '); c.send(handle); c.password(secret)
        if register:c.password(secret,'Confirm password (hidden): ')
        else:c.expect('Press Enter to continue: '); c.send(''); menu(c)
    def board():
        c=SSHConsole(settings['member_argv']); children.append(c); return c
    try:
        with socket.socket() as outage:
            outage.bind(('127.0.0.1',0)); endpoint=outage.getsockname(); outage.close(); require_closed_endpoint(endpoint)
        store=root/'mail'
        r=subprocess.run([str(a.checkpoint_fixture),str(store),'archive-pending'],capture_output=True,timeout=30)
        assert r.returncode==0, 'checkpoint fixture seed failed'
        command('queue',extra=(subject,body))
        ca=root/'public-ca.pem';ca.write_bytes(a.public_ca.read_bytes());ca.chmod(0o600)
        config=root/'home.toml';private(config,'version=1\naccount="alice@home.test"\nendpoint='+json.dumps('127.0.0.1:'+str(endpoint[1]))+'\ndomain="home.test"\nca='+json.dumps(str(ca))+'\npin="'+'ab'*32+'"\n')
        catalog=root/'boards.toml';private(catalog,'version=1\nhome_mail={store='+json.dumps(str(store))+',config='+json.dumps(str(config))+'}\n[[boards]]\nid="approved"\nlabel="Owned menu mail board"\nprofile='+json.dumps(settings['profile'])+'\n')
        before=snapshot();assert before['archive']['ready'] and before['archive']['cursor_present'] and before['archive']['checkpoint']['after']
        assert len(before['history'])==3 and len(before['pending'])==2
        assert all(m.encode() not in before['canonical'] for m in markers)
        failure=command('sync',config,extra=(home_password,),ok=False)
        require_connection_refused(failure.returncode,failure.stdout,failure.stderr)
        assert_unchanged(before,snapshot())
        handle=settings['member_handle']+secrets.token_hex(3)
        c=board();login(c,handle,password,True);c.expect('Registration pending; sysop approval required.');c.finish()
        sysop=board();login(sysop,'sysop',settings['sysop_password']);sysop.raw(b's');sysop.expect('[C]reate room: ');sysop.send('a');sysop.expect('Member handle: ');sysop.send(handle);sysop.expect('Status updated; previous sessions expired.');sysop.expect('Press Enter to return to menu: ');sysop.send('');menu(sysop);sysop.raw(b'q');sysop.expect('Logged out.');sysop.finish()
        plain=board();login(plain,handle,password);require_menu_mail(plain.transcript,OFFER);plain.raw(b'm');plain.expect('Ordinary SSH: press Enter or type back to return to the main menu.');plain.expect(OFFER.decode());plain.send('');menu(plain);plain.raw(b'q');plain.expect('Logged out.');plain.finish();checks.append(CHECKS[0]);checks.append(CHECKS[1])
        guest_before={x['pid'] for x in json.loads(remote(TERMINALS))}
        c=LauncherConsole([a.client,'--boards',catalog]+(['--unicode'] if a.unicode else []));children.append(c);c.expect(PICK);c.raw(b'a');c.expect(CONFIRM);c.send('y');login(c,handle,password)
        require_menu_mail(c.transcript,OFFER)
        visit=visit_processes(c.pid);visits.append(visit)
        tty=[x for x in json.loads(remote(TERMINALS)) if x['pid'] not in guest_before];assert len(tty)==1
        private(root/'member-tty.private.json',json.dumps(tty[0]))
        trace=json.loads(remote(TRACE_START))
        private(root/'board-trace.private.json',json.dumps(trace))
        def choice():
            c.raw(b'm');c.expect(TITLE);c.expect(OPEN);c.expect(BACK)
        def same():
            assert_same_visit(visit,visit_processes(c.pid));assert_unchanged(before,snapshot())
        remote_start=0
        choice();drain_ready(c);assert UNLOCK.encode() not in c.buffer and HOME.encode() not in c.buffer
        c.raw(b'b');menu(c);same();checks.append(CHECKS[2])
        choice();c.raw(b'oy\n');c.expect(CONFIRM_MAIL);drain_ready(c);assert UNLOCK.encode() not in c.buffer
        c.send('n');menu(c);same();checks.append(CHECKS[3])
        def begin_private():
            choice();c.raw(b'o');c.expect(CONFIRM_MAIL);c.send('y');c.expect(UNLOCK)
            assert not termios.tcgetattr(c.fd)[3]&termios.ECHO
            spans.append((remote_start,len(c.transcript)))
        begin_private();c.send('wrong synthetic local passphrase');c.expect('Enter to return to the board: ');remote_start=len(c.transcript);c.send('');menu(c);same()
        begin_private();remote_start=len(c.transcript);c.raw(b'\x03');menu(c);same();checks.append(CHECKS[4])
        begin_private();private_start=len(c.transcript)
        hostile=json.loads(remote(HOSTILE.replace('1048576','16384'),json.dumps(tty[0]).encode()));drain_ready(c)
        assert b'PUBLIC_HOSTILE_VISIT_MARKER' not in c.transcript[private_start:]
        c.send(PASS);c.expect(MAIL)
        for action,message in [(b'n','Home cycle stopped; pending ciphertext retained.'),(b'a','Catch-up stopped; earlier accepted entries may already be saved.')]:
            c.raw(action);c.password(home_password,HOME);c.expect(message);c.expect('Enter to return: ');c.send('');c.expect(MAIL)
        c.raw(b'h');c.expect('Authenticated history: 3/64')
        for _ in range(3):c.expect('Enter for next record: ');c.send('')
        c.expect('Enter to return: ');c.send('');c.expect(MAIL)
        assert subject.encode() in c.transcript[private_start:] and body.encode() in c.transcript[private_start:]
        c.raw(b'o');c.expect('Durable pending ciphertext; delivery remains unproven.')
        for pending in before['pending']:c.expect('Operation: '+pending['operation'])
        c.expect('Enter to return: ');c.send('');c.expect(MAIL)
        remote_start=len(c.transcript);c.raw(b'l');menu(c)
        assert b'PUBLIC_HOSTILE_VISIT_MARKER' not in c.transcript[private_start:]
        same();checks.extend(CHECKS[5:8])
        begin_private();c.send(PASS);c.expect(MAIL);remote_start=len(c.transcript);c.raw(b'l');menu(c);same();checks.append(CHECKS[8])
        begin_private();c.send(PASS);c.expect(MAIL);os.kill(visit['ssh'],signal.SIGTERM);remote_start=len(c.transcript);c.raw(b'l');c.expect('Connection stopped: ');c.expect(RETURN);c.send('');c.expect(PICK)
        rows=subprocess.check_output(['ps','-axo','pid,ppid,comm'],text=True).splitlines()
        assert not any(len(f:=line.split())>=3 and f[1]==str(c.pid) and Path(f[2]).name=='ssh' for line in rows)
        c.raw(b'q');c.finish();spans.append((remote_start,len(c.transcript)));checks.append(CHECKS[9])
        assert termios.tcgetattr(c.fd)[3]&termios.ECHO
        from probe_visit_mail import require_private_absent
        require_private_absent(c.transcript,spans,[m.encode() for m in markers])
        assert all(home_password.encode() not in child.transcript and PASS.encode() not in child.transcript for child in children)
        scan=validate_scan(json.loads(remote(SCAN,json.dumps(dict(markers=markers,board=settings['scan_board'],store=settings['scan_store'],log=settings['scan_logs'])).encode())))
        board_io=remote(TRACE_STOP,json.dumps(trace).encode());trace=None
        (root/'board-io.private.log').write_bytes(board_io)
        reconstruction=require_board_bound_private_absent(board_io,[m.encode() for m in markers]+[b'Authenticated history:',b'Durable pending ciphertext;',b'HOME account password',b'unread:'])
        streams,_=reconstruct_kdump_streams(board_io)
        assert all(any(token in value for key,value in streams.items() if key[2]=='read') for token in [b'@MAIL1 park ',b'@MAIL1 back ']), 'decoded board input did not contain mail controls'
        assert all(b'FACTION1;' not in value and b'@FACTION1' not in value for value in streams.values()), 'mail acquired file protocol authority'
        checks.append(CHECKS[10]);same_state=snapshot();assert_unchanged(before,same_state);require_closed_endpoint(endpoint)
        assert checks==CHECKS
        private(root/'private-inputs.json',json.dumps(dict(home_password=home_password,subject=subject,body=body,store=str(store))))
        return dict(result='pass',platform=platform.system(),scope='main-menu-local-mail-files-disabled-same-ssh',unicode=a.unicode,count=len(checks),checks=checks,files_disabled=True,transfer_authority_required=False,same_client_pid=True,same_ssh_pid_until_explicit_disconnect=True,home_transport_error='Connect',checkpoint_history_pending_cursor_exact=True,hostile_output=hostile,privacy_scan=scan,private_transcript_intervals=len(spans),actual_board_io_private_scan=True,board_io_reconstruction=reconstruction,no_file_protocol_in_board_io=True,terminal_restored=True,no_automatic_redial=True,private_fixture_retained_for_lead=True,capacity_claim=False,physical_hardware=False)
    except BaseException:
        for i,c in enumerate(children):(root/('failure-'+str(i)+'.private')).write_bytes(c.transcript+c.buffer)
        raise
    finally:
        if trace is not None:
            try:(root/'board-io-failure.private.log').write_bytes(remote(TRACE_STOP,json.dumps(trace).encode()))
            except (AssertionError,subprocess.SubprocessError):pass
        for visit in visits:
            try:
                if visit_processes(visit['client'])==visit:os.kill(visit['ssh'],signal.SIGTERM)
            except (AssertionError,ChildProcessError,ProcessLookupError):pass
        for c in children:c.close()


def main():
    p=argparse.ArgumentParser(description=__doc__)
    for name in ['client','checkpoint-fixture','mail-fixture','public-ca','settings','private-root','output']:p.add_argument('--'+name,type=Path,required=True)
    p.add_argument('--unicode',action='store_true');a=p.parse_args()
    for name in ['client','checkpoint_fixture','mail_fixture','public_ca','settings','private_root','output']:setattr(a,name,getattr(a,name).resolve())
    result=probe(a);a.output.write_text(json.dumps(result,indent=2)+'\n');print('Main-menu same-SSH mail acceptance passed:',result['count'])

if __name__=='__main__':main()
