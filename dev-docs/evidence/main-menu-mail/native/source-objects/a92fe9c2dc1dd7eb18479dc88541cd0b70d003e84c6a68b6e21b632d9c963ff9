#!/usr/bin/env python3
"""Bounded authored text replay; local application pacing, NOT network emulation.

--run records fresh timings; --check verifies saved report and source provenance.
No external dependencies, endpoints, real user content, or terminal escape output.
"""
from __future__ import annotations
import argparse
import hashlib
import json
import math
import platform
import re
import socket
import statistics
import sys
import textwrap
import threading
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
RAW = ROOT / 'dev-docs/research/results/008-text-replay.json'
REPORT = ROOT / 'dev-docs/research/0008-replay-results.md'
PROFILES = {'P1-private': (1_000_000, 0.040), 'P2-poor': (64_000, 0.400)}
SIZES = [(80, 24), (40, 20)]
SGR = re.compile(rb'\x1b\[(?:2J|H|1;36m|0m)')

def pages(case: str, cols: int, rows: int, ansi: bool) -> list[bytes]:
    if cols < 20 or rows < 8:
        raise ValueError('fixture requires >=20 columns and >=8 rows')
    if case == 'menu':
        lines = ['Faction BBS / the night shift', 'Welcome, approved member.',
                 '[M] Rooms  [C] Chat  [F] Files', '[D] Doors  [O] Other boards', '[Q] Log out']
    elif case == 'room':
        lines = ['Room: Hardware / new messages']
        for i in range(1, 9):
            lines += [f'#{i} / member{i}: Small systems',
                      'Keep the board local, the buffers bounded, and the conversation alive. '
                      'This synthetic message tests wrapping and continuation across pages.']
    elif case == 'files':
        lines = ['Local files / approved members']
        lines += [f'{i:02} notes-{i:02}.txt  1024 bytes  Public domain fixture' for i in range(1, 13)]
    else:
        raise ValueError('unknown case')
    wrapped = [part for line in lines for part in textwrap.wrap(line, cols - 1)]
    capacity = rows - 4
    result = []
    for start in range(0, len(wrapped), capacity):
        visible = wrapped[start:start + capacity]
        # The two footer lines plus a trailing empty cursor row fit the viewport.
        plain = '\r\n'.join(visible + ['', '[N] Next / [Q] Quit']) + '\r\n'
        encoded = plain.encode('ascii')
        if ansi:
            first, rest = encoded.split(b'\r\n', 1)
            encoded = b'\x1b[2J\x1b[H\x1b[1;36m' + first + b'\x1b[0m\r\n' + rest
        if len(encoded) > 2048:
            raise ValueError('page exceeds proposed 2 KiB cap')
        result.append(encoded)
    return result

def paced_send(sock: socket.socket, data: bytes, bps: int) -> float:
    if bps <= 0:
        raise ValueError('positive bit rate required')
    start = time.monotonic()
    for offset in range(0, len(data), 128):
        end = min(offset + 128, len(data))
        delay = start + end * 8 / bps - time.monotonic()
        if delay > 0:
            time.sleep(delay)
        sock.sendall(data[offset:end])
    return time.monotonic() - start

def receive(sock: socket.socket, count: int) -> bytes:
    output = bytearray()
    while len(output) < count:
        block = sock.recv(min(4096, count - len(output)))
        if not block:
            raise RuntimeError('unexpected end of replay')
        output.extend(block)
    return bytes(output)

def replay(payload: bytes, bps: int, rtt: float) -> dict:
    command = b'next\r\n'
    client, server = socket.socketpair()
    state = {}
    for sock in (client, server):
        sock.settimeout(5)
    def send():
        try:
            if receive(server, len(command)) != command:
                raise RuntimeError('request mismatch')
            time.sleep(rtt)
            state['send_s'] = paced_send(server, payload, bps)
        except Exception as exc:
            state['error'] = str(exc)
        finally:
            server.close()
    worker = threading.Thread(target=send)
    worker.start()
    try:
        start = time.monotonic()
        paced_send(client, command, bps)
        actual = receive(client, len(payload))
        elapsed = time.monotonic() - start
    finally:
        client.close()
        worker.join(timeout=6)
    if worker.is_alive() or 'error' in state:
        raise RuntimeError(state.get('error', 'replay did not terminate'))
    if actual != payload:
        raise RuntimeError('received bytes differ')
    return {'elapsed_s': elapsed, 'send_s': state['send_s'], 'received_bytes': len(actual)}

def source_hash() -> str:
    return hashlib.sha256(Path(__file__).read_bytes()).hexdigest()

