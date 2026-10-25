#!/usr/bin/env python3
"""Synthetic BBS routes inside an owned real emulator; no human input replay."""
import errno,fcntl,json,os,pty,re,select,struct,termios,time
from pathlib import Path
from probe_terminal_profile import PATTERNS

class Relay:
    def __init__(self,terminal,argv):
        size=terminal.size()
        self.terminal=terminal;self.pid,self.fd=pty.fork();self.buffer=b'';self.transcript=b'';self.done=False;self.reply=None;self.pending=b'';self.reports=[];self.noise=0;self.total=0
        if self.pid==0:
            fcntl.ioctl(0,termios.TIOCSWINSZ,struct.pack('HHHH',*size));os.closerange(3,os.sysconf('SC_OPEN_MAX'));os.execv(argv[0],argv)
    def raw(self,data):os.write(self.fd,data)
    def send(self,text):self.raw(text.encode()+b'\n')
    def pump(self,timeout=.1):
        ready=select.select([self.fd,0],[],[],timeout)[0]
        if self.fd in ready:
            try:data=os.read(self.fd,8192)
            except OSError as e:
                if e.errno!=errno.EIO:raise
                data=b''
            if data:
                self.total+=len(data);self.buffer+=data;self.transcript+=data
                if len(self.buffer)>65536 or len(self.transcript)>524288:raise RuntimeError('synthetic BBS output bound')
                self.terminal.write(data)
        if 0 in ready:
            incoming=os.read(0,128);self.pending+=incoming
            # Only structurally known replies to an owned probe can cross to BBS.
            while self.pending:
                found=None
                for kind in ['sync','kitty']:
                    m=PATTERNS[kind].search(self.pending)
                    if m:found=(kind,m);break
                if found:
                    kind,m=found;self.noise+=m.start();self.pending=self.pending[m.end():]
                    if self.reply==kind:self.raw(m[0]);self.reports.append((kind,m[0].hex()))
                    else:self.noise+=len(m[0])
                elif self.pending[0]!=27:self.noise+=1;self.pending=self.pending[1:]
                elif len(self.pending)>128:self.noise+=len(self.pending);self.pending=b''
                else:break
    def expect(self,text,timeout=15):
        self.waiting=text
        wanted=text.encode();until=time.monotonic()+timeout
        while wanted not in self.buffer:
            if time.monotonic()>=until:raise TimeoutError('synthetic BBS prompt timeout')
            self.pump()
        self.buffer=self.buffer.split(wanted,1)[1]
    def finish(self):
        end=time.monotonic()+10
        while time.monotonic()<end:
            pid,status=os.waitpid(self.pid,os.WNOHANG)
            if pid:
                self.done=True
                # Drain trusted cleanup bytes before querying image deletion.
                for _ in range(3):self.pump(.02)
                if os.waitstatus_to_exitcode(status)!=0:raise RuntimeError('synthetic BBS failed')
                return
            self.pump(.02)
        raise TimeoutError('synthetic BBS exit timeout')
    def close(self):
        if not self.done:
            try:os.kill(self.pid,15)
            except ProcessLookupError:pass
            try:os.waitpid(self.pid,0)
            except ChildProcessError:pass
        os.close(self.fd)

