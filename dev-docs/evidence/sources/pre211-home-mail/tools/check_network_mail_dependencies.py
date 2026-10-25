#!/usr/bin/env python3
"""Verify the default-off product network-mail dependency adoption, offline.

Select the complete workspace graph with faction-client/network-mail for Linux,
NetBSD and macOS. Check each selected registry package's full source tree,
including build/test/CI files, against its exact locked crate archive. Upstream
CI is inspected as source only and is never executed or imported. Unselected
platform packages are not adopted. This is a source/license/feature integrity
check, not a vulnerability, generated-code, unsafe-code or runtime security audit.
"""
import argparse
import hashlib
import json
import subprocess
import tarfile
import tomllib
from pathlib import Path, PurePosixPath

from check_omemo_dependencies import check_vendor
from check_rust_dependencies import MANIFESTS, SUFFIXES

ROOT = Path(__file__).resolve().parents[1]
INVENTORY = ROOT / 'dev-docs/dependencies/network-mail.json'
TARGETS = ['x86_64-unknown-linux-gnu', 'aarch64-unknown-netbsd', 'aarch64-apple-darwin']
REGISTRY = 'registry+https://github.com/rust-lang/crates.io-index'
# Existing product/native and qualified #208 license declarations. Adding a new
# expression requires deliberate source/license review, not merely --write.
LICENSES = {
    '(MIT OR Apache-2.0) AND Unicode-3.0', 'Apache-2.0',
    'Apache-2.0 OR BSL-1.0', 'Apache-2.0 OR MIT',
    'Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT', 'BSD-3-Clause',
    'MIT', 'MIT OR Apache-2.0', 'MIT/Apache-2.0', 'MPL-2.0', 'Unicode-3.0',
    'Unlicense OR MIT', 'Zlib OR Apache-2.0 OR MIT',
}
TRANSPORT = {
    'tokio-xmpp': {'version': '=6.0.0', 'default-features': False, 'features': ['native-tls']},
    'native-tls': '=0.2.18', 'tokio-native-tls': '=0.3.1',
    'tokio': {'version': '=1.54.1', 'default-features': False, 'features': ['rt', 'time', 'net', 'io-util', 'macros']},
    'rxml': {'version': '=0.14.0', 'default-features': False},
    'sasl': '=0.5.2',
    'futures-util': {'version': '=0.3.34', 'default-features': False, 'features': ['sink']},
    'sha2': '=0.10.9',
    'log': {'version': '=0.4.34', 'features': ['max_level_off', 'release_max_level_off']},
    'xso': {'version': '=0.4.0', 'default-features': False},
}


def require(condition, message):
    if not condition:
        raise ValueError(message)


def digest(path):
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def checksum_valid(value):
    return isinstance(value, str) and len(value) == 64 and all(c in '0123456789abcdef' for c in value)


def scan_tree(root):
    require(root.is_dir() and not root.is_symlink(), 'dependency source is not a regular directory')
    for path in root.rglob('*'):
        require(not path.is_symlink(), 'dependency source symlink')
        require(path.is_dir() or path.is_file(), 'non-regular dependency source')
        require(path.name.lower() != 'node_modules', 'dependency node_modules directory')
        if path.is_file():
            require(path.suffix.lower() not in SUFFIXES and path.name.lower() not in MANIFESTS,
                    'dependency violates no-JS source policy')


def scan_package(package, features, checksum):
    require(package.get('source') == REGISTRY, 'unreviewed registry source')
    require(package.get('license') in LICENSES, 'unreviewed or missing license expression')
    require(checksum_valid(checksum), 'missing or malformed locked archive checksum')
    scan_tree(Path(package['manifest_path']).parent)
    return dict(name=package['name'], version=package['version'],
                license=package['license'], checksum=checksum, features=sorted(features))


def scan_local(package, features, repository):
    path = Path(package['manifest_path'])
    relative = path.relative_to(repository)
    require(relative.parts[0] == 'crates' and '..' not in relative.parts
            and path.resolve().is_relative_to((repository / 'crates').resolve())
            and package.get('source') is None and package.get('license') == 'BSD-3-Clause',
            'unreviewed local package location or license')
    scan_tree(path.parent)
    return dict(name=package['name'], version=package['version'], license=package['license'],
                manifest=relative.as_posix(), manifest_sha256=digest(path), features=sorted(features))


def verify_archive_source(root, archive, checksum):
    """Read, never extract, the locked archive; check exact source membership/bytes."""
    require(root.is_dir() and not root.is_symlink(), 'registry source directory missing or linked')
    require(archive.is_file() and not archive.is_symlink(), 'locked registry archive missing or linked')
    require(checksum_valid(checksum) and digest(archive) == checksum, 'locked archive checksum changed')
    expected = {}
    with tarfile.open(archive) as tar:
        for member in tar:
            path = PurePosixPath(member.name)
            parts = path.parts
            require(not path.is_absolute() and '..' not in parts and parts
                    and parts[0] == root.name
                    and path.as_posix() == member.name.rstrip('/'), 'noncanonical registry archive path')
            if member.isdir():
                continue
            require(member.isfile() and len(parts) > 1, 'non-regular registry archive member')
            relative = PurePosixPath(*parts[1:]).as_posix()
            require(relative not in expected, 'duplicate registry archive member')
            actual = root / relative
            require(actual.is_file() and not actual.is_symlink(), 'missing or linked extracted source')
            with tar.extractfile(member) as stream:
                archived_digest = hashlib.file_digest(stream, 'sha256').hexdigest()
            require(digest(actual) == archived_digest, 'extracted registry source changed')
            expected[relative] = (archived_digest, member.size)
    actual = set()
    for path in root.rglob('*'):
        require(not path.is_symlink(), 'extracted registry source symlink')
        require(path.is_file() or path.is_dir(), 'non-regular extracted source')
        if path.is_file():
            relative = path.relative_to(root).as_posix()
            if relative not in {'.cargo-ok', '.cargo-checksum.json'}:
                actual.add(relative)
    require(actual == set(expected), 'unreviewed or missing extracted source files')
    tree = hashlib.sha256()
    for relative, (file_digest, _) in sorted(expected.items()):
        tree.update(relative.encode('utf-8') + b'\0' + bytes.fromhex(file_digest))
    return dict(source_files=len(expected), source_bytes=sum(size for _, size in expected.values()),
                source_sha256=tree.hexdigest(), archive_bytes=archive.stat().st_size)


