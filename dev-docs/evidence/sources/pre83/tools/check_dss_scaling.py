#!/usr/bin/env python3
"""Native DSS topology comparison around the frozen high-load/geometry checks."""
import argparse
import json
import subprocess
from pathlib import Path
from check_slab_scaling import execute


def check(args):
    assert args.disposable_fixture
    admin=['/usr/bin/ssh','-F','/dev/null','-o','BatchMode=yes','-o','IdentitiesOnly=yes',
           '-o','StrictHostKeyChecking=yes','-o','LogLevel=ERROR',
           '-o','UserKnownHostsFile='+str(args.known_hosts),'-i',str(args.admin_identity),
           '-p',str(args.admin_port),'root@'+args.host]
    p=subprocess.run(admin+['test -f /etc/faction-disposable-test && /usr/local/libexec/faction-dss-probe'],capture_output=True,timeout=30)
    assert p.returncode==0,'native DSS readback failed'
    preference=json.loads(p.stdout)
    assert preference=={'dss':'primary'},'DSS profile did not take effect'
    execute(args)
    data=json.loads(args.output.read_text());data['dss_readback']=preference
    args.output.write_text(json.dumps(data,indent=2)+'\n')


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--disposable-fixture',action='store_true');p.add_argument('--host',required=True)
    p.add_argument('--port',type=int,required=True);p.add_argument('--admin-port',type=int,required=True)
    for flag in ['identity','admin-identity','known-hosts','secret-file','output']:p.add_argument('--'+flag,type=Path,required=True)
    p.add_argument('--model',choices=['shared'],required=True)
    p.add_argument('--allocator',choices=['dss'],required=True)
    p.add_argument('--attribution',action='store_true');p.add_argument('--seed',action='store_true')
    p.add_argument('--max-sessions',type=int,choices=[8,32,64,100,150,200,250],default=250)
    check(p.parse_args())
