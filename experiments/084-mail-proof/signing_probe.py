#!/usr/bin/env python3
"""Record a native certificate-signing exit defect with disposable synthetic keys.

Never trust a certificate emitted by a failed signer. Core dumps are disabled for
this process and children so diagnostic dumps cannot retain fixture private keys.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import resource
import shutil
import subprocess
import tempfile


def run():
    resource.setrlimit(resource.RLIMIT_CORE, (0, 0))
    root = Path(tempfile.mkdtemp(prefix='faction-signing-control-'))
    old = Path.cwd()
    os.chdir(root)
    try:
        for name in ['ca', 'device']:
            subprocess.run(['/usr/bin/ssh-keygen', '-q', '-t', 'ed25519', '-N', '', '-f', str(root/name)], check=True, capture_output=True)
        controls = []
        for label, options in [('basic', []), ('short_validity', ['-V', '-1m:+4m']), ('clear_extensions', ['-O', 'clear']), ('forced_command', ['-O', 'force-command=/usr/bin/true'])]:
            r = subprocess.run(['/usr/bin/ssh-keygen', '-s', str(root/'ca'), '-I', 'synthetic', '-n', 'synthetic', *options, str(root/'device.pub')], capture_output=True, timeout=10)
            controls.append({'case': label, 'exit': r.returncode, 'artifact_produced': (root/'device-cert.pub').is_file()})
            (root/'device-cert.pub').unlink(missing_ok=True)
        return {'result': 'observed_failure' if any(x['exit'] for x in controls) else 'pass', 'controls': controls, 'count': len(controls), 'core_dumps_disabled': True, 'keys_removed': True, 'ssh_keygen_sha256': hashlib.sha256(Path('/usr/bin/ssh-keygen').read_bytes()).hexdigest(), 'ssh_version': subprocess.run(['/usr/bin/ssh', '-V'], capture_output=True, text=True).stderr.strip()}
    finally:
        os.chdir(old)
        shutil.rmtree(root)


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--output', type=Path, required=True)
    p.add_argument('--disposable-synthetic-fixture', action='store_true', required=True)
    a = p.parse_args()
    result = run()
    a.output.write_text(json.dumps(result, indent=2)+'\n')
    print('Synthetic signing controls recorded:', result['result'])


if __name__ == '__main__': main()
