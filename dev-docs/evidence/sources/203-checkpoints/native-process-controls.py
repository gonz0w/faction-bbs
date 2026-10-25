import json,sys,subprocess
from pathlib import Path
sys.path.insert(0,'/private/tmp');from faction203_common import remote,p,a
binary='/home/factionbuild/target203/release/examples/interop_probe'
base=a+['exec su -m factionbuild -c "exec '+binary+'"']
checks=[]
def run(label,args,data,expected,limit=20):
 r=subprocess.run(args,input=data,capture_output=True,timeout=limit)
 assert expected(r),label
 checks.append(label)
run('native_eof_clean_exit',base,b'',lambda r:r.returncode==0 and r.stdout==b'')
run('native_malformed_command_is_inert',base,b'not-a-command\n',lambda r:r.returncode==0 and r.stdout==b'err\n')
run('native_missing_final_newline_refuses',base,b'new',lambda r:r.returncode==0 and r.stdout==b'err\n')
run('native_oversize_line_drains_then_recovers',base,b'x'*70000+b'\nnot-a-command\n',lambda r:r.returncode==0 and r.stdout==b'err\nerr\n')
run('native_argv_rejects_before_state',a+['exec su -m factionbuild -c "exec '+binary+' unexpected"'],b'',lambda r:r.returncode!=0 and not r.stdout)
remote('su -m factionbuild -c "'+binary+' > /home/factionbuild/faction203/blocked-output"',timeout=10) if False else None
r=subprocess.run(a+['exec su -m factionbuild -c "'+binary+' > /home/factionbuild/faction203/blocked-output"'],input=b'new\n',capture_output=True,timeout=20)
assert r.returncode!=0 and remote('stat -f %z /home/factionbuild/faction203/blocked-output').strip()==b'0'
checks.append('native_regular_file_output_rejects_before_secrets')
r=subprocess.run(a+['exec su -m factionbuild -c "'+binary+' < /dev/null"'],capture_output=True,timeout=20)
assert r.returncode!=0 and not r.stdout
checks.append('native_nonpipe_input_rejects_before_state')
root=Path('/private/tmp/faction203-owned');(root/'native-cli-controls.json').write_text(json.dumps({'result':'pass','checks':checks,'count':len(checks)},indent=2)+'\n')
print('Native actual CLI controls passed:',len(checks))
