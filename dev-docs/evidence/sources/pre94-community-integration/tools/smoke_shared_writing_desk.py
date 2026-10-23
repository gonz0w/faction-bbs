#!/usr/bin/env python3
"""Actual restricted SSH/native writing on an explicitly marked disposable NetBSD board."""
import argparse,json,secrets,shlex,subprocess,time,tomllib
from pathlib import Path
from check_restricted_ssh import SSHConsole
from smoke_native_client_ssh import NativeConsole
from smoke_room_browser import MENU,BROWSE,CARD
from smoke_writing_desk import PROMPT,begin,command,action_end,exercise

def run(a):
    assert a.disposable_fixture
    admin=json.loads(a.admin_argv.read_text());member=json.loads(a.member_argv.read_text());guest=json.loads(a.guest_native_argv.read_text());password=a.password_file.read_text().strip()
    children=[];checks=[];calls=0
    def remote(cmd,data=None):
        p=subprocess.run(admin+['set -e; PATH=/usr/sbin:/sbin:/usr/bin:/bin:/usr/pkg/bin; export PATH; test -f /etc/faction-disposable-test; '+cmd],input=data,capture_output=True,timeout=30)
        if p.returncode:(a.output.parent/'writing-private-error.log').write_bytes(p.stdout+p.stderr)
        assert p.returncode==0,'owned writing fixture operation failed';return p.stdout.decode()
    original=remote('cat /etc/faction/service.toml');config=tomllib.loads(original);assert config['state_dir']=='/var/db/faction-profile-fixture'
    database='/var/db/faction-profile-fixture/faction.sqlite3'
    def query(sql):
        code='import sqlite3,json; db=sqlite3.connect('+repr('file:'+database+'?mode=ro')+',uri=True); print(json.dumps(db.execute('+repr(sql)+').fetchall()))'
        return json.loads(remote('python3.13 -c '+shlex.quote(code)))
    def count():return query('select count(*) from posts')[0][0]
    def connect(route='ssh',handle='sysop',register=False,plain=False):
        nonlocal calls
        if calls:time.sleep(5.1)
        calls+=1
        c=SSHConsole(member) if route=='ssh' else NativeConsole(guest if route=='guest' else [str(a.native_binary.resolve()),'--profile',str(a.native_profile.resolve())]);children.append(c)
        c.expect('[L]ogin [R]egister [Q]uit: ');c.send('r' if register else 'l');c.expect('Handle: ');c.send(handle)
        if register:
            c.password(password);c.password(password,'Confirm password (hidden): ');c.expect('Registration pending; sysop approval required.');return c
        c.password(password);c.expect('Press Enter to continue: ');c.send('');c.expect('Command (then Enter): ' if plain else MENU);return c
    def logout(c,plain=False):
        c.send('q') if plain else c.raw(b'q');c.expect('Logged out.');c.finish()
    def status(c,handle,action):
        c.raw(b's');c.expect('[C]reate room: ');c.send(action);c.expect('Member handle: ');c.send(handle);c.expect('Status updated; previous sessions expired.');action_end(c,shared=True)
    try:
        assert count()==0,'requires fresh synthetic board'
        c=connect();c.raw(b'r');c.expect(BROWSE);c.raw(b'a');c.expect(CARD);c.raw(b'p');c.expect('Reply to post ID (blank for new message): ');c.send('');c.expect('Subject (blank for generated/reply default): ');c.send('Selected room');c.expect('Message (one line, up to 1024 bytes): ');c.send('/edit');c.expect('Writing desk | room #1');c.expect(PROMPT);command(c,'Stay in the selected room.');c.send('.cancel');c.expect('Draft cancelled');action_end(c,shared=True);assert count()==0;checks.append('actual_selected_room_card_enters_writing_desk_without_room_id')
        checks+=['actual_ssh_'+s for s in exercise(c,count,shared=True)];logout(c)
        assert query('select body from posts where id=1')[0][0]=='First signal.\n\nSecond signal.'
        # The native device keeps the same terminal controls and Unicode path.
        for route in ['native','guest']:
            c=connect(route);c.raw(b't');c.expect('[D]one: ');c.send('u');c.expect('[D]one: ');c.send('d');c.expect(MENU)
            begin(c,subject='Échos 界');command(c,'Café 界');command(c,'Native paragraph.');command(c,'.preview');assert 'Café 界'.encode() in c.transcript
            c.send('.send');c.expect('Posted #');action_end(c,shared=True);logout(c);checks.append('actual_'+route+'_unicode_multiline_preview_and_send')
        c=connect();begin(c);command(c,'Keep completed lines.');c.resize(40,18);c.expect('Terminal resized; completed draft lines kept');c.expect(PROMPT);command(c,'.preview');assert b'Keep completed lines.' in c.transcript;c.send('.cancel');c.expect('Draft cancelled');action_end(c,shared=True);logout(c);checks.append('shared_resize_preserves_completed_lines_and_classic_geometry')
        before=count();c=connect();begin(c);command(c,'Lost draft.');c.close();children.remove(c);c=connect();begin(c);c.send('.send');c.expect('empty draft');c.expect(PROMPT);c.send('.cancel');c.expect('Draft cancelled');action_end(c,shared=True);assert count()==before;checks.append('actual_disconnect_reconnect_does_not_restore_a_draft')
        # A lock on this synthetic board forces the existing same-request retry.
        begin(c);command(c,'One explicit send, one durable post.')
        code='import sqlite3,sys; d=sqlite3.connect('+repr(database)+'); d.execute("BEGIN IMMEDIATE"); print("ready",flush=True); sys.stdin.readline(); d.rollback()'
        lock=subprocess.Popen(admin+['test -f /etc/faction-disposable-test && python3.13 -c '+shlex.quote(code)],stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=subprocess.PIPE)
        try:
            assert lock.stdout.readline()==b'ready\n';c.send('.send');c.expect('Database busy. Retry this same submission? [y/N]: ')
        finally:
            lock.communicate(b'finish\n',timeout=10)
        c.send('y');c.expect('Posted #');action_end(c,shared=True);assert count()==before+1;checks.append('database_busy_retry_keeps_one_explicit_submission')
        logout(c)
        handle='writer-'+secrets.token_hex(3);pending=connect(handle=handle,register=True);pending.finish();checks.append('pending_member_has_no_writing_desk')
        sysop=connect();status(sysop,handle,'a');writer=connect(handle=handle);begin(writer);command(writer,'Revoked before send.');before=count();status(sysop,handle,'v');writer.send('.send')
        # Rejected member must close or return to admission, never create the post.
        writer.expect('session');assert count()==before;logout(sysop);checks.append('revocation_prevents_sending_an_existing_draft')
        remote('/etc/rc.d/faction_broker onestop')
        remote('cat > /etc/faction/service.toml',original.replace('ansi=true','ansi=false').encode())
        remote('/etc/rc.d/faction_broker onestart')
        c=connect(plain=True);checks+=['actual_shared_plain_'+s for s in exercise(c,count,plain=True,shared=True)];logout(c,True);assert b'\x1b' not in c.transcript
        remote('/etc/rc.d/faction_broker onestop');remote('cat > /etc/faction/service.toml',original.encode());remote('/etc/rc.d/faction_broker onestart')
        assert query('pragma user_version')==[[10]]
        assert query('select count(*) from unread')==[[0]],'preview/quote advanced unread state'
        assert all(password.encode() not in c.transcript for c in children)
    finally:
        for c in children:c.close()
        remote('/etc/rc.d/faction_broker onestop || true');remote('cat > /etc/faction/service.toml',original.encode())
    result=dict(result='pass',count=len(checks),checks=checks,normal_sessions=8,admission_refill_ms=5000,capacity_claim=False,persistent_drafts=False,schema_change=False)
    a.output.write_text(json.dumps(result,indent=2,sort_keys=True)+'\n');print('Actual SSH/native writing desk passed',len(checks),'controls.')

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__)
    for n in ['admin-argv','member-argv','password-file','native-binary','native-profile','guest-native-argv','output']:p.add_argument('--'+n,type=Path,required=True)
    p.add_argument('--disposable-fixture',action='store_true',required=True);run(p.parse_args())
