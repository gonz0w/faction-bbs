#!/usr/bin/env python3
"""Actual SSH SIGWINCH recovery in an owned emulator and synthetic board only."""
import argparse,fcntl,json,os,struct,termios,time
from pathlib import Path
from probe_terminal_profile import Terminal
from exercise_terminal_bbs import Relay

def run(a):
    cfg=json.loads(a.bbs_config.read_text());t=Terminal();results=[]
    try:
        for mode in ['menu','graphics']:
            time.sleep(5.1);c=Relay(t,cfg['ssh_argv']);d={'mode':mode,'result':'fail','resize_scope':'owned SSH PTY/SIGWINCH, not OS-window manipulation'}
            try:
                c.expect('[L]ogin [R]egister [Q]uit: ');c.send('l');c.expect('Handle: ');c.send('sysop');c.expect('Password (hidden): ');c.send(cfg['password']);c.expect('Press Enter to continue: ');c.send('');c.expect('Ctrl-C exits; Esc: Enter to recover.')
                if mode=='graphics':
                    c.raw(b't');c.expect('[D]one: ');c.reply='kitty';c.send('g');c.expect('[D]one: ');c.reply=None
                    if b'a=T,t=d,f=24' not in c.transcript:raise RuntimeError('graphics path unavailable')
                size=t.size();rows,cols=(24,80) if size[:2]==[18,40] else (18,40)
                fcntl.ioctl(c.fd,termios.TIOCSWINSZ,struct.pack('HHHH',rows,cols,0,0));d['new_pty_geometry']=[rows,cols]
                if mode=='menu':
                    c.expect('Command (then Enter): ');c.send('q');c.expect('Logged out.');c.finish();d['classic_recovered']=True
                else:
                    until=time.monotonic()+10
                    while time.monotonic()<until:
                        pid,status=os.waitpid(c.pid,os.WNOHANG)
                        if pid:
                            c.done=True
                            for _ in range(3):c.pump(.02)
                            if os.waitstatus_to_exitcode(status)==0:raise RuntimeError('expected graphics resize closure')
                            break
                        c.pump(.02)
                    else:raise TimeoutError('graphics resize closure')
                    reply=t.query(b'\x1b_Ga=p,i=17391,C=1,c=20,r=5;\x1b\\','image');d['image_after_close']=reply
                    if b';ENOENT' not in bytes.fromhex(reply['reply_hex']):raise RuntimeError('image retained after close')
                    d['image_deleted']=True
                d['unexpected_input_bytes']=c.noise+len(c.pending);d['result']='pass' if not c.noise and not c.pending else 'fail'
            except Exception as e:d['error']=type(e).__name__;d['stage']=getattr(c,'waiting','startup')
            finally:c.close()
            results.append(d)
    finally:t.close();a.output.write_text(json.dumps({'schema':1,'result':'pass' if len(results)==2 and all(d['result']=='pass' for d in results) else 'fail','cases':results},indent=2,sort_keys=True)+'\n');a.output.chmod(0o600)
if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--bbs-config',type=Path,required=True);p.add_argument('--output',type=Path,required=True);run(p.parse_args())
