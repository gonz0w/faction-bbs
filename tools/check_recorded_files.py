#!/usr/bin/env python3
"""Check pinned file bytes and exact selected-folder membership, without execution."""
import argparse
import hashlib
import json
from pathlib import Path, PurePosixPath
import re

SHA256 = re.compile(r"[a-f0-9]{64}")


def safe_path(root, name):
    if not isinstance(name, str):
        raise ValueError("file path must be repository-relative text")
    relative = PurePosixPath(name)
    if (not relative.parts or relative.is_absolute() or ".." in relative.parts
            or str(relative) != name or "\\" in name):
        raise ValueError(f"unsafe file path: {name}")
    path = root
    for part in relative.parts:
        path = path / part
        if path.is_symlink():
            raise ValueError(f"symlink is not a recorded file: {name}")
    if not path.resolve().is_relative_to(root):
        raise ValueError(f"file escapes repository: {name}")
    return path


def checked_bytes(path, expected):
    if not isinstance(expected, str) or not SHA256.fullmatch(expected):
        raise ValueError("expected SHA-256 must contain 64 lowercase hex digits")
    if not path.is_file():
        raise ValueError(f"missing regular file: {path}")
    raw = path.read_bytes()
    if hashlib.sha256(raw).hexdigest() != expected:
        raise ValueError(f"recorded file checksum changed: {path}")
    return raw


def verify(root, relative_manifest, expected_hash):
    """Return the verified reference count; raise for changed or unregistered bytes."""
    root = Path(root).resolve(strict=True)
    manifest = safe_path(root, relative_manifest)
    data = json.loads(checked_bytes(manifest, expected_hash))
    if (not isinstance(data, dict) or type(data.get("schema")) is not int
            or data["schema"] != 1 or not isinstance(data.get("files"), dict)
            or not data["files"]):
        raise ValueError("manifest requires schema 1 and a nonempty files mapping")
    files = data["files"]
    for name, digest in files.items():
        checked_bytes(safe_path(root, name), digest)
    actual = set()
    for path in manifest.parent.rglob("*"):
        name = path.relative_to(root).as_posix()
        safe_path(root, name)
        if path.is_file() and path != manifest:
            actual.add(name)
        elif not path.is_file() and not path.is_dir():
            raise ValueError(f"unsupported selected entry: {name}")
    parent = PurePosixPath(relative_manifest).parent
    selected = {name for name in files if PurePosixPath(name).is_relative_to(parent)}
    if actual != selected:
        raise ValueError("selected folder inventory differs from manifest references")
    return len(files)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("manifest", help="repository-relative manifest path")
    parser.add_argument("sha256", help="expected manifest SHA-256")
    args = parser.parse_args()
    try:
        count = verify(Path(__file__).resolve().parents[1], args.manifest, args.sha256)
    except (OSError, ValueError) as error:
        parser.exit(1, f"{error}\n")
    print(f"Verified {count} recorded files; saved bytes only.")
