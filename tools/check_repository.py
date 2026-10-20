#!/usr/bin/env python3
"""Validate current docs, no-JS file policy, issue provenance, and source archives.

Uses Python 3.10+ standard library only. It makes no network calls and is not a
replacement for semantic documentation review or dependency/security analysis.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import sys
from pathlib import Path
from urllib.parse import unquote, urlsplit


ROOT = Path(__file__).resolve().parents[1]
IGNORED = {'.git', '__pycache__', '.venv', 'target', 'local', 'node_modules'}
JS_SUFFIXES = {'.js', '.jsx', '.mjs', '.cjs', '.ts', '.tsx', '.mts', '.cts'}
JS_MANIFESTS = {'package.json', 'package-lock.json', 'npm-shrinkwrap.json',
                'yarn.lock', 'pnpm-lock.yaml', 'bun.lock', 'bun.lockb',
                'deno.json', 'deno.jsonc'}
LINK = re.compile(r'!?\[[^\]\n]*\]\(([^)\n]+)\)')


def read_json(root: Path, relative: str, errors: list[str]):
    try:
        return json.loads((root / relative).read_text())
    except (OSError, ValueError) as exc:
        errors.append(f'{relative}: cannot read valid JSON ({exc})')
        return None


def validate(root: Path, *, allow_unpublished: bool = False) -> list[str]:
    errors: list[str] = []
    root = root.resolve()
    required = [
        'README.md', 'AGENTS.md', 'CONTRIBUTING.md', 'SECURITY.md', 'LICENSE',
        'CHANGELOG.md', '.gitignore', '.gitattributes', '.editorconfig',
        'docs/README.md', 'docs/project.md', 'docs/session.md',
        'docs/security-privacy.md', 'dev-docs/README.md', 'dev-docs/workflow.md',
        'dev-docs/roadmap.md', 'dev-docs/issue-index.md',
        'dev-docs/issue-map.json', 'dev-docs/archive/manifest.json',
        '.github/workflows/documentation.yml',
        '.github/ISSUE_TEMPLATE/research.yml', '.github/ISSUE_TEMPLATE/feature.yml',
        '.github/ISSUE_TEMPLATE/bug_report.yml', '.github/pull_request_template.md',
    ]
    for relative in required:
        if not (root / relative).is_file():
            errors.append(f'Missing required foundation file: {relative}')
    if (root / '.planning').exists():
        errors.append('Retired .planning directory must not exist at repository root')

    for path in root.rglob('*'):
        relative = path.relative_to(root)
        # Still flag a JS dependency directory even though its contents are skipped.
        if path.is_dir() and path.name == 'node_modules' and '.git' not in relative.parts:
            errors.append(f'JavaScript dependency directory is prohibited: {relative}')
        if not path.is_file() or any(part in IGNORED for part in relative.parts):
            continue
        if path.suffix.lower() in JS_SUFFIXES or path.name in JS_MANIFESTS:
            errors.append(f'JavaScript/TypeScript source or dependency manifest is prohibited: {relative}')
        if relative.parts[:2] == ('dev-docs', 'archive'):
            continue
        if path.suffix not in {'.md', '.py', '.json', '.yml', '.yaml'}:
            continue
        try:
            content = path.read_text(encoding='utf-8')
        except (OSError, UnicodeError) as exc:
            errors.append(f'{relative}: not readable UTF-8 ({exc})')
            continue
        if not content.endswith('\n'):
            errors.append(f'{relative}: missing final newline')
        if re.search(r'^(<<<<<<< |=======$|>>>>>>> )', content, re.M):
            errors.append(f'{relative}: unresolved merge conflict')
        if path.suffix == '.md':
            if relative.as_posix() not in {'AGENTS.md', 'dev-docs/migration.md'} and '.planning/' in content:
                errors.append(f'{relative}: active reference to retired planning paths')
            # Ignore fenced examples; validate normal inline Markdown file targets.
            prose = re.sub(r'```.*?```', '', content, flags=re.S)
            for raw in LINK.findall(prose):
                target = raw.strip()
                if target.startswith('<'):
                    target = target[1:target.find('>')]
                else:
                    target = target.split(' "', 1)[0].split(" '", 1)[0]
                parts = urlsplit(target)
                if parts.scheme or target.startswith('//') or not parts.path:
                    continue
                destination = (path.parent / unquote(parts.path)).resolve()
                if not destination.is_relative_to(root):
                    errors.append(f'{relative}: link leaves repository: {target}')
                elif not destination.exists():
                    errors.append(f'{relative}: broken local link: {target}')

    readme_path = root / 'README.md'
    if readme_path.is_file():
        readme = readme_path.read_text()
        for heading in ['# Faction BBS', '## Status', '## License and heritage']:
            if heading not in readme:
                errors.append(f'README.md: missing required section {heading}')
        if not re.search(r'updated \d{4}-\d{2}-\d{2}', readme):
            errors.append('README.md: status must carry an explicit update date')
    if (root / 'LICENSE').is_file():
        license_text = (root / 'LICENSE').read_text()
        if not all(text in license_text for text in ['BSD 3-Clause License',
                'Neither the name of the copyright holder', 'THIS SOFTWARE IS PROVIDED']):
            errors.append('LICENSE: expected BSD-3-Clause terms are missing')

    # The shell-only public checkout intentionally avoids JS-based marketplace actions.
    workflow = root / '.github/workflows/documentation.yml'
    if workflow.is_file():
        text = workflow.read_text()
        if re.search(r'^\s*(?:-\s*)?uses\s*:', text, re.M):
            errors.append('Foundation CI must use shell/Python steps, without external actions')
        if 'contents: read' not in text or 'pull_request_target' in text:
            errors.append('Foundation CI must retain read-only permissions and unprivileged PR events')

    manifest = read_json(root, 'dev-docs/archive/manifest.json', errors)
    if manifest is not None:
        if not isinstance(manifest, list) or not manifest:
            errors.append('Archive manifest must be a non-empty list')
        else:
            original_paths, archived_paths = set(), set()
            for entry in manifest:
                if not isinstance(entry, dict) or not {'original', 'archived', 'sha256'} <= entry.keys():
                    errors.append('Archive manifest contains a malformed entry')
                    continue
                original, archived = entry['original'], entry['archived']
                if original in original_paths or archived in archived_paths:
                    errors.append(f'Duplicate archive manifest mapping: {original}')
                original_paths.add(original)
                archived_paths.add(archived)
                path = (root / archived).resolve()
                archive_root = root / 'dev-docs/archive'
                if not path.is_relative_to(archive_root) or not path.is_file():
                    errors.append(f'Missing or invalid archive target: {archived}')
                elif hashlib.sha256(path.read_bytes()).hexdigest() != entry['sha256']:
                    errors.append(f'Archive integrity mismatch: {archived}')
            # An unlisted historical source file must not slip past the hash check.
            for subdir in ['planning-2026-10-19', 'seed']:
                for path in (root / 'dev-docs/archive' / subdir).rglob('*'):
                    if path.is_file() and path.relative_to(root).as_posix() not in archived_paths:
                        errors.append(f'Historical file missing from manifest: {path.relative_to(root)}')

    mapping = read_json(root, 'dev-docs/issue-map.json', errors)
    if mapping is not None:
        if not isinstance(mapping, list):
            errors.append('Issue map must be a list')
        else:
            keys, numbers, legacy_ids = set(), set(), set()
            index = (root / 'dev-docs/issue-index.md').read_text() if (root / 'dev-docs/issue-index.md').is_file() else ''
            for item in mapping:
                if not isinstance(item, dict) or not {'key','legacy_id','number','url','source'} <= item.keys():
                    errors.append('Issue map contains a malformed entry')
                    continue
                key = item['key']
                if key in keys:
                    errors.append(f'Duplicate stable work ID: {key}')
                keys.add(key)
                if item['legacy_id']:
                    if item['legacy_id'] in legacy_ids:
                        errors.append(f'Duplicate legacy issue mapping: {item["legacy_id"]}')
                    legacy_ids.add(item['legacy_id'])
                if not (root / item['source']).is_file():
                    errors.append(f'{key}: issue provenance source is missing')
                number = item['number']
                if number is None and allow_unpublished:
                    continue
                if not isinstance(number, int) or number < 1 or number in numbers:
                    errors.append(f'{key}: missing, invalid, or duplicate GitHub issue number')
                numbers.add(number)
                expected = f'https://github.com/gonz0w/faction-bbs/issues/{number}'
                if item['url'] != expected or expected not in index:
                    errors.append(f'{key}: GitHub issue URL/index mapping mismatch')
            if legacy_ids != {f'BG-{n:03}' for n in range(1,21)}:
                errors.append('Every legacy candidate BG-001 through BG-020 must map exactly once')
    return errors


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--allow-unpublished', action='store_true',
                        help='Allow unmapped GitHub issue numbers only during initial preparation')
    args = parser.parse_args()
    errors = validate(ROOT, allow_unpublished=args.allow_unpublished)
    if errors:
        print('\n'.join(f'ERROR: {error}' for error in errors), file=sys.stderr)
        return 1
    print('Repository checks passed: current doc links, foundation policy, issue provenance, and archive hashes.')
    print('README semantic freshness, transitive dependencies, and runtime support still require review.')
    return 0


if __name__ == '__main__':
    sys.exit(main())
