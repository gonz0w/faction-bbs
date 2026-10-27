"""Owned #254 exact source/command capture. No dependency setup or downloads."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import time

ROOT = Path('/Users/cam/DEV/faction-bbs')
FIXTURE = Path('/private/tmp/faction254-crypto-OWNED')
PATHS = [
    'Cargo.lock', 'crates/faction-omemo/Cargo.toml',
    'crates/faction-omemo/build.rs', 'crates/faction-omemo/src/lib.rs',
    'crates/faction-omemo/src/attachment.rs', 'crates/faction-omemo/src/native.rs',
    'crates/faction-omemo/src/blob.rs', 'crates/faction-omemo/native/attachment.c',
    'crates/faction-omemo/tests/blob_contract.rs',
    'crates/faction-omemo/tests/blob_native.rs',
    'crates/faction-omemo/tests/blob_guard.rs',
    'crates/faction-omemo/tests/blob_guard.c',
    'crates/faction-omemo/tests/blob_vectors.rs',
]
ENV = dict(OPENSSL_INCLUDE_DIR='/opt/homebrew/opt/openssl@3/include',
           OPENSSL_LIB_DIR='/opt/homebrew/opt/openssl@3/lib',
           CARGO_BUILD_JOBS='2', CARGO_TARGET_DIR=str(FIXTURE/'target'),
           TMPDIR=str(FIXTURE))

def capture(label):
    target = FIXTURE/'captures'/label
    target.mkdir(mode=0o700)
    present, missing = {}, []
    for name in PATHS:
        path = ROOT/name
        if not path.exists():
            missing.append(name)
            continue
        raw = path.read_bytes()
        copied = target/name
        copied.parent.mkdir(parents=True, exist_ok=True)
        copied.write_bytes(raw)
        present[name] = hashlib.sha256(raw).hexdigest()
    (target/'manifest.json').write_text(json.dumps(dict(files=present, missing=missing), indent=2)+'\n')
    return present

label, *command = sys.argv[1:]
sources = capture(label)
if command:
    env = os.environ.copy()
    env.update(ENV)
    run = FIXTURE/'runs'/label
    run.mkdir(mode=0o700)
    start = time.time()
    with (run/'command.log').open('wb') as log:
        result = subprocess.run(command, cwd=ROOT, env=env, stdout=log, stderr=subprocess.STDOUT)
    record = dict(command=command, cwd=str(ROOT), environment=ENV,
                  started_at_epoch=start, elapsed_seconds=time.time()-start,
                  exit=result.returncode, source_sha256=sources)
    (run/'record.json').write_text(json.dumps(record, indent=2)+'\n')
    print(json.dumps(dict(label=label, exit=result.returncode, elapsed_seconds=record['elapsed_seconds'])))
    print((run/'command.log').read_text())
    sys.exit(result.returncode)
