"""A missing ceiling in any live process invalidates the capacity evidence."""
import copy
import unittest
from check_confinement_evidence import validate_limits
from headroom_memory import RESOURCES


class ConfinementReadbacks(unittest.TestCase):
    def test_every_process_and_both_soft_and_hard_are_required(self):
        profiles=[]
        for files,threads in [(548,3),(8,0)]:
            profile=[[0,0] for _ in RESOURCES]
            profile[RESOURCES.index('files')]=[files,files]
            profile[RESOURCES.index('threads')]=[threads,threads]
            profiles.append(profile)
        record={'live_process_limits':[{'readback':{'resources':RESOURCES,
            'limit_profiles':profiles,'process_rows':[
                ['broker',1,1,0,1],['frontend',1,1,1,250]]}}]}
        validate_limits(record)
        for profile in range(2):
            for resource in ['files','threads','processes','locked_bytes','core_bytes']:
                for bound in range(2):
                    altered=copy.deepcopy(record)
                    altered['live_process_limits'][0]['readback']['limit_profiles'][profile][RESOURCES.index(resource)][bound]+=1
                    with self.assertRaisesRegex(AssertionError,'ceiling drift'):
                        validate_limits(altered)


if __name__=='__main__':unittest.main()
