from pathlib import Path
import subprocess,hashlib,json
R=Path('/Users/cam/DEV/faction-bbs');P=Path('/private/tmp/faction230-experiment-proof');ui=R/'experiments/055-shared-ui/ui.rs';(P/'ui-before.rs').write_bytes(ui.read_bytes());env=dict(__import__('os').environ,CARGO_BUILD_JOBS='2',CARGO_TARGET_DIR=str(R/'target/issue230-shared-ui'))
cmd=['cargo','clippy','--manifest-path','experiments/055-shared-ui/Cargo.toml','--all-targets','--release','--locked','--offline','--','-D','warnings']
red=subprocess.run(cmd,cwd=R,env=env,stdout=subprocess.PIPE,stderr=subprocess.STDOUT);(P/'red.log').write_bytes(red.stdout);assert red.returncode and b'HomeMail' in red.stdout and b'non-exhaustive' in red.stdout,red.stdout.decode()[-1000:]
s=ui.read_text();assert s.count('            | Action::Contacts')==1;ui.write_text(s.replace('            | Action::Contacts','            | Action::HomeMail\n            | Action::Contacts'));(P/'ui-after.rs').write_bytes(ui.read_bytes())
checks=[cmd,['cargo','fmt','--manifest-path','experiments/055-shared-ui/Cargo.toml','--','--check'],['cargo','test','--manifest-path','experiments/055-shared-ui/Cargo.toml','--release','--locked','--offline']];results=[]
for i,command in enumerate(checks):
 run=subprocess.run(command,cwd=R,env=env,stdout=subprocess.PIPE,stderr=subprocess.STDOUT);(P/('green'+str(i)+'.log')).write_bytes(run.stdout);results.append(dict(command=command,returncode=run.returncode));assert run.returncode==0,run.stdout.decode()[-1000:]
(P/'result.json').write_text(json.dumps(dict(result='pass',red_exit=red.returncode,checks=results,before_sha256=hashlib.sha256((P/'ui-before.rs').read_bytes()).hexdigest(),after_sha256=hashlib.sha256(ui.read_bytes()).hexdigest(),scope='Research prototype explanatory optional-section fallback only; maintained service/client untouched'),indent=2)+'\n');print('Research prototype actual non-exhaustive RED; HomeMail uses existing unavailable/Pause fallback; strictClippy,fmt and existingtest GREEN.')
