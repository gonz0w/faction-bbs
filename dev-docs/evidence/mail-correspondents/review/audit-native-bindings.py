#!/usr/bin/env python3
"""Read frozen maps and saved compiler/runtime observations; emit safe counts."""
import hashlib
import json
from pathlib import Path
import re
import sys

P=Path('/private/tmp/faction235-native-OWNED')
O=Path('/private/tmp/faction235-review-OWNED')
R=Path('/Users/cam/DEV/faction-bbs')
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def val(p):return json.loads(p.read_bytes())
def require(test,label):
    if not test:raise AssertionError(label)
try:
    names=['sources/native-final01-before.json','native-runtime/source-before.json','native-runtime/source-after.json','native-build/source-observed.json','native-build/source-after.json']
    maps=[val(P/name) for name in names]
    require(all(m==maps[0] for m in maps),'native-source-maps-identical')
    require(len(maps[0])==702,'native-source-count')
    require(all(sha(P/'source-final01'/name)==digest for name,digest in maps[0].items()),'frozen-actual-source-bytes')
    production={name:maps[0][name] for name in ['crates/faction-client/src/checkpoint_ui.rs','crates/faction-client/src/checkpoint_ui_inbox.rs']}
    require(all(sha(R/name)==digest for name,digest in production.items()),'current-production-matches-native')
    runtime=val(P/'sources/runtime-guest-match01.json')
    require(all(sha(P/'runtime-code'/Path(name).name)==digest for name,digest in runtime.items()),'runtime-harness-source-map')
    build=val(P/'native-build/build-result.json')
    require(build['result']=='pass' and all(row['returncode']==0 for row in build['steps']),'native-build-exit-status')
    text=(P/'native-build/tests-build.log').read_text()
    counts=re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored',text)
    require(counts and all(int(f)==0 for p,f,i in counts),'native-test-results')
    passed=sum(int(p) for p,f,i in counts);ignored=sum(int(i) for p,f,i in counts)
    require((passed,ignored)==(277,5),'native-test-counts')
    env=val(P/'runtime-environment.json')
    require(env['result']=='pass' and env['build_memory_mib']==2048 and env['runtime_memory_mib']==1024 and 1000*1024**2<env['physmem_bytes']<1024**3 and env['cores']==2,'separate-build-runtime-memory')
    execution=val(P/'native-runtime/execution.json')
    require(all(row['returncode']==0 for row in execution),'native-runtime-observed-status')
    require(set(row['name'] for row in execution)=={'local-classic','local-unicode','main-classic','main-unicode','inbox','reply-classic','reply-unicode'},'native-runtime-coverage')
    output=dict(result='pass',scope='separate-review-native-source-and-observation-binding',review_source_sha256=sha(Path(__file__)),source_count=702,maps_sha256={name:sha(P/name) for name in names},production_sha256=production,runtime_harness_count=len(runtime),native_passed=passed,native_ignored=ignored,build_memory_mib=2048,runtime_memory_mib=1024,native_runtime_returncodes_all_zero=True,limits=['Reviews saved source/build/runtime observations; no fresh build, runtime or hardware claim.'])
    (O/'native-binding-result.json').write_text(json.dumps(output,indent=2)+'\n')
    print('PASS: 702 exact native source bindings; unchanged current production; 277 tests passed, 5 ignored; separate build/runtime memory observations.')
except BaseException as error:
    print(json.dumps(dict(result='fail',error_type=type(error).__name__,assertion=str(error) if isinstance(error,AssertionError) else 'non-assertion error; inspect privately')))
    sys.exit(1)
