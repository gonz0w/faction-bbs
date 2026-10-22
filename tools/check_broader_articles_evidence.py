#!/usr/bin/env python3
"""Check saved broader-profile evidence; does not rerun INN or the VM."""
import hashlib, json, re
from evidence_sources import ROOT, source_bytes
from build_inn_reference import SHA256, VERSION

def check():
    d=json.loads((ROOT/'dev-docs/evidence/broader-articles-netbsd/results.json').read_text())
    assert d['schema']==1 and d['result']=='pass' and d['article_profile']==2
    assert d['host_rust_tests']==d['native_rust_tests']==133 and d['host_python_tests']==104
    assert d['shared_reader']['result']=='pass' and d['shared_reader']['count']==7
    for host in ['host','netbsd']:
        p=d['codec_profile'][host]
        assert p['iterations']==1000 and p['body_bytes']==16384 and p['wire_bytes']==17775
        assert p['parse_peak_requested_bytes']==66277 and p['serialize_peak_requested_bytes']==32256
        assert p['parse_mean_ns']>0 and p['serialize_mean_ns']>0
        inn=d['inn'][host];build=d['inn_build'][host]
        assert inn['result']=='pass' and inn['count']==12 and len(set(inn['checks']))==12
        assert 'inn_accepts_invalid_date_faction_rejects' in inn['checks']
        assert inn['version']==build['version']==VERSION
        assert inn['source_sha256']==build['archive_sha256']==SHA256
        assert inn['only_owned_loopback_refusal_fixture'] and inn['client_commands_observed']==0
        assert inn['synthetic_hostname_avoids_dns_lookup']
        assert inn['service_started'] is inn['posting_or_external_contact'] is False
        assert build['source_no_js_scan'] is True and build['installed_service'] is build['privileged_install'] is False
        assert inn['runtime_artifacts']==build['runtime_artifacts']
        assert {'frontends/inews','frontends/.libs/inews'}<=inn['runtime_artifacts'].keys()
        for digest in inn['runtime_artifacts'].values():assert re.fullmatch('[a-f0-9]{64}',digest)
    assert d['inn']['host']['checks']==d['inn']['netbsd']['checks']
    assert d['native_reference_build_tool']['no_js_scan'] and d['native_reference_build_tool']['dependencies']==[]
    assert d['fixture']=={'vcpus':2,'ram_mib':1024,'admin_loopback_only':True,'synthetic_board':True}
    assert all(value is False for value in d['claims'].values())
    assert d['teardown']=={'vm_stopped':True,'private_clone_removed':True}
    assert len(d['execution_sources'])==100
    assert {'crates/faction-articles/src/encoding.rs','crates/faction-articles/tests/broader.rs','tools/build_inn_reference.py','tools/check_inn_articles.py'}<=d['execution_sources'].keys()
    for name,digest in d['execution_sources'].items():assert hashlib.sha256(source_bytes(name,digest)).hexdigest()==digest
    print('Broader article record passed: 133 host/native Rust tests, 12 original INN controls per host, 7 shared controls; no live peer/capacity/hardening claim.')

if __name__=='__main__':check()
