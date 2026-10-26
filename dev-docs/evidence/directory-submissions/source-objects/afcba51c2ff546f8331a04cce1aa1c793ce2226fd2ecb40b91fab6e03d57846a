#!/usr/bin/env python3
"""Approved-member F1/plain guide on real local/worker PTYs; no content actions."""
import argparse,json,os,re,secrets,select,sqlite3,tempfile,time,subprocess
from pathlib import Path
from smoke_local_console import Console
from smoke_member_worker import Worker
from smoke_room_browser import MENU
from smoke_writing_desk import begin,action_end,PROMPT
from smoke_wall import settings,leave_settings,SETTINGS
END='L:topics Q:main menu'
F1=b'\x1bOP'

def run(a):
 checks=[];children=[];password=secrets.token_hex(20)
 with tempfile.TemporaryDirectory(prefix='faction-guide-') as tmp:
  root=Path(tmp);state=root/'board';theme=root/'theme';theme.mkdir(mode=0o700)
  text='version=1\nid="guide-test"\nboard="Night Radio"\nguide_title="CREW HANDBOOK"\nguide_notes="Be kind. Ask before sharing files."\n'
  for action,key,label in [('rooms','R','Room decks'),('new','N','New messages'),('history','H','Room history'),('post','P','Post a message'),('chat','Z','Night talk'),('sysop','S','Sysop controls'),('doors','D','Doors'),('terminal','T','Terminal settings'),('quit','Q','Log out')]:text+=f'[[menu]]\naction="{action}"\nkey="{key}"\nlabel="{label}"\n'
  (theme/'theme.toml').write_text(text);(theme/'theme.toml').chmod(0o600)
  def query(sql):
   with sqlite3.connect(f'file:{state}/faction.sqlite3?mode=ro',uri=True) as d:return d.execute(sql).fetchall()
  def start(handle='sysop',plain=False,worker=False,columns=80,rows=24):
   c=Worker(a.member,state,extra=['--theme',str(theme)],columns=columns,rows=rows,term="dumb" if plain else "xterm") if worker else Console(a.binary,state,'login',['--plain' if plain else '--ansi','--theme',str(theme)],columns=columns,rows=rows);children.append(c)
   if worker:c.expect('[L]ogin [R]egister [Q]uit: ');c.send('l')
   c.expect('Handle: ');c.send(handle);c.password(password);target='Command (then Enter): ' if plain else MENU;deadline=time.monotonic()+15
   while target.encode() not in c.buffer:
    if b'Press Enter to continue: ' in c.buffer:c.expect('Press Enter to continue: ');c.send('')
    assert time.monotonic()<deadline
    if select.select([c.fd],[],[],.05)[0]:b=os.read(c.fd,8192);assert b;c.buffer+=b;c.transcript+=b
   c.expect(target);return c
  def key(c,s,plain=False):c.send(s) if plain else c.raw(s.encode())
  def open_guide(c,plain=False,sequence=F1):
   mark=len(c.transcript);c.send('help') if plain else c.raw(sequence);c.expect('Guide command: ' if plain else END);return c.transcript[mark:]
  def cmd(c,s,plain=False):
   mark=len(c.transcript);key(c,s,plain);c.expect('Guide command: ' if plain else END);return c.transcript[mark:]
  def back(c,plain=False):key(c,'q',plain);c.expect('Command (then Enter): ' if plain else MENU)
  def logout(c,plain=False):key(c,'q',plain);c.expect('Logged out.');c.finish()
  try:
   c=Console(a.binary,state,'init');children.append(c);c.expect('Handle: ');c.send('sysop');c.password(password,'New password (12+ characters, hidden): ');c.password(password,'Confirm password (hidden): ');c.expect('Lobby created.');c.finish()
   modes=[('alice',False,False),('bobby',False,True),('charlie',True,False),('dora',True,True)]
   for h,_,_ in modes:
    c=Console(a.binary,state,'register');children.append(c);c.expect('Handle: ');c.send(h);c.password(password,'New password (12+ characters, hidden): ');c.password(password,'Confirm password (hidden): ');c.expect('Registered pending sysop approval.');c.finish()
   owner=start(plain=True)
   for h,_,_ in modes:owner.send('s');owner.expect('[C]reate room: ');owner.send('a');owner.expect('Member handle: ');owner.send(h);owner.expect('Status updated; previous sessions expired.');owner.expect('Command (then Enter): ')
   baseline=query('select count(*) from posts')
   for h,plain,worker in modes:
    c=start(h,plain,worker,40,18);frame=open_guide(c,plain)
    assert b'CREW HANDBOOK' in frame and b'Local sysop controls' not in frame and b'Files and transfers' not in frame
    if not plain:
     clean=re.sub(rb'\x1b\[[0-9;?]*[ -/]*[@-~]',b'',frame).replace(b'\r',b'').decode();assert len(clean.splitlines())<=18 and all(len(l)<40 for l in clean.splitlines())
    frame=cmd(c,'a',plain);assert b'Be kind. Ask before sharing files.' in frame;cmd(c,'l',plain);frame=cmd(c,'b',plain);all_keys=frame
    for _ in range(3):all_keys+=cmd(c,'n',plain)
    assert b'[Z] Night talk' in all_keys and b'[S] Sysop controls' not in all_keys and b'[D] Doors' not in all_keys
    checks.append(h+'_themed_guide_uses_actual_keys_and_filters_disabled_or_sysop_entries')
    cmd(c,'l',plain);frame=cmd(c,'c',plain)
    for _ in range(7):frame+=cmd(c,'n',plain)
    assert b'.save: explicit board checkpoint' in frame and b'Ctrl-T asks for SEND confirmation.' in frame and b'Room checkpoints are sysop-readable.' in frame
    cmd(c,'l',plain);frame=cmd(c,'d',plain)
    for _ in range(3):frame+=cmd(c,'n',plain)
    assert b'/names: joined channel participants' in frame and b'/who' not in frame and b'/create #name' not in frame
    checks.append(h+'_paged_room_writer_and_chat_references_never_execute_commands')
    back(c,plain);key(c,'?',plain);c.expect('Command (then Enter): ' if plain else MENU);assert query('select count(*) from posts')==baseline;logout(c,plain)
    if plain:assert b'\x1b' not in c.transcript
    checks.append(h+'_q_returns_to_main_and_question_mark_still_only_refreshes')
   open_guide(owner,True);frame=cmd(owner,'s',True);assert b'LOCAL SYSOP / BBS ROLE' in frame;back(owner,True);checks.append('current_sysop_role_gets_distinct_bbs_and_host_operator_reference')
   c=start('alice');open_guide(c);settings(owner,True);owner.send('e');owner.expect(SETTINGS);leave_settings(owner,True);frame=cmd(c,'l');assert b'Community places' in frame;cmd(c,'f');settings(owner,True);owner.send('d');owner.expect(SETTINGS);leave_settings(owner,True);frame=cmd(c,'n');assert b'Choose a topic' in frame and b'Community places' not in frame;back(c);logout(c);checks.append('live_disable_removes_the_current_topic_without_private_content_or_stale_menu_authority')
   for sequence in [b'\x1bOP',b'\x1b[11~']:
    c=start('alice');c.raw(sequence[:2]);time.sleep(.04);c.raw(sequence[2:]);c.expect(END);c.resize(40,18);c.expect(MENU);c.raw(F1);c.expect(END);back(c);logout(c)
   checks.append('both_fragmented_f1_encodings_and_resize_return_to_a_working_classic_menu')
   c=start('alice');begin(c,subject='Kept private subject');c.send('Guide must not steal this draft.');c.expect(PROMPT);mark=len(c.transcript);c.send('.preview');c.expect(PROMPT);assert b'CREW HANDBOOK' not in c.transcript[mark:];c.send('.cancel');c.expect('Draft cancelled');action_end(c);logout(c);checks.append('writing_and_preview_remain_separate_from_read_only_guide_navigation')
   logout(owner,True);assert query('pragma user_version')==[(19,)] and query('select count(*) from unread')==[(0,)] and query('select count(*) from posts')==baseline and query('select count(*) from room_drafts')==[(0,)] and query('select count(*) from file_transfers')==[(0,)] and all(password.encode() not in c.transcript for c in children);checks.append('guide_has_no_posts_unread_draft_transfer_or_password_side_effects')
  except Exception:
   if a.diagnostics:
    a.diagnostics.mkdir(parents=True,exist_ok=True,mode=0o700)
    for i,c in enumerate(children):(a.diagnostics/('guide-private-'+str(i))).write_bytes(c.transcript)
   raise
  finally:
   for c in children:c.close()
 d=dict(result='pass',count=len(checks),checks=checks,schema_version=19,actual_registration_and_approval=True,current_menu_keys=True,current_enabled_topics=True,classic_plain_fallback=True,question_mark_refresh_preserved=True,read_only=True,capacity_claim=False,hardening_claim=False)
 if a.output:a.output.write_text(json.dumps(d,indent=2,sort_keys=True)+'\n')
 print('Board guide passed:',len(checks),'actual local/worker controls.')
if __name__=='__main__':
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('binary',type=Path);p.add_argument('member',type=Path);p.add_argument('--output',type=Path);p.add_argument('--diagnostics',type=Path);a=p.parse_args();a.binary=a.binary.resolve();a.member=a.member.resolve();run(a)
