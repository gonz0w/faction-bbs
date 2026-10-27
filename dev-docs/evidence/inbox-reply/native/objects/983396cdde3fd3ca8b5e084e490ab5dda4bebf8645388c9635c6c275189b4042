#!/usr/bin/env python3
"""Guard ledger for #58. Same admission predicate as the historical experiment."""
from memory_profile import headroom,summarize

MIN_HEADROOM_BYTES=128*1024*1024


def guard_decision(sample,admitted,reason):
    if not isinstance(admitted,int) or not 0<=admitted<=250:
        raise ValueError('invalid admitted count')
    summary=summarize(sample)
    vm=summary['vm_bytes']
    available=vm['free']+max(0,vm['file_clean']-vm['wired'])
    return {'admitted':admitted,'reason':reason,'passed':headroom(sample),
            'minimum_headroom_bytes':MIN_HEADROOM_BYTES,
            'reclaimable_headroom_bytes':available,
            'shortfall_bytes':max(0,MIN_HEADROOM_BYTES-available),
            'snapshot':summary}


def admission_batches(decisions):
    # These bounds concern admitted sessions between checks, not continuous
    # reserve or an instantaneous maximum. KDF/replacement checks are separate.
    rows=[r for r in decisions if r['reason']=='before_admission_batch']
    return [{'admitted_before':a['admitted'],'admitted_after':b['admitted'],
             'admitted_in_batch':b['admitted']-a['admitted'],
             'headroom_drop_bytes':a['reclaimable_headroom_bytes']-b['reclaimable_headroom_bytes'],
             'after_passed':b['passed'],'after_shortfall_bytes':b['shortfall_bytes']}
            for a,b in zip(rows,rows[1:])]
