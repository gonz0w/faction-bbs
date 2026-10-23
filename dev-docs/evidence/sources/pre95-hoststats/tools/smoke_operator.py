#!/usr/bin/env python3
"""Local read-only operator UI on disposable PTYs, not a live service proof."""
import argparse,fcntl,json,os,pty,select,secrets,struct,subprocess,tempfile,termios,time
from pathlib import Path
from smoke_local_console import Console

class Operator(Console):
    def __init__(self,argv,columns=80,rows=24):
        self.pid,self.fd=pty.fork()
        if self.pid==0:
            fcntl.ioctl(0,termios.TIOCSWINSZ,struct.pack('HHHH',rows,columns,0,0));os.environ['TERM']='xterm-256color'
            os.closerange(3,os.sysconf('SC_OPEN_MAX'));os.execv(str(argv[0]),list(map(str,argv)))
        self.buffer,self.transcript,self.done=b'',b'',False

def status(path,connected=1,stamp=None,running=True):
    text=f'version=1\nsampled={int(time.time()) if stamp is None else stamp}\npid=1\nuptime=5\nrunning={int(running)}\nconnected={connected}\ncapacity=8\ncredentials=0\noutput_bytes=0\ndoors=0\ncounts=2,1,1,0,1,0,0,0,0,0\n'
    path.write_text(text);path.chmod(0o600)

def run(a):
    checks=[];children=[]
    with tempfile.TemporaryDirectory(prefix='faction-operator-ui-') as tmp:
        state=Path(tmp);path=state/'operator.status';status(path)
        args=[str(a.binary),'--state-dir',str(state),'--theme',str(a.theme.resolve(strict=True))]
        def once(*extra):return subprocess.run([*args,'--once',*extra],capture_output=True,timeout=3)
        r=once('--plain');assert r.returncode==0 and b'Connected: 1 / 8' in r.stdout and b'\x1b' not in r.stdout
        checks.append('plain_one_shot_no_terminal_controls')
        status(path,stamp=int(time.time())-60);assert b'STALE' in once().stdout
        status(path,stamp=int(time.time())+60);assert b'CLOCK UNKNOWN' in once().stdout
        status(path,running=False);assert b'STOPPED' in once().stdout
        path.unlink();assert b'UNAVAILABLE' in once().stdout
        checks.append('stale_future_stopped_and_unavailable')
        status(path);path.chmod(0o644);assert once().returncode!=0;path.chmod(0o600)
        checks.append('unsafe_snapshot_permission_denied')
        assert once('--refresh','0').returncode!=0
        checks.append('refresh_bounds')
        try:
            c=Operator([*args,'--ansi','--refresh','2']);children.append(c)
            c.expect('LOCAL SYSOP');c.expect('Connected: 1 / 8')
            c.raw(b'h');c.expect('Host CPUs:');c.expect('Board disk free:')
            c.raw(b'\x1b[C');c.expect('OS owner/root access; not a BBS role.')
            c.raw(b'b');c.expect('Connected: 1 / 8')
            c.raw(b'p');c.expect('PAUSED');status(path,connected=2);time.sleep(2.2)
            c.raw(b'p');c.expect('READ ONLY');c.expect('Connected: 2 / 8')
            c.buffer=b'';c.resize(40,18);c.expect('40x18');c.expect('Q:quit P:pause R:redraw B/H/I:views');c.raw(b'q');c.finish()
            flags=termios.tcgetattr(c.fd)[3];assert flags & termios.ECHO and flags & termios.ICANON
            checks.append('themed_panels_arrows_pause_resize_and_tty_restore')
            c=Operator([*args,'--plain']);children.append(c);c.expect('LOCAL SYSOP');c.raw(b'\x03');c.finish()
            assert b'\x1b' not in c.transcript;checks.append('plain_interactive_ctrl_c_exit')
        finally:
            for c in children:c.close()
    result=dict(result='pass',count=len(checks),checks=checks,capacity_claim=False)
    if a.output:a.output.write_text(json.dumps(result,indent=2)+'\n')
    print(f'Local operator UI passed {len(checks)} controls.')

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('binary',type=Path);p.add_argument('--output',type=Path);p.add_argument('--theme',type=Path,default=Path(__file__).resolve().parents[1]/'themes/nightwire')
    a=p.parse_args();a.binary=a.binary.resolve(strict=True);run(a)
