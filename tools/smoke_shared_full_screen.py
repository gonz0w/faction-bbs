#!/usr/bin/env python3
"""Real SSH/native full editing and room checkpoints on a marked disposable NetBSD fixture."""
import argparse,json,secrets,shlex,subprocess,time,tomllib,select
from pathlib import Path
from check_restricted_ssh import SSHConsole
from smoke_native_client_ssh import NativeConsole
from smoke_room_browser import MENU
from smoke_writing_desk import PROMPT,begin,command,action_end
from smoke_full_screen import FOOTER,SAVED,open_saved,enter_screen,edit_save,resume_send

def run(a):
    assert a.disposable_fixture
    admin=json.loads(a.admin_argv.read_text());member=json.loads(a.member_argv.read_text());guest=json.loads(a.guest_native_argv.read_text());password=a.password_file.read_text().strip();children=[];checks=[];calls=0
    def remote(cmd,data=None):
        p=subprocess.run(admin+['set -e; PATH=/usr/sbin:/sbin:/usr/bin:/bin:/usr/pkg/bin; export PATH; test -f /etc/faction-disposable-test; '+cmd],input=data,capture_output=True,timeout=30)
        if p.returncode:(a.output.parent/'full-screen-private-error.log').write_bytes(p.stdout+p.stderr)
        assert p.returncode==0,'owned full-screen fixture operation failed';return p.stdout.decode()
    original=remote('cat /etc/faction/service.toml');assert tomllib.loads(original)['state_dir']=='/var/db/faction-profile-fixture';db='/var/db/faction-profile-fixture/faction.sqlite3'
    def query(sql):
        code='import sqlite3,json; d=sqlite3.connect('+repr('file:'+db+'?mode=ro')+',uri=True); print(json.dumps(d.execute('+repr(sql)+').fetchall()))'
        return json.loads(remote('python3.13 -c '+shlex.quote(code)))
    def count():return query('select count(*) from posts')[0][0]
    def connect(route='ssh',plain=False,handle='sysop',register=False):
        nonlocal calls
        if calls:time.sleep(5.1)
        calls+=1;c=SSHConsole(member) if route=='ssh' else NativeConsole(guest if route=='guest' else [str(a.native_binary.resolve()),'--profile',str(a.native_profile.resolve())]);children.append(c)
        c.expect('[L]ogin [R]egister [Q]uit: ');c.send('r' if register else 'l');c.expect('Handle: ');c.send(handle);c.password(password)
        if register:c.password(password,'Confirm password (hidden): ');c.expect('Registration pending; sysop approval required.');return c
        c.expect('Press Enter to continue: ');c.send('');c.expect('Command (then Enter): ' if plain else MENU);return c
    def logout(c,plain=False):
        c.send('q') if plain else c.raw(b'q');c.expect('Logged out.');c.finish()
    def configure(text):
        remote('/etc/rc.d/faction_broker onestop');remote('cat > /etc/faction/service.toml',text.encode());remote('/etc/rc.d/faction_broker onestart')
    def status(c,handle,action):
        c.raw(b's');c.expect('[C]reate room: ');c.send(action);c.expect('Member handle: ');c.send(handle);c.expect('Status updated; previous sessions expired.');action_end(c,shared=True)
    try:
        assert count()==0 and query('select count(*) from room_drafts')==[[0]]
        c=connect();edit_save(c);assert count()==0 and query('select body from room_drafts')==[['bright first\ntail']];checks.append('actual_ssh_caret_paragraph_preview_and_explicit_checkpoint');c.close();children.remove(c)
        c=connect();resume_send(c,count,True);logout(c);assert query('select count(*) from room_drafts')==[[0]];checks.append('actual_disconnect_resume_subject_confirmation_and_atomic_cleanup')
        c=connect('native');c.raw(b't');c.expect('[D]one: ');c.send('u');c.expect('[D]one: ');c.send('d');c.expect(MENU);begin(c,subject='Échos 界');command(c,'Wide 界');enter_screen(c);c.raw(b'\x01inert ');c.expect('inert Wide 界');c.expect(FOOTER);c.raw(b'\x13');c.expect('Checkpoint saved');c.expect(FOOTER);c.raw(b'\x11');c.expect(PROMPT);c.send('.leave');c.expect('last explicit checkpoint kept');action_end(c,shared=True);logout(c);checks.append('actual_host_native_unicode_cells_and_checkpoint')
        c=connect('guest');c.raw(b't');c.expect('[D]one: ');c.send('u');c.expect('[D]one: ');c.send('d');c.expect(MENU);open_saved(c);enter_screen(c);c.raw(b'\x14');c.expect('Type SEND');c.expect('keep editing: ');c.send('SEND');c.expect('Posted #');action_end(c,shared=True);logout(c);assert query('select body from posts order by id desc limit 1')==[['inert Wide 界']];checks.append('actual_netbsd_native_resume_full_screen_and_confirmed_send')
        configure(original.replace('ansi=true','ansi=false'));c=connect(plain=True);begin(c,True);c.send('.screen');c.expect('Full-screen unavailable');c.expect(PROMPT);command(c,'Plain checkpoint.');command(c,'.save');c.send('.leave');c.expect('last explicit checkpoint kept');action_end(c,True,True);logout(c,True);checks.append('actual_shared_plain_save_with_no_ansi')
        c=connect(plain=True);open_saved(c,True);command(c,'.preview');c.send('.cancel');c.expect('Draft cancelled');action_end(c,True,True);logout(c,True);assert b'\x1b' not in c.transcript;checks.append('actual_shared_plain_resume_preview_cancel');configure(original)
        c=connect();begin(c);command(c,'Resize keeps me.');enter_screen(c);c.resize(40,18);c.expect(FOOTER);c.resize(30,12);c.expect('Press Enter to resume line editing: ');c.send('');c.expect(PROMPT);command(c,'.save');c.send('.leave');c.expect('last explicit checkpoint kept');action_end(c,True,True);logout(c,True);assert query('select body from room_drafts')==[['Resize keeps me.']];checks.append('actual_shared_resize_reflow_and_classic_fallback_preserve_draft')
        first=connect();open_saved(first);second=connect();open_saved(second);command(first,'.save');second.send('.save');second.expect('Save failed; previous checkpoint retained');second.expect(PROMPT);second.send('.leave');second.expect('last explicit checkpoint kept');action_end(second,shared=True);logout(second);first.send('.cancel');first.expect('Draft cancelled');action_end(first,shared=True);checks.append('concurrent_checkpoint_conflict_preserves_newer_revision')
        before=count();begin(first);command(first,'One checkpoint, one explicit send.');command(first,'.save')
        code='import sqlite3,sys; d=sqlite3.connect('+repr(db)+'); d.execute("BEGIN IMMEDIATE"); print("ready",flush=True); sys.stdin.readline(); d.rollback()'
        lock=subprocess.Popen(admin+['test -f /etc/faction-disposable-test && python3.13 -c '+shlex.quote(code)],stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=subprocess.PIPE)
        try:
            assert select.select([lock.stdout],[],[],15)[0] and lock.stdout.readline()==b'ready\n';first.send('.send');first.expect('Database busy. Retry this same submission? [y/N]: ')
        finally:lock.communicate(b'finish\n',timeout=10)
        first.send('y');first.expect('Posted #');action_end(first,shared=True);assert count()==before+1 and query('select count(*) from room_drafts')==[[0]];checks.append('checkpoint_busy_send_retry_is_single_and_clears_atomically')
        handle='writer-'+secrets.token_hex(3);pending=connect(handle=handle,register=True);pending.finish();status(first,handle,'a');writer=connect(handle=handle);begin(writer);command(writer,'Revocation clears this checkpoint.');command(writer,'.save');before=count();status(first,handle,'v');writer.send('.send');writer.expect('session expired');writer.finish(1);assert count()==before and query('select count(*) from room_drafts')==[[0]];checks.append('pending_denial_and_active_revocation_clear_checkpoint_and_prevent_send')
        # Test an unavailable future reply target on only this synthetic board.
        parent=query('select id from posts order by id limit 1')[0][0];begin(first,reply=str(parent));command(first,'Stale reply draft body.');command(first,'.save');first.send('.leave');first.expect('last explicit checkpoint kept');action_end(first,shared=True)
        code='import sqlite3; d=sqlite3.connect('+repr(db)+'); d.execute("DELETE FROM post_metadata WHERE post_id=?",('+str(parent)+',)); d.execute("DELETE FROM posts WHERE id=?",('+str(parent)+',)); d.commit()';remote('python3.13 -c '+shlex.quote(code))
        mark=len(first.transcript);first.raw(b'p');first.expect('Room ID: ');first.send('1');first.expect(SAVED);first.send('r');first.expect('reply');action_end(first,shared=True);assert b'Stale reply draft body.' not in first.transcript[mark:]
        first.raw(b'p');first.expect('Room ID: ');first.send('1');first.expect(SAVED);first.send('d');first.expect('Reply to post ID');first.close();children.remove(first);checks.append('stale_reply_resume_refused_without_redirect_or_disclosure_and_delete_available')
        assert query('pragma user_version')==[[12]] and query('select count(*) from unread')==[[0]] and query('select count(*) from room_drafts')==[[0]]
        assert all(password.encode() not in c.transcript for c in children)
        a.output.write_text(json.dumps(dict(result='pass',count=len(checks),checks=checks,schema_version=12,normal_sessions=8,admission_refill_ms=5000,capacity_claim=False),indent=2,sort_keys=True)+'\n');print('Actual shared full-screen/saved-draft controls passed:',len(checks))
    finally:
        for c in children:c.close()
        remote('/etc/rc.d/faction_broker onestop || true');remote('cat > /etc/faction/service.toml',original.encode())

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__)
    for n in ['admin-argv','member-argv','password-file','native-binary','native-profile','guest-native-argv','output']:p.add_argument('--'+n,type=Path,required=True)
    p.add_argument('--disposable-fixture',action='store_true',required=True);run(p.parse_args())
