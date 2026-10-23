#!/usr/bin/env python3
"""Actual SSH/native gatherings on an explicitly marked disposable NetBSD board."""
import argparse,json,secrets,shlex,subprocess,time,tomllib,select
from pathlib import Path
from check_restricted_ssh import SSHConsole
from smoke_native_client_ssh import NativeConsole
from smoke_room_browser import MENU
from smoke_gatherings import SETTINGS,LIST,CARD,TEXT,REVIEW,settings,enter,view,create,rsvp

def run(a):
    assert a.disposable_fixture
    admin=json.loads(a.admin_argv.read_text());member=json.loads(a.member_argv.read_text());guest=json.loads(a.guest_native_argv.read_text());password=a.password_file.read_text().strip();children=[];checks=[];calls=0
    def remote(cmd,data=None):
        x=subprocess.run(admin+['set -e; PATH=/usr/sbin:/sbin:/usr/bin:/bin:/usr/pkg/bin; export PATH; test -f /etc/faction-disposable-test; '+cmd],input=data,capture_output=True,timeout=30)
        if x.returncode:(a.output.parent/'gatherings-private-error.log').write_bytes(x.stdout+x.stderr)
        assert x.returncode==0,'owned gathering fixture operation failed';return x.stdout.decode()
    original=remote('cat /etc/faction/service.toml');assert tomllib.loads(original)['state_dir']=='/var/db/faction-profile-fixture';db='/var/db/faction-profile-fixture/faction.sqlite3'
    def query(sql):
        code='import sqlite3,json; d=sqlite3.connect('+repr('file:'+db+'?mode=ro')+',uri=True); print(json.dumps(d.execute('+repr(sql)+').fetchall()))';return json.loads(remote('python3.13 -c '+shlex.quote(code)))
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
    def status(c,handle,key):c.raw(b's');c.expect('[C]reate room: ');c.send(key);c.expect('Member handle: ');c.send(handle);c.expect('Status updated; previous sessions expired.');c.expect('Press Enter to return to menu: ');c.send('');c.expect(MENU)
    def edit(c,title):
        c.send('e');c.expect('Title (1-64 ASCII, blank keeps): ');c.send(title)
        c.expect('Start UTC YYYY-MM-DD HH:MM (blank keeps): ');c.send('')
        c.expect('Duration 1-1440 minutes (blank keeps): ');c.send('')
        c.expect("Place: room:ID, #channel, '-' none, blank keeps: ");c.send('')
        c.expect(TEXT);c.send('.');c.expect(REVIEW)
    def saved(c):c.send('s');c.expect('Gathering saved locally.');c.expect(CARD)
    def main(c):c.send('q');c.expect(LIST);back(c)
    try:
        # Reset only the explicitly marked, fixed synthetic profile fixture, stopped.
        remote('/etc/rc.d/faction_broker onestop')
        code="import sqlite3; d=sqlite3.connect("+repr(db)+"); d.execute('PRAGMA foreign_keys=ON'); d.execute('DELETE FROM gatherings'); d.execute(\"DELETE FROM sqlite_sequence WHERE name=\\'gatherings\\'\"); d.execute(\"UPDATE gathering_settings SET enabled=0,name=\\'Gatherings\\',compact=0,member_organizers=0,reminders=0,public_counts=1\"); d.commit()"
        remote('python3.13 -c '+shlex.quote(code))
        remote('/etc/rc.d/faction_broker onestart')
        assert query('select count(*) from gatherings')==[[0]]
        c=connect();settings(c)
        for key in ['e','o','r','v']:c.send(key);c.expect(SETTINGS)
        back(c);enter(c);when=query("select strftime('%Y-%m-%d %H:%M','now','+1 hour')")[0][0]
        create(c,when,place='#lobby');assert query('select count(*) from gatherings')==[[0]];saved(c);main(c);logout(c)
        checks.append('actual_ssh_sysop_policy_utc_notes_preview_and_save')
        c=connect('native');assert b'Open Gatherings from the menu.' in c.transcript;enter(c);view(c);rsvp(c,1);main(c);logout(c)
        checks.append('actual_host_native_reminder_and_explicit_rsvp')
        c=connect('guest');enter(c);view(c);c.send('j');c.expect('chat[#lobby]> ');c.send('/quit');c.expect(MENU);logout(c)
        checks.append('actual_netbsd_native_event_channel_jump_and_return')
        configure(original.replace('ansi=true','ansi=false'));c=connect(plain=True);enter(c,True);view(c);rsvp(c,2);c.send('q');c.expect(LIST);back(c,True);logout(c,True);assert b'\x1b' not in c.transcript;configure(original)
        checks.append('actual_plain_reminder_and_rsvp_has_no_controls')
        handle='gather-'+secrets.token_hex(3);pending=connect(handle=handle,register=True);pending.finish();sysop=connect();status(sysop,handle,'a');visitor=connect(handle=handle);enter(visitor);mark=len(visitor.transcript);view(visitor);assert b'Attendance counts are organizer-only.' in visitor.transcript[mark:] and b'Going:' not in visitor.transcript[mark:];rsvp(visitor,1);enter(sysop);view(sysop);assert handle.encode() not in sysop.transcript[sysop.transcript.rfind(b'#1 /'):];main(sysop)
        checks.append('actual_two_member_rsvp_counts_policy_and_no_attendee_names')
        second=connect();enter(sysop);view(sysop);enter(second);view(second);edit(sysop,'New revision wins.');edit(second,'Stale revision refused.');saved(sysop);second.send('s');second.expect('Event changed; reopen before saving.');second.expect(REVIEW);assert query('select title from gatherings')==[['New revision wins.']];second.send('c');second.expect(CARD);main(second);logout(second)
        checks.append('actual_concurrent_edits_refuse_stale_event_revision')
        edit(sysop,'Only once after busy.');code='import sqlite3,sys; d=sqlite3.connect('+repr(db)+'); d.execute("BEGIN IMMEDIATE"); print("ready",flush=True); sys.stdin.readline(); d.rollback()';lock=subprocess.Popen(admin+['test -f /etc/faction-disposable-test && python3.13 -c '+shlex.quote(code)],stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=subprocess.PIPE)
        try:
            assert select.select([lock.stdout],[],[],15)[0] and lock.stdout.readline()==b'ready\n';sysop.send('s');sysop.expect('database busy');sysop.expect(REVIEW)
        finally:lock.communicate(b'finish\n',timeout=10)
        saved(sysop);assert query('select title from gatherings')==[['Only once after busy.']];main(sysop)
        checks.append('actual_busy_save_retains_edit_for_single_retry')
        enter(sysop);view(sysop);sysop.send('e');sysop.expect('Title (1-64 ASCII, blank keeps): ');sysop.send('Resize preserves completed edit.');sysop.expect('Start UTC YYYY-MM-DD HH:MM (blank keeps): ');sysop.resize(40,18);sysop.expect('Start UTC YYYY-MM-DD HH:MM (blank keeps): ');sysop.send('');sysop.expect('Duration 1-1440 minutes (blank keeps): ');sysop.send('');sysop.expect("Place: room:ID, #channel, '-' none, blank keeps: ");sysop.send('');sysop.expect(TEXT);sysop.send('.');sysop.expect(REVIEW);saved(sysop);sysop.resize(80,24);sysop.expect(CARD);main(sysop)
        checks.append('actual_shared_resize_preserves_completed_event_fields')
        settings(sysop);sysop.send('m');sysop.expect('Gathering ID and hide|show|remove: ');sysop.send('1 hide');sysop.expect(SETTINGS);mark=len(visitor.transcript);visitor.send('n');visitor.expect('not found');visitor.expect(LIST);assert b'Bring your ANSI' not in visitor.transcript[mark:];sysop.send('m');sysop.expect('Gathering ID and hide|show|remove: ');sysop.send('1 show');sysop.expect(SETTINGS);back(sysop)
        checks.append('actual_hide_invalidates_open_foreign_card')
        view(visitor);visitor.send('r');visitor.expect('RSVP: 1 going / 2 maybe / 0 clear: ');visitor.send('2');visitor.expect('RSVP review> ');status(sysop,handle,'v');visitor.send('s');visitor.expect('session expired');visitor.finish(1);assert query("select response from gathering_rsvps r join members m on m.id=r.member where m.handle='"+handle+"'")==[[1]]
        checks.append('actual_active_revocation_prevents_reviewed_rsvp_save')
        enter(sysop);view(sysop);sysop.send('c');sysop.expect('Type CANCEL (anything else returns): ');sysop.send('CANCEL');sysop.expect(CARD);sysop.send('j');sysop.expect('event is cancelled, hidden or ended');sysop.expect(CARD);main(sysop);settings(sysop);sysop.send('m');sysop.expect('Gathering ID and hide|show|remove: ');sysop.send('1 remove');sysop.expect('Type DELETE (anything else cancels): ');sysop.send('DELETE');sysop.expect(SETTINGS);assert query('select count(*) from gathering_rsvps')==[[0]];sysop.send('e');sysop.expect(SETTINGS);back(sysop);logout(sysop)
        checks.append('actual_cancel_blocks_jump_and_explicit_remove_cascades_responses')
        assert query('pragma user_version')==[[16]] and query('select count(*) from unread')==[[0]] and all(password.encode() not in x.transcript for x in children)
        a.output.write_text(json.dumps(dict(result='pass',count=len(checks),checks=checks,schema_version=16,normal_sessions=8,admission_refill_ms=5000,capacity_claim=False),indent=2,sort_keys=True)+'\n');print('Actual shared gathering checks passed:',len(checks))
    except Exception:
        for i,c in enumerate(children):(a.output.parent/('gatherings-private-transcript-'+str(i))).write_bytes(c.transcript)
        raise
    finally:
        for c in children:c.close()
        remote('/etc/rc.d/faction_broker onestop || true');remote('cat > /etc/faction/service.toml',original.encode())
if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__)
    for n in ['admin-argv','member-argv','password-file','native-binary','native-profile','guest-native-argv','output']:p.add_argument('--'+n,type=Path,required=True)
    p.add_argument('--disposable-fixture',action='store_true',required=True);run(p.parse_args())
