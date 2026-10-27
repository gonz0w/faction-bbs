#!/usr/bin/env python3
"""A listener restart must not refresh a live monitor's lookup allowance."""
import json,time
from lookup_probe import admin,Connection,parser

def run(a):
    assert a.disposable_fixture
    admin(a,'/etc/rc.d/faction_isolated onestart; /etc/rc.d/faction_key_admission onestop; /usr/local/libexec/faction-key-admission --reset --capacity 250 --refill-ms 250 --experimental-capacity; /etc/rc.d/faction_key_admission onestart')
    c=Connection(a);rows=[]
    try:
        for _ in range(4):rows.append(c.request(a,'known'))
        assert all(r['response']=='key_ok' for r in rows)
        before=json.loads(admin(a,'/usr/local/libexec/faction-key-admission --inspect'));assert before['active']==1 and before['launches']==1 and before['reuses']==3
        inode=admin(a,'stat -f %i /var/run/faction-auth/admission.state')
        admin(a,'/etc/rc.d/faction_key_admission onestop; /etc/rc.d/faction_key_admission onestart')
        assert admin(a,'stat -f %i /var/run/faction-auth/admission.state')==inode
        for _ in range(3):rows.append(c.request(a,'known'))
        assert [r['response'] for r in rows]==['key_ok']*4+['denied','denied','disconnected']
    finally:c.close()
    time.sleep(.3);after=json.loads(admin(a,'/usr/local/libexec/faction-key-admission --inspect'))
    assert after['active']==0 and after['launches']==before['launches'] and after['reuses']==before['reuses']
    admin(a,'/etc/rc.d/faction_isolated onestop')
    a.output.write_text(json.dumps({'schema':1,'results':[{'oracle':'live monitor allowance remains exhausted across listener restart','result':'pass'}],'before':before,'after':after,'same_inode':True,'rows':rows,'scope':'independent restart oracle resets listener only for this explicit control; never during active fairness recovery'},indent=2)+'\n')
    print('Native exhausted-monitor restart denial passed')
if __name__=='__main__':run(parser().parse_args())
