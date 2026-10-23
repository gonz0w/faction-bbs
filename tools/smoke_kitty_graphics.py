#!/usr/bin/env python3
"""Owned software PTYs: trusted Kitty preview, refusal, malformed/late and cleanup."""
import argparse,base64,re,secrets,tempfile,time
from pathlib import Path
from smoke_local_console import Console
from smoke_member_worker import Worker
QUERY='\x1b_Gi=17392,s=1,v=1,a=q,t=d,f=24;AAAA\x1b\\'
REPLY=b'\x1b_Gi=17392;OK\x1b\\'
DELETE=b'\x1b_Ga=d,d=I,i=17391,q=2;\x1b\\'
def validate(data):
    chunks=re.findall(rb'\x1b_G([^;]*);([^\x1b]*)\x1b\\',data)
    frames=[(h,b) for h,b in chunks if b and b!=b'AAAA']
    assert len(frames)==13, len(frames)
    assert frames[0][0]==b'a=T,t=d,f=24,s=160,v=80,i=17391,c=20,r=5,q=2,m=1'
    assert frames[-1][0]==b'm=0,q=2'
    assert all(len(h)+len(b)+5<=4096 and len(b)<=4000 for h,b in frames)
    pixels=base64.b64decode(b''.join(b for _,b in frames),validate=True)
    assert len(pixels)==38400
    return pixels
def smoke(binary,member):
    children=[];checks=[];password=secrets.token_hex(20)
    with tempfile.TemporaryDirectory(prefix='faction-kitty-') as root:
        board=Path(root)/'board'
        def login(mode):
            if mode=='worker':
                c=Worker(member,board);children.append(c);c.expect('[L]ogin [R]egister [Q]uit: ');c.send('l')
            else:
                c=Console(binary,board,'login',['--ansi'] if mode=='ansi' else [],columns=40,rows=18);children.append(c)
            c.expect('Handle: ');c.send('sysop');c.password(password)
            if mode=='worker':c.expect('Press Enter to continue: ');c.send('')
            c.expect('Command (then Enter): ' if mode=='plain' else 'Ctrl-C exits; Esc: Enter to recover.')
            c.send('t') if mode=='plain' else c.raw(b't')
            c.expect('[D]one: ');return c
        def done(c,mode):
            c.send('d')
            if mode!='plain':c.expect('Press Enter to return to menu: ');c.send('')
            c.expect('Command (then Enter): ' if mode=='plain' else 'Ctrl-C exits; Esc: Enter to recover.')
            c.send('q') if mode=='plain' else c.raw(b'q')
            c.expect('Logged out.');c.finish()
        try:
            c=Console(binary,board,'init');children.append(c);c.expect('Handle: ');c.send('sysop');c.password(password,'New password (12+ characters, hidden): ');c.password(password,'Confirm password (hidden): ');c.expect('Lobby created.');c.finish()
            c=login('plain');c.send('g');c.expect('ANSI disabled; probe refused.');c.expect('[D]one: ');assert QUERY.encode() not in c.transcript;done(c,'plain');checks.append('plain_never_probes_or_emits_bitmap')
            for mode in ['ansi','worker']:
                c=login(mode);start=len(c.transcript);c.send('g');c.expect(QUERY);c.raw(REPLY[:7]);c.raw(REPLY[7:]);c.expect('Kitty preview (original calibration artwork).');c.expect('[D]one: ')
                validate(c.transcript[start:]);c.send('c');c.expect('[D]one: ');assert c.transcript[start:].count(DELETE)>=2;done(c,mode);checks.append(mode+'_fragmented_positive_bounded_rgb_classic_cleanup')
                for reply in [b'',b'\x1b_Gi=17392;ENOTSUP:unsupported\x1b\\']:
                    c=login(mode);c.send('g');c.expect(QUERY)
                    if reply:c.raw(reply)
                    c.expect('Kitty unavailable; classic fallback.');c.expect('[D]one: ');assert b'a=T,' not in c.transcript;done(c,mode)
                checks.append(mode+'_unknown_or_refused_preserves_classic')
                for reply in [b'\x1b_Gi=17392;OK\x1b\\P\n', b'\x1b_Gi=17392;OK', b'\x1b]52;c;bad\x07']:
                    c=login(mode);c.send('g');c.expect(QUERY);c.raw(reply);c.finish(1);assert b'a=T,' not in c.transcript and b'Room ID: ' not in c.transcript
                c=login(mode);c.send('g');c.expect(QUERY);c.expect('[D]one: ');c.raw(REPLY+b'P\n');c.finish(1);assert b'Room ID: ' not in c.transcript
                checks.append(mode+'_malformed_incomplete_and_late_replies_cannot_post')
        finally:
            for c in children:c.close()
    print('Kitty software PTYs passed '+str(len(checks))+' controls; no GUI rendering certification.')
    return checks
if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('binary',type=Path);p.add_argument('member',type=Path);a=p.parse_args();smoke(a.binary.resolve(),a.member.resolve())
