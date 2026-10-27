#!/usr/bin/env python3
"""Optional owned GNU screen fixture: one Kitty query, no BBS or public listener."""
import argparse,json,os,shutil,subprocess,sys,tempfile
from pathlib import Path

def probe(binary):
    with tempfile.TemporaryDirectory(prefix='faction-screen-') as root:
        p=Path(root);p.chmod(0o700);(p/'sockets').mkdir(mode=0o700)
        (p/'probe.py').write_text('''import os,termios,select,time,json,sys
saved=termios.tcgetattr(0);raw=termios.tcgetattr(0);raw[3]&=~(termios.ICANON|termios.ECHO);raw[6][termios.VMIN]=1;raw[6][termios.VTIME]=0;termios.tcsetattr(0,termios.TCSANOW,raw)
try:
 os.write(1,b"\\x1b_Gi=17392,s=1,v=1,a=q,t=d,f=24;AAAA\\x1b\\\\");response=b"";end=time.monotonic()+.5
 while time.monotonic()<end:
  if select.select([0],[],[],.05)[0]:
   response+=os.read(0,32)
   if len(response)>32:raise RuntimeError("fixture response overflow")
 open(sys.argv[1],"w").write(json.dumps({"reply_hex":response.hex(),"only_inline_rgb_query":True,"graphics_enabled":False}))
finally:termios.tcsetattr(0,termios.TCSANOW,saved)
''')
        env=dict(os.environ,SCREENDIR=str(p/'sockets'),TERM='xterm-256color')
        v=subprocess.run([str(binary),'-v'],capture_output=True,text=True,timeout=5)
        assert v.stdout.startswith('Screen version')
        subprocess.run([str(binary),'-U','-c','/dev/null','-D','-m','-S','faction-owned',sys.executable,str(p/'probe.py'),str(p/'result.json')],env=env,capture_output=True,timeout=10,check=True)
        return {'program':'GNU screen','version':v.stdout.strip(),'exit_code':0,'result':json.loads((p/'result.json').read_text())}

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--binary',type=Path,default=shutil.which('screen'));p.add_argument('--output',type=Path);a=p.parse_args()
    if a.binary is None:raise SystemExit('GNU screen is unavailable; no multiplexer claim')
    result=probe(a.binary);raw=json.dumps(result,indent=2)+'\n'
    if a.output:a.output.write_text(raw)
    print(raw,end='')
