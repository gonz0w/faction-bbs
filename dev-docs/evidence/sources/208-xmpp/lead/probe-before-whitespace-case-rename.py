#!/usr/bin/env python3
"""Private loopback TLS/XMPP fixture for the #208 research binary, not a server.

No real accounts, external peers, transcript output or product runtime adoption.
Only named outcomes/counters survive; generated keys and received auth stay private.
"""
import argparse,base64,hashlib,json,os,platform,re,secrets,socket,ssl,subprocess,tempfile,threading,time
from pathlib import Path
from xml.etree import ElementTree as ET

class FixtureFailure(Exception):pass
class Reader:
 def __init__(self,channel,limit=65536):self.channel=channel;self.limit=limit;self.total=0;self.buffer=b''
 def take(self,pattern):
  while True:
   match=re.search(pattern,self.buffer)
   if match:
    data,self.buffer=self.buffer[:match.end()],self.buffer[match.end():];return data
   data=self.channel.recv(4096)
   if not data:raise FixtureFailure('incomplete fixture input')
   self.total+=len(data)
   if self.total>self.limit:raise FixtureFailure('fixture input limit')
   self.buffer+=data
   if b'<!' in self.buffer:raise FixtureFailure('unsupported fixture XML')
 def stream(self):return self.take(rb'<stream:stream\b[^>]*>')
 def element(self,name):return self.take(rb'</(?:[A-Za-z_][\w.-]*:)?'+name.encode()+rb'\s*>').strip()

def parse_native_time(raw):
 """NetBSD time -l only. Reject unknown stderr instead of publishing it."""
 first=None;peak=None
 labels={'maximum resident set size','average shared memory size','average shared text size','average unshared data size','average unshared stack size','page reclaims','page faults','swaps','block input operations','block output operations','messages sent','messages received','signals received','voluntary context switches','involuntary context switches'}
 for line in raw.splitlines():
  if not line.strip():continue
  t=re.fullmatch(r'\s*([0-9.]+)\s+real\s+([0-9.]+)\s+user\s+([0-9.]+)\s+sys\s*',line)
  if t:
   if first is not None:raise FixtureFailure('duplicate time summary')
   first=list(map(float,t.groups()));continue
  stat=re.fullmatch(r'\s*(\d+)\s+([a-z ]+)\s*',line)
  if not stat or stat[2].strip() not in labels:raise FixtureFailure('unrecognized native time diagnostic')
  if stat[2].strip()=='maximum resident set size':
   if peak is not None:raise FixtureFailure('duplicate peak usage')
   peak=int(stat[1])
 if first is None or peak is None or peak<=0:raise FixtureFailure('native time usage missing')
 return dict(max_rss_kib=peak,wall_seconds=first[0],user_seconds=first[1],system_seconds=first[2],scope='one whole native research-client process; fixture/compiler excluded')

def validated_outcome(name,status,tls_version):
 if tls_version not in {None,'TLSv1.2','TLSv1.3'} or (status=='ok' and tls_version is None):raise FixtureFailure('unqualified or unobserved TLS success')
 return dict(check=name,result='pass',status=status,attempts=1,tls_version=tls_version)

def validate_summary(d):
 assert d['result']=='pass' and d['count']==len(d['checks'])==len(set(d['checks'])) and d['count']>0
 assert d['private_fixture_removed'] is True
 assert d['product_adopted'] is False and d['live_federation'] is False

def _command(args):
 p=subprocess.run(args,stdin=subprocess.DEVNULL,capture_output=True,timeout=30)
 if p.returncode:raise FixtureFailure('fixture certificate generation failed')

def certificates(root,openssl):
 config=root/'openssl-fixture.cnf';config.write_text('[req]\ndistinguished_name=dn\n[dn]\n')
 def request(name,cn,ca=False,issuer='ca'):
  key=root/(name+'.key');csr=root/(name+'.csr');pem=root/(name+'.pem')
  if ca:
   _command([openssl,'req','-config',str(config),'-x509','-newkey','rsa:2048','-nodes','-sha256','-days','2','-subj','/CN='+cn,'-addext','basicConstraints=critical,CA:TRUE','-keyout',str(key),'-out',str(pem)])
  else:
   _command([openssl,'req','-config',str(config),'-new','-newkey','rsa:2048','-nodes','-sha256','-subj','/CN='+cn,'-keyout',str(key),'-out',str(csr)])
   ext=root/(name+'.ext');ext.write_text('basicConstraints=critical,CA:FALSE\nsubjectAltName=DNS:'+cn+'\nextendedKeyUsage=serverAuth\nkeyUsage=critical,digitalSignature,keyEncipherment\n')
   _command([openssl,'x509','-req','-in',str(csr),'-CA',str(root/(issuer+'.pem')),'-CAkey',str(root/(issuer+'.key')),'-set_serial',str(secrets.randbits(64) or 1),'-days','2','-sha256','-extfile',str(ext),'-out',str(pem)])
  key.chmod(0o600);return pem,key
 request('ca','Faction synthetic CA',True);request('untrusted-ca','Other synthetic CA',True)
 pairs={n:request(n,cn) for n,cn in [('good','home.test'),('changed','home.test'),('wrong-name','other.test')]}
 return pairs

