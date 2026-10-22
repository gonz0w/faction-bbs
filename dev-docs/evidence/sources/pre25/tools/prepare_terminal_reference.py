#!/usr/bin/env python3
"""Optional macOS reference copies. No installs or personal config changes.

Ghostty is copied from an explicit installed app; Kitty/Alacritty DMGs are pinned
release artifacts. Private bundle identity/signing changes are NOT stock-package
certification. This fixture never downloads code without checking its digest.
"""
import argparse,hashlib,json,os,plistlib,shutil,stat,subprocess,urllib.request
from pathlib import Path
from terminal_reference_identity import compare
ARTIFACTS={
 'kitty':('0.49.2','https://github.com/kovidgoyal/kitty/releases/download/v0.49.2/kitty-0.49.2.dmg','e524b894145d89cb76b584e3a9691196ae56bfa31d51384a44aa774252b7cef7',77825600,'kitty.app'),
 'alacritty':('0.17.0','https://github.com/alacritty/alacritty/releases/download/v0.17.0/Alacritty-v0.17.0.dmg','ad8d7de35fb38e43184776cac6dfee05ca325caa0b6639a06a55e54e4b026620',6136240,'Alacritty.app'),
}
def prepare(a):
    a.work_dir.mkdir(mode=0o700);mounted=False;mount=a.work_dir/'mount';metadata={'kind':a.kind,'product_runtime_dependency':False}
    try:
        if a.kind=='ghostty':
            if not a.app:raise ValueError('explicit installed Ghostty app required')
            source=a.app.resolve();metadata['source']='explicit local installed application'
        else:
            version,url,digest,size,name=ARTIFACTS[a.kind];archive=a.work_dir/'reference.dmg'
            with urllib.request.urlopen(url,timeout=45) as src,archive.open('wb') as dst:
                count=0
                while chunk:=src.read(1024*1024):
                    count+=len(chunk)
                    if count>size:raise ValueError('reference artifact too large')
                    dst.write(chunk)
            if count!=size or hashlib.sha256(archive.read_bytes()).hexdigest()!=digest:raise ValueError('reference artifact identity mismatch')
            mount.mkdir();subprocess.run(['hdiutil','attach','-readonly','-nobrowse','-noautoopen','-mountpoint',str(mount),str(archive)],stdout=subprocess.DEVNULL,check=True,timeout=60);mounted=True;source=mount/name
            metadata.update(version=version,url=url,archive_sha256=digest)
        stage=a.work_dir/'ReferenceFixture'
        def copy_plain(src,dst):shutil.copyfile(src,dst);os.chmod(dst,stat.S_IMODE(os.stat(src).st_mode));return dst
        def ignore(directory,names):return {'doc'} if a.kind=='kitty' and Path(directory)==source/'Contents/Resources' else set()
        shutil.copytree(source,stage,symlinks=True,copy_function=copy_plain,ignore=ignore)
        plist=stage/'Contents/Info.plist';info=plistlib.loads(plist.read_bytes());executable=info['CFBundleExecutable'];metadata['bundle_version']=info.get('CFBundleShortVersionString')
        info['CFBundleIdentifier']='dev.faction.fixture.'+a.kind+'118';info['SUEnableAutomaticChecks']=False;info['SUAutomaticallyUpdate']=False;plist.write_bytes(plistlib.dumps(info))
        forbidden={'.js','.jsx','.mjs','.cjs','.ts','.tsx','.mts','.cts'}
        if any(p.is_file() and p.suffix.lower() in forbidden for p in stage.rglob('*')):raise ValueError('prohibited reference filename')
        subprocess.run(['codesign','--force','--deep','--sign','-','--timestamp=none',str(stage)],capture_output=True,check=True,timeout=60)
        metadata.update(compare(source/'Contents/MacOS'/executable,stage/'Contents/MacOS'/executable));metadata['no_js_selected_files']=True;metadata['documentation_excluded']=a.kind=='kitty'
        target=a.work_dir/(a.kind+'Fixture.app');stage.rename(target);metadata['fixture_app']=target.name;(a.work_dir/'reference.json').write_text(json.dumps(metadata,indent=2,sort_keys=True)+'\n');print(str(target))
    finally:
        if mounted:subprocess.run(['hdiutil','detach',str(mount)],stdout=subprocess.DEVNULL,check=True,timeout=30)
if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--kind',choices=['ghostty','kitty','alacritty'],required=True);p.add_argument('--work-dir',type=Path,required=True);p.add_argument('--app',type=Path);a=p.parse_args();a.work_dir=a.work_dir.resolve();prepare(a)
