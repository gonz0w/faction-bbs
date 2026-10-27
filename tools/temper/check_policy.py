#!/usr/bin/env python3
"""Inspect explicit policy selections as inert data; never decide authority."""

import argparse
from datetime import date
import json
import os
from pathlib import Path
import re
import stat
import sys
import tomllib


MAX_MANIFEST = 64 * 1024
MAX_REFERENCE = 256 * 1024
MAX_ENTRIES = 64
ID = re.compile(r'[a-z][a-z0-9_.-]{0,63}\Z')
DATE = re.compile(r'[0-9]{4}-[0-9]{2}-[0-9]{2}\Z')
MESSAGES = {
    'unsafe_reference': 'Use a project-relative regular file with no traversal, fragment or symlink components.',
    'missing_reference': 'Restore the declared file or correct the selected reference.',
    'unreadable_input': 'Check input availability and read permissions under project policy.',
    'input_too_large': 'Input exceeds the documented byte limit.',
    'invalid_reference': 'Referenced policy documents must be UTF-8 text.',
    'invalid_manifest': 'Check TOML, schema_version=1, allowed fields and bounded module/exception arrays.',
    'invalid_module': 'Use valid id/path/owner fields and unique bounded requires/conflicts ID lists.',
    'invalid_exception': 'Use valid id/path/modules and an ISO date or nonempty expiry-event description.',
    'duplicate_id': 'Each module ID and each exception ID must be unique within its own collection.',
    'missing_dependency': 'Select all declared required modules or correct the dependency.',
    'selected_conflict': 'Resolve the declared conflict through the authorized policy process.',
    'dependency_cycle': 'Resolve the selected module dependency cycle.',
    'unknown_exception_module': 'Exception module references must name selected modules.',
    'expired_exception': 'Declared date expiry precedes as-of; inspect the active exception record.',
}


def identifier(value):
    return isinstance(value, str) and ID.fullmatch(value) is not None


def text_field(value):
    return isinstance(value, str) and 0 < len(value) <= 128 and bool(value.strip()) and value.isprintable()


def id_list(value):
    return (isinstance(value, list) and len(value) <= MAX_ENTRIES
            and all(identifier(item) for item in value) and len(set(value)) == len(value))


def calendar_date(value):
    if type(value) is date:
        return value
    if not isinstance(value, str) or not DATE.fullmatch(value):
        raise ValueError('Invalid ISO calendar date')
    return date.fromisoformat(value)


class Inspection:
    def __init__(self):
        self.errors = []

    def error(self, code, location):
        item = {'code': code, 'location': location, 'message': MESSAGES[code]}
        if item not in self.errors:
            self.errors.append(item)

    def directory(self, path, location):
        try:
            for parent in (*reversed(path.parents), path):
                if not stat.S_ISDIR(parent.lstat().st_mode):
                    self.error('unsafe_reference', location)
                    return False
            return True
        except FileNotFoundError:
            self.error('missing_reference', location)
        except (OSError, ValueError):
            self.error('unreadable_input', location)
        return False

    def path(self, root, relative, location, suffix):
        if (not isinstance(relative, str) or not relative or len(relative) > 512
                or not relative.isprintable() or any(c in relative for c in '\\:#?')
                or any(part in ('', '.', '..') for part in relative.split('/'))
                or not relative.endswith(suffix)):
            self.error('unsafe_reference', location)
            return None
        path = root / relative
        if path.is_absolute() and not path.is_relative_to(root):
            self.error('unsafe_reference', location)
            return None
        return path

    def read(self, path, location, limit, text_error):
        if not self.directory(path.parent, location):
            return None
        try:
            info = path.lstat()
            if not stat.S_ISREG(info.st_mode):
                self.error('unsafe_reference', location)
                return None
            if info.st_size > limit:
                self.error('input_too_large', location)
                return None
            flags = os.O_RDONLY | getattr(os, 'O_NOFOLLOW', 0) | getattr(os, 'O_NONBLOCK', 0)
            descriptor = os.open(path, flags)
            with os.fdopen(descriptor, 'rb') as stream:
                if not stat.S_ISREG(os.fstat(stream.fileno()).st_mode):
                    self.error('unsafe_reference', location)
                    return None
                data = stream.read(limit + 1)
            if len(data) > limit:
                self.error('input_too_large', location)
                return None
            return data.decode('utf-8')
        except FileNotFoundError:
            self.error('missing_reference', location)
        except UnicodeDecodeError:
            self.error(text_error, location)
        except (OSError, ValueError):
            self.error('unreadable_input', location)
        return None

    def reference(self, root, relative, location):
        path = self.path(root, relative, location, '.md')
        if path is not None:
            self.read(path, location, MAX_REFERENCE, 'invalid_reference')
        return path is not None

    def modules(self, root, entries):
        modules = {}
        for index, entry in enumerate(entries):
            location = f'modules[{index}]'
            if (not isinstance(entry, dict) or not {'id', 'path', 'owner'} <= entry.keys()
                    or entry.keys() - {'id', 'path', 'owner', 'requires', 'conflicts'}
                    or not identifier(entry.get('id')) or not text_field(entry.get('owner'))
                    or not id_list(entry.get('requires', []))
                    or not id_list(entry.get('conflicts', []))):
                self.error('invalid_module', location)
                continue
            name = entry['id']
            if name in modules:
                self.error('duplicate_id', f'modules/{name}')
                continue
            if not self.reference(root, entry['path'], f'modules/{name}'):
                continue
            modules[name] = {'id': name, 'path': entry['path'], 'owner': entry['owner'],
                             'requires': sorted(entry.get('requires', [])),
                             'conflicts': sorted(entry.get('conflicts', []))}
        for name, module in sorted(modules.items()):
            if any(required not in modules for required in module['requires']):
                self.error('missing_dependency', f'modules/{name}')
            if any(conflict in modules for conflict in module['conflicts']):
                self.error('selected_conflict', f'modules/{name}')
        remaining = {name: set(module['requires']) & modules.keys() for name, module in modules.items()}
        while remaining:
            ready = {name for name, needs in remaining.items() if not needs}
            if not ready:
                self.error('dependency_cycle', 'modules')
                break
            remaining = {name: needs - ready for name, needs in remaining.items() if name not in ready}
        return modules

    def exceptions(self, root, entries, modules, as_of):
        observed = {}
        for index, entry in enumerate(entries):
            location = f'exceptions[{index}]'
            if (not isinstance(entry, dict) or not {'id', 'path', 'modules'} <= entry.keys()
                    or entry.keys() - {'id', 'path', 'modules', 'expires_on', 'ends_when'}
                    or not identifier(entry.get('id')) or not id_list(entry.get('modules'))
                    or not entry['modules'] or not {'expires_on', 'ends_when'} & entry.keys()
                    or ('ends_when' in entry and not text_field(entry['ends_when']))):
                self.error('invalid_exception', location)
                continue
            name = entry['id']
            if name in observed:
                self.error('duplicate_id', f'exceptions/{name}')
                continue
            expiry = None
            if 'expires_on' in entry:
                try:
                    expiry = calendar_date(entry['expires_on'])
                except ValueError:
                    self.error('invalid_exception', f'exceptions/{name}')
                    continue
            if not self.reference(root, entry['path'], f'exceptions/{name}'):
                continue
            if any(module not in modules for module in entry['modules']):
                self.error('unknown_exception_module', f'exceptions/{name}')
            if expiry is not None and expiry < as_of:
                self.error('expired_exception', f'exceptions/{name}')
            observed[name] = {'id': name, 'path': entry['path'], 'modules': sorted(entry['modules']),
                              'expires_on': expiry.isoformat() if expiry else None,
                              'event_expiry': 'unverified' if 'ends_when' in entry else 'not_declared'}
        return observed


