#!/usr/bin/env python3
"""Measure one source-qualified research process on NetBSD; no product claim."""
import argparse,json,platform,subprocess,tempfile
from pathlib import Path
import probe_xmpp_transport as fixture

def measure(binary,openssl):
 if platform.system()!='NetBSD':raise fixture.FixtureFailure('resource probe requires native NetBSD time -l units')
 usage=[];original=subprocess.run
 def timed(args,*a,**kw):
  if args and args[0]==str(binary):
   p=original(['/usr/bin/time','-l',*args],*a,**kw)
   usage.append(fixture.parse_native_time(p.stderr.decode()))
   p.stderr=b''
   return p
  return original(args,*a,**kw)
 with tempfile.TemporaryDirectory(prefix='faction208-private-cost-') as tmp:
  root=Path(tmp).resolve();root.chmod(0o700);pairs=fixture.certificates(root,openssl)
  try:
   fixture.subprocess.run=timed
   outcome=fixture.run_case(binary,root,pairs,'native_whole_process_cost',{'ok'})
  finally:fixture.subprocess.run=original
 assert len(usage)==1 and not root.exists()
 return dict(result='pass',usage=usage[0],control=outcome,private_fixture_removed=True,compiler_included=False,fixture_included=False,capacity_claim=False)
if __name__=='__main__':
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--binary',type=Path,required=True);p.add_argument('--openssl',default='openssl');p.add_argument('--output',type=Path,required=True);a=p.parse_args()
 d=measure(a.binary.resolve(),a.openssl);a.output.write_text(json.dumps(d,indent=2)+'\n');print('Whole native research-process resource case passed; private fixture removed.')
