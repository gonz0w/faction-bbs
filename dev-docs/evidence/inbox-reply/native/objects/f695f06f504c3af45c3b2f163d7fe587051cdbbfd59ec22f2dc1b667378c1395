#!/usr/bin/env python3
"""Build a NetBSD-only experimental SSH listener gate; never install or enable."""
import argparse,os,subprocess,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output-dir',type=Path,required=True);p.add_argument('--cc',default=os.environ.get('CC','cc'));a=p.parse_args()
    if not sys.platform.startswith('netbsd'):raise SystemExit('The SSH gate requires native NetBSD validation.')
    a.output_dir.mkdir(parents=True,exist_ok=True)
    common=[a.cc,'-std=c11','-O2','-Wall','-Wextra','-Werror','-fstack-protector-strong','-D_FORTIFY_SOURCE=2','-Wl,-z,relro,-z,now']
    source=str(ROOT/'native/faction-ssh-gate/gate.c')
    subprocess.run(common+['-fPIC','-shared','-Wl,-z,defs',source,'-o',str(a.output_dir/'faction-ssh-gate.so')],check=True)
    subprocess.run(common+['-DGATE_TOOL','-fPIE','-pie',source,'-o',str(a.output_dir/'faction-ssh-gate')],check=True)
    for name in ['faction-ssh-gate.so','faction-ssh-gate']:(a.output_dir/name).chmod(0o755)
    print('Built experimental listener-only gate and root diagnostic tool; no installation or host changes.')
if __name__=='__main__':main()
