#!/usr/bin/env python3
"""Sanitize a private native kdump into aggregate helper syscall classes.

Never save raw trace paths, PIDs, key lines, addresses or protocol I/O. Trace only
CALL/RET/namei, and only a marked disposable listener and its descendants.
"""
import argparse,collections,json,re
from pathlib import Path

def summarize(text):
    calls=collections.defaultdict(collections.Counter);keys=collections.Counter();execs=collections.Counter()
    for line in text.splitlines():
        m=re.match(r'\s*(\d+)\s+\d+\s+faction-key-admi\s+(CALL|RET|NAMI)\s+(.*)',line)
        if not m:continue
        pid,kind,value=m.groups()
        if kind=='CALL':calls[pid][value.split('(')[0]]+=1
        if kind=='RET' and value=='execve JUSTRETURN':execs[pid]+=1
        if kind=='NAMI' and value=='"/etc/ssh/faction/authorized_keys"':keys[pid]+=1
    assert calls and set(calls)==set(execs) and all(n==1 for n in execs.values()),'incomplete helper trace'
    classes=collections.Counter()
    for pid,c in calls.items():classes[(c['fsync'],c['pwrite'],c['ftruncate'],c['__sysctl'],keys[pid],c['write'])]+=1
    rows=[dict(zip(['fsync','pwrite','ftruncate','sysctl','key_path_lookups','write','helpers'],[*k,v])) for k,v in sorted(classes.items())]
    return {'helper_executions':sum(execs.values()),'classes':rows,'totals':{k:sum(r[k]*r['helpers'] for r in rows) for k in ['fsync','pwrite','ftruncate','sysctl','key_path_lookups','write']},'scope':'native ktrace cn; helper processes only; sysctl totals include libc/NSS/visibility and parent queries; no protocol IO tracing'}
if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('input',type=Path);p.add_argument('output',type=Path);a=p.parse_args();a.output.write_text(json.dumps(summarize(a.input.read_text()),indent=2)+'\n')
