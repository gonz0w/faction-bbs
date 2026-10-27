#!/usr/bin/env python3
"""Sanitized NetBSD metric aggregation; RSS is never presented as USS/PSS."""
import math

ROLES={'member','broker','frontend','ssh'}


def summarize(sample):
    vm=sample['vm'];page=vm['page_bytes']
    if page<=0 or page&(page-1):raise ValueError('invalid page size')
    if any(not isinstance(v,int) or v<0 for v in vm.values()):raise ValueError('invalid VM counter')
    out={'vm_bytes':{k.removesuffix('_pages'):v*page for k,v in vm.items() if k.endswith('_pages')},
         'roles':{},'limits':dict(sample['limits'])}
    for row in sample['processes']:
        if set(row)!={'role','rss_kib','vsz_bytes','high_water_rss_kib','cpu_us'} or row['role'] not in ROLES:
            raise ValueError('unexpected process data; do not save identities or arguments')
        if any(not isinstance(v,int) or v<0 for k,v in row.items() if k!='role'):
            raise ValueError('invalid process metric')
        role=out['roles'].setdefault(row['role'],{'count':0,'rss_sum_bytes':0,'high_water_rss_sum_bytes':0,'vsz_sum_bytes':0,'cpu_us_sum':0})
        role['count']+=1;role['rss_sum_bytes']+=row['rss_kib']*1024
        role['high_water_rss_sum_bytes']+=row['high_water_rss_kib']*1024
        role['vsz_sum_bytes']+=row['vsz_bytes'];role['cpu_us_sum']+=row['cpu_us']
    # This includes all mandatory app/access/crypto processes; SSH admin fixture
    # is excluded by the native collector's dedicated-listener ancestry filter.
    out['all_roles_rss_sum_bytes']=sum(v['rss_sum_bytes'] for v in out['roles'].values())
    return out


def interval(before,after,seconds,vcpus=2):
    if not math.isfinite(seconds) or seconds<=0:raise ValueError('invalid interval')
    a,b=before['cpu_ticks'],after['cpu_ticks'];idle=before['cpu_idle_index']
    if len(a)!=len(b) or after['cpu_idle_index']!=idle or not 0<=idle<len(a):raise ValueError('CPU shape')
    delta=[y-x for x,y in zip(a,b)]
    if min(delta)<0 or sum(delta)==0:raise ValueError('CPU counters reset or no elapsed ticks')
    x,y=summarize(before),summarize(after)
    out={'seconds':round(seconds,3),'whole_vm_cpu_percent_one_core':round(100*vcpus*(sum(delta)-delta[idle])/sum(delta),3),
         'vm_delta_bytes':{k:y['vm_bytes'][k]-x['vm_bytes'][k] for k in x['vm_bytes']},
         'stable_roles_cpu_percent_one_core':{}}
    for role,old in x['roles'].items():
        new=y['roles'].get(role)
        if new and old['count']==new['count'] and new['cpu_us_sum']>=old['cpu_us_sum']:
            out['stable_roles_cpu_percent_one_core'][role]=round((new['cpu_us_sum']-old['cpu_us_sum'])/(seconds*10000),3)
    return out


def headroom(sample):
    vm=sample['vm'];limits=sample['limits']
    return ((vm['free_pages']+max(0,vm['file_clean_pages']-vm['wired_pages']))*vm['page_bytes']>=128*1024*1024
            and vm['swap_used_pages']==0
            and limits['processes']+64<limits['max_processes']
            and limits['open_files']+128<limits['max_files'])
