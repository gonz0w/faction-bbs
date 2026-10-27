#!/usr/bin/env python3
"""Privacy-bounded live limit ledger and unchanged headroom evaluation."""
ROLES={'broker','frontend','ssh_listener','ssh_privileged','ssh_session','owner_tool'}
RESOURCES=['cpu_seconds','file_bytes','data_bytes','stack_bytes','core_bytes','rss_bytes',
           'locked_bytes','processes','files','socket_bytes','address_bytes','threads']


def live_limits(raw):
    if set(raw)!={'resources','processes'} or raw['resources']!=RESOURCES:
        raise ValueError('unexpected native resource schema')
    if not 1<=len(raw['processes'])<=4096:raise ValueError('invalid process count')
    profiles=[];rows=[]
    for row in raw['processes']:
        if set(row)!={'role','rss_bytes','virtual_bytes','limits'} or row['role'] not in ROLES:
            raise ValueError('unexpected process data; do not record identities')
        for k in ['rss_bytes','virtual_bytes']:
            if type(row[k]) is not int or row[k]<0:raise ValueError('invalid process counter')
        limits=row['limits']
        if len(limits)!=len(RESOURCES):raise ValueError('incomplete resource limits')
        for pair in limits:
            if len(pair)!=2 or any(v!='unlimited' and (type(v) is not int or v<0) for v in pair):
                raise ValueError('invalid limit')
            soft,hard=pair
            if hard!='unlimited' and (soft=='unlimited' or soft>hard):raise ValueError('inconsistent limit')
        if limits not in profiles:profiles.append(limits)
        rows.append([row['role'],row['rss_bytes'],row['virtual_bytes'],profiles.index(limits)])
    return {'resources':RESOURCES,'limit_profiles':profiles,
            'process_columns':['role','rss_bytes','virtual_bytes','limit_profile'],'process_rows':rows}


def sample_guard(sample):
    """Same 128-MiB predicate, applied to a saved summarized sample."""
    vm=sample['vm_bytes'];limits=sample['limits']
    available=vm['free']+max(0,vm['file_clean']-vm['wired'])
    return {'reclaimable_headroom_bytes':available,'minimum_headroom_bytes':128*1024*1024,
            'passed':available>=128*1024*1024 and vm['swap_used']==0
                     and limits['processes']+64<limits['max_processes']
                     and limits['open_files']+128<limits['max_files']}
