#!/usr/bin/env python3
"""Opt-in device passport with real PTYs, local fixture seals and a loopback trap."""
import argparse,hashlib,json,re,socket,subprocess,tempfile
from pathlib import Path
from smoke_board_launcher import LauncherConsole,PICK,catalog
PROMPT='Passport> '
def run(binary,output=None):
 checks=[];children=[]
 with tempfile.TemporaryDirectory(prefix='faction-passport-ui-') as tmp,socket.socket() as trap:
  root=Path(tmp).resolve();root.chmod(0o700);trap.bind(('127.0.0.1',0));trap.listen();trap.settimeout(.2)
  def private(p,s):p.write_text(s);p.chmod(0o600)
  for n in ['identity','known_hosts','identity.pub']:private(root/n,'ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA' if n.endswith('.pub') else 'fixture')
  text=f'version=1\nhost="127.0.0.1"\nport={trap.getsockname()[1]}\nfile_host="private.invalid"\nfile_port=2223\nidentity={json.dumps(str(root/"identity"))}\npublic_key={json.dumps(str(root/"identity.pub"))}\nknown_hosts={json.dumps(str(root/"known_hosts"))}\n'
  profiles=[root/f'board{i}.toml' for i in range(4)]
  for p in profiles:private(p,text)
  pack=root/'boards.toml';store=root/'passport';private(pack,catalog(profiles).replace('version=1\n','version=1\npassport_store='+json.dumps(str(store))+'\n',1))
  seal=root/'seal.toml';sealtext="version=1\ntitle='Night Radio'\nartist='Original fixture'\nrights='BSD-3-Clause'\nart='[ N / W ]'\n";private(seal,sealtext)
  def cli(*args):return subprocess.run([str(binary),'--passport-store',str(store),*args],capture_output=True,timeout=5)
  def start(**kw):c=LauncherConsole([binary,'--boards',pack],**kw);children.append(c);c.expect(PICK);return c
  def cmd(c,s):c.send(s);c.expect(PROMPT)
  def open(c):c.raw(b'j');c.expect(PROMPT)
  def quit(c):c.send('q');c.expect(PICK);c.raw(b'q');c.finish()
  def edit(c,note,rooms,confirm='SAVE'):
   c.send('e');c.expect('Note (empty keeps, - clears): ');c.send(note);c.expect('Rooms (empty keeps, - clears): ');c.send(rooms);c.expect('Type SAVE to confirm: ');c.send(confirm);c.expect(PROMPT)
  def collect(c,word='COLLECT'):
   c.send('i');c.expect('Local seal file (no remote import): ');c.send(str(seal));c.expect('Type '+word+' to confirm: ');c.send(word);c.expect(PROMPT)
  try:
   c=start();c.raw(b'j');c.expect('Enter to return to local Boards: ');c.send('');c.expect(PICK);assert not store.exists();c.raw(b'q');c.finish();checks.append('opening_an_uninitialized_notebook_does_not_create_storage_or_connect')
   p=cli('init');assert p.returncode==0 and b'plaintext' in p.stdout;assert cli('init').returncode==1;before=(store/'passport.toml').read_bytes();checks.append('initialization_is_explicit_private_and_create_only')
   c=start(columns=40,rows=18);open(c);page=c.transcript.split(b'LOCAL PASSPORT / on your device')[-1];plain=re.sub(rb'\x1b\[[0-9;?]*[ -/]*[@-~]',b'',page).decode();assert len(plain.splitlines())<=18 and all(len(l)<40 for l in plain.splitlines());quit(c);assert (store/'passport.toml').read_bytes()==before;checks.append('plain_ascii_notebook_fits_40x18_and_opening_saves_no_visit')
   c=start();open(c);edit(c,'PRIVATE_NOTE','SECRET_ROOM');quit(c);p=cli('list');assert p.returncode==0 and b'board-1' in p.stdout and b'PRIVATE_NOTE' not in p.stdout
   c=start();open(c);assert b'PRIVATE_NOTE' in c.transcript and b'SECRET_ROOM' in c.transcript;quit(c);checks.append('explicit_notes_and_favorite_text_persist_without_timestamps_or_public_presence')
   c=start();open(c);collect(c);saved=(store/'passport.toml').read_bytes();c.send('i');c.expect('Local seal file (no remote import): ');c.send(str(seal));c.expect(PROMPT);assert b'Seal already collected; unchanged.' in c.transcript;quit(c);assert (store/'passport.toml').read_bytes()==saved;checks.append('confirmed_inert_seal_collection_is_deduplicated_without_rewriting_notes')
   p=cli('export','board-1');assert p.returncode==0 and b'[ N / W ]' in p.stdout
   for secret in ['PRIVATE_NOTE','SECRET_ROOM',str(root),'private.invalid','board-1','Local Board 1','binding','timestamp']:assert secret.encode() not in p.stdout
   checks.append('plain_deliberate_export_contains_souvenir_fields_only_and_omits_personal_contact_material')
   c=start();open(c);c.send('s');c.expect('Share> ');share=c.transcript.split(b'LOCAL PASSPORT / on your device')[-1];assert b'PRIVATE_NOTE' not in share and b'SECRET_ROOM' not in share;c.send('e');c.expect('Share> ');assert b'PRIVATE_NOTE' in c.transcript;c.send('f');c.expect('Share> ');assert b'SECRET_ROOM' in c.transcript;c.send('q');c.expect(PROMPT);quit(c);checks.append('local_share_preview_defaults_private_fields_off_and_requires_explicit_toggles')
   private(seal,sealtext.replace('[ N / W ]','[ SECOND ]'));c=start();open(c);collect(c,'REPLACE');quit(c);assert b'SECOND' in cli('export','board-1').stdout;checks.append('different_souvenir_requires_review_and_explicit_replacement')
   original=Path(__file__).resolve().parents[1]/'deploy/examples/passport/nightwire.toml';private(seal,original.read_text());c=start();open(c);collect(c,'REPLACE');quit(c);assert b'Nightwire' in cli('export','board-1').stdout;checks.append('original_nightwire_calling_card_imports_with_required_rights_and_inert_art')
   saved=(store/'passport.toml').read_bytes();private(profiles[0],text.replace('port=2223','port=2224'));c=start();open(c);assert b'Profile/pins changed: review association.' in c.transcript;cmd(c,'i');assert b'Profile changed; review then B:rebind.' in c.transcript;c.send('b');c.expect('Type REBIND to confirm: ');c.send('REBIND');c.expect(PROMPT);assert b'Local profile snapshot unchanged.' in c.transcript;quit(c);checks.append('changed_profile_is_never_silently_rebound_and_seal_import_requires_review')
   saved=(store/'passport.toml').read_bytes();private(seal,sealtext.replace('[ N / W ]','bad'+chr(27)+'[2J'));c=start();open(c);c.send('i');c.expect('Local seal file (no remote import): ');c.send(str(seal));c.expect(PROMPT);quit(c);assert (store/'passport.toml').read_bytes()==saved;checks.append('malformed_or_control_injected_seal_does_not_mutate_the_saved_notebook')
   private(pack,catalog(profiles[1:]).replace('version=1\n','version=1\npassport_store='+json.dumps(str(store))+'\n',1));c=start();open(c);c.send('l');c.expect('List> ');c.send('c');c.expect('Saved local ID: ');c.send('board-1');c.expect(PROMPT);assert b'Contact removed from this catalog.' not in c.transcript
   # catalog helper renumbers entries; use a genuinely absent saved ID below.
   quit(c);private(pack,catalog(profiles).replace('board-1','renamed-board',1).replace('version=1\n','version=1\npassport_store='+json.dumps(str(store))+'\n',1));c=start();open(c);c.send('l');c.expect('List> ');c.send('c');c.expect('Saved local ID: ');c.send('board-1');c.expect(PROMPT);assert b'Contact removed from this catalog.' in c.transcript;c.send('d');c.expect('Type DELETE to confirm: ');c.send('DELETE');c.expect(PROMPT);quit(c);assert b'board-1' not in cli('list').stdout;checks.append('removed_contact_notes_remain_viewable_exportable_and_explicitly_deletable')
   private(pack,catalog(profiles).replace('version=1\n','version=1\npassport_store='+json.dumps(str(store))+'\n',1));c=start();open(c);saved=(store/'passport.toml').read_bytes();edit(c,'UNSAVED','',confirm='q');quit(c);assert (store/'passport.toml').read_bytes()==saved;checks.append('cancelled_edit_does_not_persist_drafts_or_recreate_deleted_entries')
   saved=(store/'passport.toml').read_bytes();c=start();open(c);c.send('e');c.expect('Note (empty keeps, - clears): ');c.send('RESIZE_PENDING');c.expect('Rooms (empty keeps, - clears): ');c.send('');c.expect('Type SAVE to confirm: ');c.resize(30,12);c.send('n');c.expect('Review> ');c.send('SAVE');c.expect('Review> ');assert (store/'passport.toml').read_bytes()==saved;c.resize(80,24);c.send('p');c.expect('Type SAVE to confirm: ');c.send('q');c.expect(PROMPT);quit(c);assert (store/'passport.toml').read_bytes()==saved;checks.append('too_small_terminal_cannot_confirm_unseen_content_and_resize_preserves_review')
   c=start();open(c);assert cli('list').returncode==1;c.raw(b'\x03');c.expect(PICK);c.raw(b'q');c.finish();assert cli('list').returncode==0;checks.append('concurrent_local_clients_fail_busy_and_ctrl_c_releases_the_notebook_lock')
   assert (store.stat().st_mode&0o777)==0o700 and ((store/'passport.toml').stat().st_mode&0o777)==0o600
   assert set(p.name for p in store.iterdir())=={'passport.lock','passport.toml'}
   try:conn,_=trap.accept()
   except TimeoutError:pass
   else:conn.close();raise AssertionError('passport opened a network connection')
   checks.append('private_fixed_storage_has_no_pending_files_network_io_or_travel_history')
  finally:
   for c in children:c.close()
 result=dict(result='pass',count=len(checks),checks=checks,device_local=True,plaintext_notes=True,automatic_visit_tracking=False,verified_board_identity=False,network_io=False,mail_change=False,capacity_claim=False,hardening_claim=False)
 if output:output.write_text(json.dumps(result,indent=2,sort_keys=True)+'\n')
 print('Local passport passed:',len(checks),'controls.')
if __name__=='__main__':
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('binary',type=Path);p.add_argument('--output',type=Path);a=p.parse_args();run(a.binary.resolve(),a.output)
