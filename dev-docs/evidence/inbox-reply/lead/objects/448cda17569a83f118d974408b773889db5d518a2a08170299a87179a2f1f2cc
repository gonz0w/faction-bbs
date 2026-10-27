#!/usr/bin/env python3
"""Bounded long native sampler reader; aggregate process rows on the test host."""
import json
import statistics
from credential_peak import Stream
from memory_profile import summarize


class SustainedStream(Stream):
    def _read(self):
        total=0
        try:
            for line in iter(lambda:self.process.stdout.readline(1024*1024),b''):
                total += len(line)
                if not line.endswith(b'\n') or total > 128*1024*1024 or len(self.events)>1000:
                    raise ValueError('sustained stream exceeded bound')
                row=json.loads(line)
                if row['event']=='sample':
                    row['summary']=summarize(row.pop('metrics'))
                self.events.append(row)
                if row['event']=='sample':self.ready.set()
        except BaseException as exc:self.error=exc;self.ready.set()

    def finish(self):
        try:
            self.process.wait(timeout=30);self.reader.join(timeout=2)
            if self.reader.is_alive():raise TimeoutError('sustained sampler did not finish')
            if self.error:raise self.error
            if self.process.returncode:raise ValueError('native sustained sampler failed')
            start,stop=self.events[0],self.events[-1];rows=self.events[1:-1]
            assert start['event']=='start' and stop['event']=='stop'
            assert len(rows)==start['count']==905
            samples=[]
            for i,e in enumerate(rows):
                assert e['event']=='sample' and e['index']==i
                assert e['finished_ns']>=e['monotonic_ns']
                samples.append({'monotonic_ns':e['monotonic_ns'],'finished_ns':e['finished_ns'],**e['summary']})
            assert all(a['finished_ns']<=b['monotonic_ns'] for a,b in zip(samples,samples[1:]))
            gaps=[(b['monotonic_ns']-a['monotonic_ns'])/1e6 for a,b in zip(samples,samples[1:])]
            return {'samples':samples,'sample_count':len(samples),'requested_interval_ms':start['interval_ms'],
                    'guest_seconds':(stop['monotonic_ns']-start['monotonic_ns'])/1e9,
                    'gap_ms':{'min':min(gaps),'median':statistics.median(gaps),'max':max(gaps)},
                    'markers':{'start_ns':start['monotonic_ns'],'stop_ns':stop['monotonic_ns']}}
        finally:self.abort()
