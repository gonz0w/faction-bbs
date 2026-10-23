#!/usr/bin/env python3
"""Native local gate on real PTYs; owned refused loopback SSH, no board data."""
import argparse,json,secrets,socket,subprocess,tempfile
from pathlib import Path
from smoke_native_client_ssh import NativeConsole
from smoke_kitty_graphics import QUERY,REPLY,DELETE

def smoke(binary):
    checks=[];children=[]
    with tempfile.TemporaryDirectory(prefix='faction-native-kitty-') as root, socket.socket() as listener:
        p=Path(root);p.chmod(0o700);key=p/'identity'
        subprocess.run(['/usr/bin/ssh-keygen','-q','-t','ed25519','-N','','-f',str(key)],check=True)
        known=p/'known_hosts';known.write_text('');known.chmod(0o600)
        listener.bind(('127.0.0.1',0));port=listener.getsockname()[1]
        profile=p/'profile.toml';profile.write_text('version=1\nhost="127.0.0.1"\nfile_host="127.0.0.1"\nport='+str(port)+'\nfile_port='+str(port)+'\nidentity='+json.dumps(str(key))+'\npublic_key='+json.dumps(str(key)+'.pub')+'\nknown_hosts='+json.dumps(str(known))+'\n');profile.chmod(0o600)
        def start(flag=True):
            c=NativeConsole([str(binary),'--profile',str(profile)]+(['--kitty'] if flag else []));children.append(c);return c
        try:
            c=start(False);c.finish(1);assert QUERY.encode() not in c.transcript;checks.append('native_default_blocks_graphics_without_probe')
            for reply,positive in [(REPLY,True),(b'',False),(b'\x1b_Gi=17392;ENOTSUP:unsupported\x1b\\',False)]:
                c=start();c.expect(QUERY)
                if reply:c.raw(reply[:7]);c.raw(reply[7:])
                c.expect('Kitty gate ready' if positive else 'Kitty unavailable; classic fallback.');c.finish(1)
                assert (DELETE in c.transcript)==positive
            checks.append('native_explicit_local_positive_or_refusal_before_ssh')
            for reply in [b'\x1b_Gi=17392;OK\x1b\\P\n',b'\x1b_Gi=17392;OK',b'\x1b]52;c;bad\x07']:
                c=start();c.expect(QUERY);c.raw(reply);c.finish(1);assert b'Kitty gate ready' not in c.transcript
            checks.append('native_local_malformed_or_incomplete_never_unlocks_gate')
        finally:
            for c in children:c.close()
    print('Native Kitty local-gate PTYs passed '+str(len(checks))+' controls; no remote board or GUI claim.')
    return checks
if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('binary',type=Path);a=p.parse_args();smoke(a.binary.resolve())
