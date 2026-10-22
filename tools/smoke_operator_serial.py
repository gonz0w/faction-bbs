#!/usr/bin/env python3
"""Normal OS login/console exit on an owned, already booted disposable serial VM.

Requires the fixture's existing serial controller/log/FIFO. Never installs login
profiles, changes credentials, boots an image or writes a terminal transcript.
"""
import argparse,json,re,time
from pathlib import Path


def run(a):
    assert a.disposable_fixture
    assert re.fullmatch(r'[a-z][a-z0-9_-]{0,30}',a.login)
    for value in [a.binary,a.state_dir]:assert re.fullmatch(r'/[A-Za-z0-9/_-]+',value)
    secret=a.password_file.read_text().strip().encode();assert len(secret)>=12 and secret.isascii()
    at=a.serial_log.stat().st_size;seen=b''
    def expect(wanted,timeout=20):
        nonlocal at,seen
        until=time.monotonic()+timeout
        while time.monotonic()<until:
            with a.serial_log.open('rb') as stream:
                stream.seek(at);data=stream.read(65536);at+=len(data)
            seen+=data;assert len(seen)<=1048576,'bounded serial observation exceeded'
            assert secret not in seen,'fixture password unexpectedly echoed'
            if wanted in seen:
                _,seen=seen.split(wanted,1);return
            time.sleep(.05)
        raise AssertionError('serial console did not reach expected state')
    def send(raw):
        with a.console_fifo.open('wb',buffering=0) as pipe:pipe.write(raw)
    send(b'\n');expect(b'login:');send(a.login.encode()+b'\n');expect(b'Password:');send(secret+b'\n')
    # Establish an authenticated shell, then run exactly the read-only command.
    send(b"printf '\\nFACTION_SHELL_READY\\n'\n");expect(b'\r\nFACTION_SHELL_READY\r\n')
    send(("test -f /etc/faction-disposable-test || exit 1; stty columns 80 rows 24; before=$(stty -g); export TERM=vt100; "+a.binary+' --state-dir '+a.state_dir+" --ansi --refresh 2; after=$(stty -g); if [ \"$before\" = \"$after\" ]; then printf '\\nTTY_RESTORED\\n'; fi\n").encode())
    expect(b'LOCAL SYSOP');send(b'h');expect(b'RAM used:');expect(b'Net RX/TX:');send(b'd');expect(b'RECOVERY / historical bundle check');send(b'i');expect(b'OS owner/root access; not a BBS role.')
    send(b'q');expect(b'\x1b[24;1H\x1b[0m');expect(b'\r\nTTY_RESTORED\r\n');send(b'exit\n');expect(b'login:')
    result=dict(result='pass',checks=['normal_os_password_login','owner_console_launch_and_panels','reserved_row_exit_and_termios_restore','logout_returns_to_getty'],count=4,login_profile_changed=False,autologin=False,physical_console_claim=False)
    a.output.write_text(json.dumps(result,indent=2)+'\n');a.output.chmod(0o600)
    print('Owned VM serial-console login/launch/exit/logout passed 4 controls.')


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--disposable-fixture',action='store_true',required=True)
    for name in ['serial-log','console-fifo','password-file','output']:p.add_argument('--'+name,type=Path,required=True)
    p.add_argument('--login',required=True);p.add_argument('--binary',default='/usr/local/libexec/faction-sysop');p.add_argument('--state-dir',default='/var/db/faction-profile-fixture');run(p.parse_args())
