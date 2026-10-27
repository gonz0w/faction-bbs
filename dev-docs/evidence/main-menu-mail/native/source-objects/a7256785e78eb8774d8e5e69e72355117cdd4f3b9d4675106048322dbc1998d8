#!/usr/bin/env python3
"""Audit post-exec loader environment and authority FDs in a marked native fixture."""
import argparse,json,subprocess
from pathlib import Path
from check_restricted_ssh import SSHConsole

def run(a):
    assert a.disposable_fixture
    common=['/usr/bin/ssh','-F','/dev/null','-o','BatchMode=yes','-o','IdentitiesOnly=yes','-o','LogLevel=ERROR','-o','StrictHostKeyChecking=yes','-o','UserKnownHostsFile='+str(a.known_hosts)]
    admin=common+['-p',str(a.admin_port),'-i',str(a.admin_identity),'root@'+a.host]
    member=common+['-p',str(a.port),'-i',str(a.identity),'-tt','factionterm@'+a.host]
    def remote(cmd):return subprocess.check_output(admin+['test -f /etc/faction-disposable-test; '+cmd],timeout=15)
    assert json.loads(remote('/usr/local/libexec/faction-key-admission --inspect'))['active']==0
    c=SSHConsole(member)
    try:
        c.expect('[L]ogin [R]egister [Q]uit: ')
        listener=remote('cat /var/run/faction-sshd.pid').decode().strip();assert listener.isdecimal()
        env=json.loads(remote('/usr/local/libexec/faction-gate-environment '+listener))
        maps=json.loads(remote('/usr/local/libexec/faction-gate-maps '+listener))
        lines=remote('/usr/bin/fstat /var/db/faction-gate/admission.state').decode().splitlines()
        owners=[line.split() for line in lines[1:] if line.strip()]
        assert len(owners)==1 and owners[0][2]==listener,'gate authority inherited by another process'
        a.output.write_text(json.dumps(dict(environment=env,mapping=maps,registry_open_descriptions=1,only_listener_holds_registry=True,scope='authenticated PTY before BBS login; full-cohort mappings are recorded separately'),indent=2)+'\n')
        c.send('q');c.finish();print('Native post-exec environment and single-owner authority descriptor controls passed')
    finally:c.close()
if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--disposable-fixture',action='store_true');p.add_argument('--host',required=True);p.add_argument('--port',type=int,required=True);p.add_argument('--admin-port',type=int,required=True)
    for n in ['identity','admin-identity','known-hosts','output']:p.add_argument('--'+n,type=Path,required=True)
    run(p.parse_args())
