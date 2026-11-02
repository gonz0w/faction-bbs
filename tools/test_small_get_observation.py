"""Fail-closed log qualification; no provider process is executed here."""
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

import check_small_get_observation as check

NAME = 'authority::services::publication::tests::composed_loop_tests::characterize_one_small_get_through_both_maintained_loops'
HOST_SHAPE = '\nrunning 1 test\ntest ' + NAME + ' ... ready 0\nready 0\nok\n\ntest result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 211 filtered out; finished in 0.24s\n\n'


def valid_logs():
    # The optional private reference is the preserved actual host stdout, not a
    # fresh execution. The fallback preserves its ready/libtest interleaving.
    reference = os.environ.get('FACTION326_REFERENCE_STDOUT')
    stdout = Path(reference).read_text() if reference else HOST_SHAPE
    points = [
        (13362,0,0,0,5,2,0,0), (29674,1,6,0,0,0,0,0),
        (29677,1,6,0,1,0,29677,129677), (32739,0,1,0,0,0,0,0),
        (32739,0,1,0,4,1,0,0), (33577,1,0,0,6,2,0,0),
        (33577,1,0,0,7,2,0,0), (33596,1,6,0,2,0,0,0),
        (33602,1,6,0,4,2,0,0), (33662,0,0,0,6,2,0,0),
        (33662,0,0,0,7,2,0,0), (71876,0,1,1,0,0,0,0),
        (71878,0,1,1,4,2,0,0),
    ]
    rows=[]
    for sequence,(micros,role,kind,attempt,event,outcome,first,last) in enumerate(points,1):
        rows.append(f'small_get_point schema=1 sequence={sequence} micros={micros} role={role} kind={kind} attempt={attempt} event={event} outcome={outcome} first_us={first} last_us={last}')
    for index,start,end,outcome,attempt,busy,planned,actual in [(0,1,5030,0,1,0,0,0),(1,5033,32794,1,2,1,15078,22628),(2,32794,71939,2,3,1,45238,54506)]:
        rows.append(f'small_get_call schema=1 index={index} start_us={start} end_us={end} outcome={outcome} diagnostic=1 phase=6 attempted={attempt} connections={attempt} busy={busy} planned_wait_us={planned} actual_wait_us={actual}')
    rows.append('small_get_summary schema=1 points=13 calls=3 setup_ok=1 transport_ok=1 exact=1 response_bytes=17 response_offset=0 response_total=17 trace_conclusive=1 timestamp_overlap=0 busy_count=1 busy_before_finish=1 busy_after_finish=0 loops_ok=1 physical_zero=1 inactive=1 drain_us=5537 root_removed=1 token_retained=0 clock_state_retained=0 attempts=3 connections=3 outer_collects=2 inner_collects=1 outer_finish_before=1 outer_finish_after=1 inner_finish_before=1 inner_finish_after=1 expected_kinds=1 calls_ok=1')
    reference_stderr=os.environ.get('FACTION326_REFERENCE_STDERR')
    prefix=Path(reference_stderr).read_text() if reference_stderr else 'small_get setup_ok=true exact=true\n'
    return stdout,prefix+'\n'+'\n'.join(rows)+'\n'


