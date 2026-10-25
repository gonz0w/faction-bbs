#!/usr/bin/env python3
"""Compare trusted signing executables using only disposable synthetic keys.

Nonzero exits never become success because an artifact exists. Certificate-backed
SSHSIG verification runs through the separate verifier; no daemon/listener starts.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import resource
import subprocess
import tempfile


def accepted(exit_code, artifact, signature_verified):
    return exit_code == 0 and artifact and signature_verified


def _run(argv, data=None):
    env = os.environ.copy()
    env.pop('SSH_AUTH_SOCK', None)
    try:
        return subprocess.run(list(map(str, argv)), input=data, capture_output=True,
                              timeout=10, env=env)
    except subprocess.TimeoutExpired:
        # subprocess.run kills and waits for the timed-out child. Do not export
        # its captured output; it can contain fixture paths or key fingerprints.
        return subprocess.CompletedProcess(argv, None, b'', b'')


def run(keygen, verifier, parent=None):
    keygen, verifier = Path(keygen).resolve(strict=True), Path(verifier).resolve(strict=True)
    resource.setrlimit(resource.RLIMIT_CORE, (0, 0))
    controls = []
    with tempfile.TemporaryDirectory(prefix='faction-signing-exit-', dir=parent) as tmp:
        root = Path(tmp)
        for name in ['ca', 'device', 'wrong-ca']:
            result = _run([verifier, '-q', '-t', 'ed25519', '-N', '', '-f', root/name])
            if result.returncode != 0:
                raise RuntimeError('synthetic key generation failed')
        allowed = root/'allowed'
        wrong = root/'wrong'
        options = 'synthetic cert-authority,namespaces="faction92" '
        allowed.write_text(options+(root/'ca.pub').read_text())
        wrong.write_text(options+(root/'wrong-ca.pub').read_text())
        cert = root/'device-cert.pub'
        message = b'synthetic certificate exit control\n'
        for label, flags in [('basic', []), ('short_validity', ['-V', '-1m:+4m']),
                             ('clear_extensions', ['-O', 'clear']),
                             ('forced_command', ['-O', 'force-command=/usr/bin/true']),
                             ('quiet', ['-q'])]:
            cert.unlink(missing_ok=True)
            result = _run([keygen, '-s', root/'ca', '-I', 'synthetic', '-n', 'synthetic', *flags, root/'device.pub'])
            artifact = cert.is_file()
            checks = dict(signature_verified=False, tamper_denied=False,
                          wrong_principal_denied=False, wrong_ca_denied=False)
            if result.returncode == 0 and artifact:
                signed = _run([verifier, '-q', '-Y', 'sign', '-f', cert, '-n', 'faction92'], message)
                if signed.returncode == 0:
                    signature = root/'message.sig'
                    signature.write_bytes(signed.stdout)
                    def verify(body, principal='synthetic', authority=allowed):
                        return _run([verifier, '-Y', 'verify', '-f', authority, '-I', principal,
                                     '-n', 'faction92', '-s', signature], body).returncode
                    checks = dict(signature_verified=verify(message) == 0,
                                  tamper_denied=(verify(b'tampered\n') or 0) > 0,
                                  wrong_principal_denied=(verify(message, 'other') or 0) > 0,
                                  wrong_ca_denied=(verify(message, authority=wrong) or 0) > 0)
            controls.append(dict(case=label, exit=result.returncode, timed_out=result.returncode is None, artifact_produced=artifact,
                                 accepted=accepted(result.returncode, artifact, all(checks.values())), **checks))
        cert.unlink(missing_ok=True)
        invalid = _run([keygen, '-s', root/'absent-ca', '-I', 'synthetic', '-n', 'synthetic', root/'device.pub'])
        invalid_ca_denied = (invalid.returncode or 0) > 0 and not cert.exists()
    return dict(result='pass' if all(c['accepted'] for c in controls) and invalid_ca_denied else 'observed_failure',
                controls=controls, count=len(controls), invalid_ca_denied=invalid_ca_denied,
                core_dumps_disabled=True, keys_removed=not root.exists(),
                keygen_sha256=hashlib.sha256(keygen.read_bytes()).hexdigest(),
                verifier_sha256=hashlib.sha256(verifier.read_bytes()).hexdigest())


if __name__ == '__main__':
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--keygen', type=Path, required=True)
    p.add_argument('--verifier', type=Path, default=Path('/usr/bin/ssh-keygen'))
    p.add_argument('--output', type=Path, required=True)
    p.add_argument('--disposable-synthetic-fixture', action='store_true', required=True)
    a = p.parse_args()
    result = run(a.keygen, a.verifier)
    a.output.write_text(json.dumps(result, indent=2)+'\n')
    print('Synthetic signing matrix:', result['result'], '/', result['count'], 'cases; no keys exported.')
