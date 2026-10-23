#!/usr/bin/env python3
"""Local private-catalog PTY checks; no SSH connection or real board required."""
import argparse,fcntl,json,os,pty,re,socket,struct,subprocess,tempfile,termios
from pathlib import Path
from smoke_local_console import Console

class LauncherConsole(Console):
    def __init__(self,argv,columns=80,rows=24):
        self.pid,self.fd=pty.fork()
        if self.pid==0:
            fcntl.ioctl(0,termios.TIOCSWINSZ,struct.pack('HHHH',rows,columns,0,0))
            os.environ['TERM']='xterm-256color';os.closerange(3,os.sysconf('SC_OPEN_MAX'))
            os.execv(str(argv[0]),list(map(str,argv)))
        self.buffer,self.transcript,self.done=b'',b'',False

PICK='A-D/Enter: choose  Q: quit'
RETURN='Enter to return to local Boards: '
CONFIRM='Connect using this local profile? [y/N]: '

def catalog(paths):
    return 'version=1\n'+''.join('[[boards]]\nid="board-'+str(i+1)+'"\nlabel="Local Board '+str(i+1)+'"\nprofile='+json.dumps(str(path))+'\n' for i,path in enumerate(paths))

def run(binary,output=None):
    checks=[];children=[]
    with tempfile.TemporaryDirectory(prefix='faction-launcher-') as tmp, socket.socket() as trap:
        root=Path(tmp);root.chmod(0o700);trap.bind(('127.0.0.1',0));trap.listen();trap.settimeout(.2)
        def private(path,text):path.write_text(text);path.chmod(0o600)
        for name in ['identity','known_hosts','identity.pub']:
            private(root/name,'ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA' if name.endswith('.pub') else 'fixture')
        profile='version=1\nhost="127.0.0.1"\nport='+str(trap.getsockname()[1])+'\nfile_host="127.0.0.1"\nfile_port=2223\nidentity='+json.dumps(str(root/'identity'))+'\npublic_key='+json.dumps(str(root/'identity.pub'))+'\nknown_hosts='+json.dumps(str(root/'known_hosts'))+'\n'
        paths=[root/f'board{i}.toml' for i in range(16)]
        for path in paths:private(path,profile)
        pack=root/'boards.toml';private(pack,catalog(paths))
        def start(**kw):
            c=LauncherConsole([binary,'--boards',pack],**kw);children.append(c);c.expect(PICK);return c
        try:
            c=start();assert b'[L]ogin' not in c.transcript;c.raw(b'q');c.finish();checks.append('startup_local_catalog_has_no_automatic_connection')
            c=start();c.raw(b'\x1b[B');c.expect('> [B] Local Board 2');c.raw(b'n');c.expect('Page 2 / 4');c.raw(b'a');c.expect('LOCAL CONNECTION / Local Board 5');c.expect(CONFIRM);c.send('n');c.expect(RETURN);c.send('');c.expect(PICK);c.raw(b'q');c.finish();checks.append('arrows_pages_hotkeys_and_canceled_confirmation')
            c=start(columns=40,rows=18);screen=re.sub(rb'\x1b\[[0-9;?]*[ -/]*[@-~]',b'',c.transcript).decode();assert len(screen.splitlines())<=18 and all(len(s)<=39 for s in screen.splitlines());c.raw(b'p');c.expect('Page 4 / 4');c.raw(b'q');c.finish();checks.append('compact_maximum_catalog_paging_and_terminal_restore')
            paths[0].unlink();c=start();c.raw(b'a');c.expect('Connection stopped: ');c.expect(RETURN);c.send('');c.expect(PICK);assert b'[L]ogin' not in c.transcript;c.raw(b'q');c.finish();checks.append('selected_profile_revalidated_and_failure_returns_to_launcher')
            c=start();c.raw(b'\x03');c.finish();checks.append('local_quit_ctrl_c_restores_terminal')
            pack.chmod(0o644);p=subprocess.run([binary,'--boards',str(pack)],capture_output=True,timeout=5);assert p.returncode==1;checks.append('loose_catalog_rejected_before_terminal_or_network')
            try:conn,_=trap.accept()
            except TimeoutError:pass
            else:conn.close();raise AssertionError('launcher connected before explicit confirmed choice')
            checks.append('no_connection_before_choice_or_after_cancel_or_bad_profile')
        finally:
            for c in children:c.close()
    result=dict(result='pass',count=len(checks),checks=checks,capacity_claim=False,remote_profile_import=False,federation_claim=False)
    if output:output.write_text(json.dumps(result,indent=2,sort_keys=True)+'\n')
    print(f'Local board launcher passed {len(checks)} controls.')

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('binary',type=Path);p.add_argument('--output',type=Path);a=p.parse_args();run(str(a.binary.resolve()),a.output)
