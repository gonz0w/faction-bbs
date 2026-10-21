#!/usr/bin/env python3
"""Exercise the actual Lua evaluator with root-owned disposable script fixtures.

Run as an unprivileged user. --scripts must contain root-owned copies of the
checked-in examples and tools/fixtures/lua-doors/*.lua. This tool does not install
accounts or change host policy; a same-UID fixture tests runtime/protocol bounds.
Separate game/BBS UID integration is covered by the actual NetBSD SSH rehearsal.
"""
import argparse
import json
import os
from pathlib import Path
import socket
import subprocess
import tempfile
import time


def run(binary, scripts):
    assert os.geteuid()!=0,'run the evaluator as an unprivileged user'
    checks=[]
    with tempfile.TemporaryDirectory(prefix='faction-door-check-') as directory:
        endpoint=Path(directory)/'door.sock'
        argv=[str(binary),'--socket',str(endpoint),'--owner-uid',str(os.getuid()),'--scripts',str(scripts)]
        server=subprocess.Popen(argv,stdin=subprocess.DEVNULL,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
        try:
            deadline=time.monotonic()+3
            while not endpoint.exists():
                assert server.poll() is None,'evaluator failed startup'
                assert time.monotonic()<deadline,'evaluator startup timed out'
                time.sleep(.01)
            def request(game,state='',input='',raw=None):
                with socket.socket(socket.AF_UNIX,socket.SOCK_STREAM) as s:
                    s.settimeout(3);s.connect(str(endpoint))
                    s.sendall(raw if raw is not None else f'{game}|{state}|{input}\n'.encode())
                    s.shutdown(socket.SHUT_WR);out=b''
                    while True:
                        part=s.recv(4096)
                        if not part:break
                        out+=part;assert len(out)<=4096
                    return out.decode('ascii')
            menu=request('doors');assert 'After Hours Arcade' in menu and 'game:signal-hunt' in menu
            checks.append('real_lua_menu')
            hunt=request('signal-hunt');state=next(x[6:] for x in hunt.splitlines() if x.startswith('STATE|'))
            secret=state.split(':')[0];won=request('signal-hunt',state,secret);assert 'LOCKED ON!' in won and 'SCORE|700\n' in won
            checks.append('signal_hunt_win_and_state_roundtrip')
            assert 'SCORE|500\n' in request('signal-hunt','42:5','42')
            assert 'SCORE|0\n' in request('signal-hunt','42:1','41')
            assert 'SCORE|' not in request('signal-hunt','42:5','q')
            assert 'SCORE|' not in request('signal-hunt','42:5','bad')
            assert 'SCORE|' not in request('signal-hunt')
            checks.append('hunt_scoring_win_loss_abandon_and_invalid_input')
            guide=request('arcade-guide');assert 'arcade guide' in guide and 'seven guesses' in guide
            checks.append('installed_guide_explains_both_games')
            state=''
            for _ in range(6):
                result=request('neon-salvage',state,'S' if state else '')
                state=next(x[6:] for x in result.splitlines() if x.startswith('STATE|'))
            assert 'dock before dawn' in result and 'SCORE|' in result
            checks.append('neon_salvage_complete_run')
            landed=request('neon-salvage','2:4:17','L');assert 'SCORE|17\n' in landed
            assert 'SCORE|' not in request('neon-salvage','1:4:0','S')
            checks.append('salvage_early_landing_scores_only_completed_turn')
            for game in ['loop','memory','controls','bad-api','partial','duplicate-score','negative-score','active-score']:

                assert request(game)=='',game+' was not rejected'
                checks.append('reject_'+game)
            began=time.monotonic();assert request('native-stall')==''
            elapsed=time.monotonic()-began;assert .15<elapsed<2,'native-library stall was not bounded'
            checks.append('native_library_stall_killed_and_reaped')
            for raw in [b'../private||\n',b'doors|bad\x00state|\n',b'doors|bad\x1bstate|\n']:
                assert request('',raw=raw)==''
            checks.append('forged_paths_and_control_bytes_rejected')
            assert 'After Hours Arcade' in request('doors')
            checks.append('healthy_after_all_failures')
        finally:
            server.terminate()
            try:server.wait(timeout=3)
            except subprocess.TimeoutExpired:server.kill();server.wait();raise AssertionError('evaluator did not stop')
        assert not endpoint.exists(),'socket not removed on orderly shutdown'
        checks.append('orderly_shutdown_and_socket_cleanup')
    return {'schema':1,'result':'pass','checks':checks,'native_stall_seconds':round(elapsed,3)}


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--binary',type=Path,required=True);p.add_argument('--scripts',type=Path,required=True)
    p.add_argument('--output',type=Path)
    a=p.parse_args();result=run(a.binary.resolve(),a.scripts.resolve())
    if a.output:a.output.write_text(json.dumps(result,indent=2)+'\n')
    print(f"Lua evaluator passed {len(result['checks'])} native runtime/protocol/game/failure checks.")
