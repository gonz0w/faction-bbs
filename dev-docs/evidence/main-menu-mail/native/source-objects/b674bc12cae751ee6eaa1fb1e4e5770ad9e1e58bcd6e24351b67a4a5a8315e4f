#!/usr/bin/env python3
"""Device-local connection checks on real PTYs; no remote SSH or network probe."""
import argparse,base64,hashlib,json,re,socket,subprocess,tempfile
from pathlib import Path
from smoke_board_launcher import LauncherConsole,PICK,catalog
PROMPT='Desk> '
def run(binary,output=None):
 checks=[];children=[]
 with tempfile.TemporaryDirectory(prefix='faction-connection-') as tmp,socket.socket() as trap:
  root=Path(tmp);root.chmod(0o700);trap.bind(('127.0.0.1',0));trap.listen();trap.settimeout(.2);port=trap.getsockname()[1]
  blob=b'\0\0\0\x0bssh-ed25519\0\0\0\x20'+bytes(range(32));key=base64.b64encode(blob).decode();fp='SHA256:'+base64.b64encode(hashlib.sha256(blob).digest()).decode().rstrip('=')
  blob2=blob[:-1]+b'\x7f';key2=base64.b64encode(blob2).decode();fp2='SHA256:'+base64.b64encode(hashlib.sha256(blob2).digest()).decode().rstrip('=')
  def private(p,text):p.write_text(text);p.chmod(0o600)
  identity=root/'private-admission';public=root/'public-admission';known=root/'private-known-hosts'
  private(identity,'LOCKED-KEY-FIXTURE-not-an-SSH-key');private(public,'ssh-ed25519 '+key+' private-comment')
  pins=f'[127.0.0.1]:{port} ssh-ed25519 {key} private-host-comment\n[files.private.invalid]:2223 ssh-ed25519 {key2}\n'
  private(known,pins)
  text=f'version=1\nhost="127.0.0.1"\nport={port}\nfile_host="files.private.invalid"\nfile_port=2223\nidentity={json.dumps(str(identity))}\npublic_key={json.dumps(str(public))}\nknown_hosts={json.dumps(str(known))}\n'
  profiles=[root/f'board-{i}.toml' for i in range(4)]
  for p in profiles:private(p,text)
  pack=root/'boards.toml';private(pack,catalog(profiles))
  def start(**kw):c=LauncherConsole([binary,'--boards',pack],**kw);children.append(c);c.expect(PICK);return c
  def cmd(c,s):c.send(s);c.expect(PROMPT)
  def leave(c):c.send('q');c.expect(PICK);c.raw(b'q');c.finish()
  def cli():return subprocess.run([str(binary),'--check-profile',str(profiles[0])],capture_output=True,timeout=5)
  try:
   c=start(columns=40,rows=18);c.raw(b'\x1b[B');c.expect('> [B] Local Board 2');c.raw(b'i');c.expect(PROMPT)
   view=c.transcript.split(b'LOCAL CONNECTION DESK')[-1];plain=re.sub(rb'\x1b\[[0-9;?]*[ -/]*[@-~]',b'',view).decode();assert len(plain.splitlines())<18 and all(len(l)<40 for l in plain.splitlines())
   cmd(c,'n');assert b'SSH admission: not tested' in c.transcript;cmd(c,'p');c.send('q');c.expect('> [B] Local Board 2');c.expect(PICK);c.raw(b'q');c.finish();checks.append('selected_board_local_desk_pages_fit_40x18_and_preserve_highlight')
   p=cli();assert p.returncode==0;assert b'Admission key file: readable' in p.stdout and b'Private-key unlock: not tested' in p.stdout
   for secret in [str(root),'127.0.0.1','files.private.invalid',key,key2,fp,fp2,str(port),'Local Board']:
    assert secret.encode() not in p.stdout+p.stderr,secret
   checks.append('plain_non_tty_share_report_redacts_all_contact_paths_keys_labels_and_endpoints')
   c=start();c.raw(b'i');c.expect(PROMPT);c.send('k');c.expect('Checking local pins; Ctrl-C cancels.');c.expect(PROMPT)
   for _ in range(5):cmd(c,'n')
   assert fp.encode() in c.transcript and fp2.encode() in c.transcript
   assert key.encode() not in c.transcript and b'private-host-comment' not in c.transcript
   leave(c);checks.append('explicit_fixed_openssh_lookup_reports_distinct_terminal_and_files_fingerprints_without_raw_material')
   before={p.name:hashlib.sha256(p.read_bytes()).hexdigest() for p in root.iterdir()}
   c=start();c.raw(b'i');c.expect(PROMPT);cmd(c,'s');summary=c.transcript.split(b'LOCAL CONNECTION DESK')[-1]
   for secret in [str(root),'127.0.0.1','files.private.invalid',key,fp,'Local Board']:assert secret.encode() not in summary
   cmd(c,'k');cmd(c,'s');leave(c);assert before=={p.name:hashlib.sha256(p.read_bytes()).hexdigest() for p in root.iterdir()};checks.append('share_preview_is_deliberate_redacted_and_does_not_write_a_report_or_history')
   identity.unlink();p=cli();assert p.returncode==1 and b'Admission key file: needs attention' in p.stdout and b'Admission public key: valid Ed25519 format' in p.stdout and b'Host-pin file: readable' in p.stdout
   c=start();c.raw(b'i');c.expect(PROMPT);assert b'Admission key file: needs attention' in c.transcript;private(identity,'restored-fixture');cmd(c,'r');assert b'Admission key file: readable' in c.transcript;leave(c);checks.append('missing_key_is_distinct_from_other_files_and_reload_recovers_without_connecting')
   private(profiles[0],text+'\nunknown="private-sentinel\\u001b[31m"\n');p=cli();assert p.returncode==1 and b'Profile structure: needs attention' in p.stdout and b'private-sentinel' not in p.stdout+p.stderr and b'\x1b' not in p.stdout+p.stderr
   c=start();c.raw(b'i');c.expect(PROMPT);assert b'Profile cannot be parsed/read safely.' in c.transcript;leave(c);private(profiles[0],text);checks.append('malformed_profile_never_echoes_parser_content_or_terminal_controls')
   known.unlink();known.symlink_to(public);p=cli();assert p.returncode==1 and b'Host-pin file: needs attention' in p.stdout;known.unlink();private(known,pins);known.chmod(0o666);assert cli().returncode==1;known.chmod(0o600);checks.append('unsafe_symlink_and_writable_host_pin_files_fail_local_readiness')
   subprocess.run(['/usr/bin/ssh-keygen','-H','-f',str(known)],check=True,capture_output=True,timeout=5)
   hashed_before=known.read_bytes();c=start();c.raw(b'i');c.expect(PROMPT);c.send('k');c.expect('Checking local pins; Ctrl-C cancels.');c.expect(PROMPT)
   for _ in range(5):cmd(c,'n')
   assert fp.encode() in c.transcript and fp2.encode() in c.transcript;leave(c);assert known.read_bytes()==hashed_before;checks.append('hashed_known_hosts_are_matched_by_base_openssh_without_mutating_pins')
   private(known,'');c=start();c.raw(b'i');c.expect(PROMPT);c.send('k');c.expect('Checking local pins; Ctrl-C cancels.');c.expect(PROMPT);assert b'No ordinary matching host pins.' in c.transcript;leave(c);checks.append('empty_matches_never_claim_live_identity_or_local_admission_success')
   private(known,pins);c=start();c.raw(b'\x1b[B');c.expect('> [B] Local Board 2');c.raw(b'i');c.expect(PROMPT);c.raw(b'\x03');c.expect('> [B] Local Board 2');c.expect(PICK);c.raw(b'q');c.finish();checks.append('ctrl_c_returns_to_local_picker_without_losing_selection')
   c=start();c.raw(b'i');c.expect(PROMPT);c.raw(b'k\r\x03');c.expect('Checking local pins; Ctrl-C cancels.');c.expect(PICK);c.raw(b'q');c.finish();checks.append('ctrl_c_cancels_an_active_pin_lookup_and_reaps_before_return')
   c=start();c.raw(b'i');c.expect(PROMPT);c.resize(30,12);cmd(c,'n');assert b'Resize to 40x18.' in c.transcript;c.resize(80,24);cmd(c,'r');assert b'DEVICE CHECK / no connection' in c.transcript;leave(c);checks.append('small_resize_uses_recovery_screen_and_reload_returns_to_bounded_pages')
   private(known,pins.splitlines()[0]+'\n');known.write_text(known.read_text()*1000);known.chmod(0o600);c=start();c.raw(b'i');c.expect(PROMPT);c.send('k');c.expect('Checking local pins; Ctrl-C cancels.');c.expect(PROMPT)
   assert b'Lookup unavailable; inspect locally.' in c.transcript;leave(c);checks.append('lookup_output_is_bounded_and_failure_returns_to_the_desk')
   try:conn,_=trap.accept()
   except TimeoutError:pass
   else:conn.close();raise AssertionError('local connection desk opened a network connection')
   checks.append('all_diagnostics_and_pin_checks_make_no_network_connection')
  finally:
   for c in children:c.close()
 result=dict(result='pass',count=len(checks),checks=checks,plain_report=True,local_only=True,helper='/usr/bin/ssh-keygen',helper_seconds=2,helper_bytes=32768,fingerprints_per_endpoint=4,network_probe=False,host_verification=False,admission_claim=False,tor_support=False,capacity_claim=False,hardening_claim=False)
 if output:output.write_text(json.dumps(result,indent=2,sort_keys=True)+'\n')
 print('Local connection desk passed:',len(checks),'controls.')
if __name__=='__main__':
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('binary',type=Path);p.add_argument('--output',type=Path);a=p.parse_args();run(a.binary.resolve(),a.output)
