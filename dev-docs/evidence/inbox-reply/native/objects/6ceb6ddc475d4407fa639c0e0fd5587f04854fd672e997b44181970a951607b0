#!/usr/bin/env python3
"""Compile and compare local research session servers; never a BBS benchmark.

Local Unix sockets in private temp dirs; synthetic ALLOW is NOT authentication.
--run needs cc/rustc/go/ps, --check verifies saved workload/source provenance.
"""
import argparse
import hashlib
import json
import os
import platform
import resource
import shutil
import socket
import statistics
import subprocess
import tempfile
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DIR = ROOT / 'experiments/004-native-sessions'
RAW = ROOT / 'dev-docs/research/results/004-native-sessions.json'

def hashes():
    paths = [Path(__file__), *sorted(DIR.glob('session*')), DIR/'Cargo.toml', DIR/'Cargo.lock']
    return {p.name: hashlib.sha256(p.read_bytes()).hexdigest() for p in paths}

def recvline(sock):
    result = bytearray()
    while not result.endswith(b'\n'):
        part = sock.recv(1)
        if not part:
            raise RuntimeError('unexpected close')
        result.extend(part)
        if len(result) > 128:
            raise RuntimeError('unbounded response')
    return bytes(result)

def command(sock, request, expected):
    start = time.perf_counter()
    sock.sendall(request)
    if recvline(sock) != expected:
        raise RuntimeError('command result differs')
    return (time.perf_counter() - start) * 1000

def rss_kib(pid):
    return int(subprocess.check_output(['ps', '-o', 'rss=', '-p', str(pid)], text=True).strip())

def cpu_seconds(pid):
    text = subprocess.check_output(['ps', '-o', 'time=', '-p', str(pid)], text=True).strip()
    fields = text.split(':')
    result = 0.0
    for field in fields:
        result = result * 60 + float(field)
    return result

def connect(path):
    sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
    sock.settimeout(3)
    sock.connect(str(path))
    return sock

def one_run(binary, directory):
    path = directory / 's'
    path.unlink(missing_ok=True)
    process = subprocess.Popen([str(binary), str(path)], stdout=subprocess.DEVNULL, stderr=subprocess.PIPE)
    clients = []
    try:
        deadline = time.monotonic() + 5
        while not path.exists():
            if process.poll() is not None or time.monotonic() > deadline:
                raise RuntimeError('server failed to start')
            time.sleep(.01)
        zero = [rss_kib(process.pid) for _ in range(3)]
        for _ in range(250):
            client = connect(path); clients.append(client)
            if len(clients) == 1:
                command(client, b'READ\n', b'DENIED\n')
            command(client, b'ALLOW\n', b'OK\n')
        idle_rss = [rss_kib(process.pid) for _ in range(3)]
        start_cpu, start_wall = cpu_seconds(process.pid), time.monotonic()
        time.sleep(1)
        idle_cpu = max(0, cpu_seconds(process.pid) - start_cpu) / (time.monotonic() - start_wall) * 100
        extra = connect(path)
        try:
            if recvline(extra) != b'FULL\n': raise RuntimeError('connection cap differs')
        finally:
            extra.close()
        control = clients[0]
        for request in (b'READ\n', b'POST\n', b'CHAT\n'):
            command(control, b'REVOKE\n', b'OK\n')
            command(control, request, b'DENIED\n')
        command(control, b'ALLOW\n', b'OK\n')
        control.sendall(b'RE')
        command(control, b'AD\n', b'ROOM synthetic local message\n')
        command(control, b'UNKNOWN\n', b'ERROR\n')
        latency = {'READ': [], 'POST': [], 'CHAT': []}
        replies = {'READ': b'ROOM synthetic local message\n',
                   'POST': b'POSTED synthetic only\n', 'CHAT': b'CHAT synthetic only\n'}
        active_cpu, active_wall = cpu_seconds(process.pid), time.monotonic()
        active_rss = []
        for i in range(500):
            operation = 'READ' if i % 10 < 6 else ('POST' if i % 10 < 8 else 'CHAT')
            client = clients[i % 25]
            latency[operation].append(command(client, operation.encode() + b'\n', replies[operation]))
            if i % 100 == 0: active_rss.append(rss_kib(process.pid))
        active_time = time.monotonic() - active_wall
        active_cpu_delta = max(0, cpu_seconds(process.pid) - active_cpu)
        # Flood one non-reading peer; an independent peer must still progress.
        stalled = clients[249]
        stalled.settimeout(.01)
        try: stalled.sendall(b'READ\n' * 20000)
        except (TimeoutError, BrokenPipeError, ConnectionResetError): pass
        slow_peer_ms = command(clients[1], b'READ\n', replies['READ'])
        # A too-long command closes only its session. Avoid buffered earlier output.
        bad = clients[248]
        bad.sendall(b'x' * 1025 + b'\n')
        try:
            if bad.recv(1) != b'': raise RuntimeError('oversized command was not closed')
        except ConnectionResetError: pass
        return {'connections': 250, 'operations': 500,
                'zero_rss_kib': zero, 'connected_rss_kib': idle_rss,
                'active_rss_kib': active_rss, 'idle_cpu_one_core_percent': idle_cpu,
                'active_cpu_s': active_cpu_delta, 'active_wall_s': active_time,
                'latency_ms': latency, 'slow_peer_progress_ms': slow_peer_ms,
                'oracles': ['approval_flag_denial', 'revocation_flag_denial', 'fragmented_command',
                            'unknown_command', 'connection_cap', 'oversized_close', 'slow_peer_progress']}
    finally:
        for client in clients: client.close()
        process.terminate()
        try: process.communicate(timeout=3)
        except subprocess.TimeoutExpired: process.kill(); process.communicate()
        path.unlink(missing_ok=True)

