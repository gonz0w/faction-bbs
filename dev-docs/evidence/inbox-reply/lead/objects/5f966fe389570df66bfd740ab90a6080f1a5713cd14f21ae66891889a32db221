#!/usr/bin/env python3
"""Measure one complete manual native UI cycle in the owned NetBSD fixture."""
import argparse,json,os,platform,pwd,re,shlex,shutil
from pathlib import Path
from smoke_native_client_ssh import NativeConsole
from probe_xmpp_transport import FixtureFailure,parse_native_time

def resource_usage(transcript):
    match=re.search(r'(?m)^\s*[0-9.]+\s+real\s+[0-9.]+\s+user\s+[0-9.]+\s+sys\s*$',transcript)
    if match is None:raise FixtureFailure('native usage summary missing')
    result=parse_native_time(transcript[match.start():])
    result['scope']='one complete faction-client manual UI process, including checkpoint unlock and one TLS/account/publication/pending-write cycle'
    return result

def invocation(binary,home):
    return ['/usr/bin/time','-l',str(binary),'--mail-online',str(home['store']),str(home['config'])]

def measure(binary,private_fixture):
    if platform.system()!='NetBSD':raise FixtureFailure('NetBSD native resource units required')
    marker=Path('/etc/faction-disposable-test')
    if not marker.is_file() or b'209' not in marker.read_bytes():raise FixtureFailure('owned disposable marker required')
    path=Path(private_fixture)
    if path.is_symlink() or path.stat().st_uid!=os.getuid() or path.stat().st_mode&0o077:raise FixtureFailure('private fixture profile required')
    profile=json.loads(path.read_text());home=profile['alice'];argv=invocation(binary,home)
    if home.get('run_as'):
        user=home['run_as'];entry=pwd.getpwnam(user)
        if entry.pw_uid==0:raise FixtureFailure('unprivileged member required')
        argv=[shutil.which('su'),'-l',user,'-c',shlex.join(argv)]
    child=NativeConsole(argv)
    try:
        child.password(home['local_password'],'LOCAL checkpoint passphrase (hidden): ')
        child.expect('[L] Lock and exit');child.raw(b'n')
        child.password(home['password'],'HOME account password (hidden; never sent to a BBS): ')
        child.expect('Manual cycle complete.');child.expect('Enter to return: ');child.send('');child.expect('[L] Lock and exit')
        child.raw(b'l');child.finish()
        for secret in [home['local_password'],home['password']]:
            if secret.encode() in child.transcript:raise FixtureFailure('hidden input appeared in terminal output')
        usage=resource_usage(child.transcript.decode('utf-8'))
    finally:child.close()
    return dict(result='pass',usage=usage,whole_client=True,includes_unlock=True,includes_network_cycle=True,compiler_included=False,server_fixture_included=False,capacity_claim=False,terminal_restored=True)
if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--client',type=Path,required=True);p.add_argument('--private-fixture',type=Path,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args()
    try:
        result=measure(a.client.resolve(),a.private_fixture);a.output.write_text(json.dumps(result,indent=2)+'\n')
        print('Whole manual native-client resource control passed; private terminal transcript withheld.')
    except (FixtureFailure,AssertionError,OSError,ValueError,KeyError):raise SystemExit('Native client resource control failed; private diagnostics withheld') from None
