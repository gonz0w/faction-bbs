"""Emit structural layout metadata only; never captured payload bytes."""
from pathlib import Path
import json
import re

source = Path('/private/tmp/faction230-native-OWNED/privacy-postrun-rescan/host_classic.post-hex.private.log')
header = re.compile(r'\s*(\d+)(?:\s+(\d+))?\s+faction-service\s+GIO\s+fd\s+(\d+)\s+(read|wrote)\s+(\d+)\s+bytes')
row = re.compile(r'\t([0-9a-f]{3,})   (.{48}) (.{1,16})')
categories = {}
examples = []
for line in source.read_text(encoding='ascii').splitlines():
    if not line.strip() or header.fullmatch(line) or row.fullmatch(line):
        continue
    prefix = re.match(r'([ \t]*)([0-9a-f]+)([ \t]+)', line)
    category = 'offset-shaped' if prefix else 'other'
    categories[category] = categories.get(category, 0) + 1
    if len(examples) < 12:
        metadata = {'category': category, 'line_length': len(line), 'starts_tab': line.startswith('\t'), 'ends_space': line.endswith(' ')}
        if prefix:
            metadata.update(leading_whitespace=len(prefix[1]), offset_digits=len(prefix[2]), offset_padding=len(prefix[3]), remaining_width=len(line)-prefix.end())
            remainder = line[prefix.end():]
            hex_prefix = re.match(r'(?:[0-9a-f]{2}[ ]+)+', remainder)
            metadata['possible_hex_prefix_width'] = len(hex_prefix[0]) if hex_prefix else 0
            metadata['whitespace_runs'] = [len(m[0]) for m in re.finditer(' +', remainder)]
        examples.append(metadata)
print(json.dumps({'categories': categories, 'structural_examples': examples}, indent=2))
