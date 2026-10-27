#!/usr/bin/env python3
"""Validate recorded article software evidence; this does not rerun the VM."""
import hashlib, json
from evidence_sources import ROOT, source_bytes

def check():
    d=json.loads((ROOT/'dev-docs/evidence/authored-articles-netbsd/results.json').read_text())
    assert d['schema']==1 and d['result']=='pass'
    assert d['host_rust_tests']==d['native_rust_tests']==127
    assert d['host_python_tests']==104
    assert d['shared_reader']['result']=='pass' and d['shared_reader']['count']==7
    assert len(set(d['shared_reader']['checks']))==7
    assert {'shared_reply_default_subject','shared_authored_subject_override'}<=set(d['shared_reader']['checks'])
    assert d['independent_mime_parser']=={'host':True,'netbsd':True,'news_server_interop':False}
    for host in ['host','netbsd']:
        p=d['codec_profile'][host]
        assert p['iterations']==1000 and p['body_bytes']==16384 and p['wire_bytes']==17123
        assert p['parse_peak_requested_bytes']==65745 and p['serialize_peak_requested_bytes']==32256
        assert p['parse_mean_ns']>0 and p['serialize_mean_ns']>0
    assert d['fixture']=={'vcpus':2,'ram_mib':1024,'admin_loopback_only':True,'synthetic_board':True}
    assert all(value is False for value in d['claims'].values())
    assert d['teardown']=={'vm_stopped':True,'private_clone_removed':True}
    assert len(d['execution_sources'])==95
    assert {'crates/faction-core/src/room_article.rs','crates/faction-core/tests/room_articles.rs',
            'crates/faction-articles/examples/profile_articles.rs','tools/check_article_interop.py'}<=d['execution_sources'].keys()
    for name,digest in d['execution_sources'].items():
        assert hashlib.sha256(source_bytes(name,digest)).hexdigest()==digest
    print('Authored article record passed: 127 host/native Rust tests, 7 shared controls; no news-server/capacity/hardening claim.')

if __name__=='__main__':check()
