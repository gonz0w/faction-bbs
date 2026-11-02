"""Read-only qualification of one ignored small-Get test; never echo raw logs."""
from dataclasses import dataclass
from pathlib import Path
import re
import sys

NAME = 'authority::services::publication::tests::composed_loop_tests::characterize_one_small_get_through_both_maintained_loops'
MAX_LOG_BYTES = 1 << 20
SCHEMAS = {
    'small_get_setup': 'schema sequence role kind ordinal phase part start_us end_us complete'.split(),
    'small_get_setup_summary': 'schema points conclusive scope_clean'.split(),
    'small_get_point': 'schema sequence micros role kind attempt event outcome first_us last_us'.split(),
    'small_get_call': 'schema index start_us end_us outcome diagnostic phase attempted connections busy planned_wait_us actual_wait_us'.split(),
    'small_get_summary': 'schema points calls setup_ok transport_ok exact response_bytes response_offset response_total trace_conclusive timestamp_overlap busy_count busy_before_finish busy_after_finish loops_ok physical_zero inactive drain_us root_removed token_retained clock_state_retained attempts connections outer_collects inner_collects outer_finish_before outer_finish_after inner_finish_before inner_finish_after expected_kinds calls_ok'.split(),
}


@dataclass(frozen=True)
class Validation:
    ok: bool
    lines: tuple[str, ...]


def _selection(stdout):
    lines = stdout.splitlines()
    running = re.findall(r'^running ([0-9]+) tests?$', stdout, re.M)
    selected = []
    for index, line in enumerate(lines):
        match = re.fullmatch(r'test ([A-Za-z0-9_:]+) \.\.\.(.*)', line)
        if match:
            selected.append((index, match[1], match[2].strip()))
    result = re.findall(r'^test result: (ok|FAILED)\. ([0-9]+) passed; ([0-9]+) failed; ([0-9]+) ignored; ([0-9]+) measured; ([0-9]+) filtered out; finished in [0-9]+(?:\.[0-9]+)?s$', stdout, re.M)
    footers = [index for index, line in enumerate(lines) if line.startswith('test result:')]
    success = [index for index, line in enumerate(lines) if line.strip() == 'ok']
    contradictory = any(line.strip() in {'FAILED', 'ignored'} for line in lines)
    named_ok = False
    if running == ['1'] and len(selected) == 1 and selected[0][1] == NAME and len(footers) == 1:
        index, _, suffix = selected[0]
        if index < footers[0] and not contradictory:
            # libtest may finish inline or after the maintained loops' ready0
            # lines. Count exactly one outcome; never echo an unknown suffix.
            named_ok = (suffix == 'ok' and not success) or (
                suffix in {'', 'ready 0'} and len(success) == 1
                and index < success[0] < footers[0]
            )
    return named_ok, len(result) == 1 and result[0][:5] == ('ok', '1', '0', '0', '0')


def _record(line):
    words = line.split()
    if not words or words[0] not in SCHEMAS or not line.isascii() or len(line) > 4096:
        return None
    keys = SCHEMAS[words[0]]
    values = {}
    for field in words[1:]:
        match = re.fullmatch(r'([a-z_]+)=([0-9]{1,20})', field)
        if not match or match[1] in values:
            return None
        number = int(match[2])
        if number > (1 << 64) - 1:
            return None
        values[match[1]] = number
    schema = values.get('schema')
    if schema == 2 and words[0] == 'small_get_setup':
        keys = keys + ['outcome']
    if set(values) != set(keys) or schema not in (
            (1, 2) if words[0] in {'small_get_setup', 'small_get_setup_summary'} else (1,)):
        return None
    return words[0], values


def _points_valid(points, summary):
    if not 1 <= len(points) <= 64 or [p['sequence'] for p in points] != list(range(1, len(points) + 1)):
        return False
    if any(p['role'] not in (0, 1) or p['kind'] not in (0, 1, 6) or p['event'] > 7 or p['outcome'] > 4 or p['attempt'] > 7 for p in points):
        return False
    issues = [p for p in points if p['event'] == 5]
    if len(issues) != 1 or issues[0]['role'] != 0 or issues[0]['kind'] != 0 or issues[0]['outcome'] != 2:
        return False
    for role, expected_kind, count_key in [(0, 1, 'outer_collects'), (1, 6, 'inner_collects')]:
        before = [p for p in points if p['role'] == role and p['event'] == 6]
        after = [p for p in points if p['role'] == role and p['event'] == 7]
        if len(before) != 1 or len(after) != 1 or before[0]['micros'] > after[0]['micros'] or before[0]['sequence'] >= after[0]['sequence']:
            return False
        if any(p['kind'] != 0 or p['outcome'] != 2 for p in before + after):
            return False
        events = [p for p in points if p['role'] == role and p['event'] <= 4]
        attempts = sorted({p['attempt'] for p in events})
        if not 1 <= len(attempts) <= 8 or attempts != list(range(len(attempts))) or summary[count_key] != len(attempts):
            return False
        for attempt in attempts:
            run = [p for p in events if p['attempt'] == attempt]
            shape = [p['event'] for p in run]
            if shape not in ([[0, 4]] if role == 0 else [[0, 1, 2, 4], [0, 3, 4]]):
                return False
            if any(p['kind'] != expected_kind for p in run) or run[-1]['outcome'] not in (1, 2) or any(p['outcome'] != 0 for p in run[:-1]):
                return False
            if len(run) == 4:
                window = run[1]
                if window['first_us'] > window['last_us'] or window['last_us'] - window['first_us'] > 100_000 or run[2]['micros'] < window['first_us']:
                    return False
    return all(p['first_us'] == p['last_us'] == 0 for p in points if p['event'] != 1)