def validate_observations(raw: dict) -> None:
    expected = {(profile, f'{cols}x{rows}', case, 'ANSI' if ansi else 'ASCII'):
                pages(case, cols, rows, ansi)
                for profile in PROFILES for cols, rows in SIZES
                for case in ('menu', 'room', 'files') for ansi in (False, True)}
    if raw['schema'] != 1 or raw['profiles'] != {
            k: {'each_direction_bps': v[0], 'injected_rtt_s': v[1]} for k, v in PROFILES.items()}:
        raise ValueError('unexpected recorded controls')
    seen = set()
    for cell in raw['cells']:
        key = tuple(cell[k] for k in ('profile', 'size', 'case', 'mode'))
        if key in seen or key not in expected:
            raise ValueError('duplicate or unknown observation cell')
        seen.add(key)
        output = expected[key]
        if cell['bytes'] != len(output[0]) or cell['pages'] != len(output) or len(cell['samples']) != 5:
            raise ValueError('observations differ from authored workload')
        for sample in cell['samples']:
            if sample['received_bytes'] != len(output[0]):
                raise ValueError('received byte count differs')
            if not all(math.isfinite(sample[k]) and sample[k] > 0 for k in ('elapsed_s', 'send_s')):
                raise ValueError('invalid timing observation')
    if seen != expected.keys():
        raise ValueError('missing observation cell')

def report(raw: dict) -> str:
    validate_observations(raw)
    lines = ['# 0008: Measured local text replay', '',
        'Generated by `python3 tools/measure_text_replay.py --run`; verify with `--check`.', '',
        '**Application-paced local socket pairs, not TCP/network or BBS performance.** '
        'Five repetitions per cell; actual received bytes verified. One request, injected nominal RTT, '
        '128-byte cumulative pacing per response. No loss/jitter/crypto/server/terminal costs.', '',
        f"Environment: {raw['environment']}. Source SHA-256: `{raw['source_sha256']}`.", '',
        'Every page fits the proposed 2 KiB cap; room/file cases continue across pages. '
        'The table times the first page only. ANSI clears/homes and colors its first line; '
        'removing that authored subset exactly reproduces plain text. This is no general escape sanitizer.', '',
        '| Profile | Size | Case | Mode | Pages | First bytes | Median ms | Min–max ms | Send rate bit/s |',
        '| --- | --- | --- | --- | ---: | ---: | ---: | --- | ---: |']
    for cell in raw['cells']:
        samples = cell['samples']
        times = [s['elapsed_s'] * 1000 for s in samples]
        rate = statistics.median(cell['bytes'] * 8 / s['send_s'] for s in samples)
        lines.append(f"| {cell['profile']} | {cell['size']} | {cell['case']} | {cell['mode']} | "
                     f"{cell['pages']} | {cell['bytes']} | {statistics.median(times):.2f} | "
                     f"{min(times):.2f}–{max(times):.2f} | {rate:.0f} |")
    lines += ['', 'Send rate is measured payload bits divided by sender pacing duration, excluding injected RTT. '
              'Short writes and scheduler wakeups can undershoot configured rates. Timings include Python '
              'scheduling; compare against configured controls, not an unmeasured real link. '
              'No host model/clock normalization, confidence interval, packet impairment, terminal visual '
              'compatibility, login, reconnect, storage, or 250-user server was tested.', '',
              'Raw observations: [008-text-replay.json](results/008-text-replay.json). '
              'Interpretation and follow-on validation: [interaction research](0008-interaction-budget.md).', '']
    return '\n'.join(lines)

def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument('--run', action='store_true')
    mode.add_argument('--check', action='store_true')
    args = parser.parse_args()
    if args.run:
        raw = {'schema': 1, 'retrieved': '2026-10-19', 'source_sha256': source_hash(),
               'environment': f'{platform.system()} / {platform.machine()} / Python {platform.python_version()}',
               'profiles': {k: {'each_direction_bps': v[0], 'injected_rtt_s': v[1]} for k, v in PROFILES.items()},
               'cells': []}
        for profile, (bps, rtt) in PROFILES.items():
            for cols, rows in SIZES:
                for case in ('menu', 'room', 'files'):
                    plain = pages(case, cols, rows, False)
                    for ansi in (False, True):
                        output = pages(case, cols, rows, ansi)
                        if [SGR.sub(b'', p) for p in output] != plain:
                            raise RuntimeError('fallback differs')
                        raw['cells'].append({'profile': profile, 'size': f'{cols}x{rows}', 'case': case,
                            'mode': 'ANSI' if ansi else 'ASCII', 'pages': len(output), 'bytes': len(output[0]),
                            'samples': [replay(output[0], bps, rtt) for _ in range(5)]})
                print(f'{profile} {cols}x{rows}: replay complete', flush=True)
        RAW.parent.mkdir(parents=True, exist_ok=True)
        RAW.write_text(json.dumps(raw, indent=2) + '\n')
        REPORT.write_text(report(raw))
    else:
        raw = json.loads(RAW.read_text())
        if raw['source_sha256'] != source_hash() or REPORT.read_text() != report(raw):
            print('Replay provenance/report stale; rerun deliberately.', file=sys.stderr)
            return 1
        print('Saved text replay provenance and report match; timing not rerun.')
    return 0

if __name__ == '__main__':
    raise SystemExit(main())
