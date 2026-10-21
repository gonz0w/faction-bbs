#!/usr/bin/env python3
"""Effective-slab verification around the frozen #58 high-load driver.

Marked disposable NetBSD guest only. No change to the workload, guard, access
contract or credentials in check_headroom_scaling.run; retain its frozen source.
"""
import argparse
import json
import subprocess
from pathlib import Path
from check_headroom_scaling import run


def execute(args):
    assert args.disposable_fixture
    admin=['/usr/bin/ssh','-F','/dev/null','-o','BatchMode=yes','-o','IdentitiesOnly=yes',
           '-o','StrictHostKeyChecking=yes','-o','LogLevel=ERROR',
           '-o','UserKnownHostsFile='+str(args.known_hosts),'-i',str(args.admin_identity),
           '-p',str(args.admin_port),'root@'+args.host]
    command='test -f /etc/faction-disposable-test && /usr/local/libexec/faction-slab-probe'
    p=subprocess.run(admin+[command],capture_output=True,timeout=30)
    assert p.returncode==0,'effective native slab readback failed'
    geometry=json.loads(p.stdout);page=geometry['page_bytes']
    assert page==4096 and 1<=len(geometry['bins'])<=128
    assert geometry['columns']==['size_bytes','regions','slab_bytes','shards']
    for size,regions,slab,shards in geometry['bins']:
        assert size>0 and regions==slab//size and slab%page==0 and shards>=1
        if args.allocator=='packed':assert slab==((size+page-1)//page)*page,'slab profile did not take effect'
    # run validates native allocator options, the private fixture and all member
    # authority/credentials. It checks retain true only for the retained variant.
    run(args)
    data=json.loads(args.output.read_text());data['slab_geometry']=geometry
    args.output.write_text(json.dumps(data,indent=2)+'\n')


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--disposable-fixture',action='store_true');p.add_argument('--host',required=True)
    p.add_argument('--port',type=int,required=True);p.add_argument('--admin-port',type=int,required=True)
    for flag in ['identity','admin-identity','known-hosts','secret-file','output']:p.add_argument('--'+flag,type=Path,required=True)
    p.add_argument('--model',choices=['shared'],required=True)
    p.add_argument('--allocator',choices=['compact','retained','packed'],required=True)
    p.add_argument('--attribution',action='store_true');p.add_argument('--seed',action='store_true')
    p.add_argument('--max-sessions',type=int,choices=[8,32,64,100,150,200,250],default=250)
    execute(p.parse_args())
