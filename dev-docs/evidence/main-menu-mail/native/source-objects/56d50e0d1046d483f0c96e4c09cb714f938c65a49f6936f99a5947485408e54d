#!/usr/bin/env python3
"""Native state-format/birth controls; destructive marked synthetic fixture only."""
import argparse,json,shlex,subprocess,time
from pathlib import Path
from lookup_probe import admin,Connection

def run(a):
    assert a.disposable_fixture
    results=[]
    def remote(cmd,ok=True,data=None):
        argv=['/usr/bin/ssh','-F','/dev/null','-o','BatchMode=yes','-o','IdentitiesOnly=yes','-o','StrictHostKeyChecking=yes','-o','UserKnownHostsFile='+str(a.known_hosts),'-i',str(a.admin_identity),'-p',str(a.admin_port),'root@'+a.host]
        p=subprocess.run(argv+['test -f /etc/faction-disposable-test && '+cmd],input=data,capture_output=True,timeout=20)
        assert not ok or p.returncode==0,'state fixture operation failed';return p
    def passed(s):results.append({'oracle':s,'result':'pass'});print('Passed: '+s,flush=True)
    def state():return json.loads(admin(a,'/usr/local/libexec/faction-key-admission --inspect'))
    def write(payload):
        code="import os,fcntl,sys; f=os.open('/var/run/faction-auth/admission.state',os.O_RDWR); fcntl.flock(f,fcntl.LOCK_EX); b=sys.stdin.buffer.read(); os.pwrite(f,b,0); os.ftruncate(f,len(b)); os.fsync(f)"
        remote('/usr/pkg/bin/python3.13 -c '+shlex.quote(code),data=payload)
    import struct
    def checked(payload):
        h=14695981039346656037
        for b in payload:h=((h^b)*1099511628211)&((1<<64)-1)
        return bytes(payload)+struct.pack('=Q',h)
    remote('/etc/rc.d/faction_key_admission onestop; /usr/local/libexec/faction-key-admission --reset --refill-ms 250 --experimental-capacity; /etc/rc.d/faction_key_admission onestart')
    assert state()['active']==0
    inode=remote('stat -f %i /var/run/faction-auth/admission.state').stdout
    c=Connection(a)
    try:
        assert c.request(a,'known')['response']=='key_ok';assert state()['active']==1
        current=remote('cat /var/run/faction-auth/admission.state').stdout;assert len(current)==8104
        # Simulated PID reuse: retain a real live PID but change its birth field.
        b=bytearray(current[:-8]);slots=struct.unpack_from('=750Q',b,96)
        index=next(i for i in range(250) if slots[3*i]);birth=96+24*index+8
        struct.pack_into('=Q',b,birth,struct.unpack_from('=Q',b,birth)[0]+1)
        write(checked(b));assert state()['active']==0
        write(current);assert state()['active']==1
        passed('exact monitor birth is live and a mismatched birth is stale (simulated PID reuse)')
        # Preserve the original v1 layout/checksum, including the live identity.
        old=bytearray(current[:6096]);struct.pack_into('=Q',old,8,1);old=checked(old);assert len(old)==6104
        write(old)
        assert remote('/usr/local/libexec/faction-key-admission --inspect',ok=False).returncode!=0
        assert remote('/usr/local/libexec/faction-key-admission --reset',ok=False).returncode!=0
        assert remote('/usr/local/libexec/faction-key-admission --migrate-v1',ok=False).returncode!=0
        assert remote('/usr/pkg/bin/python3.13 -I -B /usr/local/libexec/check_key_admission_config.py --migrate-v1',ok=False).returncode!=0
        assert remote('cat /var/run/faction-auth/admission.state').stdout==old
        passed('v1 is refused without silent reset and migration refuses the live listener')
        remote('/etc/rc.d/faction_key_admission onestop')
        assert remote('/usr/local/libexec/faction-key-admission --migrate-v1',ok=False).returncode!=0
        assert remote('/usr/pkg/bin/python3.13 -I -B /usr/local/libexec/check_key_admission_config.py --migrate-v1',ok=False).returncode!=0
        assert remote('cat /var/run/faction-auth/admission.state').stdout==old
        passed('stopped listener cannot migrate over a surviving monitor')
    finally:c.close()
    time.sleep(.5)
    bad=bytearray(old);bad[-1]^=1;write(bad)
    assert remote('/usr/pkg/bin/python3.13 -I -B /usr/local/libexec/check_key_admission_config.py --migrate-v1',ok=False).returncode!=0
    assert remote('cat /var/run/faction-auth/admission.state').stdout==bad
    passed('corrupt legacy bytes cannot be migrated or silently reset')
    write(old)
    remote('/usr/pkg/bin/python3.13 -I -B /usr/local/libexec/check_key_admission_config.py --migrate-v1')
    assert remote('stat -f %i /var/run/faction-auth/admission.state').stdout==inode
    s=state();assert s['version']==2 and s['lookup_limit']==4 and s['active']==0 and s['capacity']==8 and s['refill_ms']==250
    assert len(remote('cat /var/run/faction-auth/admission.state').stdout)==8104
    passed('explicit quiescent migration preserves inode and creates the checked v2 layout')
    assert remote('/usr/local/libexec/faction-key-admission --migrate-v1',ok=False).returncode!=0
    remote('/etc/rc.d/faction_key_admission onestart')
    c=Connection(a)
    try:assert c.request(a,'known')['response']=='key_ok'
    finally:c.close()
    time.sleep(.3);assert state()['active']==0
    passed('migrated v2 rejects repeated migration and admits a fresh real query')
    a.output.write_text(json.dumps({'schema':1,'results':results,'migration':{'old_bytes':6104,'new_bytes':8104,'same_inode':True,'quiescent':True,'implicit':False},'scope':'marked synthetic VM; birth mismatch is simulated, not observed natural PID recycling'},indent=2)+'\n')
if __name__=='__main__':
    from lookup_probe import parser
    run(parser().parse_args())
