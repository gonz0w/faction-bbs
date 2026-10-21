#!/usr/bin/env python3
"""Build opt-in NetBSD profile binaries; never installs setuid files or policy."""
import argparse,hashlib,os,shutil,subprocess,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]

def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output-dir',type=Path,required=True);p.add_argument('--target-dir',type=Path,required=True);a=p.parse_args()
    assert sys.platform.startswith('netbsd'),'native NetBSD only'
    out=a.output_dir.resolve();target=a.target_dir.resolve();out.mkdir(parents=True,exist_ok=True)
    cc=os.environ.get('CC','cc')
    flags=['-std=c11','-O2','-Wall','-Wextra','-Werror','-fstack-protector-strong','-D_FORTIFY_SOURCE=2','-fPIE']
    source=ROOT/'native/faction-isolation/startup.c';obj=out/'service_startup.o'
    subprocess.run([cc,*flags,'-c',str(source),'-o',str(obj)],check=True)
    # Cargo fingerprints the link argument, not external object contents.
    # A content-addressed path forces relink whenever native startup changes.
    identity=hashlib.sha256(obj.read_bytes()).hexdigest()
    sealed=out/('service_startup-'+identity+'.o');shutil.copyfile(obj,sealed);obj=sealed
    subprocess.run(['cargo','rustc','--target-dir',str(target),'-p','faction-service','--bin','faction-service','--release','--frozen','--','-C','link-arg='+str(obj),'-C','link-arg=-Wl,-z,now'],cwd=ROOT,check=True)
    shutil.copyfile(target/'release/faction-service',out/'faction-service');(out/'faction-service').chmod(0o700)
    subprocess.run([cc,*flags,'-DFACTION_TERMINAL=1','-pie','-Wl,-z,relro,-z,now',
        str(ROOT/'native/faction-tty/main.c'),str(ROOT/'crates/faction-unix/src/terminal_fd.c'),
        str(ROOT/'crates/faction-unix/src/process_security.c'),str(source),'-o',str(out/'faction-tty')],check=True)
    (out/'faction-tty').chmod(0o755)
    print('Built profile binaries. No privilege bits, root installation, mounts or firewall changes applied.')

if __name__=='__main__':main()
