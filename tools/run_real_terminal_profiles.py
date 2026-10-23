#!/usr/bin/env python3
"""Run fresh isolated macOS emulator copies, never existing user windows/config.

References must be prepared privately with distinct fixture bundle IDs. This
runner never installs tools, edits personal configuration or requests capture.
"""
import argparse,json,os,plistlib,signal,subprocess,tempfile,time,shlex,fcntl,shutil,stat
from pathlib import Path

def processes(executable):
    lines=subprocess.check_output(['ps','-axo','pid=,rss=,comm='],text=True).splitlines();out={}
    for line in lines:
        fields=line.strip().split(None,2)
        if len(fields)==3 and (fields[2]==str(executable) or fields[2].startswith(str(executable)+' ')):out[int(fields[0])]=int(fields[1])
    return out

def run_case(a,root,name,app,font,route):
    private=root/name;private.mkdir(mode=0o700);result=private/'result.json'
    if name.startswith('ghostty-'):
        # Swift/AppKit restoration is per bundle, even for a second -n process.
        # A fresh fixture-only identity prevents one profile restoring another.
        stage=private/'GhosttyFixture'
        def copy_plain(source,target):
            shutil.copyfile(source,target);os.chmod(target,stat.S_IMODE(os.stat(source).st_mode));return target
        shutil.copytree(app,stage,symlinks=True,copy_function=copy_plain)
        info=stage/'Contents/Info.plist';meta=plistlib.loads(info.read_bytes());meta['CFBundleIdentifier']='dev.faction.fixture.ghostty118.'+root.name.rsplit('-',1)[1]+'.'+name
        meta['SUEnableAutomaticChecks']=False;meta['SUAutomaticallyUpdate']=False
        info.write_bytes(plistlib.dumps(meta));subprocess.run(['codesign','--force','--deep','--sign','-','--timestamp=none',str(stage)],capture_output=True,check=True,timeout=30)
        app=private/'GhosttyFixture.app';stage.rename(app)
    with (app/'Contents/Info.plist').open('rb') as f:meta=plistlib.load(f)
    if not meta['CFBundleIdentifier'].startswith('dev.faction.fixture.'):raise ValueError('use only a private fixture bundle ID')
    executable=app/'Contents/MacOS'/meta['CFBundleExecutable'];before=set(processes(executable));rss=0
    if before:raise RuntimeError('close stale private fixture instances before another profile')
    gate=private/'start-go'
    command=[str(a.python),str(Path(__file__).with_name('probe_terminal_profile.py')),'--presenter',str(a.presenter.resolve()),'--output',str(result),'--start-gate',str(gate)]
    if a.bbs_config:command+=['--bbs-config',str(a.bbs_config.resolve())]
    if route=='tmux':
        if not a.tmux:raise ValueError('private tmux path required')
        command=[str(a.tmux.resolve()),'-S',str(private/'tmux.sock'),'-f','/dev/null','new-session','-s','owned',*command]
    elif route=='screen':command=['/usr/bin/env','TERM=xterm-256color',str(a.screen.resolve()),'-U','-c','/dev/null','-S','faction-owned-'+name,*command]
    status_file=private/'wrapper-status'
    start=private/'start.sh'
    start.write_text('#!/bin/sh\n'+shlex.join(command)+' 2>'+shlex.quote(str(private/'child-error.log'))+'\nstatus=$?\nprintf "%s" "$status" > '+shlex.quote(str(status_file))+'\nexit "$status"\n');start.chmod(0o700)
    command=[str(start)]
    label=name.split('-')[0]
    if label=='ghostty':
        args=['/usr/bin/open','-n','-a',str(app),'--args','--config-default-files=false','--window-save-state=never','--initial-window=true','--quit-after-last-window-closed=true','--abnormal-command-exit-runtime=0','--shell-integration=none','--font-family='+font,'--window-width=80','--window-height=24','--working-directory='+str(private),'-e',*command]
    elif label=='kitty':
        args=[str(executable),'--config','NONE','--start-as=minimized','--override','font_family='+font,'--override','shell_integration=disabled','--override','update_check_interval=0','--override','remember_window_size=no','--override','initial_window_width=80c','--override','initial_window_height=24c','--override','confirm_os_window_close=0','--override','macos_quit_when_last_window_closed=yes','--directory',str(private),*command]
    else:
        config=private/'alacritty.toml';config.write_text('[general]\nlive_config_reload=false\n[window]\ndimensions={columns=80,lines=24}\n[font]\nnormal={family='+json.dumps(font)+'}\n')
        args=['/usr/bin/open','-n','-g','-a',str(app),'--args','--config-file',str(config),'--working-directory',str(private),'-e',*command]
    prior=int(subprocess.check_output([str(a.mac_control),'front'],text=True)) if a.mac_control else None
    child=None
    try:
        with (private/'launch.log').open('wb') as log:
            if label=='kitty':child=subprocess.Popen(args,stdout=log,stderr=log,start_new_session=True)
            else:subprocess.run(args,stdout=log,stderr=log,check=True,timeout=15)
        deadline=time.monotonic()+(180 if a.bbs_config else 45)
        while time.monotonic()<deadline and not result.exists():
            if status_file.exists() and not result.exists():break
            current=processes(executable);rss=max(rss,max((v for k,v in current.items() if k not in before),default=0))
            if gate.with_suffix('.ready').exists() and not gate.exists():
                if prior is not None:
                    for pid in set(current)-before:subprocess.run([str(a.mac_control),'restore',str(prior),str(pid)],check=True,timeout=5)
                gate.write_text('go\n')
            time.sleep(.1)
        if not result.exists():return {'profile':name,'result':'unavailable','reason':'owned GUI agent did not produce a result','wrapper_status':status_file.read_text() if status_file.exists() else None,'gui_max_sampled_rss_kib':rss}
        d=json.loads(result.read_text());d.update(profile=name,font_requested=font,route=route,route_environment_override={'TERM':'xterm-256color'} if route=='screen' else {},gui_max_sampled_rss_kib=rss,gui_visual_pixels_captured=False)
        return d
    finally:
        if a.tmux and (private/'tmux.sock').exists():subprocess.run([str(a.tmux),'-S',str(private/'tmux.sock'),'kill-server'],stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL,timeout=5)
        for pid in set(processes(executable))-before:
            if pid in processes(executable):os.kill(pid,signal.SIGTERM)
        until=time.monotonic()+5
        while time.monotonic()<until and set(processes(executable))-before:time.sleep(.1)
        if child is not None:
            try:child.wait(timeout=5)
            except subprocess.TimeoutExpired:child.terminate();child.wait(timeout=5)