OPEN=b"<stream:stream xmlns='jabber:client' xmlns:stream='http://etherx.jabber.org/streams' from='home.test' id='synthetic-208' version='1.0'>"
FEATURE=b"<stream:features><mechanisms xmlns='urn:ietf:params:xml:ns:xmpp-sasl'><mechanism>PLAIN</mechanism></mechanisms></stream:features>"
BIND=b"<stream:features><bind xmlns='urn:ietf:params:xml:ns:xmpp-bind'/></stream:features>"
SUCCESS=b"<success xmlns='urn:ietf:params:xml:ns:xmpp-sasl'/>"

class Peer:
 def __init__(self,root,pair,password,payload,mode):
  self.password=password;self.payload=payload;self.mode=mode;self.stop=threading.Event();self.auth_seen=False;self.auth_ok=False;self.message_seen=False;self.connections=0;self.failure=None;self.tls_version=None
  self.context=ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER);self.context.minimum_version=ssl.TLSVersion.TLSv1_2;self.context.load_cert_chain(*map(str,pair))
  if mode=='tls12':self.context.maximum_version=ssl.TLSVersion.TLSv1_2
  self.listener=socket.socket();self.listener.bind(('127.0.0.1',0));self.listener.listen(1);self.listener.settimeout(15);self.port=self.listener.getsockname()[1]
  self.thread=threading.Thread(target=self.run,name='faction208-owned-tls',daemon=True);self.thread.start()
 def send(self,c,data):
  if self.mode=='fragmented':
   for i in range(0,len(data),3):
    if self.stop.is_set():break
    c.sendall(data[i:i+3])
  else:c.sendall(data)
 def run(self):
  try:
   plain,_=self.listener.accept();self.connections+=1;plain.settimeout(14)
   with self.context.wrap_socket(plain,server_side=True) as c:
    self.tls_version=c.version();r=Reader(c);r.stream()
    if self.mode=='stalled':self.stop.wait(12);return
    if self.mode=='trickle':
     for b in OPEN+FEATURE:
      if self.stop.wait(0.15):return
      c.sendall(bytes([b]))
     return
    self.send(c,OPEN+(FEATURE if self.mode!='no-plain' else FEATURE.replace(b'PLAIN',b'SCRAM-SHA-256')))
    auth=ET.fromstring(r.element('auth'));self.auth_seen=True
    if auth.get('mechanism')!='PLAIN':raise FixtureFailure('unqualified authentication mechanism')
    decoded=base64.b64decode(auth.text or '',validate=True)
    self.auth_ok=decoded==b'\0alice\0'+self.password.encode()
    if self.mode=='auth-fail' or not self.auth_ok:
     self.send(c,b"<failure xmlns='urn:ietf:params:xml:ns:xmpp-sasl'><not-authorized/></failure>");return
    coalesced=self.mode=='coalesced-restart'
    self.send(c,SUCCESS+(OPEN+BIND if coalesced else b''));r.stream()
    if not coalesced:self.send(c,OPEN+BIND)
    iq=ET.fromstring(r.element('iq'));ident=iq.get('id','')
    if not re.fullmatch('[A-Za-z0-9_.:-]{1,128}',ident):raise FixtureFailure('invalid binding identifier')
    self.send(c,("<iq xmlns='jabber:client' type='result' id='"+ident+"'><bind xmlns='urn:ietf:params:xml:ns:xmpp-bind'><jid>alice@home.test/faction208</jid></bind></iq>").encode())
    msg=ET.fromstring(r.element('message'));self.message_seen=True
    assert msg.get('from')=='alice@home.test/faction208' and msg.get('to')=='bob@away.test/device1'
    assert msg.find('{jabber:client}body') is None and msg.find('{jabber:client}subject') is None
    enc=msg.find('{urn:xmpp:omemo:2}encrypted');assert enc is not None
    content=enc.find('{urn:xmpp:omemo:2}payload');assert content is not None and content.text==self.payload
    if self.mode=='lost':return
    if self.mode=='malformed':out=b'<message><x></message>'
    elif self.mode=='depth':out=b'<message>'+b'<n>'*40+b'</n>'*40+b'</message>'
    elif self.mode=='events':out=b'<message>'+b'<n/>'*300+b'</message>'
    elif self.mode=='token':out=b'<message><n>'+b'A'*70000+b'</n></message>'
    elif self.mode=='stanza':out=b'<message>'+((b'<n>'+b'A'*512+b'</n>')*34)+b'</message>'
    elif self.mode=='whitespace':out=b' '*70000
    else:out=("<message xmlns='jabber:client' from='bob@away.test/device1' to='alice@home.test/faction208' type='chat' id='synthetic-208'><encrypted xmlns='urn:xmpp:omemo:2'><payload>"+self.payload+"</payload></encrypted></message>").encode()
    self.send(c,out)
    try:
     while not self.stop.is_set() and c.recv(4096):pass
    except (OSError,ssl.SSLError):pass
  except (OSError,ssl.SSLError,FixtureFailure,ET.ParseError):
   # Negative identity/cancellation cases may close at any handshake stage. The
   # client status and observed phase below decide whether that is expected.
   pass
  except BaseException as exc:self.failure=type(exc).__name__
  finally:self.listener.close()
 def close(self):
  self.stop.set();self.listener.close();self.thread.join(timeout=16)
  if self.thread.is_alive():raise FixtureFailure('owned TLS fixture did not stop')
  if self.failure:raise FixtureFailure('owned TLS fixture failed internally')


