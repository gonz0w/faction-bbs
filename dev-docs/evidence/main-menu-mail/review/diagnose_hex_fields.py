"""Report only hex-field shape metadata, never any captured value."""
import json
from pathlib import Path
import re

raw = Path('/private/tmp/faction230-native-OWNED/privacy-postrun-rescan/host_classic.post-hex.private.log').read_text(encoding='ascii')
examples = []
count = 0
for line in raw.splitlines():
    row = re.fullmatch(r'\t([0-9a-f]{3,})   (.{48}) (.{1,16})', line)
    if row:
        tokens = row[2].split()
        bad = [token for token in tokens if not re.fullmatch(r'[0-9a-f]{2}', token)]
        if bad:
            count += 1
            if len(examples) < 10:
                examples.append({'hex_field_width': len(row[2]), 'token_count': len(tokens), 'token_lengths': [len(v) for v in tokens], 'invalid_token_count': len(bad), 'invalid_token_character_classes': [{'decimal_digits': sum(c.isdigit() for c in v), 'lowercase_hex_letters': sum(c in 'abcdef' for c in v), 'uppercase_hex_letters': sum(c in 'ABCDEF' for c in v), 'other_letters': sum(c.isalpha() and c not in 'abcdefABCDEF' for c in v), 'punctuation': sum(not c.isalnum() for c in v)} for v in bad], 'field_whitespace_runs': [len(m[0]) for m in re.finditer(' +', row[2])]})
print(json.dumps({'invalid_hex_field_rows': count, 'examples': examples}, indent=2))