class SmallGetOutput(unittest.TestCase):
    def test_actual_host_selection_shape_and_complete_numeric_records_pass(self):
        stdout,stderr=valid_logs()
        self.assertIn(' ... ready 0\nready 0\nok\n',stdout)
        result=check.validate(stdout,stderr,0)
        self.assertTrue(result.ok)
        self.assertEqual(sum(line.startswith('small_get_point ') for line in result.lines),13)
        self.assertTrue(all(all(value.isascii() and value.isdecimal() for value in (part.split('=',1)[1] for part in line.split()[1:])) for line in result.lines))

    def test_normal_inline_named_success_also_passes(self):
        stdout,stderr=valid_logs()
        inline=stdout.replace(' ... ready 0\nready 0\nok\n',' ... ok\n',1)
        self.assertNotEqual(inline,stdout)
        self.assertTrue(check.validate(inline,stderr,0).ok)

    def test_missing_failed_ignored_and_duplicate_named_outcomes_fail(self):
        stdout,stderr=valid_logs()
        self.assertEqual(stdout.count('\nok\n'),1)
        inline=stdout.replace(' ... ready 0\nready 0\nok\n',' ... ok\n',1)
        cases={
            'missing_interleaved_ok':stdout.replace('\nok\n','\n',1),
            'named_failed':stdout.replace(' ... ready 0',' ... FAILED',1),
            'named_ignored':stdout.replace(' ... ready 0',' ... ignored',1),
            'duplicate_standalone_ok':stdout.replace('\nok\n','\nok\nok\n',1),
            'inline_plus_standalone_ok':inline.replace(' ... ok\n',' ... ok\nok\n',1),
        }
        for name,value in cases.items():
            self.assertNotEqual(value,stdout)
            with self.subTest(case=name):self.assertFalse(check.validate(value,stderr,0).ok)

    def test_command_failure_always_fails_despite_success_looking_logs(self):
        stdout,stderr=valid_logs()
        for status in [1,7,101,124,137]:
            with self.subTest(status=status):
                result=check.validate(stdout,stderr,status)
                self.assertFalse(result.ok)
                self.assertTrue(any(line.startswith('small_get_point ') for line in result.lines))

    def test_zero_duplicate_wrong_and_ignored_selection_fail(self):
        stdout,stderr=valid_logs()
        variants=[stdout.replace('running 1 test','running 0 tests').replace('1 passed','0 passed'),stdout+stdout,stdout.replace(NAME,'wrong::selected_test'),stdout.replace('0 ignored','1 ignored'),stdout.replace('1 passed','0 passed'),stdout.replace('0 failed','1 failed')]
        for value in variants:
            with self.subTest(value=value[:30]):self.assertFalse(check.validate(value,stderr,0).ok)

    def test_missing_duplicate_or_failed_final_result_fail(self):
        stdout,stderr=valid_logs()
        summary=next(line for line in stdout.splitlines() if line.startswith('test result:'))
        for value in [stdout.replace(summary,''),stdout+summary+'\n',stdout.replace('test result: ok.','test result: FAILED.')]:
            self.assertFalse(check.validate(value,stderr,0).ok)

    def test_malformed_truncated_duplicate_and_out_of_bounds_records_fail(self):
        stdout,stderr=valid_logs();lines=stderr.splitlines()
        point=next((line for line in lines if line.startswith('small_get_point ')),None)
        self.assertIsNotNone(point)
        removed=stderr.replace(point+'\n','',1)
        duplicated=stderr.replace(point,point+' micros=2',1)
        self.assertNotIn(point+'\n',removed)
        self.assertIn(point+' micros=2\n',duplicated)
        self.assertEqual(duplicated.count(point+' micros=2\n'),1)
        variants=[removed,stderr.replace('schema=1','schema=2',1),stderr+lines[-1]+'\n',stderr.replace('role=1','role=2',1),stderr.replace('kind=6','kind=7',1),stderr.replace('event=5','event=8',1),stderr.replace('sequence=1 ','sequence=2 ',1),stderr.replace('drain_us=5537','drain_us=5000000'),stderr.replace('root_removed=1','root_removed=0'),stderr.replace('timestamp_overlap=0','timestamp_overlap=1'),stderr.replace('outcome=2','outcome=4',1),stderr.replace('last_us=129677','last_us=129678'),stderr.replace('micros=13362','micros=18446744073709551616'),duplicated,stderr.replace(lines[-1],'')]
        for i,value in enumerate(variants):
            with self.subTest(case=i):self.assertFalse(check.validate(stdout,value,0).ok)

    def test_missing_role_or_call_cannot_pass_with_summary_flags(self):
        stdout,stderr=valid_logs()
        for value in ['\n'.join(line for line in stderr.splitlines() if not ('small_get_point ' in line and 'role=1' in line)),'\n'.join(line for line in stderr.splitlines() if not line.startswith('small_get_call schema=1 index=1 '))]:
            self.assertFalse(check.validate(stdout,value,0).ok)

    def test_cli_keeps_raw_logs_and_secret_exceptions_private(self):
        stdout,stderr=valid_logs();secret='PRIVATE_TOKEN_PKI_STORE_REQUEST_SENTINEL'
        with tempfile.TemporaryDirectory(prefix='faction326-validator-') as directory:
            root=Path(directory);(root/'OWNED').write_text('Owned synthetic validator input\n')
            out=root/'stdout';err=root/'stderr';out.write_text(stdout+secret+'\n');err.write_text(stderr+secret+'\n')
            command=[sys.executable,str(Path(check.__file__)),'--status','0','--stdout',str(out),'--stderr',str(err)]
            result=subprocess.run(command,capture_output=True,text=True)
            self.assertEqual(result.returncode,0)
            self.assertNotIn(secret,result.stdout+result.stderr)
            self.assertNotIn(NAME,result.stdout+result.stderr)
            err.write_text(stderr+'small_get_point schema=1 payload='+secret+'\n')
            result=subprocess.run(command,capture_output=True,text=True)
            self.assertNotEqual(result.returncode,0);self.assertNotIn(secret,result.stdout+result.stderr)
            err.unlink();command[-1]=str(root/secret)
            result=subprocess.run(command,capture_output=True,text=True)
            self.assertNotEqual(result.returncode,0);self.assertNotIn(secret,result.stdout+result.stderr)

    def test_cli_nonzero_status_and_oversized_input_fail(self):
        stdout,stderr=valid_logs()
        with tempfile.TemporaryDirectory(prefix='faction326-validator-') as directory:
            root=Path(directory);out=root/'stdout';err=root/'stderr';out.write_text(stdout);err.write_text(stderr)
            command=[sys.executable,str(Path(check.__file__)),'--status','101','--stdout',str(out),'--stderr',str(err)]
            self.assertNotEqual(subprocess.run(command,capture_output=True).returncode,0)
            command[3]='0';err.write_bytes(b'x'*((1<<20)+1))
            self.assertNotEqual(subprocess.run(command,capture_output=True).returncode,0)


if __name__=='__main__':unittest.main()
