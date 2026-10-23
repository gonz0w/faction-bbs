#!/usr/bin/env python3
"""Render a trusted local theme example to SVG using a fixed reference palette.

Python stdlib only. This authoring artifact is not a GUI screenshot or a terminal
font/graphics certification. Input is the compiled example's canonical output;
reject controls outside the renderer grammar rather than interpreting raw art.
"""
import argparse
import re
import subprocess
from pathlib import Path
from xml.sax.saxutils import escape

ART_GLYPHS = frozenset(bytes(range(128, 256)).decode('cp437') + '⌂')

PALETTE = ('#101018', '#c45b6e', '#7dbe8b', '#d8b46b', '#415b94', '#b482ce',
           '#70b6ba', '#c5c5d0', '#707080', '#ff8194', '#a4e4af', '#ffe0a0',
           '#829fe0', '#dfa5ff', '#a0e5eb', '#f3edf9')

def cells(frame, columns, rows=18, allow_newlines=False):
    grid = [[(' ', 7, 0, False) for _ in range(columns)] for _ in range(rows)]
    row = col = 0
    fg, bg, bold = 7, 0, False
    pos = 0
    while pos < len(frame):
        if frame[pos] == '\x1b':
            m = re.match(r'\x1b\[([0-9;]*)([HJm])', frame[pos:])
            if not m:
                raise ValueError('unsupported preview control')
            params = [int(x) for x in m[1].split(';')] if m[1] else []
            if m[2] == 'H':
                row, col = (params or [1, 1]); row -= 1; col -= 1
                if not (0 <= row < rows and 0 <= col < columns):
                    raise ValueError('cursor outside preview')
            elif m[2] == 'J':
                if params != [2]:
                    raise ValueError('unsupported clear')
                grid = [[(' ', fg, bg, bold) for _ in range(columns)] for _ in range(rows)]
            else:
                for p in params or [0]:
                    if p == 0: fg, bg, bold = 7, 0, False
                    elif p == 1: bold = True
                    elif p == 22: bold = False
                    elif 30 <= p <= 37: fg = p - 30
                    elif 90 <= p <= 97: fg = p - 90 + 8
                    elif 40 <= p <= 47: bg = p - 40
                    elif 100 <= p <= 107: bg = p - 100 + 8
                    else: raise ValueError('unsupported preview style')
            pos += len(m[0])
        elif allow_newlines and frame[pos] in '\r\n':
            if frame[pos] == '\n': row += 1
            col = 0; pos += 1
            if row >= rows: raise ValueError('scrolling login preview')
        else:
            c = frame[pos]
            if not (' ' <= c <= '~' or c in ART_GLYPHS) or not (row < rows and col < columns):
                raise ValueError('unsupported glyph, wrapped or scrolling preview')
            grid[row][col] = c, fg, bg, bold
            col += 1; pos += 1
    return grid

def svg(grid, login=False, paged=False):
    width, height = len(grid[0]) * 10 + 24, len(grid) * 22 + 24
    out = [f'<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}" role="img" aria-labelledby="title description">',
           ('<title id="title">Faction BBS seasonal login preview</title>' if login else '<title id="title">Faction BBS main-menu authoring preview</title>'),
           ('<desc id="description">Original seasonal login template, reference palette. Not an emulator screenshot or final identity-art acceptance.</desc>' if login else '<desc id="description">Reference palette, not an emulator screenshot. Arrow lightbar, hotkeys, action descriptions and classic recovery instructions.</desc>'),
           f'<rect width="{width}" height="{height}" fill="{PALETTE[0]}"/>']
    for y, line in enumerate(grid):
        x = 0
        while x < len(line):
            start = x; _, fg, bg, bold = line[x]
            while x < len(line) and line[x][1:] == (fg, bg, bold): x += 1
            text = ''.join(c[0] for c in line[start:x])
            if bg:
                out.append(f'<rect x="{12 + start * 10}" y="{12 + y * 22}" width="{(x-start)*10}" height="22" fill="{PALETTE[bg]}"/>')
            if text.strip():
                out.append(f'<text x="{12+start*10}" y="{29+y*22}" fill="{PALETTE[fg]}" font-family="monospace" font-size="16" font-weight="{700 if bold else 400}" xml:space="preserve" textLength="{(x-start)*10}" lengthAdjust="spacingAndGlyphs">{escape(text)}</text>')
    document = '\n'.join([*out, '</svg>', ''])
    if paged:
        document = document.replace('seasonal login preview','paged login preview').replace('Original seasonal login template','Original long-form login template')
    return document

def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('binary', type=Path)
    p.add_argument('--theme', required=True, type=Path)
    p.add_argument('--season', type=Path)
    p.add_argument('--theme-schedule', type=Path)
    p.add_argument('--date')
    p.add_argument('--login', action='store_true')
    p.add_argument('--page',type=int,choices=range(1,17))
    p.add_argument('--columns', type=int, choices=range(40, 241), default=80, metavar='40..240')
    p.add_argument('--output', required=True, type=Path)
    a = p.parse_args()
    cmd = [str(a.binary.resolve(strict=True)), '--theme', str(a.theme), '--columns', str(a.columns)]
    if a.season: cmd += ['--season', str(a.season)]
    if a.theme_schedule: cmd += ['--theme-schedule', str(a.theme_schedule)]
    if a.date: cmd += ['--date', a.date]
    if a.login: cmd += ['--login']
    if a.page: cmd += ['--page',str(a.page)]
    result = subprocess.run(cmd, check=True, capture_output=True, timeout=10)
    if len(result.stdout) > (17408 if a.login else 4096): raise ValueError('preview exceeds output budget')
    a.output.write_text(svg(cells(result.stdout.decode('utf-8'), a.columns, rows=24 if a.login and a.columns>=80 else 18, allow_newlines=a.login), login=a.login,paged=a.page is not None))
    print('Wrote reference-palette SVG; not a GUI or artwork approval.')

if __name__ == '__main__': main()