def _complete(points, calls, summary):
    ones = 'setup_ok transport_ok exact trace_conclusive loops_ok physical_zero inactive root_removed expected_kinds calls_ok outer_finish_before outer_finish_after inner_finish_before inner_finish_after'.split()
    if any(summary[key] != 1 for key in ones) or any(summary[key] != 0 for key in ['timestamp_overlap', 'token_retained', 'clock_state_retained']):
        return False
    if (summary['response_bytes'], summary['response_offset'], summary['response_total']) != (17, 0, 17) or summary['drain_us'] >= 5_000_000:
        return False
    if summary['points'] != len(points) or not 2 <= len(calls) <= 7 or summary['calls'] != len(calls):
        return False
    if summary['attempts'] != len(calls) or summary['connections'] != len(calls) or summary['outer_collects'] != len(calls) - 1:
        return False
    if [c['index'] for c in calls] != list(range(len(calls))) or calls[0]['outcome'] != 0 or calls[-1]['outcome'] != 2:
        return False
    for index, call in enumerate(calls):
        if call['start_us'] > call['end_us'] or call['diagnostic'] != 1 or call['phase'] > 6 or call['attempted'] != index + 1 or call['connections'] != index + 1:
            return False
        if index and (call['start_us'] < calls[index - 1]['end_us'] or call['outcome'] not in (1, 2)):
            return False
        if 0 < index < len(calls) - 1 and call['outcome'] != 1:
            return False
    busy = sum(c['outcome'] == 1 for c in calls)
    if summary['busy_count'] != busy or summary['busy_before_finish'] + summary['busy_after_finish'] != busy or calls[-1]['busy'] != busy:
        return False
    if sum(p['role'] == 0 and p['event'] == 4 and p['outcome'] == 1 for p in points) != busy:
        return False
    return _points_valid(points, summary)


