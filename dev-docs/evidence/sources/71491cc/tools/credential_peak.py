#!/usr/bin/env python3
"""Bounded persistent native metric stream parser. No credentials or identities."""
import json
import statistics
import subprocess
import threading
import time
from memory_profile import summarize


def ledger(events):
    if not 3<=len(events)<=12002:raise ValueError('invalid event count')
    start,stop=events[0],events[-1]
    if start['event']!='start' or stop['event']!='stop':raise ValueError('missing stream markers')
    rows=events[1:-1]
    if len(rows)!=start['count']:raise ValueError('incomplete native stream')
    times=[];cost=[];samples=[]
    for i,row in enumerate(rows):
        if row['event']!='sample' or row['index']!=i:raise ValueError('invalid sample order')
        began,finished=row['monotonic_ns'],row['finished_ns']
        if finished<began or (times and began<=times[-1]):raise ValueError('invalid monotonic sample time')
        times.append(began);cost.append((finished-began)/1e6)
        samples.append({'monotonic_ns':began,'finished_ns':finished,**summarize(row['metrics'])})
    gaps=[(b-a)/1e6 for a,b in zip(times,times[1:])]
    if not start['monotonic_ns']<=times[0]<times[-1]<=stop['monotonic_ns']:raise ValueError('invalid markers')
    return {'requested_interval_ms':start['interval_ms'],'sample_count':len(samples),
            'guest_seconds':(stop['monotonic_ns']-start['monotonic_ns'])/1e9,
            'gap_ms':{'min':min(gaps),'median':statistics.median(gaps),'max':max(gaps)},
            'collector_query_ms':{'median':statistics.median(cost),'max':max(cost)},
            'samples':samples,'markers':{'start_ns':start['monotonic_ns'],'stop_ns':stop['monotonic_ns']}}


class Stream:
    def __init__(self, command):
        self.started=time.monotonic();self.events=[];self.error=None
        self.ready=threading.Event()
        self.process=subprocess.Popen(command,stdout=subprocess.PIPE,stderr=subprocess.PIPE)
        self.reader=threading.Thread(target=self._read,daemon=True);self.reader.start()
        if not self.ready.wait(10):self.abort();raise TimeoutError('native sampler did not start')
        if self.error:self.abort();raise self.error

    def _read(self):
        total=0
        try:
            for line in iter(lambda:self.process.stdout.readline(1024*1024),b''):
                total+=len(line)
                if not line.endswith(b'\n') or total>64*1024*1024 or len(self.events)>12001:
                    raise ValueError('native stream exceeded bound')
                row=json.loads(line);self.events.append(row)
                if row['event']=='sample':self.ready.set()
        except BaseException as exc:self.error=exc;self.ready.set()

    def finish(self):
        try:
            self.process.wait(timeout=30);self.reader.join(timeout=2)
            if self.reader.is_alive():raise TimeoutError('native stream reader did not finish')
            if self.error:raise self.error
            if self.process.returncode:raise ValueError('native sampler failed')
            result=ledger(self.events);result['host_wall_seconds']=time.monotonic()-self.started
            return result
        finally:self.abort()

    def abort(self):
        if self.process.poll() is None:self.process.kill()
        self.process.wait()
        self.process.stdout.close();self.process.stderr.close()


def allocation_window(run, allocation):
    start,end=allocation['touched_ns'],allocation['released_ns']
    before=[s['vm_bytes']['anon'] for s in run['samples'] if s['finished_ns']<allocation['began_ns']]
    during=[s['vm_bytes']['anon'] for s in run['samples'] if start<=s['monotonic_ns'] and s['finished_ns']<=end]
    if len(before)<3 or not during:raise ValueError('calibration window not sampled')
    baseline=int(statistics.median(before[-10:]))
    return {'known_touched_bytes':allocation['bytes'],'fully_inside_samples':len(during),
            'baseline_anon_bytes':baseline,'max_anon_bytes':max(during),
            'observed_anon_delta_bytes':max(during)-baseline,'allocation_markers':allocation}
