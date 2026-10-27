#!/usr/bin/env python3
"""Copy preserved public execution inputs; never reconstruct or execute product code."""
from pathlib import Path, PurePosixPath
import hashlib, json, re, sys

OUT = Path('/private/tmp/faction230-packaging-OWNED')
ROOTS = {owner: Path('/private/tmp/faction230-' + owner + '-OWNED') for owner in ('server', 'client', 'lead')}
FROZEN_LEAD_FILES = ['OWNED', 'bind_guard_platform.py', 'check-menu-evidence-green01.py', 'guard-registration-green-test.py', 'guard-registration-green.log', 'guard-registration-red-test.py', 'guard-registration-red.log', 'guard-registration.json', 'guard-test-before-registration', 'host-integrated-0.log', 'host-integrated-1.log', 'host-integrated-2.log', 'host-integrated-after.json', 'host-integrated-before.json', 'host-integrated.json', 'host-python-sources.json', 'host-python.log', 'host-setup-failed01.log', 'host_integrated.py', 'host_integrated_failed01.py', 'menu-evidence-green01.log', 'menu-evidence-green02.log', 'menu-evidence-red.log', 'names-binding.json', 'names-green-guard.py', 'names-green-test.py', 'names-green.log', 'names-guard-before.py', 'names-red-test.py', 'names-red.log', 'names-test-before.py', 'platform-binding.json', 'platform-green-guard.py', 'platform-green-test.py', 'platform-green.log', 'platform-guard-before.py', 'platform-red-test.py', 'platform-red.log', 'platform-test-before.py', 'register_guard.py', 'supplement-binding.json', 'supplement-failed01-guard.py', 'supplement-failed01-test.py', 'supplement-green.log', 'supplement-green02-guard.py', 'supplement-green02-test.py', 'supplement-green02.log', 'supplement-guard-before.py', 'supplement-red-test.py', 'supplement-red.log', 'supplement-test-before.py', 'supplement_guard.py', 'supplement_names.py', 'test-menu-evidence-green01.py', 'test-menu-evidence-green02.py', 'test-menu-evidence-red.py', 'workflow-after-registration.yml', 'workflow-before-registration']
HEX = re.compile(r'[0-9a-f]{64}')
PRIVATE_BLOCK = re.compile(rb'(?m)^-----BEGIN (?:OPENSSH |RSA |EC |DSA |ENCRYPTED )?PRIVATE KEY-----\r?$')
SOURCE_SUFFIXES = {'.rs', '.py', '.toml', '.lock', '.sql', '.c', '.h', '.m', '.lua', '.yml', '.md', '.article', '.json', '.pem'}
SOURCE_NAMES = {'LICENSE-HACL-APACHE', 'LICENSE-HACL-MIT', 'LICENSE-ISC', 'SHA256SUMS'}
objects = {}
logical_names = {}
maps = []
artifacts = []
stages = []
details = []
input_hashes = {}


def digest(data):
    return hashlib.sha256(data).hexdigest()


def read(path):
    if path.is_symlink() or not path.is_file():
        raise ValueError('Nonregular evidence input refused')
    data = path.read_bytes()
    key = str(path)
    h = digest(data)
    if key in input_hashes and input_hashes[key] != h:
        raise ValueError('Evidence input changed while packaging')
    input_hashes[key] = h
    return data


def public_bytes(data):
    if b'\x00' in data or PRIVATE_BLOCK.search(data):
        raise ValueError('Private-key or binary material refused')
    data.decode('utf-8')
    return data


def write(relative, data):
    public_bytes(data)
    p = OUT / relative
    p.parent.mkdir(parents=True, exist_ok=True)
    if p.exists() and p.read_bytes() != data:
        raise ValueError('Refusing to overwrite an existing export variant')
    p.write_bytes(data)
    return {'path': relative, 'sha256': digest(data), 'bytes': len(data)}


