#!/usr/bin/env python3
"""Bounded synthetic activity; called only by the disposable NetBSD driver."""
import json
import secrets
import socket
import time
from sustained_stream import SustainedStream
from headroom_memory import sample_guard
from memory_profile import interval, summarize


def percentile(values, fraction):
    ordered = sorted(values)
    return ordered[min(len(ordered)-1, int((len(ordered)-1)*fraction))]


def active_interval(clients, admin, metric, remote, login, drop, alive, output):
    """Fifteen minutes, one broadcast/second target; never a throughput ceiling."""
    assert len(clients) == 250 and len({c.profile for c in clients}) == 250
    duration = 900
    before = json.loads(remote(metric).stdout)
    stream = SustainedStream(admin+[metric+' --stream 1000 905'])
    began = time.monotonic()
    rounds = []; operations = []; held = []; abuse = None
    next_ui = 30; next_login = 60; next_sample_check = 0
    outcome = 'completed'; failure = None
    try:
        while time.monotonic()-began < duration:
            elapsed = time.monotonic()-began
            if elapsed >= next_sample_check:
                samples = [e for e in stream.events if e.get('event') == 'sample']
                assert samples, 'native sustained sampler missing'
                assert all(sample_guard(e['summary'])['passed'] for e in samples), 'sustained headroom guard failed'
                assert not stream.error, 'native sustained sampler failed'
                next_sample_check = elapsed+1
            if elapsed >= 720 and abuse is None:
                offers = drops = 0
                # The native probabilistic drop policy needs retries to fill its
                # eight silent slots. Fixed total offers, fixed socket deadlines.
                for _ in range(64):
                    s = socket.create_connection(active_interval.endpoint, 2)
                    s.settimeout(2); offers += 1
                    try: banner = s.recv(256)
                    except (ConnectionResetError, TimeoutError): banner = b''
                    if banner.startswith(b'SSH-2.0-'): held.append(s)
                    else: s.close(); drops += 1
                    if len(held) == 8: break
                assert len(held) == 8, 'silent preauth cohort not filled'
                for _ in range(16):
                    s = socket.create_connection(active_interval.endpoint, 2); s.settimeout(2); offers += 1
                    try: banner = s.recv(256)
                    except (ConnectionResetError, TimeoutError): banner = b''
                    s.close(); assert not banner.startswith(b'SSH-2.0-'), 'preauth cap exceeded'
                    drops += 1
                abuse = {'at_seconds': time.monotonic()-began, 'silent_accepted': 8,
                         'offers': offers, 'excess_drops': drops, 'listener_reset': False}
            if elapsed >= next_ui and elapsed < 720:
                c = clients[(len(operations)*17) % len(clients)]
                start = time.monotonic(); c.send('/quit'); c.expect('Chat closed.')
                c.expect('Ctrl-C exits; Esc: Enter to recover.')
                # A bounded invalid-key burst must not starve other connections.
                c.raw(b'x'*2048)
                witness = 'fairness-'+secrets.token_hex(8)
                peer = next(x for x in clients if x is not c); peer.send(witness)
                for x in clients:
                    if x is not c: x.expect(witness)
                fair_seconds = time.monotonic()-start
                c.raw(b'r'); c.expect('More rooms? [y/N]: '); c.send('n')
                c.expect('Press Enter to return to menu: '); c.send('')
                c.expect('Ctrl-C exits; Esc: Enter to recover.')
                c.raw(b'p'); c.expect('Room ID: '); c.send('1')
                c.expect('Reply to post ID'); c.send(''); c.expect('Subject (blank for generated/reply default): ');c.send('');c.expect('Message (one line')
                c.send('sustained synthetic '+secrets.token_hex(8)); c.expect('Submission ID: ')
                c.expect('Posted #'); c.expect('Press Enter to return to menu: '); c.send('')
                c.expect('Ctrl-C exits; Esc: Enter to recover.')
                c.raw(b'n'); c.expect('Room ID: '); c.send('50'); c.expect('More? [y/N]: '); c.send('n')
                c.expect('Press Enter to return to menu: '); c.send('')
                c.expect('Ctrl-C exits; Esc: Enter to recover.'); c.raw(b'c'); c.expect('chat[#lobby]> ')
                operations.append({'kind':'room_read_post_menu_input_burst', 'at_seconds':elapsed,
                                   'seconds':time.monotonic()-start, 'invalid_keys':2048,
                                   'other_chat_recipients':249, 'witness_seconds':fair_seconds})
                assert fair_seconds < 10, 'input burst starved chat witness'
                next_ui += 30
            if elapsed >= next_login and elapsed < 720:
                c = clients.pop(0); n = c.profile; start = time.monotonic()
                c.send('/quit'); c.expect('Chat closed.'); c.expect('Ctrl-C exits; Esc: Enter to recover.')
                c.raw(b'q'); c.expect('Logged out.'); c.finish(); drop(c); time.sleep(.4)
                c = login(n); c.raw(b'c'); c.expect('chat[#lobby]> ')
                operations.append({'kind':'fresh_member_credential', 'at_seconds':elapsed,
                                   'seconds':time.monotonic()-start, 'distinct_accounts':250})
                next_login += 60
            marker = 'sustain-'+secrets.token_hex(8)
            start = time.monotonic(); clients[len(rounds) % len(clients)].send(marker)
            for c in clients: c.expect(marker)
            latency = time.monotonic()-start
            assert latency < 10, 'sustained all-member delivery exceeded bound'
            assert len(clients) == 250 and len({c.profile for c in clients}) == 250
            assert all(alive(c) for c in clients), 'member lost during sustained activity'
            rounds.append({'at_seconds':time.monotonic()-began, 'recipients':250, 'seconds':latency})
            for c in clients:
                c.transcript = b''
                assert len(c.buffer) <= 32768, 'host driver output backlog'
            if len(rounds) % 60 == 0:
                print('Sustained interval: %d broadcasts, %.1f seconds, 250 distinct members' % (len(rounds), time.monotonic()-began), flush=True)
                output.with_suffix('.active.partial.json').write_text(json.dumps({'rounds':rounds,'operations':operations,'abuse':abuse})+'\n')
            time.sleep(max(0, start+1-time.monotonic()))
    except (AssertionError, OSError, TimeoutError) as exc:
        outcome = 'failed'; failure = {'type':type(exc).__name__, 'detail':str(exc)}
    finally:
        for s in held: s.close()
        # Keep a complete native stream if it can finish within its fixed tail;
        # failed early campaigns retain only actually collected samples instead.
        if outcome == 'completed': observed = stream.finish()
        else:
            stream.abort()
            observed = {'samples':[{'monotonic_ns':e['monotonic_ns'], 'finished_ns':e['finished_ns'], **e['summary']}
                                   for e in stream.events if e.get('event') == 'sample']}
    after = json.loads(remote(metric).stdout)
    latencies = [x['seconds'] for x in rounds]
    result = {'requested_seconds':duration, 'active_seconds':time.monotonic()-began,
              'outcome':outcome, 'failure':failure, 'rounds':rounds, 'operations':operations,
              'abuse':abuse, 'native_stream':observed,
              'whole_vm_interval':interval(before,after,time.monotonic()-began),
              'delivery_latency_seconds':{'p50':percentile(latencies,.5),'p95':percentile(latencies,.95),'max':max(latencies)} if latencies else {},
              'final_distinct_members':len({c.profile for c in clients}),
              'scope':'bounded 15-minute active test; sampled guard; no long-soak or production capacity claim'}
    output.with_suffix('.active.json').write_text(json.dumps(result,indent=2)+'\n')
    assert outcome == 'completed', 'sustained activity failed; preserved native samples'
    assert all(sample_guard(s)['passed'] for s in observed['samples']), 'sustained stream guard failed'
    assert summarize(after)['roles']['ssh']['count'] == 501, 'silent preauth did not clear without restart'
    return result
