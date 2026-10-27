"""Controlled child ownership, publication observation and recovery assertions."""
import copy
import io
import signal
import unittest
from pathlib import Path
from unittest.mock import Mock, patch
import probe_archive_process_death as fixture


class Child:
    def __init__(self, polls, killed_code=-signal.SIGKILL, broken_input=False):
        self.polls=iter(polls); self.last=None; self.returncode=None
        self.stdin=Mock() if broken_input else io.BytesIO()
        if broken_input:self.stdin.write.side_effect=BrokenPipeError()
        self.kills=0; self.reaps=0; self.killed_code=killed_code
    def poll(self):
        if self.returncode is not None:return self.returncode
        self.last=next(self.polls,self.last)
        return self.last
    def kill(self):self.kills+=1;self.returncode=self.killed_code
    def communicate(self, timeout):
        self.reaps+=1
        if self.returncode is None:self.returncode=self.last
        return b'private captured output',b'private captured error'


def home():
    return dict(store='/private/store',config='/private/config',local_password='local-private',password='home-private',run_as='memberfixture')


def row(subject, direction='received', sent_at=42):
    return dict(direction=direction,subject=subject,body=subject+' body',sent_at=sent_at,address='bob@home.test')


class ArchiveProcessDeath(unittest.TestCase):
    def observe(self, child, inodes, clocks=None):
        with patch.object(fixture.platform,'system',return_value='Linux'), patch.object(fixture.subprocess,'Popen',return_value=child) as launch, patch.object(fixture,'canonical_inode',side_effect=inodes), patch.object(fixture.time,'monotonic',side_effect=clocks or [0,0,0,0,0]), patch.object(fixture.time,'sleep'):
            result=fixture.kill_after_publication(Path('/binary'),home())
        args,kwargs=launch.call_args
        self.assertEqual(args[0],['/binary','catchup','/private/store','/private/config'])
        self.assertNotIn('local-private',str(args));self.assertNotIn('home-private',str(args))
        self.assertEqual(kwargs['user'],'memberfixture');self.assertEqual(kwargs['group'],'memberfixture')
        return result

    def test_kills_only_owned_live_child_after_observed_publication(self):
        child=Child([None,None,None])
        result=self.observe(child,[(1,10),(1,10),(1,11)])
        self.assertEqual(result,dict(signal='SIGKILL',publication_observed=True))
        self.assertEqual(child.kills,1);self.assertEqual(child.reaps,1)

    def test_normal_finish_or_lost_kill_race_is_never_a_pass(self):
        child=Child([0])
        with self.assertRaises(fixture.FixtureFailure):self.observe(child,[(1,10)])
        self.assertEqual(child.kills,0);self.assertGreaterEqual(child.reaps,1)
        child=Child([None,None],killed_code=0)
        with self.assertRaises(fixture.FixtureFailure):self.observe(child,[(1,10),(1,11)])
        self.assertEqual(child.kills,1)

    def test_no_publication_timeout_and_input_failure_reap_owned_child(self):
        child=Child([None])
        with self.assertRaises(fixture.FixtureFailure):self.observe(child,[(1,10)],clocks=[0,16])
        self.assertEqual(child.kills,1);self.assertEqual(child.reaps,1)
        child=Child([None],broken_input=True)
        with self.assertRaises(fixture.FixtureFailure):self.observe(child,[(1,10)])
        self.assertEqual(child.kills,1);self.assertEqual(child.reaps,1)

    def test_canonical_stat_failure_reaps_only_its_child(self):
        child=Child([None])
        with self.assertRaises(fixture.FixtureFailure):self.observe(child,[(1,10),OSError('unavailable')])
        self.assertEqual(child.kills,1);self.assertEqual(child.reaps,1)

    def event_observation(self, child, *, register_error=False, wait_error=False, launch_error=False):
        events=[];queue=Mock();fd=71;change=object()
        def control(changes, count, timeout):
            events.append('register' if changes else 'wait')
            if changes and register_error:raise OSError('watch unavailable')
            if not changes and wait_error:raise OSError('watch lost')
            return []
        queue.control.side_effect=control;queue.close.side_effect=lambda:events.append('queue-close')
        native_select=Mock(KQ_FILTER_VNODE=1,KQ_EV_ADD=2,KQ_EV_CLEAR=4,KQ_NOTE_WRITE=8)
        native_select.kqueue.return_value=queue;native_select.kevent.return_value=change
        native_os=Mock(O_RDONLY=0,O_DIRECTORY=1,O_CLOEXEC=2)
        native_os.open.side_effect=lambda *a:events.append('open') or fd
        native_os.close.side_effect=lambda n:events.append('fd-close')
        def launch(*a,**kw):
            events.append('launch')
            if launch_error:raise OSError('launch unavailable')
            return child
        with patch.object(fixture,'select',native_select,create=True),patch.object(fixture,'os',native_os,create=True),patch.object(fixture.platform,'system',return_value='NetBSD'),patch.object(fixture.subprocess,'Popen',side_effect=launch) as popen,patch.object(fixture,'canonical_inode',side_effect=[(1,10),(1,10),(1,11)]),patch.object(fixture.time,'monotonic',return_value=0),patch.object(fixture.time,'sleep') as polling:
            try:result=fixture.kill_after_publication(Path('/binary'),home())
            except fixture.FixtureFailure:result=None
        return result,events,queue,native_select,native_os,popen,polling,change

    def test_netbsd_registers_vnode_watch_before_child_and_checks_inode_before_kill(self):
        child=Child([None,None,None]);result,events,queue,sel,osmod,launch,polling,change=self.event_observation(child)
        self.assertEqual(result,dict(signal='SIGKILL',publication_observed=True))
        self.assertIn('register',events);self.assertLess(events.index('register'),events.index('launch'))
        sel.kevent.assert_called_once_with(71,filter=1,flags=6,fflags=8)
        self.assertEqual(queue.control.call_args_list[0].args,([change],0,0))
        self.assertEqual(queue.control.call_args_list[1].args,(None,1,15))
        polling.assert_not_called();self.assertEqual(child.kills,1);self.assertEqual(child.reaps,1)
        queue.close.assert_called_once();osmod.close.assert_called_once_with(71)

    def test_netbsd_registration_failure_closes_watch_without_starting_child(self):
        result,events,queue,sel,osmod,launch,polling,change=self.event_observation(Child([None,None,None]),register_error=True)
        self.assertIsNone(result);launch.assert_not_called();queue.close.assert_called_once();osmod.close.assert_called_once_with(71)

    def test_netbsd_wait_failure_kills_reaps_owned_child_and_closes_both_handles(self):
        child=Child([None,None,None]);result,events,queue,sel,osmod,launch,polling,change=self.event_observation(child,wait_error=True)
        self.assertIsNone(result);self.assertEqual(child.kills,1);self.assertEqual(child.reaps,1)
        queue.close.assert_called_once();osmod.close.assert_called_once_with(71)

    def test_netbsd_launch_failure_closes_registered_watch(self):
        result,events,queue,sel,osmod,launch,polling,change=self.event_observation(Child([None]),launch_error=True)
        self.assertIsNone(result);queue.close.assert_called_once();osmod.close.assert_called_once_with(71)

    def test_netbsd_early_exit_remains_failure_and_closes_watch(self):
        child=Child([0]);result,events,queue,sel,osmod,launch,polling,change=self.event_observation(child)
        self.assertIsNone(result);self.assertEqual(child.kills,0);self.assertGreaterEqual(child.reaps,1)
        queue.close.assert_called_once();osmod.close.assert_called_once_with(71)

    def test_reopened_history_must_preserve_prefix_pending_ids_and_authenticated_new_rows(self):
        prior=dict(history=[row('old')],pending=[dict(operation='a'*32,address='bob@home.test')])
        expected=[row('new'+str(i)) for i in range(3)]
        for count in range(4):
            current=copy.deepcopy(prior);current['history']+=expected[:count]
            self.assertEqual(fixture.recovered_history(prior,current,expected),count)
        changes=[lambda d:d['history'][0].update(body='changed'),lambda d:d['pending'].clear(),lambda d:d['history'].append(row('unknown')),lambda d:d['history'].extend([expected[0],expected[0]]),lambda d:d['history'].append(dict(expected[0],sent_at=43)),lambda d:d['history'].append(dict(expected[0],direction='sent'))]
        for change in changes:
            value=copy.deepcopy(prior);change(value)
            with self.assertRaises(fixture.FixtureFailure):fixture.recovered_history(prior,value,expected)

    def test_public_result_requires_real_kill_and_complete_recovery(self):
        good=dict(signal='SIGKILL',publication_observed=True)
        result=fixture.public_result(good,3,9)
        self.assertEqual(result['signal'],'SIGKILL');self.assertEqual(result['recovered_new_count'],3)
        self.assertFalse({'cursor','epoch','subject','body','password','stdout','stderr'}&result.keys())
        for killed,count,total in [(dict(good,signal='normal-exit'),3,9),(dict(good,publication_observed=False),3,9),(good,2,9),(good,3,8)]:
            with self.assertRaises(fixture.FixtureFailure):fixture.public_result(killed,count,total)


if __name__=='__main__':unittest.main()
