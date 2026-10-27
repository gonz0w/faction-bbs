import unittest
from pathlib import Path
import measure_home_mail as cost
class HomeCost(unittest.TestCase):
    def test_usage_tail_never_exports_private_terminal_content(self):
        transcript='private local menu text\r\n0.02 real 0.01 user 0.00 sys\r\n 14000 maximum resident set size\r\n 0 page faults\r\n'
        result=cost.resource_usage(transcript)
        self.assertEqual(result['max_rss_kib'],14000)
        self.assertNotIn('private',str(result))
        for bad in ['private terminal only',transcript+'unknown diagnostic\n',transcript.replace('14000','0')]:
            with self.assertRaises(cost.FixtureFailure):cost.resource_usage(bad)
    def test_resource_argv_contains_no_secret_or_environment_override(self):
        home=dict(store='/private/owned/store',config='/private/owned/home.toml',local_password='LOCAL-SECRET',password='HOME-SECRET')
        argv=cost.invocation(Path('/private/owned/faction-client'),home)
        self.assertEqual(argv[:2],['/usr/bin/time','-l'])
        self.assertNotIn('SECRET',str(argv))
