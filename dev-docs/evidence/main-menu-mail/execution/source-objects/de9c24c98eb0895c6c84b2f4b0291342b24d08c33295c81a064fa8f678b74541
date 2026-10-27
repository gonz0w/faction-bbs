#!/usr/bin/env python3
"""Actual repeated-chat regression, disposable private NetBSD fixture only."""
import json,secrets,shlex,subprocess,time
from pathlib import Path
import argparse
from check_restricted_ssh import SSHConsole
parser=argparse.ArgumentParser(description='Three fresh-marker real SSH chat re-entry cycles in the marked private profiling fixture only.')
parser.add_argument('--disposable-fixture',action='store_true')
for flag in ['host','identity','admin-identity','known-hosts','secret-file','output']:parser.add_argument('--'+flag,required=True)
for flag in ['port','admin-port']:parser.add_argument('--'+flag,type=int,required=True)
args=parser.parse_args();assert args.disposable_fixture
assert Path(args.secret_file).stat().st_mode&0o077==0
secret=Path(args.secret_file).read_text().strip();assert len(secret)>=24
common=['/usr/bin/ssh','-F','/dev/null','-o','BatchMode=yes','-o','StrictHostKeyChecking=yes','-o','UserKnownHostsFile='+args.known_hosts,'-o','IdentitiesOnly=yes','-o','LogLevel=ERROR']
a=common+['-i',args.admin_identity,'-p',str(args.admin_port),'root@'+args.host]
def remote(cmd):
 x=subprocess.run(a+['set -e; PATH=/usr/sbin:/sbin:/usr/bin:/bin:/usr/pkg/bin; export PATH; '+cmd],capture_output=True,timeout=30);assert x.returncode==0,x.stderr.decode();return x.stdout.decode().strip()
remote('test -f /etc/faction-disposable-test; /etc/rc.d/faction_broker onestatus >/dev/null || /etc/rc.d/faction_broker onestart; /etc/rc.d/faction_sshd onestatus >/dev/null || /etc/rc.d/faction_sshd onestart; /etc/rc.d/faction_broker onestop')
binary=remote('sha256 -q /usr/local/libexec/shared-bbs');term=remote('id -u factionterm')
command=f'nohup /usr/local/libexec/shared-bbs --socket /var/run/faction/terminal.sock --terminal-uid {term} --state-dir /var/db/faction-profile >/home/factiond/profile-broker.log 2>&1 </dev/null &'
remote('su - factiond -c '+shlex.quote(command));time.sleep(.5);pid=remote('cat /var/run/faction/terminal.pid');assert pid.isdecimal()
argv=common+['-i',args.identity,'-p',str(args.port),'-tt','factionterm@'+args.host];children=[];passed=0;error=None
try:
 for n in range(2):
  child=SSHConsole(argv);children.append(child);child.expect('[L]ogin [R]egister [Q]uit: ');child.send('l');child.expect('Handle: ');child.send(f'profile{n:04}');child.expect('Password (hidden): ');child.send(secret);child.expect('Press Enter to continue: ');child.send('');child.expect('Ctrl-C exits; Esc: Enter to recover.')
 for cycle in range(3):
  for child in children:child.raw(b'c');child.expect('chat[#lobby]> ')
  marker='reentry '+secrets.token_hex(8);children[0].send(marker)
  for child in children:child.expect(marker)
  for child in children:child.send('/quit');child.expect('Chat closed.');child.expect('Ctrl-C exits; Esc: Enter to recover.')
  passed+=1
except AssertionError as exc:
 error=type(exc).__name__
finally:
 for child in children:
  assert secret.encode() not in child.transcript,'hidden password echoed'
  child.close()
 remote('kill -TERM '+pid)
 for _ in range(80):
  if subprocess.run(a+['kill -0 '+pid],capture_output=True,timeout=30).returncode!=0:break
  time.sleep(.1)
out={'binary_sha256':binary,'completed_cycles':passed,'requested_cycles':3,'result':'pass' if passed==3 else 'fail','exception':error,'fresh_unique_markers':True}
Path(args.output).write_text(json.dumps(out,indent=2)+'\n');print('Real SSH chat re-entry: '+str(passed)+'/3 fresh-marker cycles')
raise SystemExit(0 if passed==3 else 1)
