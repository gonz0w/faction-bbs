#!/usr/bin/env python3
"""Run a synthetic Rust/Lua boundary probe, or verify saved provenance only.

--run needs rustc, cc and a Lua 5.4/5.5 development prefix. Compiles in a private
temporary directory; no network, Lua standard libraries, wallets or board data.
No OS sandbox/whole-process-memory/NetBSD-support claim follows from these tests.
"""
import argparse
import hashlib
import json
import platform
import subprocess
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
EXPERIMENT = ROOT/'experiments/010-lua-boundary'
SAVED = ROOT/'dev-docs/research/results/010-lua-boundary.json'
ORACLES = {*(f'dispatch_{i}' for i in range(6)),
           *(f'reject_{n}' for n in ['memory','loop','controls','length','events','badapi']),
           'child_crash_reaped','child_deadline_reaped','healthy_after_failures',
           'forged_api_and_control_bytes_denied'}

def sources():
    paths = [EXPERIMENT/'worker.c',EXPERIMENT/'supervisor.rs',Path(__file__).resolve()]
    paths += sorted((EXPERIMENT/'fixtures').glob('*.lua'))
    return {str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest() for p in paths}

def capture(args, **kwargs):
    return subprocess.check_output(args,text=True,stderr=subprocess.STDOUT,timeout=30,**kwargs).strip()

def check(data):
    assert data['schema']==1 and data['sources']==sources(), 'saved source hash mismatch'
    assert {o['name'] for o in data['oracles']}==ORACLES and len(data['oracles'])==16, 'oracle set'
    assert all(o['result']=='pass' for o in data['oracles']), 'failed oracle'
    assert data['limits']=={'lua_heap_bytes':1048576,'hook_interval':1000,'hook_ticks':20,
                            'events':8,'body_bytes':120,'script_bytes':4096,'parent_seconds':2}, 'limits'

def run(prefix):
    include = prefix/'include/lua'
    if not (include/'lua.h').is_file():
        include = prefix/'include'
    assert (include/'lua.h').is_file(), 'Lua development headers required'
    library=prefix/'lib/liblua.dylib' if platform.system()=='Darwin' else prefix/'lib/liblua.so'
    assert library.is_file(), 'shared Lua library required'
    with tempfile.TemporaryDirectory(prefix='faction-lua-probe-') as work:
        work=Path(work);worker=work/'worker';supervisor=work/'supervisor'
        subprocess.run(['cc','-std=c11','-O2','-Wall','-Wextra','-Werror','-I'+str(include),
                        str(EXPERIMENT/'worker.c'),'-L'+str(prefix/'lib'),
                        '-Wl,-rpath,'+str(prefix/'lib'),'-llua','-o',str(worker)],check=True,timeout=30)
        subprocess.run(['rustc','--edition','2024','-O',str(EXPERIMENT/'supervisor.rs'),
                        '-o',str(supervisor)],check=True,timeout=30)
        output=capture([str(supervisor),str(worker),str(EXPERIMENT/'fixtures')])
        oracles=[]
        for line in output.splitlines():
            if not line: continue
            assert line.startswith('PASS '), 'unexpected output'
            name,*observation=line[5:].split(' ',1)
            oracles.append({'name':name,'result':'pass','observation':observation[0] if observation else ''})
    data={'schema':1,'date':'2026-10-19','scope':'synthetic developer-host Rust/Lua worker boundary; not NetBSD runtime or BBS',
          'environment':{'os':platform.system(),'release':platform.release(),'machine':platform.machine(),
                         'rustc':capture(['rustc','--version']), 'cc':capture(['cc','--version']).splitlines()[0],
                         'lua':capture([str(prefix/'bin/lua'),'-v']),
                         'lua_shared_library_sha256':hashlib.sha256(library.read_bytes()).hexdigest()},
          'limits':{'lua_heap_bytes':1048576,'hook_interval':1000,'hook_ticks':20,
                    'events':8,'body_bytes':120,'script_bytes':4096,'parent_seconds':2},
          'oracles':oracles,'sources':sources()}
    check(data);SAVED.write_text(json.dumps(data,indent=2)+'\n')
    print('16 Rust/Lua boundary oracles passed; native temporary build cleaned up')

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--run',action='store_true');p.add_argument('--check',action='store_true')
    p.add_argument('--lua-prefix',type=Path)
    args=p.parse_args()
    if args.run:
        if not args.lua_prefix:p.error('--run requires --lua-prefix')
        run(args.lua_prefix.resolve())
    else:
        check(json.loads(SAVED.read_text()));print('Saved Lua boundary evidence/provenance valid; runtime not rerun')
