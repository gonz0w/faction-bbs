#!/usr/bin/env python3
"""Build pinned INN inews without installation. Optional reference tool only.

Requires Python 3.12+, C make/build tools and Perl used by upstream configure.
Pass an explicitly downloaded official archive and a new private output directory.
"""
import argparse, hashlib, json, os, subprocess, sys, tarfile
from pathlib import Path
VERSION='2.7.4'
URL='https://downloads.isc.org/isc/inn/inn-2.7.4.tar.gz'
SHA256='80fc7e801e1996cb17bb430d104dd9c5b29aa571fd4b29a9c821b4fadb051d3b'
FORBIDDEN={'.js','.jsx','.mjs','.cjs','.ts','.tsx','.mts','.cts'}

def runtime_artifacts(root):
    paths=[root/'frontends/inews']
    if (root/'frontends/.libs/inews').is_file():paths.append(root/'frontends/.libs/inews')
    paths += [p for p in (root/'lib/.libs').glob('libinn*') if p.is_file() and not p.is_symlink()]
    return {str(p.relative_to(root)):hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(paths)}

def build(archive, work):
    if sys.version_info<(3,12):raise SystemExit('Optional reference build requires Python 3.12+ for safe tar extraction')
    assert hashlib.sha256(archive.read_bytes()).hexdigest()==SHA256,'INN archive differs from reviewed source'
    work.mkdir(mode=0o700);work=work.resolve()
    with tarfile.open(archive) as source:
        for entry in source.getmembers():
            path=Path(entry.name)
            assert not path.is_absolute() and '..' not in path.parts
            assert path.suffix.lower() not in FORBIDDEN and path.name not in {'package.json','yarn.lock','package-lock.json','pnpm-lock.yaml'}
        source.extractall(work,filter='data')
    root=work/('inn-'+VERSION)
    options=['--prefix='+str(work/'install'),'--without-perl','--without-python','--without-openssl','--without-canlock','--without-sasl']
    with (work/'build.log').open('wb') as log:
        for command in [['./configure',*options],['make','-C','include','-j2'],['make','-C','lib','-j2'],['make','-C','frontends','inews','-j2']]:
            result=subprocess.run(command,cwd=root,stdout=log,stderr=subprocess.STDOUT,timeout=300)
            if result.returncode:
                log.flush()
                print((work/'build.log').read_text(errors='replace')[-12000:],file=sys.stderr)
                raise RuntimeError('INN reference build failed; inspect private build.log')
    binary=root/'frontends/inews'
    assert binary.is_file() and os.access(binary,os.X_OK)
    record={'version':VERSION,'source_url':URL,'archive_sha256':SHA256,
            'license_sha256':hashlib.sha256((root/'LICENSE').read_bytes()).hexdigest(),
            'binary_sha256':hashlib.sha256(binary.read_bytes()).hexdigest(),
            'runtime_artifacts':runtime_artifacts(root),
            'configure_options':[option for option in options if not option.startswith('--prefix=')],
            'source_no_js_scan':True,'installed_service':False,'privileged_install':False}
    (work/'reference.json').write_text(json.dumps(record,indent=2,sort_keys=True)+'\n')
    print('Built original INN '+VERSION+' inews in private output directory; no installation or service.')

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--archive',type=Path,required=True);p.add_argument('--work-dir',type=Path,required=True)
    args=p.parse_args();build(args.archive.resolve(),args.work_dir)