def logical_source(name):
    p = PurePosixPath(name)
    if p.is_absolute() or '..' in p.parts or any(x in {'.git', 'target', '.DS_Store', '__pycache__'} for x in p.parts):
        return False
    if not (name.startswith(('crates/', 'tools/', '.github/', 'native/')) or name in {'Cargo.toml', 'Cargo.lock', 'rust-toolchain.toml'}):
        return False
    return p.suffix in SOURCE_SUFFIXES or p.name in SOURCE_NAMES


def add_object(data, logical=None):
    public_bytes(data)
    h = digest(data)
    if h in objects and objects[h] != data:
        raise ValueError('Source digest collision')
    objects[h] = data
    if logical:
        logical_names.setdefault(h, set()).add(logical)
    return h


def flat_map(data):
    try:
        value = json.loads(data)
    except (ValueError, UnicodeDecodeError):
        return None
    if isinstance(value, dict) and value and all(isinstance(k, str) and isinstance(v, str) and HEX.fullmatch(v) for k, v in value.items()):
        return value
    return None


def add_map(owner, stage, label, path, base=None):
    data = read(path)
    mapping = flat_map(data)
    if mapping is None:
        return False
    if not all(logical_source(name) for name in mapping):
        raise ValueError('Map contains a non-public source path')
    ref = write('source-maps/' + owner + '/' + stage + '/' + label + '.json', data)
    maps.append({'owner': owner, 'stage': stage, 'phase': label, 'mapping': mapping, 'ref': ref, 'base': base})
    for name, h in mapping.items():
        if base is not None:
            p = base / name
            if p.is_file() and not p.is_symlink():
                b = read(p)
                if digest(b) != h:
                    raise ValueError('Executed map does not match retained source bytes')
                add_object(b, name)
    return True


def artifact(owner, path, root, category='metadata'):
    data = read(path)
    rel = path.relative_to(root).as_posix()
    ref = write(category + '/' + owner + '/' + rel, data)
    artifacts.append({'owner': owner, 'input_name': rel, **ref})
    return ref


# Only known source snapshots and their immediate public records are admitted.
for owner in ('server', 'client'):
    root = ROOTS[owner]
    for stage in sorted(p for p in root.iterdir() if p.is_dir()):
        if owner == 'server' and (stage / 'manifest.json').is_file():
            add_map(owner, stage.name, 'before', stage / 'manifest.json', stage / 'sources')
            if (stage / 'manifest-after.json').is_file():
                add_map(owner, stage.name, 'after', stage / 'manifest-after.json', stage / 'sources-after')
        elif owner == 'client' and (stage / 'manifest.json').is_file():
            add_map(owner, stage.name, 'snapshot', stage / 'manifest.json', stage)
        for p in sorted(q for q in stage.iterdir() if q.is_file()):
            if p.name in {'manifest.json', 'manifest-after.json'}:
                continue
            if p.name in {'source-before.json', 'source-after.json'}:
                add_map(owner, stage.name, p.stem, p)
            elif p.name == 'status' or p.suffix == '.json':
                artifact(owner, p, root)
            elif p.suffix == '.log':
                artifact(owner, p, root, 'logs')
    for p in sorted(q for q in root.iterdir() if q.is_file()):
        if p.name == 'OWNED':
            continue
        if p.suffix == '.json':
            if not add_map(owner, 'root', p.stem, p):
                artifact(owner, p, root)
        elif p.suffix == '.log':
            artifact(owner, p, root, 'logs')
        elif p.suffix == '.py':
            artifact(owner, p, root, 'provenance')

# Lead objects are immutable public Rust/Python inputs addressed by their digest.
lead = ROOTS['lead']
for dirname in ('host-source-objects', 'host-python-source-objects'):
    for p in sorted((lead / dirname).iterdir()):
        if not HEX.fullmatch(p.name):
            raise ValueError('Unexpected source-object name')
        data = read(p)
        if digest(data) != p.name:
            raise ValueError('Lead source object hash mismatch')
        add_object(data)
