"""Read-only original host notice capture audit; never emit generated values."""
from pathlib import Path
import hashlib, json, os, re, stat, sys, time
BASE=Path('/private/tmp/faction239-native-OWNED')
OUT=Path('/private/tmp/faction239-review-OWNED')
CODE=BASE/'runtime-code03'
sys.dont_write_bytecode=True
sys.path.insert(0,str(CODE))
import probe_menu_mail_notices as proof
import probe_menu_mail as stream
import probe_visit_mail as visit
sha=lambda b:hashlib.sha256(b).hexdigest()
read=lambda p:json.loads(p.read_bytes())
def identity(p):
 s=p.lstat();assert stat.S_ISREG(s.st_mode) and not p.is_symlink() and s.st_uid==os.getuid()
 return dict(sha256=sha(p.read_bytes()),bytes=s.st_size,mtime_ns=s.st_mtime_ns,mode=oct(stat.S_IMODE(s.st_mode)))
def inventory(p):return {str(f.relative_to(p)):identity(f) for f in sorted(p.rglob('*')) if f.is_file()}
source_map=read(BASE/'sources/runtime-code03.json')
for name,digest in source_map.items():
 p=CODE/Path(name).name;assert sha(p.read_bytes())==digest
 copy=OUT/'host-audit-sources'/Path(name).name;copy.parent.mkdir(mode=0o700,exist_ok=True)
 
 if copy.exists():assert copy.read_bytes()==p.read_bytes()
 else:copy.write_bytes(p.read_bytes());copy.chmod(0o600)
compiled=read(BASE/'sources/host-compile-before02.json')
assert compiled==read(BASE/'sources/host-compile-after02.json')
runtime=read(BASE/'sources/host-runtime-final.json')
assert set(compiled)<=set(runtime)
delta={p:s for p,s in runtime.items() if compiled.get(p)!=s}
assert set(delta)<={'tools/probe_menu_mail_notices.py','tools/test_probe_menu_mail_notices.py'}
for name in ['crates/faction-client/src/main.rs','crates/faction-client/src/visit_mail.rs']:
 assert compiled[name]==runtime[name]
