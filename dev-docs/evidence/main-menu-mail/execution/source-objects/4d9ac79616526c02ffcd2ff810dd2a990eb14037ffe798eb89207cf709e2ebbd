#!/usr/bin/env python3
"""Three-minute lookup-abuse/useful-member experiment; marked VM only."""
import concurrent.futures,json,secrets,socket,time
from credential_peak import Stream
from headroom_memory import sample_guard
from memory_profile import summarize
from lookup_probe import cohort

def legitimate(a):
    import paramiko
    t=None
    try:
        t=paramiko.Transport(socket.create_connection((a.host,a.port),3));t.start_client(timeout=4)
        assert paramiko.HostKeys(str(a.known_hosts)).check('['+a.host+']:'+str(a.port),t.get_remote_server_key()),'host key changed'
        key=paramiko.Ed25519Key.from_private_key_file(str(a.identity));t.auth_publickey('factionterm',key)
        return t.is_authenticated()
    except (paramiko.SSHException,OSError,EOFError):return False
    finally:
        if t is not None:t.close()

def active(clients,admin,metric,remote,login,drop,alive,args):
    assert len(clients)==250 and len({c.profile for c in clients})==250
    stream=Stream(admin+[metric+' --stream 1000 185']);began=time.monotonic()
    rounds=[];attempts=[];abuse=None;removed=None;restored=False;next_probe=15
    initial=json.loads(remote('/usr/local/libexec/faction-key-admission --inspect').stdout)
    listener=remote('cat /var/run/faction-sshd.pid; stat -f %i /var/run/faction-auth/admission.state').stdout
    effective=remote('/usr/sbin/sshd -T -f /etc/ssh/faction/key_admission_sshd_config').stdout.decode()
    penalties=next(x.split(' ',1)[1] for x in effective.splitlines() if x.startswith('persourcepenalties '));assert penalties!='no'
    failure=None;pool=concurrent.futures.ThreadPoolExecutor(max_workers=1);future=None
    try:
        while time.monotonic()-began<180:
            elapsed=time.monotonic()-began
            if elapsed>=5 and future is None:future=pool.submit(cohort,args,'full',8)
            if future is not None and future.done() and abuse is None:
                abuse=future.result();abuse['finished_at_seconds']=time.monotonic()-began
                assert abuse['before']['active']==250 and abuse['after']['active']==250 and abuse['charged_lookups']==0
                assert abuse['after']['full']-abuse['before']['full']==24
                # Leave one slot for an actual same-source recovery check. The
                # other 249 distinct approved members must remain useful.
                c=clients.pop();removed=c.profile;c.send('/quit');c.expect('Chat closed.');c.expect('Ctrl-C exits; Esc: Enter to recover.');c.raw(b'q');c.expect('Logged out.');c.finish();drop(c)
            if abuse is not None and not restored and elapsed>=next_probe:
                accepted=legitimate(args);attempts.append({'at_seconds':time.monotonic()-began,'accepted':accepted})
                if accepted:
                    time.sleep(.4);c=login(removed);c.raw(b'c');c.expect('chat[#lobby]> ');restored=True
                next_probe=elapsed+5
            marker='lookup-witness-'+secrets.token_hex(8);sender=clients[len(rounds)%len(clients)]
            start=time.monotonic();sender.send(marker)
            for c in clients:
                if c is not sender:c.expect(marker)
            # Sender echo is deliberately excluded from confirmed receipts.
            assert time.monotonic()-start<10 and all(alive(c) for c in clients)
            rounds.append({'at_seconds':time.monotonic()-began,'members':len(clients),'confirmed_peer_receipts':len(clients)-1,'seconds':time.monotonic()-start})
            for c in clients:c.transcript=b''
            samples=[e for e in stream.events if e.get('event')=='sample']
            assert not stream.error and samples and all(sample_guard(summarize(e['metrics']))['passed'] for e in samples),'lookup activity sampled guard failed'
            if len(rounds)%30==0:print('Lookup active: %d rounds, %.1fs, %d approved members'%(len(rounds),time.monotonic()-began,len(clients)),flush=True)
            time.sleep(max(0,start+1-time.monotonic()))
        assert restored and attempts and attempts[-1]['accepted'],'same-source recovery deadline'
        observed=stream.finish()
    except BaseException as exc:
        failure={'type':type(exc).__name__,'detail':str(exc)};stream.abort()
        observed={'samples':[summarize(e['metrics']) for e in stream.events if e.get('event')=='sample']}
    finally:pool.shutdown(wait=True);stream.abort()
    same=remote('cat /var/run/faction-sshd.pid; stat -f %i /var/run/faction-auth/admission.state').stdout==listener
    result={'requested_seconds':180,'seconds':time.monotonic()-began,'outcome':'failed' if failure else 'completed','failure':failure,'rounds':rounds,'abuse':abuse,'same_source_attempts':attempts,'same_listener_and_inode':same,'effective_source_penalties':penalties,'listener_reset':False,'registry_reset':False,'native_stream':observed,'before':initial,'after':json.loads(remote('/usr/local/libexec/faction-key-admission --inspect').stdout),'final_distinct_members':len({c.profile for c in clients}),'scope':'250 members during full-capacity unsigned cohort; 249 during same-source fresh admission recovery; return to 250; sampled guard, not sustained flood or capacity claim'}
    args.output.with_suffix('.lookup.json').write_text(json.dumps(result,indent=2)+'\n')
    assert not failure and same and result['final_distinct_members']==250,'lookup workload failed; native partial evidence retained'
    return result
