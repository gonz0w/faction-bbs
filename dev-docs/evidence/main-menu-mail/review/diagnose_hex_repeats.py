from pathlib import Path
import json
import re

lines = Path('/private/tmp/faction230-native-OWNED/privacy-postrun-rescan/host_classic.post-hex.private.log').read_text(encoding='ascii').splitlines()
counts = {'ellipsis_repeat_rows': 0, 'unknown_short_rows': 0}
examples = []
for i, line in enumerate(lines):
    if line.startswith('\t') and len(line) == 4:
        known = line == '\t...'
        counts['ellipsis_repeat_rows' if known else 'unknown_short_rows'] += 1
        previous = re.match(r'\t([0-9a-f]+)   ', lines[i-1])
        following = re.match(r'\t([0-9a-f]+)   ', lines[i+1]) if i+1 < len(lines) else None
        examples.append({'recognized_ellipsis': known, 'previous_offset': int(previous[1],16) if previous else None, 'next_offset': int(following[1],16) if following else None, 'followed_by_record_or_eof': following is None})
print(json.dumps({'counts': counts, 'layout': examples}, indent=2))
