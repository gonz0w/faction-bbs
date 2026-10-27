#!/usr/bin/env python3
"""Owned real-emulator TTY measurements, never a mocked terminal responder.

Run only in a fresh synthetic terminal. Unknown keyboard input is discarded and
counted, never recorded or forwarded. A contaminated run cannot be a pass.
"""
import argparse,fcntl,json,os,re,select,struct,subprocess,termios,time,tty
from pathlib import Path

SYNC=b'\x1b[?2026$p'
KITTY=b'\x1b_Gi=17392,s=1,v=1,a=q,t=d,f=24;AAAA\x1b\\'
PATTERNS={
    'sync':re.compile(rb'\x1b\[\?2026;[0-4]\$y'),
    'kitty':re.compile(rb'\x1b_Gi=17392;[ -~]{1,100}\x1b\\'),
    'image':re.compile(rb'\x1b_Gi=17391;[ -~]{1,100}\x1b\\'),
    'cursor':re.compile(rb'\x1b\[[0-9]{1,4};[0-9]{1,4}R'),
    'sgr':re.compile(rb'\x1bP(?:0\$r|1\$r[0-9;:]*m)\x1b\\'),
}
class ReplyCollector:
    """Retain one structurally known report, never arbitrary keyboard text."""
    def __init__(self,kind):self.kind=kind;self.pending=b'';self.valid=None;self.unexpected=0
    def feed(self,incoming):
        if len(incoming)>128:
            self.unexpected+=len(incoming)+len(self.pending);self.pending=b'';return False
        self.pending+=incoming;match=PATTERNS[self.kind].search(self.pending);accepted=False
        if match:
            if self.valid is not None:self.unexpected+=len(match[0])
            else:self.valid=match[0];accepted=True
            self.unexpected+=match.start();self.pending=self.pending[match.end():]
        if len(self.pending)>128:self.unexpected+=len(self.pending);self.pending=b''
        return accepted
    def finish(self):
        self.unexpected+=len(self.pending);self.pending=b'';return self.valid

class Terminal:
    def __init__(self):
        self.saved=termios.tcgetattr(0);tty.setraw(0);self.sent=0;self.received=0;self.unexpected=0
    def close(self):
        try:self.write(b'\x1b[0m\x1b[?25h')
        finally:termios.tcsetattr(0,termios.TCSANOW,self.saved)
    def write(self,data):
        deadline=time.monotonic()+5;offset=0
        old=fcntl.fcntl(1,fcntl.F_GETFL)
        try:
            fcntl.fcntl(1,fcntl.F_SETFL,old|os.O_NONBLOCK)
            while offset<len(data):
                left=deadline-time.monotonic()
                if left<=0 or not select.select([],[1],[],left)[1]:raise TimeoutError('owned terminal output timeout')
                try:offset+=os.write(1,data[offset:])
                except BlockingIOError:pass
            self.sent+=len(data)
        finally:fcntl.fcntl(1,fcntl.F_SETFL,old)
    def query(self,data,kind,timeout=.3):
        self.write(data);start=time.monotonic();reports=ReplyCollector(kind);first=None
        while time.monotonic()-start<timeout:
            if not select.select([0],[],[],min(.01,max(0,timeout-(time.monotonic()-start))))[0]:continue
            incoming=os.read(0,128);self.received+=len(incoming)
            if reports.feed(incoming):first=round((time.monotonic()-start)*1000,3)
        valid=reports.finish();self.unexpected+=reports.unexpected
        return {'reply_hex':valid.hex() if valid else '', 'first_reply_ms':first, 'window_ms':round((time.monotonic()-start)*1000,3)}

    def size(self):return list(struct.unpack('HHHH',fcntl.ioctl(0,termios.TIOCGWINSZ,b'\0'*8)))

def cursor(reply):
    raw=bytes.fromhex(reply['reply_hex'])
    match=re.fullmatch(rb'\x1b\[([0-9]+);([0-9]+)R',raw)
    return [int(v) for v in match.groups()] if match else None

def run(a):
    terminal=Terminal();result={'schema':1,'pid':os.getpid(),'ppid':os.getppid(),'term_hint':os.environ.get('TERM',''),'geometry':terminal.size(),'cases':[]}
    try:
        if a.start_gate:
            a.start_gate.with_suffix('.ready').write_text('ready\n')
            end=time.monotonic()+10
            while not a.start_gate.exists():
                if time.monotonic()>end:raise TimeoutError('owned fixture start gate')
                time.sleep(.01)
            termios.tcflush(0,termios.TCIFLUSH)
        result['sync']=terminal.query(SYNC,'sync')
        result['kitty']=terminal.query(KITTY,'kitty')
        result['rgb_sgr_status']=terminal.query(b'\x1b[38;2;17;34;51m\x1bP$qm\x1b\\','sgr');terminal.write(b'\x1b[0m')
        rows=subprocess.check_output([str(a.presenter)],text=True).splitlines()
        for line in rows:
            name,raw,native,nw,classic,cw=line.split('\t')
            for mode,data,expected in [('native',native,int(nw)),('classic',classic,int(cw))]:
                reply=terminal.query(b'\x1b[2J\x1b[H'+bytes.fromhex(data)+b'\x1b[6n','cursor')
                pos=cursor(reply);result['cases'].append({'name':name,'mode':mode,'rendered_hex':data,'expected_cells':expected,'cursor':pos,'pass':pos==[1,expected+1],'first_reply_ms':reply['first_reply_ms']})
            if name in ['combining','zwj','flag','modifier']:
                reply=terminal.query(b'\x1b[2J\x1b[H'+bytes.fromhex(raw)+b'\x1b[6n','cursor');result['cases'].append({'name':name,'mode':'raw_observation_only','rendered_hex':raw,'cursor':cursor(reply),'first_reply_ms':reply['first_reply_ms']})
        before=terminal.size();terminal.write(b'\x1b[8;18;40t');time.sleep(.15)
        result['resize_request']={'before':before,'after':terminal.size(),'request':'CSI 8;18;40t','fixture_only':True}
        if a.bbs_config:
            from exercise_terminal_bbs import run as bbs_run
            result['bbs_routes']=bbs_run(terminal,a.bbs_config)
        result['keyboard']='classic application input only; physical keys/extended keys not certified';result['mouse']='disabled, not negotiated'
        result['unexpected_input_bytes']=terminal.unexpected;result['sent_bytes']=terminal.sent;result['received_bytes']=terminal.received
        classic=all(c.get('pass',True) for c in result['cases'] if c['mode']=='classic')
        native=all(c.get('pass',True) for c in result['cases'] if c['mode']=='native')
        routes=all(c['result']=='pass' for c in result.get('bbs_routes',[]))
        result['classic_cells_pass']=classic;result['native_cells_pass']=native
        result['result']=('pass' if native else 'classic-only') if not terminal.unexpected and classic and routes else 'fail'
    except Exception as e:result['result']='fail';result['error']=type(e).__name__
    finally:
        terminal.close();temporary=a.output.with_suffix('.tmp');temporary.write_text(json.dumps(result,indent=2,sort_keys=True)+'\n');temporary.chmod(0o600);temporary.replace(a.output)
    return result['result']=='pass'
if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--start-gate',type=Path);p.add_argument('--bbs-config',type=Path);p.add_argument('--presenter',required=True,type=Path);p.add_argument('--output',required=True,type=Path);a=p.parse_args();raise SystemExit(0 if run(a) else 1)
