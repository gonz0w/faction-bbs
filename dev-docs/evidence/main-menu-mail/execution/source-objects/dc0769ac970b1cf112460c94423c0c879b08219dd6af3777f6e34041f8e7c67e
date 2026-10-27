#!/usr/bin/env python3
"""Compile/run bounded C allocation probe, or verify saved source/result provenance."""
import argparse
import hashlib
import json
import platform
import subprocess
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / 'experiments/028-session-buffers/probe.c'
RAW = ROOT / 'dev-docs/research/results/028-buffer-probe.json'

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    group = parser.add_mutually_exclusive_group(required=True)
    group.add_argument('--run', action='store_true')
    group.add_argument('--check', action='store_true')
    args = parser.parse_args()
    hashes = {name: hashlib.sha256(path.read_bytes()).hexdigest() for name, path in
              [('probe.c', SOURCE), ('measure_buffer_probe.py', Path(__file__))]}
    if args.run:
        raw = {'schema': 1, 'date': '2026-10-19', 'source_hashes': hashes,
               'environment': f'{platform.system()} / {platform.machine()}',
               'compiler': subprocess.check_output(['cc', '--version'], text=True).splitlines()[0],
               'flags': ['-std=c11', '-O2', '-Wall', '-Wextra', '-Werror'], 'runs': []}
        with tempfile.TemporaryDirectory(prefix='faction-probe-') as directory:
            binary = Path(directory) / 'probe'
            subprocess.run(['cc', *raw['flags'], str(SOURCE), '-o', str(binary)], check=True)
            for _ in range(5):
                for mode in ('copy', 'shared'):
                    raw['runs'].append(json.loads(subprocess.check_output([str(binary), mode], text=True)))
        RAW.parent.mkdir(parents=True, exist_ok=True)
        RAW.write_text(json.dumps(raw, indent=2) + '\n')
    else:
        raw = json.loads(RAW.read_text())
    if raw['source_hashes'] != hashes:
        raise SystemExit('Buffer probe source provenance is stale; rerun deliberately.')
    if len(raw['runs']) != 10 or {r['sessions'] for r in raw['runs']} != {250}:
        raise SystemExit('Unexpected probe workload/result count.')
    expected = 250 * sum(i % 251 for i in range(256 * 256))
    if {r['checksum'] for r in raw['runs']} != {expected}:
        raise SystemExit('Probe delivery checksum differs.')
    for mode in ('copy', 'shared'):
        runs = [r for r in raw['runs'] if r['mode'] == mode]
        if len(runs) != 5 or len({(r['pointer_bytes'], r['requested_live_bytes']) for r in runs}) != 1:
            raise SystemExit('Probe allocation observations differ between repetitions.')
        print(f"{mode}: {runs[0]['requested_live_bytes']} requested live bytes; 5 matching runs")
    print('Allocation requests only: excludes allocator overhead, RSS, OS, crypto, storage, server state.')

if __name__ == '__main__':
    main()
