#!/usr/bin/env python3
"""Read retained private captures; emit hashes/counts/verdicts, never private data."""
import copy
import hashlib
import importlib
import json
import os
from pathlib import Path
import re
import sys
import tarfile
import traceback

BASE=Path('/private/tmp/faction235-native-OWNED')
OUT=Path('/private/tmp/faction235-review-OWNED')
PIN='318c8daa5f9918c17f3c4f18588ef0f855f6e24d81d27e99884d59f565778599'

def sha(data): return hashlib.sha256(data).hexdigest()
def raw(path): return path.read_bytes()
def val(path): return json.loads(raw(path))
def require(test, label):
    if not test: raise AssertionError(label)

def audit():
    sys.dont_write_bytecode=True
    sys.path.insert(0,str(BASE/'runtime-code'))
    proof=importlib.import_module('probe_mail_correspondents')
    menu=importlib.import_module('probe_menu_mail')
    visit=importlib.import_module('probe_visit_mail')
    require(sha(raw(BASE/'runtime-code/probe_mail_correspondents.py'))==PIN,'probe-source-pin')
    associations=val(BASE/'capture-associations.json')
    require(associations['result']=='pass' and set(associations['routes'])=={'host_classic','host_unicode','netbsd_classic','netbsd_unicode'},'route-matrix')
    roots=list(BASE.glob('correspondents-menu-OWNED-*'))+list((BASE/'native-private').glob('correspondents-menu-OWNED-*'))
    require(len(roots)==4,'four-captures')
    captures={val(p/'private-input-association.json')['capture_id']:p for p in roots}
    require(len(captures)==4,'unique-ids')
    candidates={sha(raw(p)):p for folder in [BASE/'host-main',BASE/'native-runtime'] for p in folder.glob('*.json')}
    reports=[]
    for name,link in associations['routes'].items():
        folder=captures[link['capture_id']]
        markerraw=raw(folder/'private-markers.json'); markers=json.loads(markerraw)
        scanraw=raw(folder/'board-file-scan-input.private.json'); scaninput=json.loads(scanraw)
        trace=raw(folder/'board-io.private.log')
        hashes={'trace_sha256':sha(trace),'marker_set_sha256':sha(markerraw),'file_scan_input_sha256':sha(scanraw)}
        require(hashes==link['private_input_hashes']==val(folder/'capture-hash-association.private.json'),'private-hash-association')
        beforelink=val(folder/'private-input-association.json')
        require(beforelink==dict(marker_set_sha256=hashes['marker_set_sha256'],capture_id=link['capture_id'],source_sha256=PIN,generated_before_capture=True),'pre-generated-association')
        trace_link=val(folder/'trace-input-association.private.json')
        require(trace_link['capture_id']==link['capture_id'] and trace_link['source_sha256']==PIN and trace_link['marker_set_sha256']==hashes['marker_set_sha256']==trace_link['marker_file_sha256_at_trace_start'],'trace-start-association')
        resultlink=val(folder/'capture-result-association.private.json')
        require(resultlink==dict(capture_id=link['capture_id'],source_sha256=PIN,result_sha256=link['result_sha256'],**hashes),'result-association')
        require(link['source_sha256']==PIN and link['generated_before_capture'] is True and link['input_trace_unchanged'] is True,'frozen-association')
        require(link['result_sha256'] in candidates,'result-source-resolved')
        result=val(candidates[link['result_sha256']])
        require(result['result']=='pass' and result['capture_id']==link['capture_id'] and result['source_sha256']==PIN and result['private_input_hashes']==hashes,'actual-result-binding')
        require(result['platform']==('Darwin' if name.startswith('host') else 'NetBSD') and result['unicode'] is name.endswith('unicode'),'platform-mode')
        require(result['same_ssh_pid_until_normal_close'] is True and result['automatic_redial_observed'] is False and result['actual_home_transport_error']=='Connect' and result['files_disabled'] is True,'recorded-same-ssh-outage')
        require(result['exact_canonical_ack_invariants'] is True,'recorded-canonical-invariants')
        require(scaninput['markers']==markers and len(markers)==len(set(markers))==result['marker_count'],'scan-actual-markers')
        require(all(scaninput[k] for k in ['board','store','log']),'scan-roots-present')
        visit.validate_scan(result['privacy_scan'])
        stats=menu.require_board_bound_private_absent(trace,[m.encode() for m in markers])
        streams,count=menu.reconstruct_kdump_streams(trace)
        require(stats==result['board_io_reconstruction'],'reconstructed-stats-exact')
        require({key[0] for key in streams}=={trace_link['trace']['pid']},'trace-process-id')
        require(all(key[2] in ['read','wrote'] and key[1]>=0 for key in streams),'per-process-fd-direction')
        require(all(any(token in stream for key,stream in streams.items() if key[2]=='read') for token in [b'@MAIL1 park ',b'@MAIL1 back ']),'actual-park-back')
        require(all(b'FACTION1;' not in stream and b'@FACTION1' not in stream for stream in streams.values()),'no-file-protocol')
        before=val(folder/'canonical-before.private.json'); after=val(folder/'canonical-after.private.json')
        proof.require_only_read_change(before,after,4)
        require(len(before['history'])==8 and len(before['pending'])==1,'canonical-fixture-shape')
        require(all(row[field] in markers for row in before['history'] for field in ['address','subject','body','identity']),'actual-history-marker-coverage')
        peers={row['address'] for row in before['history']}
        require(len(peers)==2 and all(re.fullmatch(r'(peer|other)-[0-9a-f]{12}@away\.test',peer) for peer in peers),'generated-peers')
        require(any(re.fullmatch(r'[0-9a-f]{48}',m) for m in markers),'generated-password-marker')
        terminal=raw(folder/'terminal-2.private')
        require(all(peer.encode() in terminal for peer in peers),'private-peer-local-terminal-positive-control')
        require(all(token in terminal for token in [b'[LOCAL CORRESPONDENTS]',b'[LOCAL CORRESPONDENT INBOX]',b'Home cycle stopped; pending ciphertext retained.',b'No unread received records.',b'Reply cancelled; no ciphertext queued.',b'Logged out.']),'local-journey-positive-controls')
        require(terminal.count(proof.UNLOCK.encode())>=3,'fresh-unlock-reentries')
        require(result['private_matches']==result['file_protocol_matches']==0,'recorded-absence')
        native=name.startswith('netbsd')
        if native:
            with tarfile.open(BASE/'native-main-captures.private.tar.gz','r:gz') as archive:
                for filename in ['board-io.private.log','private-markers.json','canonical-before.private.json','canonical-after.private.json','board-file-scan-input.private.json']:
                    members=[m for m in archive.getmembers() if m.name.endswith(folder.name+'/'+filename)]
                    require(len(members)==1,'archive-member-unique')
                    require(archive.extractfile(members[0]).read()==raw(folder/filename),'archive-extracted-exact')
        reports.append(dict(route=name,capture_id=link['capture_id'],source_sha256=PIN,result_sha256=link['result_sha256'],private_input_hashes=hashes,marker_count=len(markers),stream_stats=stats,canonical_only_read_index=4,canonical_pending_and_archive_unchanged=True,private_values_replayed_absent=True,recorded_board_file_scan_verified=True,independent_fresh_board_file_scan=False,actual_local_positive_controls=True,native_archive_members_exact=native))
    return dict(result='pass',scope='separate-review-retained-actual-private-captures',routes=reports,review_source_sha256=sha(raw(Path(__file__))),oracle_source_sha256={name:sha(raw(BASE/'runtime-code'/name)) for name in ['probe_mail_correspondents.py','probe_menu_mail.py','probe_visit_mail.py']},limits=['Read-only replay of retained captures, not a new runtime execution.','Board-file scan replay validates saved observation and exact scan input; no independent fresh remote scan.','Same-SSH PID and TCP refusal are recorded runtime assertions; retained trace and local journey provide supporting observations.'])

if __name__=='__main__':
    try:
        result=audit()
        (OUT/'audit-result.json').write_text(json.dumps(result,indent=2)+'\n')
        print('PASS: four actual capture associations, strict GIO private-marker replay, canonical-only acknowledgement and local journey controls.')
    except BaseException as error:
        safe=dict(result='fail',error_type=type(error).__name__,assertion=str(error) if isinstance(error,AssertionError) else 'non-assertion error; inspect privately',line=traceback.extract_tb(error.__traceback__)[-1].lineno)
        (OUT/'audit-status.json').write_text(json.dumps(safe,indent=2)+'\n')
        print(json.dumps(safe));sys.exit(1)
