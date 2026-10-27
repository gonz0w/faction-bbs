#!/usr/bin/env python3
"""OpenSSH key-option enforcement and signed-client compatibility; marked VM only."""
import json,shlex,time
from lookup_probe import admin,Connection,parser
from lookup_workload import legitimate

def run(a):
    assert a.disposable_fixture
    # Public fixture operation uses a protected, byte-exact temporary copy on
    # the guest. The key line and endpoints never enter the saved result.
    command="test ! -e /root/faction-lookup.keys; install -m 600 /etc/ssh/faction/authorized_keys /root/faction-lookup.keys; /etc/rc.d/faction_key_admission onestop"
    def remote(cmd):
        # admin() requires JSON only at its callers, and returns raw bytes here.
        return admin(a,cmd)
    mode=remote('stat -f %Lp /etc/ssh/faction/authorized_keys').decode().strip();assert mode in ['644','640']
    original=remote('cat /etc/ssh/faction/authorized_keys');assert original.startswith(b'restrict,pty ')
    remote(command)
    try:
        code="from pathlib import Path;p=Path('/etc/ssh/faction/authorized_keys');p.write_bytes(b'from=\"192.0.2.1\",'+p.read_bytes());p.chmod(0o"+mode+")"
        remote('/usr/pkg/bin/python3.13 -c '+shlex.quote(code)+'; /usr/local/libexec/faction-key-admission --reset --refill-ms 250 --experimental-capacity; /etc/rc.d/faction_key_admission onestart')
        remote("su -m factionauth -c 'test -r /etc/ssh/faction/authorized_keys'")
        c=Connection(a);rows=[]
        try:
            for _ in range(4):
                r=c.request(a,'known');rows.append(r)
                if r['response']=='disconnected':break
        finally:c.close()
        assert [r['response'] for r in rows]==['denied','denied','disconnected']
    finally:
        remote('/etc/rc.d/faction_key_admission onestop; install -m '+mode+' /root/faction-lookup.keys /etc/ssh/faction/authorized_keys; rm /root/faction-lookup.keys; /usr/local/libexec/faction-key-admission --reset --refill-ms 250 --experimental-capacity; /etc/rc.d/faction_key_admission onestart')
        assert remote('cat /etc/ssh/faction/authorized_keys')==original
    before=json.loads(remote('/usr/local/libexec/faction-key-admission --inspect'))
    assert legitimate(a),'ordinary signed Paramiko client rejected'
    time.sleep(.4);after=json.loads(remote('/usr/local/libexec/faction-key-admission --inspect'))
    charged=after['launches']+after['reuses']-before['launches']-before['reuses'];assert 1<=charged<=4 and after['active']==0
    a.output.write_text(json.dumps({'schema':1,'results':[{'oracle':'OpenSSH from option still denies a matching invitation key','result':'pass'},{'oracle':'ordinary signed Paramiko client succeeds after byte-exact key restoration','result':'pass'}],'options_rows':rows,'signed_client_charged_lookups':charged,'transport_only':True,'key_file_restored_exactly':True,'scope':'synthetic excluding from option; no helper key parser or BBS authority'},indent=2)+'\n')
    print('Native key options, exact restoration and signed-client compatibility passed')
if __name__=='__main__':
    from pathlib import Path
    p=parser();p.add_argument('--identity',type=Path,required=True);run(p.parse_args())
