#!/usr/bin/env python3
"""Measure canonical Nightwire output in an owned real emulator, without pixels.

Only compiled local authoring output is sent; this is not a member-content relay.
DSR reports test cursor/width behavior, not glyph shape or the displayed RGB palette.
"""
import argparse
import hashlib
import json
import re
import subprocess
import termios
import time
from pathlib import Path
from probe_terminal_profile import Terminal, cursor
from render_theme_preview import ART_GLYPHS, cells

ROOT = Path(__file__).resolve().parents[1]
CONTROL = re.compile(r'\x1b\[([0-9;]*)([HJm])')


def frames(binary):
    result = []
    for name, pack, season, count in [
        ('long', 'nightwire', None, 3), ('short', 'nightwire-short', None, 1),
        ('autumn', 'nightwire', 'nightwire-seasons/autumn', 3),
        ('winter', 'nightwire', 'nightwire-seasons/winter', 3),
    ]:
        for mode, columns, page in ([('ansi', 80, p) for p in range(1, count + 1)]
                                    + [('compact', 40, 1), ('plain', 80, 1)]):
            args = [str(binary), '--theme', str(ROOT / 'themes' / pack), '--login', '--columns', str(columns)]
            if season:
                args += ['--season', str(ROOT / 'themes' / season)]
            if mode == 'plain':
                args += ['--plain']
            elif mode == 'ansi':
                args += ['--page', str(page)]
            raw = subprocess.check_output(args, stderr=subprocess.DEVNULL, timeout=10)
            if len(raw) > 17408:
                raise ValueError('preview budget')
            cells(raw.decode('utf-8'), columns, 24 if columns == 80 else 18, True)
            if mode == 'plain' and (b'\x1b' in raw or not raw.isascii()):
                raise ValueError('classic fallback is not printable ASCII')
            result.append(dict(name=f'{name}-{mode}-{page}', mode=mode, columns=columns, raw=raw))
    for columns in [80, 40]:
        raw = subprocess.check_output([str(binary), '--theme', str(ROOT / 'themes/nightwire'),
                                       '--columns', str(columns)], stderr=subprocess.DEVNULL, timeout=10)
        cells(raw.decode('utf-8'), columns)
        if len(raw) > 4096:
            raise ValueError('menu budget')
        result.append(dict(name=f'menu-{columns}', mode='menu', columns=columns, raw=raw))
    return result


def checkpoints(raw, columns, rows):
    """Split *unchanged* canonical bytes at cursor moves, measuring every text row.

Validate the narrow grammar first. DSR leaves pending autowrap alone; the next
product CUP resets it. Plain LF gets explicit CR for this raw research TTY only.
"""
    text = raw.decode('utf-8')
    row = col = 1
    pending = False
    start = pos = 0
    visible = False
    out = []
    while pos < len(text):
        if text[pos] == '\x1b':
            match = CONTROL.match(text, pos)
            if not match:
                raise ValueError('noncanonical control')
            values = [int(x) for x in match[1].split(';')] if match[1] else []
            if match[2] == 'H':
                if visible:
                    out.append((text[start:pos].encode(), [row, col]))
                    start = pos
                    visible = False
                row, col = values or [1, 1]
                pending = False
                if not (1 <= row <= rows and 1 <= col <= columns):
                    raise ValueError('frame does not fit real terminal')
            elif match[2] == 'J' and values != [2]:
                raise ValueError('unsupported clear')
            elif match[2] == 'm' and any(p not in [0, 1, 22] and not 30 <= p <= 37
                                            and not 40 <= p <= 47 and not 90 <= p <= 97
                                            and not 100 <= p <= 107 for p in values):
                raise ValueError('unsupported SGR')
            pos = match.end()
        else:
            ch = text[pos]
            if ch == '\r':
                col = 1
                pending = False
            elif ch == '\n':
                row += 1
                col = 1
                pending = False
            elif ' ' <= ch <= '~' or ch in ART_GLYPHS:
                if pending:
                    row += 1
                    col = 1
                pending = col == columns
                if not pending:
                    col += 1
                visible = True
            else:
                raise ValueError('unsupported glyph')
            if row > rows:
                raise ValueError('frame would scroll real terminal')
            pos += 1
    if start < len(text):
        out.append((text[start:].encode(), [row, col] if visible else None))
    return out


def run(a):
    terminal = Terminal()
    result = dict(schema=1, geometry=terminal.size(), frames=[], glyphs=[], visual_pixels_captured=False)
    try:
        if a.start_gate:
            a.start_gate.with_suffix('.ready').write_text('ready\n')
            end = time.monotonic() + 10
            while not a.start_gate.exists():
                if time.monotonic() > end:
                    raise TimeoutError('owned fixture start gate')
                time.sleep(.01)
            termios.tcflush(0, termios.TCIFLUSH)
        rows, columns = terminal.size()[:2]
        if columns < 40 or rows < 18:
            raise ValueError('real viewport must be at least 40x18')
        rendered = frames(a.binary)
        if columns < 80 or rows < 24:
            rendered = [f for f in rendered if f['mode'] == 'compact' or f['name'] == 'menu-40']
        for case in rendered:
            raw = case['raw']
            # Product adapters preserve the ordinary TTY OPOST/ONLCR path.
            # This raw response-measurement TTY emulates that output processing.
            wire = raw.replace(b'\n', b'\r\n')
            terminal.write(b'\x1b[0m\x1b[2J\x1b[H')
            checks = []
            for data, expected in checkpoints(wire, columns, rows):
                if expected is None:
                    terminal.write(data)
                    continue
                reply = terminal.query(data + b'\x1b[6n', 'cursor', timeout=.08)
                actual = cursor(reply)
                checks.append(dict(expected=expected, actual=actual, pass_=actual == expected))
            result['frames'].append(dict(name=case['name'], mode=case['mode'], columns=case['columns'],
                bytes=len(raw), sha256=hashlib.sha256(raw).hexdigest(), checkpoints=checks,
                pass_=bool(checks) and all(c['pass_'] for c in checks)))
        glyphs = sorted({ch for case in rendered for ch in case['raw'].decode() if ch in ART_GLYPHS})
        for glyph in glyphs:
            # A repeated run tests accumulation rather than merely one glyph.
            reply = terminal.query(b'\x1b[2J\x1b[H' + (glyph * 32).encode() + b'\x1b[6n', 'cursor', timeout=.08)
            actual = cursor(reply)
            result['glyphs'].append(dict(codepoint=f'U+{ord(glyph):04X}', expected=[1, 33],
                                         actual=actual, pass_=actual == [1, 33]))
        terminal.write(b'\x1b[0m\x1b[2J\x1b[H')
        result['unclassified_input_bytes'] = terminal.unexpected
        result['result'] = 'pass' if all(c['pass_'] for c in result['frames'] + result['glyphs']) and not terminal.unexpected else 'fail'
        result['raw_tty_onlcr_emulation'] = True
        result['sgr_palette_and_glyph_shape_certified'] = False
    finally:
        terminal.close()
    a.output.write_text(json.dumps(result, indent=2, sort_keys=True) + '\n')
    a.output.chmod(0o600)
    return result


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--start-gate', type=Path)
    args = parser.parse_args()
    args.binary = args.binary.resolve(strict=True)
    run(args)
