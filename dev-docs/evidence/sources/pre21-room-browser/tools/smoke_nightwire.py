#!/usr/bin/env python3
"""Original artwork login routes on disposable local/worker PTYs; no listener."""
import argparse
import json
import secrets
import tempfile
from pathlib import Path
from smoke_local_console import Console
from smoke_member_worker import Worker
FOOTER='Ctrl-C exits; Esc: Enter to recover.'
PAGE=lambda n:f'Art page {n}/3. Enter: next; S+Enter: skip: '

def smoke(a):
    password=secrets.token_hex(20);children=[];checks=[]
    with tempfile.TemporaryDirectory(prefix='faction-nightwire-') as tmp:
        state=Path(tmp)/'board'
        def enter(c):c.expect('Handle: ');c.send('sysop');c.password(password)
        def finish(c,plain=False):
            if not plain:c.expect('Press Enter to continue: ');c.send('');c.expect(FOOTER);c.raw(b'q')
            else:c.expect('Command (then Enter): ');c.send('q')
            c.expect('Logged out.');c.finish()
        try:
            c=Console(a.binary,state,'init');children.append(c);c.expect('Handle: ');c.send('sysop');c.password(password,'New password (12+ characters, hidden): ');c.password(password,'Confirm password (hidden): ');c.expect('Lobby created.');c.finish()
            for name,pack,season,mode in [('long','nightwire',None,'wide'),('short','nightwire-short',None,'wide'),('autumn','nightwire','autumn','wide'),('winter','nightwire','winter','wide'),('compact','nightwire',None,'compact'),('plain','nightwire',None,'plain')]:
                opts=['--theme',str(a.themes/pack),'--plain' if mode=='plain' else '--ansi']
                if season:opts+=['--season',str(a.themes/'nightwire-seasons'/season)]
                c=Console(a.binary,state,'login',opts,columns=40 if mode=='compact' else 80,rows=18 if mode=='compact' else 24);children.append(c);enter(c)
                if mode=='wide':
                    c.expect('NIGHTWIRE / 01')
                    if name!='short':
                        c.expect(PAGE(1));c.send('');c.expect('NIGHTWIRE / 02');c.expect(PAGE(2));c.send('');c.expect('NIGHTWIRE / 03')
                    if season:c.expect(season.upper()+' / independent frequencies.')
                finish(c,mode=='plain')
                if mode!='wide':assert b'F A C T I O N' in c.transcript and b'Art page ' not in c.transcript
                if mode=='plain':assert b'\x1b[' not in c.transcript
                if name=='short':assert b'Art page ' not in c.transcript and b'NIGHTWIRE / 02' not in c.transcript
                checks.append(name+'_original_art_login')
            c=Worker(a.member,state,['--theme',str(a.themes/'nightwire')]);children.append(c);c.expect('[L]ogin [R]egister [Q]uit: ');c.send('l');enter(c);c.expect(PAGE(1));c.send('');c.expect(PAGE(2));c.send('s');finish(c);assert b'NIGHTWIRE / 03' not in c.transcript;checks.append('worker_original_art_skip_to_ascii')
            assert all(password.encode() not in c.transcript and b'SAUCE' not in c.transcript for c in children)
            if a.output:a.output.write_text(json.dumps({'result':'pass','count':len(checks),'checks':checks,'capacity_claim':False},indent=2)+'\n')
            print(f'Original Nightwire artwork passed {len(checks)} local/worker login routes.')
        finally:
            for c in children:c.close()

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('binary',type=Path);p.add_argument('member',type=Path);p.add_argument('--themes',type=Path,default=Path('themes'));p.add_argument('--output',type=Path)
    a=p.parse_args();a.binary=a.binary.resolve(strict=True);a.member=a.member.resolve(strict=True);a.themes=a.themes.resolve(strict=True);smoke(a)