def check(raw):
    if raw['source_hashes'] != hashes() or len(raw['candidates']) != 4:
        raise ValueError('native probe provenance/count stale')
    for candidate in raw['candidates']:
        if len(candidate['runs']) != 5 or candidate['binary_bytes'] <= 0:
            raise ValueError('missing native observations')
        for run in candidate['runs']:
            if run['connections'] != 250 or run['operations'] != 500 or len(run['oracles']) != 7:
                raise ValueError('native workload/oracle mismatch')
            if {k:len(v) for k,v in run['latency_ms'].items()} != {'READ':300,'POST':100,'CHAT':100}:
                raise ValueError('native operation count mismatch')

def main():
    p = argparse.ArgumentParser(description=__doc__)
    g = p.add_mutually_exclusive_group(required=True)
    g.add_argument('--run', action='store_true'); g.add_argument('--check', action='store_true')
    args = p.parse_args()
    if args.run:
        soft, hard = resource.getrlimit(resource.RLIMIT_NOFILE)
        resource.setrlimit(resource.RLIMIT_NOFILE, (min(4096, hard) if hard != resource.RLIM_INFINITY else 4096, hard))
        raw = {'schema':1, 'date':'2026-10-19', 'source_hashes':hashes(),
               'environment':f'{platform.system()} / {platform.machine()}', 'candidates':[]}
        with tempfile.TemporaryDirectory(prefix='faction-native-', dir='/tmp') as temp:
            directory = Path(temp)
            mio_dir = directory/'mio'; mio_dir.mkdir()
            for name in ('Cargo.toml','Cargo.lock','session_mio.rs'):
                shutil.copyfile(DIR/name,mio_dir/name)
            mio_env = dict(os.environ,CARGO_HOME=str(directory/'cargo-home'),CARGO_TARGET_DIR=str(directory/'cargo-target'))
            subprocess.run(['cargo','build','--locked','--release','--manifest-path',str(mio_dir/'Cargo.toml')],env=mio_env,check=True,capture_output=True)
            builds = [('C', ['cc','--version'], ['cc','-std=c11','-O2','-Wall','-Wextra','-Werror',str(DIR/'session.c')]),
                      ('Rust', ['rustc','--version'], ['rustc','--edition=2024','-C','opt-level=2','-C','panic=abort','-C','strip=symbols',str(DIR/'session.rs')]),
                      ('Go', ['go','version'], ['go','build','-trimpath','-ldflags=-s -w'])]
            for name, version, cmd in builds:
                binary = directory / name.lower()
                env = dict(os.environ, GOCACHE=str(directory/'go-cache'), CGO_ENABLED='0', GOTOOLCHAIN='local', GO111MODULE='off')
                build = cmd + ['-o', str(binary)] + ([str(DIR/'session.go')] if name == 'Go' else [])
                subprocess.run(build, env=env, check=True, capture_output=True)
                candidate = {'name':name, 'compiler':subprocess.check_output(version,text=True).splitlines()[0],
                             'binary_bytes':binary.stat().st_size, 'runs':[]}
                for _ in range(5): candidate['runs'].append(one_run(binary,directory))
                raw['candidates'].append(candidate)
                print(f'{name}: five session runs complete',flush=True)
            binary=directory/'cargo-target/release/session-mio'
            candidate={'name':'Rust+Mio','compiler':subprocess.check_output(['rustc','--version'],text=True).strip(),
                       'binary_bytes':binary.stat().st_size,'runs':[]}
            for _ in range(5): candidate['runs'].append(one_run(binary,directory))
            raw['candidates'].append(candidate)
            print('Rust+Mio: five session runs complete',flush=True)
            metadata=json.loads(subprocess.check_output(['cargo','metadata','--locked','--format-version','1',
                '--filter-platform',subprocess.check_output(['rustc','-vV'],text=True).split('host: ')[1].splitlines()[0],
                '--manifest-path',str(mio_dir/'Cargo.toml')],env=mio_env,text=True))
            active={node['id'] for node in metadata['resolve']['nodes']}
            raw['mio_dependencies']=[]
            for package in metadata['packages']:
                if package['id'] not in active or package['name']=='faction-session-research': continue
                package_dir=Path(package['manifest_path']).parent
                prohibited=[p.name for p in package_dir.rglob('*') if p.is_file() and
                    (p.suffix.lower() in {'.js','.jsx','.mjs','.cjs','.ts','.tsx'} or p.name=='package.json')]
                if prohibited: raise RuntimeError('JS/TS found in selected dependency package')
                raw['mio_dependencies'].append({'name':package['name'],'version':package['version'],
                                                'license':package['license'],'source_js_file_scan':'none found'})
        RAW.write_text(json.dumps(raw,indent=2)+'\n')
    else: raw=json.loads(RAW.read_text())
    check(raw)
    for candidate in raw['candidates']:
        rss=statistics.median(max(r['active_rss_kib']) for r in candidate['runs'])
        latency=[v for r in candidate['runs'] for v in r['latency_ms']['READ']]
        print(f"{candidate['name']}: {candidate['binary_bytes']} binary bytes; median sampled active RSS {rss} KiB; median READ {statistics.median(latency):.3f} ms")
    print('Local cleartext synthetic sessions only; no authentication, durable posts, fan-out, TCP or crypto.')

if __name__ == '__main__': main()
