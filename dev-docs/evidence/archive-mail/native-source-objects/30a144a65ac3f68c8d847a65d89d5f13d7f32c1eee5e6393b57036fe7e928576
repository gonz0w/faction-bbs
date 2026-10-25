import importlib.util,io,sys,unittest,tempfile
from unittest.mock import patch
from pathlib import Path
spec=importlib.util.spec_from_file_location('probe_xmpp_transport',Path(__file__).with_name('probe_xmpp_transport.py'))
fixture=importlib.util.module_from_spec(spec)
spec.loader.exec_module(fixture)
class Channel:
 def __init__(self,chunks):self.chunks=list(chunks)
 def recv(self,n):return self.chunks.pop(0) if self.chunks else b''
class FixtureReader(unittest.TestCase):
 def test_fragmented_and_coalesced_elements_keep_unread_tail(self):
  r=fixture.Reader(Channel([b'<a',b'>one</a><b>two</b>']))
  self.assertEqual(r.element('a'),b'<a>one</a>')
  self.assertEqual(r.element('b'),b'<b>two</b>')
 def test_unfinished_and_oversized_input_never_grows_without_bound(self):
  with self.assertRaises(fixture.FixtureFailure):fixture.Reader(Channel([b'<a>'])).element('a')
  with self.assertRaises(fixture.FixtureFailure):fixture.Reader(Channel([b'x'*100]),limit=64).element('a')
 def test_stream_restart_preserves_next_element(self):
  r=fixture.Reader(Channel([b"<?xml version='1.0'?><stream:stream xmlns:stream='http://etherx.jabber.org/streams'><auth>private</auth>"]))
  self.assertIn(b'<stream:stream',r.stream())
  self.assertEqual(r.element('auth'),b'<auth>private</auth>')
 def test_certificate_fixture_has_explicit_private_config_not_host_defaults(self):
  commands=[]
  def command(args):
   commands.append(args)
   if '-keyout' in args:Path(args[args.index('-keyout')+1]).write_text('synthetic-placeholder')
  with tempfile.TemporaryDirectory() as tmp:
   with patch.object(fixture,'_command',command):fixture.certificates(Path(tmp),'openssl')
   requests=[args for args in commands if args[1]=='req']
   self.assertTrue(requests)
   self.assertTrue(all('-config' in args for args in requests))
   for args in requests:self.assertTrue(Path(args[args.index('-config')+1]).is_file())
 def test_cost_parser_requires_real_usage_and_refuses_unknown_diagnostics(self):
  raw='0.02 real 0.01 user 0.00 sys\n 14000 maximum resident set size\n 0 page faults\n'
  self.assertEqual(fixture.parse_native_time(raw)['max_rss_kib'],14000)
  for bad in ['',raw+'unexpected secret-like diagnostics\n',raw.replace('14000','0')]:
   with self.assertRaises(fixture.FixtureFailure):fixture.parse_native_time(bad)
 def test_tls_outcome_never_claims_unobserved_or_obsolete_success(self):
  self.assertEqual(fixture.validated_outcome('synthetic','ok','TLSv1.3')['tls_version'],'TLSv1.3')
  for version in [None,'TLSv1','SSLv3']:
   with self.assertRaises(fixture.FixtureFailure):fixture.validated_outcome('synthetic','ok',version)
 def test_success_summary_rejects_failure_overclaims_and_missing_teardown(self):
  good=dict(result='pass',count=1,checks=['synthetic'],private_fixture_removed=True,product_adopted=False,live_federation=False)
  fixture.validate_summary(good)
  for field,value in [('count',2),('private_fixture_removed',False),('product_adopted',True),('result','fail')]:
   d=dict(good);d[field]=value
   with self.assertRaises(AssertionError):fixture.validate_summary(d)
if __name__=='__main__':unittest.main()
