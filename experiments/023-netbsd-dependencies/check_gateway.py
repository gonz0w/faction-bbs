#!/usr/bin/env python3
"""Destructive only to the documented disposable guest's synthetic approval flag.

Requires the manually provisioned isolated VM in README. Never aim at a live board.
Emits sanitized oracle results only; no keys, endpoints or SSH transcripts.
Python 3.10+ stdlib; the SSH client is external. This is not production tooling.
"""
import argparse
import json
import select
import subprocess
import tempfile
import time
from pathlib import Path

PROBE = '/usr/local/libexec/faction-netbsd-probe'
DB = '/var/db/faction-probe/gateway.db'

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--host', required=True)
    parser.add_argument('--port', required=True, type=int)
    parser.add_argument('--identity', required=True, type=Path)
    parser.add_argument('--known-hosts', required=True, type=Path)
    parser.add_argument('--output', required=True, type=Path)
    args = parser.parse_args()
    common = ['ssh', '-F', '/dev/null', '-p', str(args.port), '-i', str(args.identity),
              '-o', 'IdentitiesOnly=yes', '-o', 'BatchMode=yes', '-o', 'ConnectTimeout=5',
              '-o', 'StrictHostKeyChecking=yes', '-o', f'UserKnownHostsFile={args.known_hosts}']
    results = []
    def call(options=(), command=(), data='', admin=False, identity=None):
        a = list(common)
        if identity is not None:
            a[a.index('-i')+1] = str(identity)
        return subprocess.run(a+list(options)+[('root' if admin else 'bbstest')+'@'+args.host]+list(command),
                              input=data, text=True, capture_output=True, timeout=12)
    def approval(value):
        p = call(command=[f'{PROBE} approval {DB} {value}'], admin=True)
        assert p.returncode == 0, 'guest admin approval control failed'
    def passed(name):
        results.append({'oracle': name, 'result': 'pass'})
    approval(0)
    p = call(data='READ\nPOST\nQUIT\n', options=['-T'])
    assert p.returncode == 0 and p.stdout.count('DENIED') == 2 and 'ROOM' not in p.stdout
    passed('key_auth_forced_probe_pending_denied')
    with tempfile.TemporaryDirectory(prefix='faction-unknown-key-') as d:
        key=Path(d)/'key'
        subprocess.run(['ssh-keygen','-q','-t','ed25519','-N','','-f',str(key)],check=True)
        p=call(options=['-T'],data='QUIT\n',identity=key)
        assert p.returncode == 255 and 'Permission denied' in p.stderr
    passed('unknown_key_denied')
    p=call(options=['-T'],command=['printf FACTION_HOST_SHELL_ESCAPE'])
    assert p.returncode != 0 and 'FACTION_HOST_SHELL_ESCAPE' not in p.stdout
    assert 'exec/subsystem denied' in p.stderr
    passed('exec_request_denied_no_host_shell')
    p=call(options=['-s'],command=['sftp'])
    assert p.returncode != 0 and p.stdout == ''
    passed('sftp_subsystem_denied')
    p=call(options=['-W','127.0.0.1:22'])
    assert p.returncode == 255 and 'administratively prohibited' in p.stderr
    passed('direct_tcp_forward_denied')
    p=call(options=['-N','-o','ExitOnForwardFailure=yes','-R','0:127.0.0.1:22'])
    assert p.returncode == 255 and 'remote port forwarding failed' in p.stderr
    passed('remote_tcp_forward_denied')
    p=call(options=['-N','-o','ExitOnForwardFailure=yes','-R','/tmp/faction-denied.sock:127.0.0.1:22'])
    assert p.returncode == 255 and 'remote port forwarding failed' in p.stderr
    passed('remote_unix_forward_denied')
    p=call(options=['-T'],data='x'*2048+'\n')
    assert p.returncode != 0 and 'oversized or unterminated command' in p.stderr
    passed('oversized_command_disconnect')
    # One PTY session stays connected across approval and revocation.
    child=subprocess.Popen(common+['-tt','bbstest@'+args.host],stdin=subprocess.PIPE,
                           stdout=subprocess.PIPE,stderr=subprocess.DEVNULL,bufsize=0)
    pending=bytearray()
    def until(marker):
        deadline=time.monotonic()+8
        while marker not in pending:
            assert time.monotonic()<deadline, 'PTY response timeout'
            if select.select([child.stdout],[],[],.1)[0]:
                chunk=child.stdout.read(1)
                assert chunk, 'PTY closed unexpectedly'
                pending.extend(chunk)
        end=pending.index(marker)+len(marker)
        del pending[:end]
    def send(command,marker):
        child.stdin.write(command+b'\n');child.stdin.flush();until(marker)
    try:
        until(b'FACTION DEPENDENCY PROBE')
        send(b'READ',b'DENIED')
        approval(1);send(b'READ',b'ROOM');send(b'POST',b'POSTED')
        approval(0);send(b'READ',b'DENIED');send(b'POST',b'DENIED')
        child.stdin.write(b'QUIT\n');child.stdin.flush()
        assert child.wait(timeout=8)==0
    finally:
        if child.poll() is None: child.kill();child.wait()
        child.stdin.close();child.stdout.close()
        approval(0)
    passed('pty_live_approval_and_revocation_read_post')
    args.output.write_text(json.dumps({'schema':1,'scope':'NetBSD ARM64 isolated synthetic SSH gateway',
                                      'results':results},indent=2)+'\n')
    print(f'{len(results)} SSH gateway oracles passed')

if __name__=='__main__': main()
