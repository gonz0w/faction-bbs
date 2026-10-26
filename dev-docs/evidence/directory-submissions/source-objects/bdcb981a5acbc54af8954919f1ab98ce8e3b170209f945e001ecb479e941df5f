#!/usr/bin/env python3
"""Actual SSH/native polls on an explicitly marked disposable NetBSD board."""
import argparse,json,secrets,shlex,subprocess,time,tomllib,select
from pathlib import Path
from check_restricted_ssh import SSHConsole
from smoke_native_client_ssh import NativeConsole
from smoke_room_browser import MENU
from smoke_polls import SETTINGS,LIST,CARD,TEXT,REVIEW,settings,enter,view,create,vote

def run(a):
    assert a.disposable_fixture
    admin=json.loads(a.admin_argv.read_text());member=json.loads(a.member_argv.read_text());guest=json.loads(a.guest_native_argv.read_text());password=a.password_file.read_text().strip();children=[];checks=[];calls=0
    def remote(cmd,data=None):
        x=subprocess.run(admin+['set -e; PATH=/usr/sbin:/sbin:/usr/bin:/bin:/usr/pkg/bin; export PATH; test -f /etc/faction-disposable-test; '+cmd],input=data,capture_output=True,timeout=30)
        if x.returncode:(a.output.parent/'polls-private-error.log').write_bytes(x.stdout+x.stderr)
        assert x.returncode==0,'owned poll fixture operation failed';return x.stdout.decode()
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
    def saved(c):c.send('s');c.expect('Poll saved locally.');c.expect(CARD)
    def main(c):c.send('q');c.expect(LIST);back(c)
    def edit(c,question):
        c.send('e');c.expect('Question (1-96 ASCII, blank keeps): ');c.send(question);c.expect("Choice (1-48 ASCII), '.' finishes: ");c.send('.');c.expect('Results A:always V:after vote C:after close (blank keeps): ');c.send('');c.expect(TEXT);c.send('.');c.expect(REVIEW)
    def moderate(c,action,poll=2):
        settings(c);c.send('m');c.expect('Poll ID and hide|show|remove: ');c.send(str(poll)+' '+action);c.expect(SETTINGS);back(c)
    try:
        # Reset only the explicitly marked and fixed synthetic fixture, stopped.
        remote('/etc/rc.d/faction_broker onestop')
        code="import sqlite3;d=sqlite3.connect("+repr(db)+");d.execute('PRAGMA foreign_keys=ON');d.execute('DELETE FROM polls');d.execute(\"DELETE FROM sqlite_sequence WHERE name='polls'\");d.execute(\"UPDATE poll_settings SET enabled=0,name='Polls',compact=0,member_creators=0\");d.commit()"
        remote('python3.13 -c '+shlex.quote(code));remote('/etc/rc.d/faction_broker onestart')
        first=connect();settings(first);first.send('e');first.expect(SETTINGS);first.send('o');first.expect(SETTINGS);first.send('n');first.expect('Poll label (1-24 ASCII): ');first.send('Nightwire ballots');first.expect(SETTINGS);back(first);enter(first);create(first,'c');assert query('select count(*) from polls')==[[0]];saved(first);vote(first,1);assert b'Results are not available.' in first.transcript;main(first);logout(first)
        checks.append('actual_ssh_named_policy_creation_preview_and_after_close_ballot')
        c=connect('native');enter(c);view(c);vote(c,1);vote(c,2);assert query('select count(*) from poll_ballots where poll=1')==[[1]];main(c);logout(c)
        checks.append('actual_host_native_repeated_vote_change_is_one_ballot')
        c=connect('guest');enter(c);view(c);c.send('n');c.expect('3: Sunday');c.expect(CARD);c.send('n');c.expect('3: Sunday');c.expect(CARD);c.send('p');c.expect('1: Friday');c.expect(CARD);c.send('d');c.expect('Bring your terminal.');c.expect('Poll notes> ');c.send('q');c.expect(CARD);main(c);logout(c)
        checks.append('actual_netbsd_native_paged_choices_and_notes')
        configure(original.replace('ansi=true','ansi=false'));c=connect(plain=True);enter(c,True);view(c);vote(c,2);c.send('q');c.expect(LIST);back(c,True);logout(c,True);assert b'\x1b' not in c.transcript;configure(original)
        checks.append('actual_plain_ballot_has_no_ansi_and_keeps_hidden_results')
        handle='voter-'+secrets.token_hex(3);pending=connect(handle=handle,register=True);pending.finish();first=connect();status(first,handle,'a');visitor=connect(handle=handle);enter(visitor);view(visitor);mark=len(visitor.transcript);vote(visitor,1);assert b'Results are not available.' in visitor.transcript[mark:] and b'1/2' not in visitor.transcript[mark:];enter(first);view(first);first.send('f');first.expect('Type CLOSE (anything else returns): ');first.send('CLOSE');first.expect(CARD);mark=len(visitor.transcript);visitor.send('p');visitor.expect('1/2');visitor.expect(CARD);assert handle.encode() not in visitor.transcript[mark:];main(first)
        checks.append('actual_two_members_after_close_visibility_without_voter_names')
        enter(first);create(first);saved(first);second=connect();enter(second);view(second,2);edit(first,'New wording wins.');edit(second,'Stale wording refused.');saved(first);second.send('s');second.expect('Poll changed; reopen before saving.');second.expect(REVIEW);assert query('select question from polls where id=2')==[['New wording wins.']];second.send('c');second.expect(CARD);main(second);logout(second)
        checks.append('actual_concurrent_author_edits_reject_stale_revision')
        main(visitor);enter(visitor);view(visitor,2);visitor.send('v');visitor.expect('Choice number (0 withdraws): ');visitor.send('1');visitor.expect('Ballot review> ');code='import sqlite3,sys;d=sqlite3.connect('+repr(db)+');d.execute("BEGIN IMMEDIATE");print("ready",flush=True);sys.stdin.readline();d.rollback()';lock=subprocess.Popen(admin+['test -f /etc/faction-disposable-test && python3.13 -c '+shlex.quote(code)],stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=subprocess.PIPE)
        try:
            assert select.select([lock.stdout],[],[],15)[0] and lock.stdout.readline()==b'ready\n';visitor.send('s');visitor.expect('database busy');visitor.expect('Ballot review> ')
        finally:lock.communicate(b'finish\n',timeout=10)
        visitor.send('s');visitor.expect(CARD);assert query('select count(*) from poll_ballots where poll=2')==[[1]];main(first)
        checks.append('actual_busy_review_stays_readable_and_retry_saves_one_ballot')
        enter(first);create(first);first.send('e');first.expect('Question (1-96 ASCII, blank keeps): ');first.send('Resize keeps completed notes.');first.expect("Choice (1-48 ASCII), '.' finishes: ");first.send('.');first.expect('Results A:always V:after vote C:after close (blank keeps): ');first.send('');first.expect(TEXT);first.resize(40,18);first.expect(TEXT);first.send('.');first.expect(REVIEW);saved(first);first.resize(80,24);first.expect(CARD);assert query('select description from polls where id=3')==[['Bring your terminal.']];main(first)
        checks.append('actual_resize_preserves_completed_unsaved_poll_notes')
        moderate(first,'hide');mark=len(visitor.transcript);visitor.send('n');visitor.expect('not found');visitor.expect(LIST);assert b'New wording wins.' not in visitor.transcript[mark:];moderate(first,'show')
        checks.append('actual_moderation_hide_invalidates_open_foreign_card')
        view(visitor,3);visitor.send('v');visitor.expect('Choice number (0 withdraws): ');visitor.send('2');visitor.expect('Ballot review> ');status(first,handle,'v');visitor.send('s');visitor.expect('session expired');visitor.finish(1);assert query('select count(*) from poll_ballots where poll=3')==[[0]]
        checks.append('actual_active_revocation_prevents_reviewed_ballot')
        second=connect();enter(second);view(second,3);second.send('v');second.expect('Choice number (0 withdraws): ');second.send('1');second.expect('Ballot review> ');enter(first);view(first,3);first.send('f');first.expect('Type CLOSE (anything else returns): ');first.send('CLOSE');first.expect(CARD);second.send('s');second.expect('Poll changed; reopen before saving.');second.expect(CARD);assert query('select count(*) from poll_ballots where poll=3')==[[0]];main(second);logout(second);main(first)
        checks.append('actual_close_race_refuses_stale_review_without_changing_ballots')
        enter(first);view(first,2);first.send('c');first.expect('Type CANCEL (anything else returns): ');first.send('CANCEL');first.expect(CARD);main(first);settings(first);first.send('m');first.expect('Poll ID and hide|show|remove: ');first.send('1 remove');first.expect('Type DELETE (anything else cancels): ');first.send('no');first.expect(SETTINGS);assert query('select count(*) from polls where id=1')==[[1]];first.send('m');first.expect('Poll ID and hide|show|remove: ');first.send('1 remove');first.expect('Type DELETE (anything else cancels): ');first.send('DELETE');first.expect(SETTINGS);assert query('select count(*) from poll_ballots where poll=1')==[[0]];first.send('e');first.expect(SETTINGS);back(first);logout(first)
        checks.append('actual_cancel_and_explicit_delete_cascade_then_disable')
        assert query('pragma user_version')==[[19]] and query('select count(*) from unread')==[[0]] and all(password.encode() not in c.transcript for c in children)
        a.output.write_text(json.dumps(dict(result='pass',count=len(checks),checks=checks,schema_version=19,normal_sessions=8,admission_refill_ms=5000,capacity_claim=False,secret_ballot_claim=False),indent=2,sort_keys=True)+'\n');print('Actual shared polls passed:',len(checks),'controls.')
    except Exception:
        for i,c in enumerate(children):(a.output.parent/('polls-private-transcript-'+str(i))).write_bytes(c.transcript)
        raise
    finally:
        for c in children:c.close()
        remote('/etc/rc.d/faction_broker onestop || true');remote('cat > /etc/faction/service.toml',original.encode())
if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__)
    for n in ['admin-argv','member-argv','password-file','native-binary','native-profile','guest-native-argv','output']:p.add_argument('--'+n,type=Path,required=True)
    p.add_argument('--disposable-fixture',action='store_true',required=True);run(p.parse_args())