def selected_packages(metadata):
    packages = {p['id']: p for p in metadata['packages']}
    require(len(packages) == len(metadata['packages']), 'duplicate package metadata')
    nodes = metadata['resolve']['nodes']
    require(len({n['id'] for n in nodes}) == len(nodes), 'duplicate selected graph node')
    require(all(n['id'] in packages for n in nodes), 'selected graph node has no source package')
    return [(packages[node['id']], node['features']) for node in nodes]


def validate_profile(workspace, client, transport):
    require(not any('patch' in p or 'replace' in p for p in [workspace, client, transport]),
            'unreviewed dependency replacement')
    require('crates/faction-xmpp' in workspace['workspace']['exclude'], 'transport must remain optional and excluded')
    require(client['features'].get('default') == [], 'network mail must remain default-off')
    require(set(client['features']['network-mail']) == {'native-mail', 'dep:faction-xmpp', 'dep:tokio', 'dep:roxmltree'},
            'network feature profile changed')
    require(set(client['features']['native-mail']) == {'dep:faction-omemo', 'faction-omemo/native'},
            'native feature profile changed')
    deps = client['dependencies']
    require(deps['faction-xmpp'] == dict(path='../faction-xmpp', optional=True), 'transport must remain a local optional dependency')
    require(deps['tokio'] == dict(TRANSPORT['tokio'], optional=True), 'client Tokio profile changed')
    require(deps['roxmltree'] == dict(workspace=True, optional=True), 'network XML helper must remain optional')
    require(transport['package'].get('publish') is False and 'workspace' in transport,
            'standalone transport package policy changed')
    require(transport['dependencies'] == TRANSPORT, 'qualified transport dependency profile changed')
    require(transport.get('dev-dependencies') == {'base64': '=0.22.1'}, 'transport test dependencies changed')


def check_inventory(path, current):
    require(path.is_file() and path.read_text() == current,
            'network-mail inventory changed; review source/license/features, then use --write deliberately')


def inventory():
    manifest_paths = [ROOT / 'Cargo.toml', ROOT / 'crates/faction-client/Cargo.toml', ROOT / 'crates/faction-xmpp/Cargo.toml']
    validate_profile(*(tomllib.loads(p.read_text()) for p in manifest_paths))
    vendor = check_vendor()
    lock_path = ROOT / 'Cargo.lock'
    lock = tomllib.loads(lock_path.read_text())
    checks = {(p['name'], p['version'], p['source']): p.get('checksum')
              for p in lock['package'] if p.get('source')}
    graphs = {}
    verified = {}
    for target in TARGETS:
        data = json.loads(subprocess.check_output([
            'cargo', 'metadata', '--manifest-path', str(ROOT / 'Cargo.toml'),
            '--locked', '--offline', '--format-version', '1', '--filter-platform', target,
            '--features', 'faction-client/network-mail'], text=True))
        rows, local = [], []
        for package, features in selected_packages(data):
            source = package.get('source')
            path = Path(package['manifest_path'])
            if source is None:
                local.append(scan_local(package, features, ROOT))
                continue
            key = (package['name'], package['version'], source)
            require(key in checks, 'selected package missing from root lock')
            row = scan_package(package, features, checks[key])
            if key not in verified:
                root = path.parent
                archive = root.parents[2] / 'cache' / root.parent.name / (package['name'] + '-' + package['version'] + '.crate')
                verified[key] = verify_archive_source(root, archive, checks[key])
            row.update(verified[key])
            rows.append(row)
        graphs[target] = dict(registry=sorted(rows, key=lambda p: (p['name'], p['version'])),
                              local=sorted(local, key=lambda p: p['name']))
    return json.dumps(dict(schema=1, scope='product-workspace-with-default-off-network-mail',
                           selected_feature='faction-client/network-mail', upstream_ci_selected=False,
                           root_lock_sha256=digest(lock_path),
                           manifests={p.relative_to(ROOT).as_posix(): digest(p) for p in manifest_paths},
                           native_vendor_sha256=vendor, graphs=graphs), indent=2, sort_keys=True) + '\n'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument('--write', action='store_true', help='Write only after deliberate selected source/license/feature review')
    mode.add_argument('--check', action='store_true', help='Compare with the reviewed inventory (default)')
    args = parser.parse_args()
    current = inventory()
    if args.write:
        INVENTORY.write_text(current)
        print('Wrote product network-mail inventory; review exact sources, licenses and features before adoption.')
    else:
        check_inventory(INVENTORY, current)
        print('Selected Linux/NetBSD/macOS product network-mail sources match locked archives and reviewed policy; no security audit claim.')


if __name__ == '__main__':
    try:
        main()
    except (ValueError, OSError, tarfile.TarError) as error:
        raise SystemExit(str(error)) from None
