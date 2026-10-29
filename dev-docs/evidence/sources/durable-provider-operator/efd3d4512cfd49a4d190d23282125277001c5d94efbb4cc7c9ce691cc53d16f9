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

if __name__=='__main__':unittest.main()
