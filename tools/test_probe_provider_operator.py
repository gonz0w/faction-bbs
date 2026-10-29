"""Contracts for the maintained provider operator production-console proof."""
import importlib
import os
from pathlib import Path
import signal
import subprocess
import tempfile
import time
import unittest
from unittest.mock import patch

class OperatorRunnerTests(unittest.TestCase):
    def module(self):
        return importlib.import_module('probe_provider_operator')

    def test_probe_requires_separate_operator_and_member_seed_inputs(self):
        p=self.module()
        args=p.make_parser().parse_args(['--client','/client','--fixture','/seed','--operator','/operator','--owned-root','/owned','--output','/owned/result.json'])
        self.assertEqual(args.operator,Path('/operator'))
        self.assertEqual(args.fixture,Path('/seed'))
        self.assertTrue(callable(p.probe))

    def test_readiness_failure_closes_pipes_and_reaps_owned_operator(self):
        p=self.module()
        with tempfile.TemporaryDirectory(prefix='faction284-runner-OWNED-') as name:
            root=Path(name).resolve();root.chmod(0o700)
            binary=root/'unready'
            binary.write_text('#!/usr/bin/env python3\nimport time\ntime.sleep(30)\n');binary.chmod(0o700)
            children=[];original=subprocess.Popen
            def record(*args,**kwargs):
                child=original(*args,**kwargs);children.append(child);return child
            try:
                with patch.object(p.subprocess,'Popen',side_effect=record):
                    with self.assertRaises(ValueError):p.Operator(binary,root/'config.toml',root,ready_seconds=.1)
                self.assertEqual(len(children),1)
                self.assertIsNotNone(children[0].poll())
                self.assertTrue(children[0].stdout.closed and children[0].stderr.closed)
            finally:
                for child in children:
                    if child.poll() is None:child.kill()
                    child.wait(timeout=5)
                    child.stdout.close();child.stderr.close()

    def test_operator_uses_foreground_serve_and_term_within_original_drain(self):
        p=self.module()
        with tempfile.TemporaryDirectory(prefix='faction284-runner-OWNED-') as name:
            root=Path(name).resolve();root.chmod(0o700)
            binary=root/'synthetic-operator'
            binary.write_text('#!/usr/bin/env python3\nimport signal,sys,time\nassert sys.argv[1:]==["serve",'+repr(str(root/'config.toml'))+']\ndef stop(*_):\n print("stopped",flush=True)\n sys.exit(0)\nsignal.signal(signal.SIGTERM,stop)\nprint("ready 12345",flush=True)\nwhile True:time.sleep(.01)\n');binary.chmod(0o700)
            owner=p.Operator(binary,root/'config.toml',root)
            self.assertEqual(owner.port,12345)
            start=time.monotonic();owner.close();self.assertLess(time.monotonic()-start,5)
            self.assertIsNotNone(owner.process.poll())
            self.assertTrue(owner.process.stdout.closed and owner.process.stderr.closed)
            owner.close()

    def test_proposal_is_public_exact_versioned_and_bounded(self):
        p=self.module()
        text=p.proposal(['01'*32,'02'*32],[bytes([3])*32,bytes([4])*32],generation=2,reset_epoch=1,approved=[False,True],stamp=100)
        import tomllib
        v=tomllib.loads(text)
        self.assertEqual(v['version'],1)
        self.assertEqual(v['generation'],2)
        self.assertEqual(v['homes'],['home.test'])
        self.assertEqual(len(v['bindings']),2)
        self.assertFalse(v['bindings'][0]['approved'])
        self.assertNotIn('key',v['bindings'][0])
        self.assertLessEqual(len(text.encode()),65536)

class PreparedModeTests(unittest.TestCase):
    def module(self):return importlib.import_module('probe_provider_operator')

    def test_prepared_mode_is_deliberate_and_default_off(self):
        p=self.module();values=['--client','/client','--fixture','/fixture','--operator','/operator','--owned-root','/owned','--output','/owned/result.json']
        self.assertFalse(p.make_parser().parse_args(values).prepared)
        self.assertTrue(p.make_parser().parse_args(values+['--prepared']).prepared)

    def test_invalid_device_oracle_handles_existing_and_generated_toml(self):
        p=self.module()
        for text in [b'device=101\n',b'device = 101\n']:
            self.assertIn(b'202',p.invalid_device_profile(text))
            self.assertNotIn(b'101',p.invalid_device_profile(text))
        with self.assertRaises(ValueError):p.invalid_device_profile(b'device=202\n')

    def test_each_member_action_uses_separate_normal_hidden_unlock(self):
        p=self.module();events=[]
        class Console:
            def __init__(self,args,**kwargs):self.args=args;events.append(('argv',args))
            def password(self,password,prompt):events.append(('hidden',password,prompt))
            def expect(self,text):events.append(('expect',text))
            def finish(self):events.append(('finish',))
        children=[]
        with patch.object(p,'LauncherConsole',Console):
            p.prepare_member(Path('/client'),Path('/vault'),Path('/template'),Path('/new-profile'),Path('/new-card'),'synthetic password',True,children)
        self.assertEqual([event[1] for event in events if event[0]=='argv'],
            [[Path('/client'),'--device-provider-prepare',Path('/vault'),Path('/template'),Path('/new-profile'),'--unicode'],
             [Path('/client'),'--device-provider-card',Path('/vault'),Path('/new-profile'),Path('/new-card'),'--unicode']])
        self.assertEqual(sum(event[0]=='hidden' for event in events),2)
        self.assertTrue(all(event[2]==p.UNLOCK for event in events if event[0]=='hidden'))
        self.assertEqual(len(children),2)

    def test_prepared_selection_reaches_both_existing_terminal_journeys(self):
        p=self.module()
        with patch.object(p,'require_root',return_value=Path('/owned')),patch.object(p,'one_journey',return_value={'checks':['synthetic']}) as journey:
            p.probe(Path('/client'),Path('/fixture'),Path('/operator'),Path('/owned'),'openssl',prepared=True)
        self.assertEqual([call.args[-1] for call in journey.call_args_list],[False,True])
        self.assertTrue(all(call.kwargs=={'prepared':True} for call in journey.call_args_list))

if __name__=='__main__':unittest.main()