def inspect(root, manifest, as_of):
    check = Inspection()
    report = {
        'schema_version': 1, 'as_of': as_of.isoformat(), 'modules': [], 'exceptions': [],
        'policy_conformance': 'unverified', 'selection_authority': 'unverified',
        'execution_authority': 'unverified',
        'limits': [
            'Selection is declared input; PROJECT approval and policy meaning are not interpreted.',
            'Rule prose, supersession correctness, undeclared conflicts and event expiry require review.',
            'Source trust, actual check applicability, runtime and integration/release authority are not established.',
            'Only declared local references are read; other policy files and Markdown links are not inspected.',
            'Inputs must remain unchanged; concurrent hostile filesystem mutation is not supported.',
        ],
    }
    # Preserve parent components so physical checks see any preceding symlink.
    root = Path(root).absolute()
    if check.directory(root, 'root'):
        path = check.path(root, manifest, 'manifest', '.toml')
        text = check.read(path, 'manifest', MAX_MANIFEST, 'invalid_manifest') if path is not None else None
        if text is not None:
            try:
                data = tomllib.loads(text)
            except (ValueError, RecursionError):
                check.error('invalid_manifest', 'manifest')
            else:
                if (data.keys() - {'schema_version', 'modules', 'exceptions'}
                        or type(data.get('schema_version')) is not int or data['schema_version'] != 1
                        or not isinstance(data.get('modules'), list) or len(data['modules']) > MAX_ENTRIES
                        or not isinstance(data.get('exceptions', []), list)
                        or len(data.get('exceptions', [])) > MAX_ENTRIES):
                    check.error('invalid_manifest', 'manifest')
                else:
                    modules = check.modules(root, data['modules'])
                    exceptions = check.exceptions(root, data.get('exceptions', []), modules, as_of)
                    report['modules'] = [modules[key] for key in sorted(modules)]
                    report['exceptions'] = [exceptions[key] for key in sorted(exceptions)]
    report['errors'] = sorted(check.errors, key=lambda item: (item['location'], item['code']))
    return report


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--root', required=True, help='Explicit physical project directory')
    parser.add_argument('--manifest', required=True, help='Project-relative TOML selection file')
    parser.add_argument('--as-of', required=True, help='Explicit ISO calendar date, YYYY-MM-DD')
    args = parser.parse_args()
    try:
        as_of = calendar_date(args.as_of)
    except ValueError:
        parser.error('as-of must be a valid YYYY-MM-DD calendar date')
    report = inspect(args.root, args.manifest, as_of)
    print(json.dumps(report, indent=2, sort_keys=True))
    return 1 if report['errors'] else 0


if __name__ == '__main__':
    sys.dont_write_bytecode = True
    raise SystemExit(main())
