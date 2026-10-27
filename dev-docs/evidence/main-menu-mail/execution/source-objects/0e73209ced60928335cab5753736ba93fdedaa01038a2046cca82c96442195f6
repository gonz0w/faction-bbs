#!/usr/bin/env python3
"""Build the unprivileged NetBSD rc startup helper, not a resident supervisor."""
import argparse
import os
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--cc', default=os.environ.get('CC', 'cc'))
    args = parser.parse_args()
    if not sys.platform.startswith('netbsd'):
        parser.exit(1, 'Build on the target NetBSD host.\n')
    args.output.parent.mkdir(parents=True, exist_ok=True)
    subprocess.run([args.cc, '-std=c11', '-O2', '-Wall', '-Wextra', '-Werror',
                    '-fstack-protector-strong', '-D_FORTIFY_SOURCE=2', '-fPIE', '-pie',
                    '-Wl,-z,relro,-z,now', str(ROOT / 'native/faction-daemon/main.c'),
                    '-o', str(args.output)], check=True)
    args.output.chmod(0o755)
    print('Built unprivileged NetBSD startup helper. No accounts or services changed.')


if __name__ == '__main__':
    main()