def run(terminal,config):
    cfg=json.loads(Path(config).read_text());results=[]
    from smoke_local_console import Console
    state=Path(cfg['local_state'])
    if not state.exists():
        c=Console(Path(cfg['local_binary']),state,'init')
        try:
            c.expect('Handle: ');c.send('sysop');c.password(cfg['password'],'New password (12+ characters, hidden): ');c.password(cfg['password'],'Confirm password (hidden): ');c.expect('Lobby created.');c.finish()
        finally:c.close()
    cases=[('local',[cfg['local_binary'],'--state-dir',str(state),'login','--ansi']),('ssh',cfg['ssh_argv']),('native',[cfg['native_binary'],'--profile',cfg['native_profile']]),('native-kitty',[cfg['native_binary'],'--profile',cfg['native_profile'],'--kitty'])]
    for kind,argv in cases:
        if kind!='local':time.sleep(5.1) # respect normal eight-launch burst/refill across the matrix
        c=Relay(terminal,argv);case={'route':kind,'result':'fail'}
        try:
            if kind=='native-kitty':c.reply='kitty'
            if kind!='local':c.expect('[L]ogin [R]egister [Q]uit: ');c.reply=None;c.send('l')
            c.expect('Handle: ');c.send('sysop');c.expect('Password (hidden): ');c.send(cfg['password'])
            if kind!='local' or (terminal.size()[0]>=24 and terminal.size()[1]>=80):c.expect('Press Enter to continue: ');c.send('')
            c.expect('Ctrl-C exits; Esc: Enter to recover.');c.raw(b't');c.expect('[D]one: ')
            c.reply='sync';c.send('p');c.expect('[D]one: ');c.reply=None
            case['sync_reply_forwarded']=any(k=='sync' for k,_ in c.reports)
            c.reply='kitty';start=len(c.transcript);c.send('g');c.expect('[D]one: ');c.reply=None
            graphic=b'a=T,t=d,f=24' in c.transcript[start:];case['graphic_uploaded']=graphic
            if graphic:
                from smoke_kitty_graphics import validate
                validate(c.transcript[start:]);case['actual_image_retention']=terminal.query(b'\x1b_Ga=p,i=17391,C=1,c=20,r=5;\x1b\\','image')
            else:case['actual_image_retention']=None
            c.send('c');c.expect('[D]one: ')
            if graphic:case['actual_image_after_classic']=terminal.query(b'\x1b_Ga=p,i=17391,C=1,c=20,r=5;\x1b\\','image')
            else:case['actual_image_after_classic']=None
            c.send('u');c.expect('[D]one: ');c.send('3');c.expect('[D]one: ');c.send('d')
            if kind=='local':c.expect('Press Enter to return to menu: ');c.send('')
            c.expect('Ctrl-C exits; Esc: Enter to recover.');c.raw(b'p');c.expect('Room ID: ');c.send('1');c.expect('Reply to post ID (blank for new message): ');c.send('');c.expect('Subject (blank for generated/reply default): ');c.send('Profile café 界 🦀');c.expect('Message (one line, up to 1024 bytes): ');c.send('Synthetic profile café 界 🦀');c.expect('Posted #');c.expect('Press Enter to return to menu: ');c.send('');c.expect('Ctrl-C exits; Esc: Enter to recover.')
            c.raw(b'n');c.expect('Room ID: ');c.send('1');c.expect('Subject: Profile café 界 🦀');c.expect('More? [y/N]: ');c.send('n');c.expect('Press Enter to return to menu: ');c.send('');c.expect('Ctrl-C exits; Esc: Enter to recover.')
            c.raw(b't');c.expect('[D]one: ');c.send('c');c.expect('UTF-8: false');c.expect('[D]one: ');c.send('d')
            if kind=='local':c.expect('Press Enter to return to menu: ');c.send('')
            c.expect('Ctrl-C exits; Esc: Enter to recover.');c.raw(b'q');c.expect('Logged out.');c.finish()
            case['classic_recovered']=True;case['native_unicode_article']=True;case['unclassified_input_bytes']=c.noise+len(c.pending);case['bbs_output_bytes']=c.total;case['result']='pass' if not c.noise and not c.pending else 'fail'
            if graphic:
                before=bytes.fromhex(case['actual_image_retention']['reply_hex']);after=bytes.fromhex(case['actual_image_after_classic']['reply_hex'])
                case['renderer_acknowledged_image']=before.endswith(b';OK\x1b\\');case['renderer_acknowledged_deletion']=b';ENOENT' in after
                if not case['renderer_acknowledged_image'] or not case['renderer_acknowledged_deletion']:case['result']='fail'
        except Exception as e:
            case['error']=type(e).__name__;case['stage']=getattr(c,'waiting','startup')
            # Synthetic output only, private diagnostics; never dump arbitrary keyboard input.
            Path(config).with_name('last-bbs-private.log').write_bytes(c.transcript);Path(config).with_name('last-bbs-private.log').chmod(0o600)
        finally:c.close()
        results.append(case)
        if case['result']!='pass':break
    return results
