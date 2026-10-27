#!/usr/bin/env python3
"""Build the NetBSD-only bounded C SSH admission launcher; no installer side effects."""
import argparse,os,subprocess,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,required=True);p.add_argument('--cc',default=os.environ.get('CC','cc'));a=p.parse_args()
    if not sys.platform.startswith('netbsd'):raise SystemExit('The admission launcher targets native NetBSD only.')
    a.output.parent.mkdir(parents=True,exist_ok=True)
    subprocess.run([a.cc,'-std=c11','-O2','-Wall','-Wextra','-Werror','-fstack-protector-strong','-D_FORTIFY_SOURCE=2','-fPIE','-pie','-Wl,-z,relro,-z,now',str(ROOT/'native/faction-admission/main.c'),'-o',str(a.output)],check=True)
    a.output.chmod(0o755)
    print('Built experimental NetBSD SSH admission launcher; no installation or configuration changed.')
if __name__=='__main__':main()
