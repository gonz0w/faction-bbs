#!/usr/bin/env python3
"""Actual private bundle check/restore and Rust dashboard on disposable data.

Non-root developer runs exercise the shared APIs with one fixture UID. Root runs
require a marked disposable NetBSD VM and distinct non-root owner/file UIDs, and
exercise the actual production backup CLI. Never touches a running/saved board.
"""
import argparse,fcntl,hashlib,json,os,pwd,pty,secrets,sqlite3,struct,subprocess,sys,tempfile,termios,time
from pathlib import Path
from types import SimpleNamespace
import backup_status
import file_area_backup as bundle
from smoke_local_console import Console


class OwnerConsole(Console):
    def __init__(self,binary,state,action,uid):
        self.pid,self.fd=pty.fork()
        if self.pid==0:
            fcntl.ioctl(0,termios.TIOCSWINSZ,struct.pack('HHHH',24,80,0,0))
            if os.geteuid()==0:
                os.setgroups([]);os.setgid(pwd.getpwuid(uid).pw_gid);os.setuid(uid)
            os.closerange(3,os.sysconf('SC_OPEN_MAX'));os.execv(str(binary),[str(binary),'--state-dir',str(state),action])
        self.buffer,self.transcript,self.done=b'',b'',False


def run(a):
    native=os.geteuid()==0
    if native:
        assert a.disposable_fixture and Path('/etc/faction-disposable-test').is_file()
        assert a.owner_uid and a.files_uid and a.owner_uid>0 and a.files_uid>0 and a.owner_uid!=a.files_uid
    owner=a.owner_uid if native else os.geteuid();files=a.files_uid if native else owner
    checks=[];password=secrets.token_hex(24)
    with tempfile.TemporaryDirectory(prefix='faction-bundle-check-') as name:
        root=Path(name);os.chown(root,owner,-1)
        state=root/'board';restored=root/'restored';restored_store=root/'restored-store'
        x=SimpleNamespace(board=state,store=root/'store',pidfile=root/'owner.pid',bundle=root/'bundle',owner_uid=owner,files_uid=files,faction=a.binary)
        c=OwnerConsole(a.binary,state,'init',owner)
        try:
            c.expect('Handle: ');c.send('sysop');c.password(password,'New password (12+ characters, hidden): ');c.password(password,'Confirm password (hidden): ');c.expect('Lobby created.');c.finish()
        finally:c.close()
        content=bytes(range(256))*1024;object_id='a'*32
        with sqlite3.connect(state/'faction.sqlite3') as db:
            db.execute("INSERT INTO file_areas(slug,title,private,quota) VALUES('uploads','Uploads',0,1048576)")
            db.execute("INSERT INTO file_objects VALUES(?,1,1,'fixture.bin','Synthetic test data',?,?, 'published')",(object_id,len(content),hashlib.sha256(content).hexdigest()))
            db.execute("UPDATE wall_settings SET enabled=1,name='Recovery wall'")
            db.execute("INSERT INTO wall_notes(edition,member_id,request_id,body,posted_at,edited_at) VALUES(0,1,?,'Wall survives complete recovery',?,0)",('b'*32,int(time.time())))
            db.execute("INSERT INTO wall_receipts VALUES(1,?,1,?)",('b'*32,int(time.time())))
            db.execute("UPDATE page_settings SET enabled=1,name='Recovery members'")
            db.execute("INSERT INTO member_pages VALUES(1,'Recovered bio','','','Recovered project',15,1,0,?,?)",('c'*32,int(time.time())))
            db.execute("INSERT INTO member_plans(member_id,request_id,body,visible,posted_at) VALUES(1,?,'Recovered hidden plan',0,?)",('d'*32,int(time.time())))
            start=(int(time.time())//60+60)*60
            db.execute("UPDATE gathering_settings SET enabled=1,name='Recovery gatherings'")
            db.execute("INSERT INTO gatherings(organizer,request_id,revision,title,body,starts_at,ends_at,place,cancelled,hidden,changed_at) VALUES(1,?,?,'Recovered gathering','Local event survives recovery',?,?,'room:1',0,0,?)",('e'*32,'f'*32,start,start+3600,int(time.time())))
            db.execute("INSERT INTO gathering_rsvps VALUES(1,1,2)")
        db.close()
        bundle.mkdir(x.store,files);blob=x.store/object_id;blob.write_bytes(content);blob.chmod(0o600);os.chown(blob,files,-1)
        x.pidfile.write_bytes(b'fixture\n');x.pidfile.chmod(0o600);os.chown(x.pidfile,owner,-1)
        def action(op,record=False,expected=True,values=x):
            if native:
                argv=[sys.executable,str(Path(bundle.__file__).resolve()),op,'--owner-uid',str(owner),'--board',str(values.board),'--store',str(values.store),'--pidfile',str(x.pidfile),'--bundle',str(values.bundle),'--faction',str(a.binary)]
                if op!='verify':argv.extend(['--files-uid',str(files)])
                if record:argv.append('--record-status')
                p=subprocess.run(argv,capture_output=True,timeout=45)
                assert (p.returncode==0)==expected,'actual file-area CLI outcome mismatch'
                assert password.encode() not in p.stdout+p.stderr
                return
            try:
                if op=='backup':bundle.backup(values)
                if op=='restore':bundle.restore(values)
                else:
                    m=bundle.verify(values)
                    if record:backup_status.record(state,owner,dict(catalog_bytes=m['catalog']['bytes'],object_bytes=sum(v['bytes'] for v in m['objects'].values()),objects=len(m['objects'])))
            except (ValueError,OSError,subprocess.CalledProcessError):
                if record:backup_status.record(state,owner,None)
                assert not expected,'bundle API unexpectedly failed';return
            assert expected,'bundle API unexpectedly succeeded'
        def data():
            p=subprocess.run([str(a.operator),'--state-dir',str(state),'--view','data','--plain','--once'],capture_output=True,timeout=5)
            # A root operator may read the owner-private telemetry; no member role grants it.
            assert p.returncode==0 and b'\x1b' not in p.stdout
            assert all(secret not in p.stdout for secret in [password.encode(),b'fixture.bin',b'argon2id'])
            return p.stdout
        with x.pidfile.open('rb') as held:
            fcntl.flock(held,fcntl.LOCK_EX|fcntl.LOCK_NB);action('backup',expected=False);assert not x.bundle.exists()
        checks.append('held_owner_lock_refuses_backup')
        action('backup');assert not (state/'backup.status').exists()
        assert (x.bundle/'objects'/object_id).read_bytes()==content
        checks.append('complete_backup_is_opt_in_and_preserves_file_bytes')
        before={str(f.relative_to(x.bundle)):hashlib.sha256(f.read_bytes()).hexdigest() for f in x.bundle.rglob('*') if f.is_file()}
        action('verify');assert not (state/'backup.status').exists()
        assert before=={str(f.relative_to(x.bundle)):hashlib.sha256(f.read_bytes()).hexdigest() for f in x.bundle.rglob('*') if f.is_file()}
        assert not restored.exists() and not restored_store.exists()
        checks.append('verify_only_leaves_artifact_and_destinations_unchanged')
        c=OwnerConsole(a.binary,state,'login',owner)
        try:
            c.expect('Handle: ');c.send('sysop');c.password(password);c.expect('Command (then Enter): ')
            action('verify',record=True)
            c.send('q');c.expect('Logged out.');c.finish()
        finally:c.close()
        checks.append('live_local_session_survives_verify_and_receipt')
        good=backup_status.read(state,owner)
        assert good['result']==1 and good['objects']==1 and good['object_bytes']==len(content)
        assert b'Last attempt: VERIFIED' in data()
        checks.append('explicit_verified_receipt_read_by_rust_panel')
        saved=x.bundle/'objects'/object_id;saved.write_bytes(b'broken');action('verify',record=True,expected=False)
        failed=backup_status.read(state,owner);assert failed['result']==0 and failed['verified']==good['verified'] and failed['object_bytes']==len(content)
        assert b'Last attempt: FAILED' in data()
        checks.append('corrupt_recheck_records_failure_and_retains_last_success')
        target=SimpleNamespace(**vars(x));target.board=restored;target.store=restored_store
        action('restore',expected=False,values=target);assert not restored.exists() and not restored_store.exists()
        checks.append('corrupt_bundle_cannot_reserve_restore_paths')
        saved.write_bytes(content);action('verify',record=True);assert backup_status.read(state,owner)['result']==1
        action('restore',values=target);assert (restored_store/object_id).read_bytes()==content and not (restored/'backup.status').exists()
        c=OwnerConsole(a.binary,restored,'login',owner)
        try:
            c.expect('Handle: ');c.send('sysop');c.password(password);c.expect('Command (then Enter): ');c.send('q');c.expect('Logged out.');c.finish()
        finally:c.close()
        checks.append('restored_board_login_and_exact_blobs_without_receipt')
        with sqlite3.connect(f'file:{restored}/faction.sqlite3?mode=ro',uri=True) as db:
            assert db.execute('select enabled,name from wall_settings').fetchall()==[(1,'Recovery wall')]
            assert db.execute('select body from wall_notes').fetchall()==[('Wall survives complete recovery',)]
            assert db.execute('select request_id,note_id from wall_receipts').fetchall()==[('b'*32,1)]
        checks.append('complete_restore_preserves_wall_policy_note_and_cooldown_receipt')
        with sqlite3.connect(f'file:{restored}/faction.sqlite3?mode=ro',uri=True) as db:
            assert db.execute('select enabled,name from page_settings').fetchall()==[(1,'Recovery members')]
            assert db.execute('select bio,project from member_pages').fetchall()==[('Recovered bio','Recovered project')]
            assert db.execute('select body,visible from member_plans').fetchall()==[('Recovered hidden plan',0)]
        checks.append('complete_restore_preserves_member_page_and_hidden_plan')
        with sqlite3.connect(restored/'faction.sqlite3') as db:
            assert db.execute('select enabled,name from gathering_settings').fetchall()==[(1,'Recovery gatherings')]
            assert db.execute('select title,body,starts_at,ends_at,place from gatherings').fetchall()==[('Recovered gathering','Local event survives recovery',start,start+3600,'room:1')]
            assert db.execute('select gathering,member,response from gathering_rsvps').fetchall()==[(1,1,2)]
        checks.append('complete_restore_preserves_gathering_policy_event_and_response')
        target=SimpleNamespace(**vars(x));target.bundle=root/'bundle-recorded';action('backup',record=True,values=target);assert backup_status.read(state,owner)['result']==1
        checks.append('backup_command_can_explicitly_record_success')
        receipt=state/'backup.status';receipt.chmod(0o644);assert b'Backup check: UNAVAILABLE' in data();receipt.chmod(0o600)
        checks.append('unsafe_receipt_is_unavailable_to_dashboard')
        values=backup_status.read(state,owner);future=int(time.time())+3600;values.update(attempted=future,verified=future)
        receipt.write_bytes(('version=1\n'+''.join(f'{k}={values[k]}\n' for k in backup_status.FIELDS)).encode())
        assert b'CLOCK UNKNOWN' in data();receipt.unlink();assert b'Backup check: UNRECORDED' in data()
        checks.append('future_clock_and_missing_receipts_are_explicit')
    result=dict(result='pass',count=len(checks),checks=checks,native_distinct_uid_cli=native,developer_api_fixture=not native,capacity_claim=False,hardening_claim=False)
    if a.output:a.output.write_text(json.dumps(result,indent=2)+'\n')
    print(f'Backup/recovery passed {len(checks)} controls; '+('actual distinct-UID CLI.' if native else 'developer API fixture.'))


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('binary',type=Path);p.add_argument('operator',type=Path);p.add_argument('--owner-uid',type=int);p.add_argument('--files-uid',type=int);p.add_argument('--disposable-fixture',action='store_true');p.add_argument('--output',type=Path)
    a=p.parse_args();a.binary=a.binary.resolve(strict=True);a.operator=a.operator.resolve(strict=True);run(a)