assert runtime['crates/faction-client/src/main.rs']=='4f394b707498d4532c1ae51794c6d786896ca3ba5651b0e855eae2c7cf161c11'
assert runtime['crates/faction-client/src/visit_mail.rs']=='164b5aaf70442118a2ca218f3ff15f6d6cb5ad12461b57d23c02b34962876e61'
for name in ['tools/probe_menu_mail_notices.py','tools/test_probe_menu_mail_notices.py']:assert runtime[name]==source_map[name]
binaries=read(BASE/'host-network-binaries02.json')
for name,digest in binaries.items():assert sha((BASE/'bin/host-network02'/name).read_bytes())==digest
assert all(r['returncode']==0 for r in read(BASE/'host-build-results02.json'))
settings=read(BASE/'host-settings.json');profile=Path(settings['profile'])
assert settings['disposable_fixture'] is True and settings['files_disabled'] is True
profile_source=BASE/'source-final03/crates/faction-client/src/profile.rs'
assert sha(profile_source.read_bytes())==compiled['crates/faction-client/src/profile.rs']
assert 'Command::new("/usr/bin/ssh")' in profile_source.read_text()
verdicts=[]
for mode in ['classic','unicode']:
 route=BASE/'host-main02'/(mode+'.json');result=read(route)
 assert result['platform']=='Darwin' and result['unicode']==(mode=='unicode') and result['result']=='pass'
 assert result['capture_id']!='302bf032f453a9f06e85f60e61a5bce7'
 matches=[]
 for p in BASE.glob('menu-notices-OWNED-*/private-input-association.json'):
  if read(p)['capture_id']==result['capture_id']:matches.append(p.parent)
 assert len(matches)==1
 original=matches[0];before=inventory(original)
 assert original.stat().st_uid==os.getuid() and stat.S_IMODE(original.stat().st_mode)==0o700
 pre=read(original/'private-input-association.json');trace=read(original/'trace-input-association.private.json')
 association=read(original/'capture-result-association.private.json')
 proof.require_capture_association(pre,trace,result)
 assert result['source_sha256']==source_map['tools/probe_menu_mail_notices.py']
 assert result['client_binary_sha256']==binaries['faction-client']
 assert result['fixture_binary_sha256']==binaries['mail_correspondents_fixture']
 assert result['profile_sha256']==sha(profile.read_bytes())
 assert association['capture_id']==result['capture_id'] and association['source_sha256']==result['source_sha256']
 assert association['result_sha256']==sha(route.read_bytes())
 for field,digest in result['private_input_hashes'].items():assert association[field]==digest
 markers=read(original/'private-markers.json');generated=read(original/'generated-inputs.private.json')
 assert markers and len(markers)==result['marker_count'] and len(markers)==len(set(markers))
 assert all(type(m)is str and m for m in markers)
 assert all(generated[k] in markers for k in ['home_password','subject','body','peer','other'])
 assert generated['board_password'] not in markers
 assert proof.PASS in markers
 scan=read(original/'board-file-scan-input.private.json')
 assert scan['markers']==markers
 for category,field in [('board','scan_board'),('store','scan_store'),('log','scan_logs')]:assert scan[category]==settings[field] and scan[category]
 assert sha((original/'private-markers.json').read_bytes())==pre['marker_set_sha256']
 assert sha((original/'board-file-scan-input.private.json').read_bytes())==pre['file_scan_input_sha256']
 wire=(original/'board-io.private.log').read_bytes()
 assert sha(wire)==result['private_input_hashes']['trace_sha256']
 streams,records=stream.reconstruct_kdump_streams(wire)
 reconstruction=stream.require_board_bound_private_absent(wire,[m.encode() for m in markers])
 assert reconstruction==result['board_io_reconstruction']
 assert all(reconstruction[k]>0 for k in ['records','streams','decoded_bytes','read_bytes','write_bytes'])
 assert reconstruction['decoded_bytes']==reconstruction['read_bytes']+reconstruction['write_bytes']
 requests=[]
 for key,data in streams.items():
  assert len(key)==3 and type(key[0])is int and type(key[1])is int and key[2] in ['read','wrote']
  assert b'FACTION1;' not in data and b'@FACTION1' not in data
  if key[2]=='read':requests.extend(re.findall(rb'@MAIL1 [^\r\n]*(?:\r\n|\r|\n)',data))
 protocol=proof.require_mail_cycles(b''.join(requests),5,8)
 assert protocol==result['protocol_cycles']==dict(parks=5,backs=13,nonces=13)
 visit.validate_scan(result['privacy_scan'])
 assert result['file_protocol_matches']==result['private_matches']==0
 for state in ['ready','unknown']:
  prior=read(original/(state+'-canonical-before.private.json'));after=read(original/(state+'-canonical-after.private.json'))
  assert prior==after and set(prior)=={'history','pending','archive'}
  for row in prior['history']:
   assert all(row[k] in markers for k in ['identity','subject','body','address'])
  canonical=(original/('mail-'+state)/'mail.fkv').read_bytes()
  assert all(m.encode() not in canonical for m in markers)
 # Original local terminal captures corroborate each notice state; no values emitted.
 terminal=(original/'terminal-2.private').read_bytes()
 for state,counts in [('off',None),('unknown',None),('stale',(3,8,1))]:
  expected=('\r\n'.join(proof.notice_lines(state,'main',counts))+'\r\n').encode()
  assert expected in terminal
 terminal_unknown=(original/'terminal-3.private').read_bytes()
 expected=('\r\n'.join(proof.notice_lines('stale','main',(None,8,1)))+'\r\n').encode()
 assert expected in terminal_unknown
 assert before==inventory(original)
 verdicts.append(dict(route=mode,result='pass',capture_id=result['capture_id'],result_identity=identity(route),original_directory=str(original),original_files=before,protocol_cycles=protocol,board_io_reconstruction=reconstruction,marker_count=len(markers),private_matches=0,source_binary_result_bindings_verified=True,private_input_hashes_verified=True,corrected_stream_reconstruction_verified=True,generated_private_inventory_reviewed=True,canonical_saved_inspection_equal=True,canonical_raw_before_after_equality='Exact executed probe asserted equality in memory; historical before ciphertext/hash was not separately saved.',board_file_scan='Exact pre-frozen request and unchanged executed SCAN source verified; original scan counts retained inside result, no separately saved raw scan response.'))
receipt=dict(review_type='independent original host capture audit; read-only',time_utc=time.strftime('%Y-%m-%dT%H:%M:%SZ',time.gmtime()),result='pass',source_map=source_map,compiled_manifest_identity=identity(BASE/'sources/host-compile-before02.json'),runtime_manifest_identity=identity(BASE/'sources/host-runtime-final.json'),runtime_overrides=delta,binary_receipt_identity=identity(BASE/'host-network-binaries02.json'),audit_script_identity=identity(Path(__file__)),routes=verdicts,failed_earlier_capture_id_excluded='302bf032f453a9f06e85f60e61a5bce7',pending=['NetBSD main2','host/NetBSD shared2','same final publication selection scan and review','stop proof and cleanup'],no_originals_modified=True)
p=OUT/'host-original-capture-review.json';assert not p.exists();p.write_text(json.dumps(receipt,indent=2)+'\n');p.chmod(0o600)
print(json.dumps(dict(result='pass',host_routes=2,capture_ids=[v['capture_id'] for v in verdicts],receipt=str(p),receipt_sha256=sha(p.read_bytes()))))
