from pathlib import Path
import hashlib,json,os,socket,subprocess,tempfile
root=Path('/private/tmp/faction203-reference');binary=Path('/Users/cam/DEV/faction-bbs/target/release/examples/interop_probe')
checks=[]
def pipe(data,args=()):return subprocess.run([str(binary),*args],input=data,stdout=subprocess.PIPE,stderr=subprocess.PIPE,timeout=10)
r=pipe(b'new\n',['unexpected']);assert r.returncode!=0 and not r.stdout;checks.append('argv_refuses_before_protocol')
with open('/dev/null','rb') as f:
 r=subprocess.run([str(binary)],stdin=f,stdout=subprocess.PIPE,stderr=subprocess.PIPE,timeout=10)
 assert r.returncode!=0 and not r.stdout
checks.append('regular_input_refuses')
path=root/'private-output-control';fd=os.open(path,os.O_CREAT|os.O_EXCL|os.O_WRONLY,0o600)
try:
 with os.fdopen(fd,'wb') as f:
  r=subprocess.run([str(binary)],input=b'new\n',stdout=f,stderr=subprocess.PIPE,timeout=10)
 assert r.returncode!=0 and path.read_bytes()==b''
finally:path.unlink()
checks.append('regular_output_refuses')
r=pipe(b'new');assert r.returncode==0 and r.stdout==b'err\n';checks.append('incomplete_eof_err_and_clean_exit')
vector=b'convert d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a\n'
expected=b'ok d85e07ec22b0ad881537c2f44d662d1a143cf830c57aca4305d85c7a90f6b62e\n'
r=pipe(b'x'*(65568+3)+b'\n'+vector);assert r.returncode==0 and r.stdout==b'err\n'+expected;checks.append('oversized_line_drains_before_valid_successor')
r=pipe(b'x'*65568+b'\n'+vector);assert r.returncode==0 and r.stdout==b'err\n'+expected;checks.append('exact_line_cap_framing_preserves_successor')
r=pipe(b'\xff\n'+vector);assert r.returncode==0 and r.stdout==b'err\n'+expected;checks.append('non_utf8_refuses_without_desynchronizing')
left,right=socket.socketpair()
try:
 process=subprocess.Popen([str(binary)],stdin=right,stdout=right,stderr=subprocess.PIPE)
 right.close();left.settimeout(10);left.sendall(vector);left.shutdown(socket.SHUT_WR)
 output=b''
 while True:
  data=left.recv(4096)
  if not data:break
  output+=data
 assert process.wait(timeout=10)==0 and output==expected
 assert process.stderr.read()==b''
 process.stderr.close()
finally:left.close();right.close()
checks.append('duplicated_socket_stdio_accepts_private_protocol')
record={'schema':1,'result':'pass','count':len(checks),'checks':checks,'binary_sha256':hashlib.sha256(binary.read_bytes()).hexdigest(),'private_output_fixture_removed':not path.exists(),'no_private_values_in_report':True}
(root/'host-process-controls.json').write_text(json.dumps(record,indent=2)+'\n')
print('Product example process controls:',len(checks),'passed; private output fixture removed')
