#!/usr/bin/env python3
"""Check saved Unicode-subject evidence; does not rerun the software VM."""
import hashlib, json
from evidence_sources import ROOT, source_bytes


def check():
    d=json.loads((ROOT/'dev-docs/evidence/unicode-subjects-netbsd/results.json').read_text())
    assert d['schema']==1 and d['metadata_schema']==7 and d['result']=='pass'
    assert d['host_rust_tests']==d['native_rust_tests']==140 and d['host_python_tests']==104
    assert d['local_worker_pty']['host_controls']==d['local_worker_pty']['native_controls']==5
    assert len(set(d['local_worker_pty']['cases']))==5
    shared=d['shared_reader']
    assert shared['result']=='pass' and shared['count']==len(set(shared['checks']))==12
    assert {'shared_split_utf8_and_backspace','shared_160_byte_subject_clipped_by_cells','shared_unicode_reply_default','shared_malformed_incomplete_control_and_overflow_close','shared_revocation_denies_unicode_draft_submission'} <= set(shared['checks'])
    assert shared['dedicated_ssh_admission_tested'] is shared['capacity_claim'] is False
    assert d['limits']=={'subject_utf8_bytes':160,'label_ascii_cells_max':119,'migration_deadline_seconds':5,'new_runtime_dependencies':0,'new_owner_helpers':0,'normal_sessions':8}
    assert d['fixture']=={'vcpus':2,'ram_mib':1024,'admin_loopback_only':True,'synthetic_board':True}
    assert all(v is False for v in d['claims'].values())
    assert d['teardown']=={'vm_stopped':True,'private_clone_removed':True}
    assert len(d['execution_sources'])==101
    assert {'crates/faction-core/src/post_metadata_v7.sql','crates/faction-terminal/src/subject.rs','crates/faction-terminal/tests/subjects.rs','tools/smoke_unicode_subjects.py','tools/smoke_shared_room_reader.py'} <= d['execution_sources'].keys()
    for name,digest in d['execution_sources'].items():
        assert hashlib.sha256(source_bytes(name,digest)).hexdigest()==digest
    print('Unicode subject evidence passed: 140 host/native Rust tests, five local/worker PTY controls per host and twelve native shared controls; classic ASCII display, no capacity/hardening claim.')


if __name__=='__main__':check()
