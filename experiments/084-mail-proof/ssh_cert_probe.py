#!/usr/bin/env python3
"""Native NetBSD OpenSSH certificate controls, synthetic disposable VM ONLY.

This proves scoped SSH admission, NOT completed Faction member login or an online
home certificate issuer. No E2EE keys are used. No maintained BBS config changes.
"""
import argparse
import json
import os
from pathlib import Path
import pwd
import shutil
import subprocess
import tempfile
import time


def prepare(path):
    path.mkdir(mode=0o700)
    def cmd(args): subprocess.run(args, check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    for n in ['homea', 'homeb', 'stranger', 'device', 'otherdevice']:
        cmd(['/usr/bin/ssh-keygen', '-q', '-t', 'ed25519', '-N', '', '-f', str(path/n)])
    force = '/usr/bin/printf faction-roaming-admission-only'
    variants = [('valid-a','homea','board-b/alice@homea.test','-1m:+4m'), ('valid-b','homeb','board-b/alice@homeb.test','-1m:+4m'), ('wrong-board','homea','board-c/alice@homea.test','-1m:+4m'), ('wrong-namespace','homea','board-b/alice@homeb.test','-1m:+4m'), ('unknown-ca','stranger','board-b/alice@homea.test','-1m:+4m'), ('expired','homea','board-b/alice@homea.test','-3m:-2m'), ('future','homea','board-b/alice@homea.test','+1h:+2h')]
    for label, signer, principal, validity in variants:
        public = path/(label+'.pub'); shutil.copyfile(path/'device.pub', public)
        cmd(['/usr/bin/ssh-keygen','-q','-s',str(path/signer),'-I',label,'-n',principal,'-V',validity,'-O','clear','-O','force-command='+force,str(public)])
    # The test server receives public issuers and synthetic devices only.
    for n in ['homea','homeb','stranger']: (path/n).unlink()


def run(preissued=None):
    assert os.geteuid() == 0 and Path('/etc/faction-disposable-test').is_file()
    root = Path(tempfile.mkdtemp(prefix='faction-roam-private-', dir='/var/tmp'))
    public = Path(tempfile.mkdtemp(prefix='faction-roam-public-', dir='/etc'))
    public.chmod(0o755)
    user = 'factionroam'
    def command(argv, ok=True):
        r = subprocess.run(argv, capture_output=True, timeout=15)
        if ok and r.returncode:
            raise RuntimeError('synthetic command failed: '+Path(argv[0]).name+' '+r.stderr.decode(errors='replace')+r.stdout.decode(errors='replace')+' status='+str(r.returncode))
        return r
    command(['/usr/sbin/groupadd', user])
    command(['/usr/sbin/useradd', '-g', user, '-d', '/nonexistent', '-s', '/bin/sh', user])
    assert pwd.getpwnam(user).pw_uid != 0
    def key(name):
        command(['/usr/bin/ssh-keygen', '-q', '-t', 'ed25519', '-N', '', '-f', str(root/name)])
    key('host')
    if preissued:
        st = preissued.stat()
        assert st.st_uid == 0 and st.st_mode & 0o077 == 0 and not preissued.is_symlink()
        allowed_names = ['homea.pub','homeb.pub','device','device.pub','otherdevice','otherdevice.pub'] + [n+'-cert.pub' for n in ['valid-a','valid-b','wrong-board','wrong-namespace','unknown-ca','expired','future']]
        for n in allowed_names:
            path = preissued/n
            assert path.is_file() and not path.is_symlink() and path.stat().st_size < 65536
            shutil.copyfile(path, root/n)
            (root/n).chmod(0o600)
    else:
        for name in ['homea', 'homeb', 'stranger', 'device', 'otherdevice']: key(name)
    ca = {n: ' '.join((root/(n+'.pub')).read_text().split()[:2]) for n in ['homea', 'homeb']}
    def authority(a=True, b=True):
        rows = []
        for enabled, home, jid in [(a, 'homea', 'alice@homea.test'), (b, 'homeb', 'alice@homeb.test')]:
            if enabled:
                rows.append('restrict,cert-authority,principals="board-b/'+jid+'" '+ca[home])
        (public/'authorized').write_text('\n'.join(rows)+'\n')
        (public/'authorized').chmod(0o644)
    authority()
    marker = 'faction-roaming-admission-only'
    force = '/usr/bin/printf '+marker
    config = f'''Port 2200
ListenAddress 127.0.0.1
HostKey {root}/host
PidFile {root}/sshd.pid
AuthorizedKeysFile {public}/authorized
StrictModes yes
UsePAM no
UseDNS no
PasswordAuthentication no
KbdInteractiveAuthentication no
PermitEmptyPasswords no
AuthenticationMethods publickey
PubkeyAcceptedAlgorithms ssh-ed25519-cert-v01@openssh.com
CASignatureAlgorithms ssh-ed25519
AllowUsers {user}
PermitRootLogin no
DisableForwarding yes
PermitTunnel no
PermitUserRC no
PermitTTY no
MaxSessions 1
MaxAuthTries 3
MaxStartups 8
ForceCommand {force}
LogLevel VERBOSE
'''
    (root/'sshd_config').write_text(config)
    command(['/usr/sbin/sshd', '-t', '-f', str(root/'sshd_config')])
    known = '[127.0.0.1]:2200 '+' '.join((root/'host.pub').read_text().split()[:2])+'\n'
    (root/'known_hosts').write_text(known)
    server = subprocess.Popen(['/usr/sbin/sshd', '-D', '-e', '-f', str(root/'sshd_config')], stdout=subprocess.DEVNULL, stderr=(root/'sshd.log').open('wb'))
    checks, source = [], 9
    def certificate(label, signer='homea', principal='board-b/alice@homea.test', validity='-1m:+4m'):
        if preissued:
            return root/(label+'-cert.pub')
        publickey = root/(label+'.pub')
        shutil.copyfile(root/'device.pub', publickey)
        command(['/usr/bin/ssh-keygen', '-q', '-s', str(root/signer), '-I', label, '-n', principal, '-V', validity, '-O', 'clear', '-O', 'force-command='+force, str(publickey)])
        return root/(label+'-cert.pub')
    def connect(cert, device='device', extra=None):
        nonlocal source
        source += 1
        command(['/sbin/ifconfig','lo0','inet','127.0.0.'+str(source),'alias'])
        args = ['/usr/bin/ssh', '-F', '/dev/null', '-b', '127.0.0.'+str(source), '-T', '-o', 'BatchMode=yes', '-o', 'IdentitiesOnly=yes', '-o', 'StrictHostKeyChecking=yes', '-o', 'UserKnownHostsFile='+str(root/'known_hosts'), '-o', 'ConnectTimeout=3', '-o', 'ConnectionAttempts=1', '-o', 'LogLevel=ERROR', '-i', str(root/device), '-p', '2200']
        if cert:
            args += ['-o', 'CertificateFile='+str(cert)]
        else:
            args += ['-o', 'CertificateFile=none']
        if extra:
            args += extra
        args += [user+'@127.0.0.1', 'id']
        try:
            return command(args, ok=False)
        finally:
            command(['/sbin/ifconfig','lo0','inet','127.0.0.'+str(source),'delete'])
    def allowed(cert):
        r = connect(cert)
        assert r.returncode == 0 and r.stdout.decode() == marker, 'expected scoped certificate admission failed: '+str(r.returncode)+' '+r.stderr.decode(errors='replace')
    def denied(cert, **kw):
        r = connect(cert, **kw)
        assert r.returncode != 0 and marker.encode() not in r.stdout, 'invalid proof admitted'
        assert b'Connection refused' not in r.stderr and b'kex_exchange_identification' not in r.stderr, 'transport/admission failure masked certificate result'
    try:
        time.sleep(0.5)
        a = certificate('valid-a')
        b = certificate('valid-b', 'homeb', 'board-b/alice@homeb.test')
        allowed(a); checks.append('home_a_certificate_and_device_possession_accepted')
        allowed(b); checks.append('independent_home_b_same_handle_distinguished_by_full_principal')
        # Every certificate is target-scoped. Each signer has only explicit local principals.
        denied(certificate('wrong-board', principal='board-c/alice@homea.test'))
        checks.append('wrong_visited_board_principal_denied')
        denied(certificate('wrong-namespace', principal='board-b/alice@homeb.test'))
        checks.append('home_a_cannot_assert_home_b_namespace')
        denied(certificate('unknown-ca', 'stranger'))
        checks.append('uninvited_signing_authority_denied')
        denied(certificate('expired', validity='-3m:-2m'))
        checks.append('expired_certificate_denied')
        denied(certificate('future', validity='+1h:+2h'))
        checks.append('not_yet_valid_certificate_denied')
        denied(a, device='otherdevice')
        checks.append('certificate_without_device_private_key_denied')
        denied(None)
        checks.append('uncertified_raw_key_denied')
        allowed(a)
        checks.append('remote_exec_stays_in_fixed_forced_command')
        authority(a=False)
        denied(a); allowed(b)
        checks.append('visited_board_local_revoke_denies_old_certificate_other_home_still_works')
        return {'result': 'pass', 'checks': checks, 'count': len(checks), 'product_integration': False, 'online_issuer': False, 'mail_keys_used': False, 'base_source_penalties_changed': False, 'distinct_loopback_sources': True, 'external_fixture_signer': bool(preissued)}
    except Exception as e:
        raise RuntimeError(str(e)+'; private fixture server: '+(root/'sshd.log').read_text()[-2000:]) from None
    finally:
        server.terminate()
        server.wait(timeout=10)
        command(['/usr/sbin/userdel', user])
        command(['/usr/sbin/groupdel', user])
        shutil.rmtree(root)
        shutil.rmtree(public)


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--output', type=Path)
    p.add_argument('--prepare-fixture', type=Path)
    p.add_argument('--preissued-dir', type=Path)
    p.add_argument('--disposable-synthetic-fixture', action='store_true', required=True)
    a = p.parse_args()
    if a.prepare_fixture:
        prepare(a.prepare_fixture)
        return
    assert a.output is not None
    result = run(a.preissued_dir)
    a.output.write_text(json.dumps(result, indent=2)+'\n')
    print('Native scoped SSH certificate controls:', result['count'])


if __name__ == '__main__': main()
