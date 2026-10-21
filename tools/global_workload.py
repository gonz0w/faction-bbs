"""Bounded global-gate pressure while 250 approved fixture members remain useful."""
import json,secrets,shlex,subprocess,time
from credential_peak import Stream
from headroom_memory import sample_guard

def active(clients,remote,metric,admin):
    assert len(clients)==250
    command="""import socket,time,json
held=[];offers=0;banners=0;start=time.monotonic()
try:
 while time.monotonic()-start<60:
  s=socket.socket();s.settimeout(.2);s.bind(('127.0.0.'+str(31+offers%8),0));offers+=1
  try:
   s.connect(('127.0.0.1',2222));b=s.recv(256)
   if b.startswith(b'SSH-'):held.append(s);banners+=1
   else:s.close()
  except OSError:s.close()
  time.sleep(.01)
finally:
 for s in held:s.close()
print(json.dumps({'offers':offers,'banners':banners,'seconds':time.monotonic()-start}))
"""
    before=json.loads(remote('/usr/local/libexec/faction-ssh-gate --inspect').stdout)
    mapping=json.loads(remote('/usr/local/libexec/faction-gate-maps $(cat /var/run/faction-sshd.pid)').stdout)
    stream=Stream(admin+[metric+' --stream 1000 65']);rounds=[]
    flood=subprocess.Popen(admin+['/usr/pkg/bin/python3.13 -I -c '+shlex.quote(command)],stdout=subprocess.PIPE,stderr=subprocess.PIPE)
    began=time.monotonic()
    try:
        while time.monotonic()-began<61:
            n=len(rounds)%len(clients);marker='global pressure '+secrets.token_hex(8);start=time.monotonic();clients[n].send(marker)
            # Require non-sender delivery, not just terminal echo.
            for i,c in enumerate(clients):
                if i!=n:c.expect(marker)
            rounds.append(dict(at_seconds=time.monotonic()-began,confirmed_peer_receipts=249,seconds=time.monotonic()-start));time.sleep(.5)
        out,err=flood.communicate(timeout=10);assert flood.returncode==0,'private global pressure failed'
        pressure=json.loads(out);samples=stream.finish()
        guards=[sample_guard(s) for s in samples['samples']];assert all(g['passed'] for g in guards)
        after=json.loads(remote('/usr/local/libexec/faction-ssh-gate --inspect').stdout)
        key=json.loads(remote('/usr/local/libexec/faction-key-admission --inspect').stdout);assert key['active']==250
        assert pressure['offers']>100 and after['global_drops']>before['global_drops']
        assert 0<after['passed']-before['passed']<=8+int(pressure['seconds']/.25)+4
        return dict(requested_seconds=60,pressure=pressure,before=before,after=after,mapping=mapping,native_stream=samples,rounds=rounds,minimum_headroom_bytes=min(g['reclaimable_headroom_bytes'] for g in guards),active_members=250,registry_reset=False,listener_reset=False)
    finally:
        stream.abort()
        if flood.poll() is None:flood.kill();flood.wait()
