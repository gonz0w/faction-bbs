#!/usr/bin/env python3
"""Actual SSH/native member pages on an explicitly marked disposable NetBSD board."""
import argparse,json,secrets,shlex,subprocess,time,tomllib,select
from pathlib import Path
from check_restricted_ssh import SSHConsole
from smoke_native_client_ssh import NativeConsole
from smoke_room_browser import MENU
from smoke_member_pages import SETTINGS,PAGES,EDIT,CARD,TEXT,REVIEW,settings,setting,enter,review_field,save

def run(a):
    assert a.disposable_fixture
    admin=json.loads(a.admin_argv.read_text());member=json.loads(a.member_argv.read_text());guest=json.loads(a.guest_native_argv.read_text());password=a.password_file.read_text().strip();children=[];checks=[];calls=0
    def remote(cmd,data=None):
        x=subprocess.run(admin+['set -e; PATH=/usr/sbin:/sbin:/usr/bin:/bin:/usr/pkg/bin; export PATH; test -f /etc/faction-disposable-test; '+cmd],input=data,capture_output=True,timeout=30)
        if x.returncode:(a.output.parent/'pages-private-error.log').write_bytes(x.stdout+x.stderr)
        assert x.returncode==0,'owned page fixture operation failed';return x.stdout.decode()
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
    def edit(c,plain=False):enter(c,plain);c.send('e');c.expect(EDIT)
    def end_edit(c,plain=False):c.send('q');c.expect(PAGES);back(c,plain)
    def configure(text):remote('/etc/rc.d/faction_broker onestop');remote('cat > /etc/faction/service.toml',text.encode());remote('/etc/rc.d/faction_broker onestart')
    def status(c,handle,key):c.raw(b's');c.expect('[C]reate room: ');c.send(key);c.expect('Member handle: ');c.send(handle);c.expect('Status updated; previous sessions expired.');c.expect('Press Enter to return to menu: ');c.send('');c.expect(MENU)
    def find(c,handle):c.send('f');c.expect('Member handle: ');c.send(handle);c.expect(CARD)
    try:
        assert query('select count(*) from member_pages')==[[0]]
        c=connect();settings(c);c.send('e');c.expect(SETTINGS);setting(c,'n','Page label (1-24 ASCII): ','Nightwire people');setting(c,'h','Plan history (1-16) and days (7-365): ','2 7');back(c);edit(c);review_field(c,1,'Local sysop bio.');assert query('select count(*) from member_pages')==[[0]];save(c);review_field(c,4,'SECRET project hidden from members.');save(c);c.send('v');c.expect('Shared fields (1-4, all/none)');c.expect(': ');c.send('1');c.expect(REVIEW);save(c);end_edit(c);logout(c);checks.append('actual_ssh_named_pages_field_preview_save_and_explicit_visibility')
        c=connect('native');edit(c);c.send('p');c.expect(TEXT);c.send('A public dated device update.');c.expect(TEXT);c.send('.');c.expect(REVIEW);save(c);end_edit(c);logout(c);checks.append('actual_host_native_dated_plan_preview_and_save')
        c=connect('guest');edit(c);review_field(c,3,' /\\\n/  \\');save(c);c.send('p');c.expect(TEXT);c.send('SECRET hidden plan update.');c.expect(TEXT);c.send('.');c.expect(REVIEW);c.send('h');c.expect('hidden');c.expect(REVIEW);save(c);end_edit(c);logout(c);checks.append('actual_netbsd_native_original_ascii_and_hidden_plan')
        configure(original.replace('ansi=true','ansi=false'));c=connect(plain=True);enter(c,True);c.send('v');c.expect('Local sysop bio.');c.expect(CARD);c.send('q');c.expect(PAGES);back(c,True);logout(c,True);assert b'\x1b' not in c.transcript;configure(original);checks.append('actual_plain_profile_view_has_no_terminal_controls')
        handle='pages-'+secrets.token_hex(3);pending=connect(handle=handle,register=True);pending.finish();sysop=connect();status(sysop,handle,'a');visitor=connect(handle=handle);enter(visitor);mark=len(visitor.transcript);find(visitor,'sysop')
        for _ in range(5):visitor.send('n');visitor.expect(CARD)
        assert b'SECRET' not in visitor.transcript[mark:] and b'A public dated device update.' in visitor.transcript[mark:];checks.append('actual_other_member_cannot_view_hidden_fields_or_hidden_plan')
        edit(sysop);sysop.send('h');sysop.expect(REVIEW);save(sysop);end_edit(sysop);mark=len(visitor.transcript);visitor.send('n');visitor.expect('not found');visitor.expect(PAGES);assert b'SECRET' not in visitor.transcript[mark:];edit(sysop);sysop.send('h');sysop.expect(REVIEW);save(sysop);end_edit(sysop);checks.append('actual_page_hide_invalidates_already_open_view_on_next_action')
        first=sysop;second=connect();edit(first);edit(second);review_field(first,1,'New revision wins.');review_field(second,1,'Stale revision refused.');save(first);second.send('s');second.expect('Page changed; cancel and reopen before saving.');second.expect(REVIEW);assert query('select bio from member_pages where member_id=1')==[['New revision wins.']];second.send('c');second.expect(EDIT);end_edit(second);logout(second);checks.append('actual_concurrent_page_edits_refuse_stale_revision')
        first.send('p');first.expect(TEXT);first.send('Only once after busy.');first.expect(TEXT);first.send('.');first.expect(REVIEW);code='import sqlite3,sys; d=sqlite3.connect('+repr(db)+'); d.execute("BEGIN IMMEDIATE"); print("ready",flush=True); sys.stdin.readline(); d.rollback()';lock=subprocess.Popen(admin+['test -f /etc/faction-disposable-test && python3.13 -c '+shlex.quote(code)],stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=subprocess.PIPE)
        try:
            assert select.select([lock.stdout],[],[],15)[0] and lock.stdout.readline()==b'ready\n';first.send('s');first.expect('database busy');first.expect(REVIEW)
        finally:lock.communicate(b'finish\n',timeout=10)
        save(first);assert query("select count(*) from member_plans where body='Only once after busy.'")==[[1]];assert query('select count(*) from member_plans')==[[2]];checks.append('actual_busy_plan_retry_is_single_and_history_prunes_to_policy')
        first.send('1');first.expect(TEXT);first.send('Resize keeps completed text.');first.expect(TEXT);first.resize(40,18);first.expect(TEXT);first.send('.');first.expect(REVIEW);save(first);first.resize(80,24);first.expect(EDIT);end_edit(first);checks.append('actual_shared_resize_preserves_completed_page_text')
        visitor.send('e');visitor.expect(EDIT);review_field(visitor,1,'Member-owned hello.');save(visitor);visitor.send('q');visitor.expect(PAGES);back(visitor);enter(first);first.send('s');first.expect('Moderate: ');first.send(handle+' hide');first.expect('Moderation applied');first.expect(PAGES)
        # Hidden page is not listed/viewable, but author can still edit it without un-hiding it.
        edit(visitor);review_field(visitor,1,'Author revision while hidden.');save(visitor);end_edit(visitor);assert query("select moderated from member_pages p join members m on m.id=p.member_id where m.handle='"+handle+"'")==[[1]];edit(visitor);visitor.send('d');visitor.expect('Type DELETE: ');visitor.send('DELETE');visitor.expect(PAGES);visitor.send('e');visitor.expect(EDIT);review_field(visitor,1,'Recreated while moderated.');save(visitor);visitor.send('h');visitor.expect(REVIEW);save(visitor);end_edit(visitor);assert query("select moderated,published from member_pages p join members m on m.id=p.member_id where m.handle='"+handle+"'")==[[1,1]];checks.append('actual_own_content_delete_recreate_cannot_clear_sysop_hide');first.send('s');first.expect('Moderate: ');first.send(handle+' show');first.expect(PAGES);back(first);checks.append('actual_sysop_hide_preserves_authorship_and_author_edit_does_not_unhide')
        edit(visitor);review_field(visitor,4,'Revoked before save.');status(first,handle,'v');visitor.send('s');visitor.expect('session expired');visitor.finish(1);enter(first);first.send('f');first.expect('Member handle: ');first.send(handle);first.expect('not found');first.expect('Member handle: ');first.send('');first.expect(PAGES);back(first);assert query("select project from member_pages p join members m on m.id=p.member_id where m.handle='"+handle+"'")==[['']];checks.append('actual_active_revocation_hides_author_and_prevents_pending_save')
        rogue=connect();edit(rogue);rogue.send('1');rogue.expect(TEXT);rogue.raw(b'\x1b[2J');rogue.finish(1);assert query('select bio from member_pages where member_id=1')==[['Resize keeps completed text.']];checks.append('actual_escape_input_cannot_modify_page_or_execute_controls')
        settings(first);first.send('l');first.expect(SETTINGS);back(first);enter(first);first.send('v');first.expect('sysop | local page');first.expect(CARD);first.send('q');first.expect(PAGES);first.send('e');first.expect(EDIT);first.send('d');first.expect('Type DELETE: ');first.send('DELETE');first.expect('Page and retained plans removed.');first.expect(PAGES);assert query('select count(*) from member_pages where member_id=1')==[[0]] and query('select count(*) from member_plans where member_id=1')==[[0]];back(first);settings(first);first.send('d');first.expect(SETTINGS);back(first);logout(first);checks.append('actual_compact_layout_confirmed_page_history_removal_and_disable')
        assert query('pragma user_version')==[[19]] and query('select count(*) from unread')==[[0]] and all(password.encode() not in x.transcript for x in children)
        a.output.write_text(json.dumps(dict(result='pass',count=len(checks),checks=checks,schema_version=19,normal_sessions=8,admission_refill_ms=5000,capacity_claim=False),indent=2,sort_keys=True)+'\n');print('Actual shared member-page checks passed:',len(checks))
    except Exception:
        if children:(a.output.parent/'pages-private-transcript').write_bytes(children[-1].transcript)
        raise
    finally:
        for c in children:c.close()
        remote('/etc/rc.d/faction_broker onestop || true');remote('cat > /etc/faction/service.toml',original.encode())
if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__)
    for n in ['admin-argv','member-argv','password-file','native-binary','native-profile','guest-native-argv','output']:p.add_argument('--'+n,type=Path,required=True)
    p.add_argument('--disposable-fixture',action='store_true',required=True);run(p.parse_args())
