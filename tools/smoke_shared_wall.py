#!/usr/bin/env python3
"""Real SSH/native community wall on a marked private disposable NetBSD fixture."""
import argparse,json,secrets,shlex,subprocess,time,tomllib,select
from pathlib import Path
from check_restricted_ssh import SSHConsole
from smoke_native_client_ssh import NativeConsole
from smoke_room_browser import MENU
from smoke_wall import SETTINGS,WALL,TEXT,REVIEW,settings,setting,wall,draft,send

def run(a):
    assert a.disposable_fixture
    admin=json.loads(a.admin_argv.read_text());member=json.loads(a.member_argv.read_text());guest=json.loads(a.guest_native_argv.read_text());password=a.password_file.read_text().strip();children=[];checks=[];calls=0
    def remote(cmd,data=None):
        x=subprocess.run(admin+['set -e; PATH=/usr/sbin:/sbin:/usr/bin:/bin:/usr/pkg/bin; export PATH; test -f /etc/faction-disposable-test; '+cmd],input=data,capture_output=True,timeout=30)
        if x.returncode:(a.output.parent/'wall-private-error.log').write_bytes(x.stdout+x.stderr)
        assert x.returncode==0,'owned wall fixture operation failed';return x.stdout.decode()
    original=remote('cat /etc/faction/service.toml');assert tomllib.loads(original)['state_dir']=='/var/db/faction-profile-fixture';db='/var/db/faction-profile-fixture/faction.sqlite3'
    def query(sql):
        code='import sqlite3,json; d=sqlite3.connect('+repr('file:'+db+'?mode=ro')+',uri=True); print(json.dumps(d.execute('+repr(sql)+').fetchall()))'
        return json.loads(remote('python3.13 -c '+shlex.quote(code)))
    def mutate(sql):
        code='import sqlite3; d=sqlite3.connect('+repr(db)+'); d.execute('+repr(sql)+'); d.commit()';remote('python3.13 -c '+shlex.quote(code))
    def age():mutate('UPDATE wall_receipts SET last_at=unixepoch()-3601')
    def connect(route='ssh',plain=False,handle='sysop',register=False):
        nonlocal calls
        if calls:time.sleep(5.1)
        calls+=1;c=SSHConsole(member) if route=='ssh' else NativeConsole(guest if route=='guest' else [str(a.native_binary.resolve()),'--profile',str(a.native_profile.resolve())]);children.append(c)
        c.expect('[L]ogin [R]egister [Q]uit: ');c.send('r' if register else 'l');c.expect('Handle: ');c.send(handle);c.password(password)
        if register:c.password(password,'Confirm password (hidden): ');c.expect('Registration pending; sysop approval required.');return c
        c.expect('Press Enter to continue: ');c.send('');c.expect('Command (then Enter): ' if plain else MENU);return c
    def logout(c,plain=False):c.send('q') if plain else c.raw(b'q');c.expect('Logged out.');c.finish()
    def back(c,plain=False):c.send('q');c.expect('Command (then Enter): ' if plain else MENU)
    def configure(text):remote('/etc/rc.d/faction_broker onestop');remote('cat > /etc/faction/service.toml',text.encode());remote('/etc/rc.d/faction_broker onestart')
    def status(c,handle,action):
        c.raw(b's');c.expect('[C]reate room: ');c.send(action);c.expect('Member handle: ');c.send(handle);c.expect('Status updated; previous sessions expired.');c.expect('Press Enter to return to menu: ');c.send('');c.expect(MENU)
    try:
        assert query('select count(*) from wall_notes')==[[0]]
        c=connect();settings(c);c.send('e');c.expect(SETTINGS);setting(c,'n','Wall name (1-24 ASCII): ','Night signals');setting(c,'l','Limits: notes bytes days cooldown: ','8 240 7 3600');back(c);wall(c);draft(c,'Real SSH signal.\n /\\\n/  \\');assert query('select count(*) from wall_notes')==[[0]];send(c);back(c);logout(c);checks.append('actual_ssh_named_wall_ascii_review_send_and_zero_implicit_posts')
        for route in ['native','guest']:
            age();c=connect(route);wall(c);draft(c,route+' device signal.');send(c);back(c);logout(c);checks.append('actual_'+route+'_native_client_wall_post')
        configure(original.replace('ansi=true','ansi=false'));age();c=connect(plain=True);wall(c,True);draft(c,'Plain terminal signal.');send(c);c.send('b');c.expect('guest device signal.');c.expect(WALL);back(c,True);logout(c,True);assert b'\x1b' not in c.transcript;checks.append('actual_plain_wall_chronological_paging_without_escapes');configure(original)
        c=connect();wall(c);draft(c,'Cooldown blocked.');before=query('select count(*) from wall_notes');c.send('s');c.expect('wall cooldown');c.expect(REVIEW);assert query('select count(*) from wall_notes')==before;c.send('c');c.expect(WALL);checks.append('actual_cooldown_preserves_review_without_duplicate')
        age();draft(c,'Only once after busy.');code='import sqlite3,sys; d=sqlite3.connect('+repr(db)+'); d.execute("BEGIN IMMEDIATE"); print("ready",flush=True); sys.stdin.readline(); d.rollback()'
        lock=subprocess.Popen(admin+['test -f /etc/faction-disposable-test && python3.13 -c '+shlex.quote(code)],stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=subprocess.PIPE)
        try:
            assert select.select([lock.stdout],[],[],15)[0] and lock.stdout.readline()==b'ready\n';c.send('s');c.expect('database busy');c.expect(REVIEW)
        finally:lock.communicate(b'finish\n',timeout=10)
        send(c);assert query('select count(*) from wall_notes')[0][0]==before[0][0]+1;checks.append('actual_busy_send_retains_request_for_exactly_one_retry')
        age();c.send('e');c.expect(TEXT);c.send('/undo');c.expect(TEXT);c.send('Edited at 40 columns.');c.expect(TEXT);c.resize(40,18);c.expect(TEXT);c.send('.');c.expect(REVIEW);send(c);assert query('select body,edited_at>0 from wall_notes order by id desc limit 1')==[['Edited at 40 columns.',1]];c.resize(80,24);c.expect(WALL);back(c);checks.append('actual_edit_and_resize_preserve_completed_text_with_compact_wall')
        settings(c);c.send('w');c.expect('Type YES: ');c.send('YES');c.expect(SETTINGS);setting(c,'a','Retain 0-12 weekly editions: ','1');back(c);age();wall(c);draft(c,'Explicit archived signal.');send(c);ed=query('select edition from wall_settings')[0][0];mutate('update wall_notes set edition='+str(ed-1));c.send('a');c.expect('Edition ID, or Enter for current: ');c.send(str(ed-1));c.expect('Explicit archived signal.');c.expect(WALL);c.send('e');c.expect('action denied');c.expect('Press Enter to return to menu: ');c.send('');c.expect(MENU)
        wall(c);c.send('a');c.expect('Edition ID, or Enter for current: ');c.send(str(ed-1));c.expect(WALL);c.send('x');c.expect('Note deleted');c.expect(WALL);assert query('select count(*) from wall_notes')==[[0]];back(c);checks.append('actual_weekly_archive_readonly_edit_and_explicit_delete')
        handle='wall-'+secrets.token_hex(3);pending=connect(handle=handle,register=True);pending.finish();status(c,handle,'a');writer=connect(handle=handle);wall(writer);draft(writer,'Revoked before send.');status(c,handle,'v');writer.send('s');writer.expect('session expired');writer.finish(1);assert query('select count(*) from wall_notes')==[[0]];checks.append('actual_pending_denial_and_active_revocation_prevent_wall_send')
        # Strict terminal input controls cannot become a wall post or graphics.
        attacker=connect();wall(attacker);attacker.send('p');attacker.expect(TEXT);attacker.raw(b'\x1b[2J');attacker.finish(1);assert query('select count(*) from wall_notes')==[[0]];checks.append('actual_escape_injection_cannot_publish_or_execute_terminal_controls')
        settings(c);c.send('d');c.expect(SETTINGS);back(c);assert query('select enabled from wall_settings')==[[0]];logout(c);checks.append('actual_sysop_disable_returns_to_classic_menu')
        assert query('pragma user_version')==[[15]] and all(password.encode() not in x.transcript for x in children)
        a.output.write_text(json.dumps(dict(result='pass',count=len(checks),checks=checks,schema_version=15,normal_sessions=8,admission_refill_ms=5000,capacity_claim=False),indent=2,sort_keys=True)+'\n');print('Actual shared community-wall checks passed:',len(checks))
    except Exception:
        if children:(a.output.parent/'wall-private-transcript').write_bytes(children[-1].transcript)
        raise
    finally:
        for c in children:c.close()
        remote('/etc/rc.d/faction_broker onestop || true');remote('cat > /etc/faction/service.toml',original.encode())
if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__)
    for n in ['admin-argv','member-argv','password-file','native-binary','native-profile','guest-native-argv','output']:p.add_argument('--'+n,type=Path,required=True)
    p.add_argument('--disposable-fixture',action='store_true',required=True);run(p.parse_args())
