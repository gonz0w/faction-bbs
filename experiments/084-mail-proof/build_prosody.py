#!/usr/bin/env python3
"""Build inspected Prosody 13.0.7 in a fresh disposable NetBSD VM only.

Requires existing native Lua-5.4 development/runtime dependencies. No service is
enabled; no operating-system or maintained BBS configuration is changed.
"""
import argparse
import hashlib
import io
import json
from pathlib import Path
import subprocess
import tarfile
import urllib.request

SOURCE = 'https://prosody.im/downloads/source/prosody-13.0.7.tar.gz'
SHA256 = 'f671df3c7535c7269afc89579ea26fc13d1006b40ef5df4ccb71a560197f151b'
FORBIDDEN = {'.js','.jsx','.ts','.tsx','.mjs','.cjs','.mts','.cts'}
MANIFESTS = {'package.json','package-lock.json','npm-shrinkwrap.json','yarn.lock','pnpm-lock.yaml','bun.lock','bun.lockb','deno.json','deno.jsonc'}


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--admin-argv', type=Path, required=True)
    p.add_argument('--private-log', type=Path, required=True)
    p.add_argument('--disposable-synthetic-fixture', action='store_true', required=True)
    a = p.parse_args()
    admin = json.loads(a.admin_argv.read_text())
    def remote(cmd, data=None):
        x = subprocess.run(admin+['set -e; PATH=/usr/sbin:/sbin:/usr/bin:/bin:/usr/pkg/bin; export PATH; '+cmd], input=data, capture_output=True, timeout=180)
        with a.private_log.open('ab') as log: log.write(x.stdout+x.stderr)
        if x.returncode: raise RuntimeError('native fixture build failed; inspect private log')
    raw = urllib.request.urlopen(SOURCE, timeout=60).read()
    assert hashlib.sha256(raw).hexdigest() == SHA256
    with tarfile.open(fileobj=io.BytesIO(raw)) as t:
        for member in t.getmembers():
            name = Path(member.name)
            assert not name.is_absolute() and '..' not in name.parts
            assert name.suffix.lower() not in FORBIDDEN and name.name not in MANIFESTS
    remote('test -f /etc/faction-disposable-test; test ! -e /root/faction-mail-source; test ! -e /usr/local/faction-mail-probe; mkdir /root/faction-mail-source; tar -xzf - -C /root/faction-mail-source', raw)
    patch = Path(__file__).with_name('prosody-netbsd-lua54.patch').read_bytes()
    remote('cat > /root/faction-mail-source/netbsd-lua54.patch', patch)
    remote('cd /root/faction-mail-source/prosody-13.0.7; patch -p1 < ../netbsd-lua54.patch; ./configure --ostype=netbsd --no-example-certs --lua-version=5.4 --lua-suffix=5.4 --prefix=/usr/pkg; make -j 2 CFLAGS="-Wall -fPIC -D_NETBSD_SOURCE -I/usr/pkg/include -I/usr/pkg/include/lua-5.4"; mkdir -p /usr/local; cp -R . /usr/local/faction-mail-probe; chmod -R a+rX /usr/local/faction-mail-probe')
    print('Inspected Prosody 13.0.7 built natively; no service enabled.')


if __name__ == '__main__': main()
