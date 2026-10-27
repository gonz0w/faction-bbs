#!/usr/bin/env python3
"""Existing 29 native UI oracles with explicit pacing below the source quota.

The underlying destructive suite requires the marked disposable NetBSD fixture.
Pacing is a declared workload input, never a service or security-policy change.
"""
import argparse,time
from pathlib import Path
import check_isolation_oracles as ui

def run(a):
    original=ui.Terminal.__init__
    def paced(self,*args,**kwargs):
        time.sleep(.55);original(self,*args,**kwargs)
    ui.Terminal.__init__=paced
    try:ui.run(a)
    finally:ui.Terminal.__init__=original
if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--disposable-fixture',action='store_true');p.add_argument('--host',required=True);p.add_argument('--port',type=int,required=True);p.add_argument('--admin-port',type=int,required=True)
    for n in ['identity','admin-identity','known-hosts','secret-file','output']:p.add_argument('--'+n,type=Path,required=True)
    run(p.parse_args())
