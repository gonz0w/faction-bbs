#!/usr/bin/env python3
"""Run artwork probes serially in private macOS fixture apps; never user windows."""
import argparse
import fcntl
import json
import os
import plistlib
import shlex
import shutil
import signal
import stat
import subprocess
import tempfile
import time
from pathlib import Path
from run_real_terminal_profiles import processes


def run_case(a, root, label, font, route, app):
    name = label + '-' + route + ('-missing-font' if font == 'FactionDeliberatelyMissingFont25' else '-compact' if font == 'compact' else '')
    compact = font == 'compact'
    if compact:
        font = 'Menlo'
    columns, rows = (40, 18) if compact else (80, 24)
    private = root / name
    private.mkdir(mode=0o700)
    if label == 'ghostty':
        # Ghostty restores windows by bundle identity. Give every run its own.
        def copy_plain(src, dst):
            shutil.copyfile(src, dst)
            os.chmod(dst, stat.S_IMODE(os.stat(src).st_mode))
            return dst
        stage = private / 'Reference'
        shutil.copytree(app, stage, symlinks=True, copy_function=copy_plain)
        plist = stage / 'Contents/Info.plist'
        meta = plistlib.loads(plist.read_bytes())
        meta['CFBundleIdentifier'] = 'dev.faction.fixture.art25.' + root.name.rsplit('-', 1)[-1] + '.' + name
        plist.write_bytes(plistlib.dumps(meta))
        subprocess.run(['codesign', '--force', '--deep', '--sign', '-', '--timestamp=none', str(stage)], capture_output=True, check=True, timeout=60)
        app = private / 'Reference.app'
        stage.rename(app)
    meta = plistlib.loads((app / 'Contents/Info.plist').read_bytes())
    if not meta['CFBundleIdentifier'].startswith('dev.faction.fixture.'):
        raise ValueError('only isolated private fixture identities are allowed')
    executable = app / 'Contents/MacOS' / meta['CFBundleExecutable']
    if processes(executable):
        raise ValueError('private reference already running')
    output = private / 'result.json'
    gate = private / 'start-go'
    command = [str(a.python), str(Path(__file__).with_name('probe_artwork_terminal.py')),
               '--binary', str(a.binary), '--output', str(output), '--start-gate', str(gate)]
    if route == 'tmux':
        config = private / 'tmux.conf'
        config.write_text('set -g status off\n')
        command = [str(a.tmux), '-S', str(private / 'tmux.sock'), '-f', str(config), 'new-session', '-s', 'owned', *command]
    elif route == 'screen':
        command = ['/usr/bin/env', 'TERM=xterm-256color', '/usr/bin/screen', '-U', '-c', '/dev/null', '-S', 'faction-art-' + name, *command]
    wrapper = private / 'start.sh'
    status = private / 'status'
    wrapper.write_text('#!/bin/sh\n' + shlex.join(command) + ' 2>' + shlex.quote(str(private / 'probe-error.log'))
                       + '\nstatus=$?\nprintf "%s" "$status" > ' + shlex.quote(str(status)) + '\nexit "$status"\n')
    wrapper.chmod(0o700)
    if label == 'ghostty':
        argv = ['/usr/bin/open', '-n', '-a', str(app), '--args', '--config-default-files=false', '--window-save-state=never',
                '--initial-window=true', '--quit-after-last-window-closed=true', '--abnormal-command-exit-runtime=0',
                '--shell-integration=none', '--font-family=' + font, '--window-width=' + str(columns), '--window-height=' + str(rows),
                '--working-directory=' + str(private), '-e', str(wrapper)]
    elif label == 'kitty':
        argv = [str(executable), '--config', 'NONE', '--start-as=minimized', '--override', 'font_family=' + font,
                '--override', 'shell_integration=disabled', '--override', 'update_check_interval=0', '--override', 'remember_window_size=no',
                '--override', 'initial_window_width=' + str(columns) + 'c', '--override', 'initial_window_height=' + str(rows) + 'c', '--override',
                'confirm_os_window_close=0', '--override', 'macos_quit_when_last_window_closed=yes', '--directory', str(private), str(wrapper)]
    else:
        config = private / 'alacritty.toml'
        config.write_text('[general]\nlive_config_reload=false\n[window]\ndimensions={columns=' + str(columns) + ',lines=' + str(rows) + '}\n[font]\nnormal={family=' + json.dumps(font) + '}\n')
        argv = ['/usr/bin/open', '-n', '-g', '-a', str(app), '--args', '--config-file', str(config), '--working-directory', str(private), '-e', str(wrapper)]
    prior = subprocess.check_output([str(a.mac_control), 'front'], text=True).strip()
    child = None
    try:
        with (private / 'launch.log').open('wb') as log:
            if label == 'kitty':
                child = subprocess.Popen(argv, stdout=log, stderr=log, start_new_session=True)
            else:
                subprocess.run(argv, stdout=log, stderr=log, check=True, timeout=15)
        deadline = time.monotonic() + 120
        while time.monotonic() < deadline and not output.exists():
            if status.exists():
                break
            if gate.with_suffix('.ready').exists() and not gate.exists():
                for pid in processes(executable):
                    subprocess.run([str(a.mac_control), 'restore', prior, str(pid)], check=True, timeout=5)
                gate.write_text('go\n')
            time.sleep(.1)
        if not output.exists():
            raise RuntimeError('owned probe failed; inspect private logs: ' + name)
        result = json.loads(output.read_text())
        result.update(profile=name, font_requested=font, route=route, emulator=label,
                      viewport_requested=[rows, columns], tmux_status_disabled=route == 'tmux')
        print(name + ': ' + result['result'], flush=True)
        return result
    finally:
        if a.tmux and (private / 'tmux.sock').exists():
            subprocess.run([str(a.tmux), '-S', str(private / 'tmux.sock'), 'kill-server'], capture_output=True, timeout=5)
        for pid in processes(executable):
            os.kill(pid, signal.SIGTERM)
        deadline = time.monotonic() + 5
        while processes(executable) and time.monotonic() < deadline:
            time.sleep(.1)
        if processes(executable):
            raise RuntimeError('owned reference did not stop')
        if child:
            child.wait(timeout=5)


def main(a):
    root = Path(tempfile.mkdtemp(prefix='artwork-matrix-', dir=a.private_root))
    root.chmod(0o700)
    with (a.private_root / 'matrix.lock').open('w') as lock:
        fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
        results = []
        for label, app in [('ghostty', a.ghostty), ('kitty', a.kitty), ('alacritty', a.alacritty)]:
            for font, route in [('Menlo', 'direct'), ('FactionDeliberatelyMissingFont25', 'direct'),
                                ('Menlo', 'tmux'), ('Menlo', 'screen'), ('compact', 'direct')]:
                if route == 'tmux' and not a.tmux:
                    continue
                results.append(run_case(a, root, label, font, route, app))
                a.output.write_text(json.dumps(dict(schema=1, profiles=results), indent=2, sort_keys=True) + '\n')
                a.output.chmod(0o600)
    return results


if __name__ == '__main__':
    p = argparse.ArgumentParser(description=__doc__)
    for name in ['binary', 'private-root', 'output', 'mac-control', 'ghostty', 'kitty', 'alacritty']:
        p.add_argument('--' + name, type=Path, required=True)
    p.add_argument('--tmux', type=Path)
    p.add_argument('--python', type=Path, default=Path(os.sys.executable))
    args = p.parse_args()
    for name, value in vars(args).items():
        if isinstance(value, Path):
            setattr(args, name, value.resolve())
    main(args)