for name in FROZEN_LEAD_FILES:
    p = lead / name
    if name == 'OWNED':
        continue
    if p.suffix == '.json':
        if not add_map('lead', 'root', p.stem, p):
            artifact('lead', p, lead)
    elif p.suffix == '.log':
        artifact('lead', p, lead, 'logs')
    elif p.suffix in {'.py', '.yml'} or name in {'workflow-before-registration', 'guard-test-before-registration'}:
        artifact('lead', p, lead, 'provenance')
    else:
        raise ValueError('Unexpected lead root evidence file')
for stage in sorted(p for p in lead.iterdir() if p.is_dir() and p.name not in {'host-source-objects', 'host-python-source-objects'}):
    for p in sorted(q for q in stage.iterdir() if q.is_file()):
        if p.suffix == '.json':
            if not add_map('lead', stage.name, p.stem, p):
                artifact('lead', p, lead)
        elif p.suffix == '.log':
            artifact('lead', p, lead, 'logs')
        elif p.suffix == '.py':
            artifact('lead', p, lead, 'provenance')
        else:
            raise ValueError('Unexpected lead stage evidence file')

# Resolve every source reference from actual preserved bytes, never the current repo.
for record in maps:
    files = []
    for name, h in sorted(record['mapping'].items()):
        if h not in objects:
            raise ValueError('Unresolved executed source digest')
        logical_names.setdefault(h, set()).add(name)
        files.append({'name': name, 'sha256': h})
    label = '/'.join([record['owner'], record['stage'], record['phase']])
    stages.append({'label': label, 'files': files})
    details.append({'label': label, 'original_map': record['ref'], 'source_entries': len(files)})
used = {f['sha256'] for stage in stages for f in stage['files']}
for h in sorted(used):
    write('source-objects/' + h, objects[h])

# Stage records preserve command/status/log identity and honest pre/post limitations.
for stage in sorted({(m['owner'], m['stage']) for m in maps}):
    owner, name = stage
    found = [m for m in maps if m['owner'] == owner and m['stage'] == name]
    phases = {m['phase']: m for m in found}
    before = phases.get('before') or phases.get('source-before') or phases.get('host-integrated-before')
    after = phases.get('after') or phases.get('source-after') or phases.get('host-integrated-after')
    drift = None
    if before and after:
        a, b = before['mapping'], after['mapping']
        drift = sorted(k for k in a.keys() | b.keys() if a.get(k) != b.get(k))
    details.append({'stage': owner + '/' + name, 'phases': {k: v['ref'] for k, v in phases.items()}, 'changed_paths': drift, 'post_context_recorded': after is not None, 'records': [a for a in artifacts if a['owner'] == owner and (a['input_name'].startswith(name + '/') if name != 'root' else '/' not in a['input_name'])]})


def rust_counts(data):
    rows = re.findall(r'test result: \w+\. (\d+) passed; (\d+) failed; (\d+) ignored;', data.decode())
    return dict(zip(('passed', 'failed', 'ignored'), [sum(int(r[i]) for r in rows) for i in range(3)]))


def exported(rel):
    return (OUT / rel).read_bytes()