def main(a):
    root=Path(tempfile.mkdtemp(prefix='faction-terminal-matrix-',dir=a.private_root));root.chmod(0o700)
    lock=(a.private_root/"faction-terminal-profiles.lock").open("w")
    fcntl.flock(lock,fcntl.LOCK_EX|fcntl.LOCK_NB)
    results=[]
    for label,app in [('ghostty',a.ghostty),('kitty',a.kitty),('alacritty',a.alacritty)]:
        if not app:continue
        for route in (['direct','tmux','screen'] if not a.direct_only else ['direct']):
            name=label+'-'+route
            if a.only and name not in a.only:continue
            d=run_case(a,root,name,app.resolve(),'Menlo',route);results.append(d);print(name+': '+d['result'],flush=True)
        if a.missing_font and not a.only:
            name=label+'-missing-font';d=run_case(a,root,name,app.resolve(),'FactionDeliberatelyMissingFont118','direct');results.append(d);print(name+': '+d['result'],flush=True)
    a.output.write_text(json.dumps({'schema':1,'profiles':results,'private_fixture_removed':False},indent=2,sort_keys=True)+'\n');a.output.chmod(0o600)
    print('Results saved; private launch logs retained for diagnosis. No GUI pixel/physical/capacity claim.')
if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__)
    for name in ['ghostty','kitty','alacritty','tmux']:p.add_argument('--'+name,type=Path)
    p.add_argument('--mac-control',type=Path);p.add_argument('--bbs-config',type=Path);p.add_argument('--python',type=Path,default=Path(os.sys.executable));p.add_argument('--presenter',type=Path,required=True);p.add_argument('--screen',type=Path,default=Path('/usr/bin/screen'));p.add_argument('--private-root',type=Path,default=Path('/private/tmp'));p.add_argument('--output',type=Path,required=True);p.add_argument('--direct-only',action='store_true');p.add_argument('--missing-font',action='store_true');p.add_argument('--only',nargs='+');main(p.parse_args())
