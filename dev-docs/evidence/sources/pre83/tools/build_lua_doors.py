#!/usr/bin/env python3
"""Build the optional separate-process Lua door evaluator (Lua 5.4 or 5.5)."""
import argparse
import os
from pathlib import Path
import subprocess
import sys
ROOT = Path(__file__).resolve().parents[1]


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--output', type=Path, required=True)
    p.add_argument('--include', type=Path, default=Path('/usr/include'))
    p.add_argument('--library', default='lua', help='linker library name, e.g. lua or lua5.4')
    p.add_argument('--library-dir', type=Path)
    p.add_argument('--cc', default=os.environ.get('CC', 'cc'))
    a = p.parse_args()
    a.output.parent.mkdir(parents=True, exist_ok=True)
    flags = ['-std=c11', '-O2', '-Wall', '-Wextra', '-Werror', '-fstack-protector-strong']
    if sys.platform == 'darwin':
        flags += ['-Wno-deprecated-declarations']
    else:
        flags += ['-D_GNU_SOURCE', '-D_FORTIFY_SOURCE=2', '-fPIE', '-pie', '-Wl,-z,relro,-z,now']
    if a.library_dir:
        flags += ['-L' + str(a.library_dir), '-Wl,-rpath,' + str(a.library_dir)]
    subprocess.run([a.cc, *flags, '-I' + str(a.include), str(ROOT/'native/faction-doors/main.c'),
                    '-l' + a.library, '-o', str(a.output)], check=True)
    a.output.chmod(0o755)
    print('Built optional Lua door evaluator. No installation or service changes.')


if __name__ == '__main__':
    main()
