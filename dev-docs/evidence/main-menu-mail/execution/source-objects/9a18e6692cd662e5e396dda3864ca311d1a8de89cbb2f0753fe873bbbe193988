#!/usr/bin/env python3
"""Optional C-only tmux reference build in a NEW private directory; not runtime."""
import argparse,hashlib,json,os,subprocess,tarfile,urllib.request
from pathlib import Path
SOURCES={
 'libevent':('2.1.13-stable','https://github.com/libevent/libevent/releases/download/release-2.1.13-stable/libevent-2.1.13-stable.tar.gz','f7e9383b8c0baa81b687e5b5eecc01beefaf1b19b64151d95ed61647fe7a315c'),
 'tmux':('3.8','https://github.com/tmux/tmux/releases/download/3.8/tmux-3.8.tar.gz','e79c699c7e949dccd0a4a125e17b8d1261e311b16979dbc8eb34542f3966d82e'),
}
def build(a):
    a.work_dir.mkdir(mode=0o700);prefix=a.work_dir/'prefix';log=a.work_dir/'build.log';manifest={}
    for name,(version,url,digest) in SOURCES.items():
        raw=urllib.request.urlopen(url,timeout=30).read(8*1024*1024+1)
        if len(raw)>8*1024*1024 or hashlib.sha256(raw).hexdigest()!=digest:raise ValueError('reference archive identity mismatch')
        archive=a.work_dir/(name+'.tar.gz');archive.write_bytes(raw)
        with tarfile.open(archive) as src:src.extractall(a.work_dir,filter='data')
        manifest[name]={'version':version,'url':url,'archive_sha256':digest}
    forbidden={'.js','.jsx','.mjs','.cjs','.ts','.tsx','.mts','.cts'}
    if any(f.is_file() and f.suffix.lower() in forbidden for f in a.work_dir.rglob('*')):raise ValueError('prohibited reference source filename')
    def run(argv,cwd,env=None):
        with log.open('ab') as out:subprocess.run(argv,cwd=cwd,env=env,stdout=out,stderr=subprocess.STDOUT,timeout=300,check=True)
    event=a.work_dir/'libevent-2.1.13-stable'
    run(['./configure','--prefix='+str(prefix),'--disable-openssl','--disable-shared','--enable-static','--disable-samples','--disable-libevent-regress'],event)
    for command in [['make','-j',str(a.jobs)],['make','install']]:run(command,event)
    env=dict(os.environ,CPPFLAGS='-I'+str(prefix/'include')+' -I'+str(a.ncurses_prefix/'include'),LDFLAGS='-L'+str(prefix/'lib')+' -L'+str(a.ncurses_prefix/'lib'),PKG_CONFIG_PATH=str(prefix/'lib/pkgconfig')+':'+str(a.ncurses_prefix/'lib/pkgconfig'))
    tmux=a.work_dir/'tmux-3.8';run(['./configure','--prefix='+str(prefix),'--disable-utf8proc','--disable-jemalloc'],tmux,env)
    for command in [['make','-j',str(a.jobs)],['make','install']]:run(command,tmux,env)
    binary=prefix/'bin/tmux';subprocess.run([str(binary),'-V'],check=True)
    manifest['no_js_source_scan']=True;manifest['jemalloc']=manifest['utf8proc']=False;manifest['binary_sha256']=hashlib.sha256(binary.read_bytes()).hexdigest();(a.work_dir/'reference.json').write_text(json.dumps(manifest,indent=2,sort_keys=True)+'\n')
if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--work-dir',type=Path,required=True);p.add_argument('--ncurses-prefix',type=Path,required=True);p.add_argument('--jobs',type=int,choices=range(1,9),default=2);a=p.parse_args();a.work_dir=a.work_dir.resolve();a.ncurses_prefix=a.ncurses_prefix.resolve();build(a)
