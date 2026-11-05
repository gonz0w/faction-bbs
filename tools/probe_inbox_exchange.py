#!/usr/bin/env python3
"""Issue 344 production PTY exchange controls; isolated synthetic vaults, no listener."""
import argparse
import json
import os
import socket
import errno
import unicodedata
import subprocess
import traceback
from pathlib import Path
from smoke_native_client_ssh import NativeConsole

PASS = 'checkpoint synthetic fixture password'
HOME = 'HOME account password (hidden; never sent to a BBS): '
MENU = '[L] Lock and exit'
X = '[X] Home exchange'


def run(a):
    root = a.owned_root.resolve()
    info = root.stat()
    assert info.st_uid == os.getuid() and info.st_mode & 0o777 == 0o700
    assert (root/'OWNERSHIP').is_file()
    assert a.name and Path(a.name).name == a.name and a.name not in ('.', '..')
    with socket.socket() as check:
        assert check.connect_ex(('127.0.0.1', 9)) == errno.ECONNREFUSED, 'synthetic home endpoint must be refused'
    work = root/a.name
    work.mkdir(mode=0o700)
    ca = work/'ca.pem'
    subprocess.run(['openssl','req','-x509','-newkey','rsa:2048','-nodes','-keyout',str(work/'key.pem'),'-out',str(ca),'-days','1','-subj','/CN=synthetic.invalid'],check=True,capture_output=True)
    ca.chmod(0o600)
    config = work/'home.toml'
    config.write_text('version = 1\naccount = "alice@home.test"\nendpoint = "127.0.0.1:9"\ndomain = "home.test"\nca = '+json.dumps(str(ca))+'\npin = "'+'00'*32+'"\n')
    config.chmod(0o600)
    checks=[]; children=[]
    def start(name, online=True, unicode=False, mode='inbox-7'):
        path=work/name
        subprocess.run([str(a.fixture),str(path),mode],check=True,capture_output=True)
        before=(path/'mail.fkv').read_bytes()
        argv=[str(a.client),'--mail-online',str(path),str(config)] if online else [str(a.client),'--mail-checkpoints',str(path),'open']
        if unicode: argv.append('--unicode')
        c=NativeConsole(argv);children.append((name,c));c.resize(40,24)
        c.password(PASS,'LOCAL checkpoint passphrase (hidden): ');c.expect(MENU)
        return c,path,before
    def inbox(c):
        c.raw(b'i');c.expect('[LOCAL INBOX]');c.expect('[B] Back')
    def group(c):
        c.raw(b'c');c.expect('[LOCAL CORRESPONDENTS]');c.expect('[B] Back');c.raw(b'o');c.expect('[LOCAL CORRESPONDENT INBOX]');c.expect('[B] Back')
    def conversation(c):
        c.raw(b't');c.expect('[LOCAL CONVERSATION]');c.expect('[B] Back')
    def exchange(c):
        begin=len(c.transcript)
        c.raw(b'x');c.expect('[LOCAL HOME EXCHANGE]');c.expect('[B] Return')
        for row in c.transcript[begin:].decode().splitlines():
            cells=sum(2 if unicodedata.east_asian_width(ch) in ('W','F') else 1 for ch in row)
            assert cells <= 40, 'exchange submenu exceeds classic width'
    def returned(c, label): c.expect(label);c.expect('[B] Back')
    def stable(path,before): assert (path/'mail.fkv').read_bytes()==before
    try:
        c,path,before=start('entry')
        inbox(c);global_x=X.encode() in c.transcript+c.buffer
        begin=len(c.transcript);group(c);group_x=X.encode() in c.transcript[begin:]+c.buffer
        begin=len(c.transcript);conversation(c);conversation_x=X.encode() in c.transcript[begin:]+c.buffer
        c.raw(b'bbbbl');c.finish();stable(path,before)
        checks.append({'online_global_X':global_x,'group_X':group_x,'conversation_X':conversation_x})
        assert global_x and group_x and conversation_x, 'online production inbox/group/conversation has no Home exchange entry'
        if a.red_only: return checks
        for unicode in [False,True]:
            for surface in ['global','group','conversation']:
                name=surface+('-unicode' if unicode else '-classic')
                c,path,before=start(name,unicode=unicode)
                inbox(c)
                if surface!='global':group(c)
                # A literal private search and Unread survive both returns.
                c.raw(b'u');returned(c,'Unread received; newest first')
                c.raw(b's');c.expect('Search (64 bytes; blank clears): ');c.send('inbox');returned(c,'Local search active.')
                c.raw(b'j');returned(c,'selection 2/3')
                label='[LOCAL INBOX]' if surface=='global' else '[LOCAL CORRESPONDENT INBOX]'
                if surface=='conversation':conversation(c);label='[LOCAL CONVERSATION]'
                exchange(c)
                assert b'ALL pending ciphertext' in c.transcript and b'will not receive' in c.transcript
                assert HOME.encode() not in c.transcript
                c.raw(b'b');returned(c,label)
                exchange(c);c.raw(b'n');c.password('synthetic first home password',HOME)
                c.expect('Home cycle stopped; pending ciphertext retained.');c.expect('Enter to return: ');c.send('');returned(c,label)
                exchange(c);c.raw(b'a');c.password('synthetic second home password',HOME)
                c.expect('Catch-up stopped; earlier accepted entries may already be saved.');c.expect('Enter to return: ');c.send('');returned(c,label)
                tail=c.transcript[c.transcript.rfind(label.encode()):]+c.buffer
                assert b'Local search active.' in tail or b'Unread / Search' in tail
                assert b'> 05 ' in tail, 'eligible canonical selection changed after refused exchange'
                c.raw(b'bbbbl' if surface=='conversation' else b'bbbl' if surface=='group' else b'bl');c.finish();stable(path,before)
                assert HOME.encode() in c.transcript and c.transcript.count(HOME.encode())==2
                assert b'synthetic first home password' not in c.transcript
                assert b'synthetic second home password' not in c.transcript
                checks.append(name+' cancel and fresh N/A refusal return in place')
        for online in [False,True]:
            c,path,before=start('empty-'+str(online),online=online,mode='inbox-0')
            inbox(c)
            if online:exchange(c);c.raw(b'b');returned(c,'[LOCAL INBOX]')
            else:assert X.encode() not in c.transcript
            c.raw(b'bl');c.finish();stable(path,before)
            checks.append('empty online entry' if online else 'offline has no authority')
        for unicode in [False,True]:
            c,path,before=start('offline-all-'+str(unicode),online=False,unicode=unicode)
            inbox(c);group(c);conversation(c)
            assert X.encode() not in c.transcript
            c.raw(b'xbb bbl'.replace(b' ',b''));c.finish();stable(path,before)
            assert HOME.encode() not in c.transcript
            checks.append('offline global group conversation no exchange authority '+str(unicode))
            c,path,before=start('empty-group-'+str(unicode),unicode=unicode)
            inbox(c);group(c)
            c.raw(b'u');returned(c,'Unread received; newest first')
            c.raw(b's');c.expect('Search (64 bytes; blank clears): ');c.send('no matching query');returned(c,'No matching retained records.')
            exchange(c);c.raw(b'b');returned(c,'[LOCAL CORRESPONDENT INBOX]')
            assert b'No matching retained records.' in c.transcript[c.transcript.rfind(b'[LOCAL CORRESPONDENT INBOX]'):]
            conversation(c);exchange(c);c.raw(b'b');returned(c,'[LOCAL CONVERSATION]')
            assert b'No eligible retained records.' in c.transcript[c.transcript.rfind(b'[LOCAL CONVERSATION]'):]
            c.raw(b'bbbbl');c.finish();stable(path,before)
            checks.append('empty eligible group/conversation cancel stays scoped '+str(unicode))
        for control in [b'\x03', b'\x04']:
            for at_password in [False,True]:
                c,path,before=start('control-'+str(control[0])+'-'+str(at_password))
                inbox(c);exchange(c)
                if at_password:c.raw(b'n');c.expect(HOME)
                c.raw(control);c.expect('Home mail locked: local operation stopped.');c.finish(code=1);stable(path,before)
                checks.append('control locks from exchange or fresh password '+str(control[0])+' '+str(at_password))
        c,path,before=start('disabled',mode='pending')
        inbox(c);exchange(c)
        assert b'bounded live receive' in c.transcript
        c.raw(b'a');c.expect('Enable archive checkpoints with G before catch-up.');c.expect('Enter to return: ');c.send('');returned(c,'[LOCAL INBOX]')
        assert HOME.encode() not in c.transcript
        # Queued N must be discarded at X entry, leaving a cancellation prompt.
        c.raw(b'xn');c.expect('[LOCAL HOME EXCHANGE]');c.expect('[B] Return');c.raw(b'b');returned(c,'[LOCAL INBOX]')
        assert HOME.encode() not in c.transcript
        c.raw(b'bl');c.finish();stable(path,before);checks.append('disabled archive and queued input refuse connection')
        return checks
    finally:
        for name,c in children:
            c.close();(work/(name+'.pty')).write_bytes(c.transcript)


def main():
    p=argparse.ArgumentParser();p.add_argument('--client',type=Path,required=True);p.add_argument('--fixture',type=Path,required=True);p.add_argument('--owned-root',type=Path,required=True);p.add_argument('--name',required=True);p.add_argument('--red-only',action='store_true');a=p.parse_args();a.client=a.client.resolve();a.fixture=a.fixture.resolve()
    try:
        checks=run(a);print(json.dumps({'result':'pass','checks':checks}))
    except BaseException:
        traceback.print_exc();raise
if __name__=='__main__':main()