def run_case(binary,root,pairs,name,expected,mode='good',password=None,extra=None,pair='good',ca='ca',pin=None):
 payload=base64.b64encode(secrets.token_bytes(64)).decode();password=password or secrets.token_hex(20)
 peer=Peer(root,pairs[pair],password,payload,mode)
 actual_pin=hashlib.sha256(ssl.PEM_cert_to_DER_cert(pairs['good'][0].read_text())).hexdigest()
 config='\n'.join([str(peer.port),str(root/(ca+'.pem')),pin if pin is not None else actual_pin,password,payload])+'\n'
 started=time.monotonic()
 try:
  p=subprocess.run([str(binary),'probe',*(extra or [])],input=config.encode(),capture_output=True,timeout=15)
  elapsed=time.monotonic()-started
  assert not p.stderr,'prototype emitted diagnostic data'
  assert password.encode() not in p.stdout and payload.encode() not in p.stdout,'private fixture data escaped'
  result=json.loads(p.stdout);assert result['status'] in expected,(name,result)
  assert p.returncode==(0 if result['status']=='ok' else 1)
  assert result['attempts']==1 and 0<=result['received_bytes']<=65536 and 0<=result['events']<=256
  assert peer.connections==1
  if result['status']=='ok':assert peer.auth_ok and peer.message_seen
  if name in {'wrong_server_name','untrusted_certificate','changed_certificate_pin','no_plain_mechanism'}:assert not peer.auth_seen,'credentials sent before qualified identity/mechanism'
  if name=='absolute_deadline_bounds_trickling_peer':assert elapsed<13
  return validated_outcome(name,result['status'],peer.tls_version)
 finally:peer.close()


def probe(binary,openssl='openssl'):
 checks=[];outcomes=[]
 with tempfile.TemporaryDirectory(prefix='faction208-private-tls-') as tmp:
  root=Path(tmp).resolve();root.chmod(0o700);pairs=certificates(root,openssl)
  scenarios=[
   ('verified_tls_plain_auth_and_ciphertext_exchange',{'ok'},dict()),
   ('verified_tls12_profile',{'ok'},dict(mode='tls12')),
   ('fragmented_xml_exchange',{'ok'},dict(mode='fragmented')),
   ('coalesced_sasl_restart_retains_unread_bytes',{'ok'},dict(mode='coalesced-restart')),
   ('wrong_server_name',{'tls'},dict(pair='wrong-name',pin=hashlib.sha256(ssl.PEM_cert_to_DER_cert(pairs['wrong-name'][0].read_text())).hexdigest())),
   ('untrusted_certificate',{'tls'},dict(ca='untrusted-ca')),
   ('changed_certificate_pin',{'pin'},dict(pair='changed')),
   ('failed_account_authentication',{'auth'},dict(mode='auth-fail')),
   ('no_plain_mechanism',{'mechanism'},dict(mode='no-plain')),
   ('malformed_xml_rejected',{'xml_malformed'},dict(mode='malformed')),
   ('excessive_depth_rejected_before_dom',{'xml_limit'},dict(mode='depth')),
   ('excessive_events_rejected_before_dom',{'xml_limit'},dict(mode='events')),
   ('oversized_unfinished_text_rejected',{'xml_limit'},dict(mode='token')),
   ('oversized_stanza_rejected',{'xml_limit'},dict(mode='stanza')),
   ('whole_connection_whitespace_budget',{'xml_limit'},dict(mode='whitespace')),
   ('connection_loss_has_no_retry',{'lost'},dict(mode='lost')),
   ('absolute_deadline_bounds_trickling_peer',{'timeout'},dict(mode='trickle')),
   ('explicit_cancellation_bounds_stalled_peer',{'cancelled'},dict(mode='stalled',extra=['--cancel-after-ms','250']))]
  for name,expected,kwargs in scenarios:
   outcomes.append(run_case(binary,root,pairs,name,expected,**kwargs));checks.append(name)
 result=dict(result='pass',platform=platform.system(),checks=checks,count=len(checks),outcomes=outcomes,private_fixture_removed=not root.exists(),product_adopted=False,live_federation=False)
 validate_summary(result);return result

if __name__=='__main__':
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--binary',type=Path,required=True);p.add_argument('--openssl',default='openssl');p.add_argument('--output',type=Path)
 a=p.parse_args();result=probe(a.binary.resolve(),a.openssl)
 if a.output:a.output.write_text(json.dumps(result,indent=2)+'\n')
 print('Native XMPP transport fixture passed:',result['count'],'controls;',result['platform'])