def _setup_complete(records, calls, points, summary):
    setup = records['small_get_setup']
    headers = records['small_get_setup_summary']
    if len(headers) != 1:
        return False
    schema = headers[0]['schema']
    if headers[0] != {'schema':schema, 'points':len(setup), 'conclusive':1, 'scope_clean':1} or any(p['schema'] != schema for p in setup):
        return False
    if not 1 <= len(setup) <= 128 or [p['sequence'] for p in setup] != list(range(1, len(setup) + 1)):
        return False
    if any(p['role'] > 2 or p['kind'] > 2 or p['ordinal'] > 7 or p['phase'] > (5 if schema == 1 else 7) or p['part'] > 1 or p['complete'] != 1 or p['start_us'] > p['end_us'] for p in setup):
        return False
    expected_counts = {(0,0):len(calls), (0,2):len(calls)-1, (1,0):summary['inner_collects']+1, (1,1):1, (2,1):1}
    if set((p['role'],p['kind']) for p in setup) != set(expected_counts):
        return False
    for (role,kind), count in expected_counts.items():
        group = [p for p in setup if (p['role'],p['kind']) == (role,kind)]
        if not 1 <= count <= 8 or sorted({p['ordinal'] for p in group}) != list(range(count)):
            return False
        previous = None
        for ordinal in range(count):
            run = [p for p in group if p['ordinal'] == ordinal]
            # Schema1 qualifies historical cold-work records only. Current
            # schema2 requires explicit actual lookup/publication outcomes.
            if schema == 1:
                expected = {0:[(2,0),(0,0),(1,0),(3,0),(4,0)], 1:[(0,0),(1,0),(2,0),(1,1),(3,0),(4,0)], 2:[(5,0)]}[kind]
            elif kind == 2:
                expected = [(5, 0)]
                if any(p['outcome'] != 0 for p in run):
                    return False
            else:
                lookup = [p for p in run if p['phase'] == 6]
                if len(lookup) != 1 or lookup[0]['outcome'] not in (0, 1):
                    return False
                hit = lookup[0]['outcome'] == 1
                expected = [(2, 0)] if kind == 0 else []
                expected += [(6, 0)]
                if not hit:
                    expected += [(0, 0), (1, 0)]
                if kind == 1:
                    expected += [(2, 0)]
                if not hit:
                    expected += [(1, 1), (7, 0)]
                expected += [(3, 0), (4, 0)]
                if any((p['outcome'] != (1 if hit else 0) if p['phase'] == 6
                        else p['outcome'] not in (2, 3) if p['phase'] == 7
                        else p['outcome'] != 0) for p in run):
                    return False
            if [(p['phase'],p['part']) for p in run] != expected or any(a['end_us'] > b['start_us'] for a,b in zip(run,run[1:])):
                return False
            if previous is not None and previous > run[0]['start_us']:
                return False
            previous = run[-1]['end_us']
            if role == 0:
                call = calls[ordinal if kind == 0 else ordinal+1]
                if not call['start_us'] <= run[0]['start_us'] <= run[-1]['end_us'] <= call['end_us']:
                    return False
                if kind == 2:
                    tcp = next(p for p in setup if (p['role'],p['kind'],p['ordinal'],p['phase']) == (0,0,ordinal+1,2))
                    if run[-1]['end_us'] > tcp['start_us']:
                        return False
            elif role == 1 and kind == 1:
                issued = next(p for p in points if p['event'] == 5)
                submitted = next(p for p in setup if (p['role'],p['kind'],p['ordinal'],p['phase']) == (0,0,0,4))
                if run[0]['start_us'] < submitted['end_us'] or run[-1]['end_us'] > issued['micros']:
                    return False
            elif role == 1 and kind == 0 and ordinal == 0:
                issued = next(p for p in points if p['event'] == 5)
                if run[0]['start_us'] < issued['micros']:
                    return False
            elif role == 1 and ordinal:
                validated = next(p for p in points if p['role'] == 1 and p['attempt'] == ordinal-1 and p['event'] == 0)
                if run[-1]['end_us'] > validated['micros']:
                    return False
            elif role == 2:
                finished = next(p for p in points if p['role'] == 1 and p['event'] == 6)
                submitted = next(p for p in setup if (p['role'],p['kind'],p['ordinal'],p['phase']) == (1,0,0,4))
                if run[0]['start_us'] < submitted['end_us'] or run[-1]['end_us'] > finished['micros']:
                    return False
    return True


def validate(stdout, stderr, status):
    code = int(status != 0)
    if not isinstance(status, int) or not 0 <= status <= 255 or len(stdout) > MAX_LOG_BYTES or len(stderr) > MAX_LOG_BYTES:
        return Validation(False, ('small_get_validation schema=1 ok=0 status=255 selection=0 result=0 code=16',))
    selection, result = _selection(stdout)
    if not selection: code |= 2
    if not result: code |= 4
    records = {name: [] for name in SCHEMAS}
    published = []
    for line in stderr.splitlines():
        if not line.startswith('small_get_'):
            continue
        record = _record(line)
        if record is None:
            code |= 8
            continue
        name, values = record
        limit = {'small_get_point': 64, 'small_get_call': 7, 'small_get_summary': 1, 'small_get_setup': 128, 'small_get_setup_summary': 1}[name]
        if len(records[name]) >= limit:
            code |= 8
            continue
        records[name].append(values)
        keys = SCHEMAS[name] + (['outcome'] if name == 'small_get_setup' and values['schema'] == 2 else [])
        published.append(name + ' ' + ' '.join(key + '=' + str(values[key]) for key in keys))
    summaries = records['small_get_summary']
    if len(summaries) != 1 or not _complete(records['small_get_point'], records['small_get_call'], summaries[0]):
        code |= 8
    elif not _setup_complete(records, records['small_get_call'], records['small_get_point'], summaries[0]):
        code |= 8
    ok = code == 0
    published.append(f'small_get_validation schema=1 ok={int(ok)} status={status} selection={int(selection)} result={int(result)} code={code}')
    return Validation(ok, tuple(published))


def main(argv=None):
    argv = sys.argv[1:] if argv is None else argv
    try:
        if len(argv) != 6 or argv[::2] != ['--status', '--stdout', '--stderr']:
            raise ValueError
        status = int(argv[1])
        logs = []
        for name in [argv[3], argv[5]]:
            with Path(name).open('rb') as stream:
                data = stream.read(MAX_LOG_BYTES + 1)
            if len(data) > MAX_LOG_BYTES:
                raise ValueError
            logs.append(data.decode('utf-8'))
        result = validate(*logs, status)
    except (OSError, UnicodeError, ValueError):
        result = Validation(False, ('small_get_validation schema=1 ok=0 status=255 selection=0 result=0 code=16',))
    for line in result.lines:
        print(line)
    return 0 if result.ok else 1


if __name__ == '__main__':
    raise SystemExit(main())
