#!/usr/bin/env python3
"""Read back the host profile around installed shared service workloads."""
import argparse
import json
import subprocess
from pathlib import Path
from check_key_admission_scaling import run as execute


def check(args):
    assert args.disposable_fixture and args.allocator=='compact'
    admin=['/usr/bin/ssh','-F','/dev/null','-o','BatchMode=yes','-o','IdentitiesOnly=yes',
           '-o','StrictHostKeyChecking=yes','-o','LogLevel=ERROR',
           '-o','UserKnownHostsFile='+str(args.known_hosts),'-i',str(args.admin_identity),
           '-p',str(args.admin_port),'root@'+args.host]
    def readback():
        p=subprocess.run(admin+['test -f /etc/faction-disposable-test && /usr/local/libexec/faction-host-memory'],capture_output=True,timeout=30)
        assert p.returncode==0,'native host readback failed'
        host=json.loads(p.stdout)
        assert host['columns']==['service','rss_bytes','virtual_bytes','locked_mapping_virtual_bytes']
        times=[x for x in host['processes'] if x[0]=='ntpd']
        assert times and all(x[3]==0 for x in times),'time daemon missing or still locked'
        for service in ['syslogd','dhcpcd','cron','inetd','master','mdnsd']:
            assert any(x[0]==service for x in host['processes']),'base service missing: '+service
        return host
    before=readback();execute(args);after=readback()
    data=json.loads(args.output.read_text());data['host_profile']='pageable-time; all base services retained'
    data['host_service_readbacks']={'before':before,'after':after}
    args.output.write_text(json.dumps(data,indent=2)+'\n')


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--disposable-fixture',action='store_true');p.add_argument('--host',required=True)
    p.add_argument('--port',type=int,required=True);p.add_argument('--admin-port',type=int,required=True)
    for flag in ['identity','admin-identity','known-hosts','secret-file','output']:p.add_argument('--'+flag,type=Path,required=True)
    p.add_argument('--model',choices=['shared'],required=True)
    p.add_argument('--allocator',choices=['compact'],required=True)
    p.add_argument('--attribution',action='store_true');p.add_argument('--seed',action='store_true')
    p.add_argument('--max-sessions',type=int,choices=[8,32,64,100,150,200,250],default=250)
    check(p.parse_args())