summary = {
    'schema': 1, 'issue': 230,
    'packaging_attempts': [{'script': 'package_sources.failed01.py', 'log': 'package_sources.failed01.log', 'result': 'refused omitted native C source prefix before manifest creation'}, {'script': 'package_sources.py', 'result': 'current attempt'}],
    'scope': 'Immutable public source/log export only; no product tests or native runs were executed by this packaging script.',
    'host_default_workspace': {'counts': rust_counts(exported('logs/lead/host-integrated-2.log')), 'log': 'logs/lead/host-integrated-2.log', 'checks': 'metadata/lead/host-integrated.json'},
    'host_optional_client': {'default': rust_counts(exported('logs/client/final02-default.log')), 'network_mail': rust_counts(exported('logs/client/final02-network.log')), 'logs': ['logs/client/final02-default.log', 'logs/client/final02-network.log']},
    'host_python_base': {'tests': int(re.search(rb'Ran (\d+) tests', exported('logs/lead/host-python.log')).group(1)), 'log': 'logs/lead/host-python.log', 'boundary': 'This 391-test run predates later guard-test additions; focused guard runs are separate records.'},
    'focused_python_runs': {},
    'native_client_reported': {'passed': 137, 'ignored': 4, 'unused_decoder_fixture_tests': 7, 'boundary': 'Reported by native acceptance agent; actual native logs/source maps belong to its separate immutable export and must be integrated independently.'},
    'historical_failures': {'retained': True, 'boundary': 'Names containing green may still contain observed failures. Original logs/status values are authoritative; no historical source/result hashes were refreshed.'},
    'server_context_drift': [],
    'privacy': {'export_policy': 'Only public source snapshots, public execution scripts, logs and source/hash metadata; binaries, archives, runtime configurations and private key material excluded.', 'private_key_block_and_binary_scan': 'pass', 'actual_private_value_scan': 'pending separately coordinated native/combined publication scan; no private inventory is exported'},
    'platform_binding_summary_failure': {'record': 'metadata/lead/platform-binding.json', 'description': 'Original summary writer shadowed CompletedProcess with Path after successful GREEN; original script, logs and honest descriptor are preserved.'},
    'source_object_count': len(used), 'source_stage_count': len(stages),
}
for p in sorted((OUT / 'logs/lead').glob('*.log')):
    data = p.read_bytes()
    m = re.search(rb'Ran (\d+) tests', data)
    if m:
        summary['focused_python_runs'][p.relative_to(OUT).as_posix()] = {'tests': int(m.group(1)), 'result': 'pass' if re.search(rb'(?m)^OK\s*$', data) else 'observed failure; see original log'}
for d in details:
    if d.get('stage', '').startswith('server/') and d.get('changed_paths'):
        summary['server_context_drift'].append({'stage': d['stage'], 'paths': d['changed_paths'], 'boundary': 'Formatting stages intentionally mutate input bytes. Client transport and proof-driver drift lie outside the server/terminal/CLI compiled dependency graph; they are retained without a whole-context stability claim.'})
if summary['host_default_workspace']['counts']['passed'] != 473 or summary['host_optional_client']['network_mail']['passed'] != 137:
    raise ValueError('Unexpected final host counts')
write('source-stages.json', (json.dumps(stages, indent=2) + '\n').encode())
write('stage-details.json', (json.dumps(details, indent=2) + '\n').encode())
write('summary.json', (json.dumps(summary, indent=2) + '\n').encode())
write('object-index.json', (json.dumps({h: {'path': 'source-objects/' + h, 'bytes': len(objects[h]), 'logical_names': sorted(logical_names[h])} for h in sorted(used)}, indent=2) + '\n').encode())
write('OWNED', b'Owned disposable public evidence packaging for issue 230. Inputs remain intact.\n')
write('input-freeze.json', (json.dumps({'lead_root_files_frozen': FROZEN_LEAD_FILES, 'source_inputs_read': len(input_hashes), 'scope': 'Only the retained inputs recorded here were read; later-added files are separate stages.'}, indent=2) + '\n').encode())

# Manifest pins every exported byte except itself and the external invocation log.
manifest = {p.relative_to(OUT).as_posix(): digest(p.read_bytes()) for p in sorted(OUT.rglob('*')) if p.is_file() and p.name != 'manifest.json'}
write('manifest.json', (json.dumps(manifest, indent=2) + '\n').encode())
print(json.dumps({'result': 'pass', 'manifest_entries': len(manifest), 'source_objects': len(used), 'source_stages': len(stages), 'source_references': sum(len(x['files']) for x in stages), 'host_workspace': summary['host_default_workspace']['counts'], 'host_client': summary['host_optional_client'], 'host_python': summary['host_python_base']['tests'], 'focused_python_runs': summary['focused_python_runs']}, indent=2))
