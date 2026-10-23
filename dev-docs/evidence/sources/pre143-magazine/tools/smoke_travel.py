#!/usr/bin/env python3
"""Device-local travel UI on disposable PTYs; loopback trap proves no early connection."""
import argparse,hashlib,json,re,socket,subprocess,tempfile
from pathlib import Path
from smoke_board_launcher import LauncherConsole,PICK,RETURN,CONFIRM,catalog
PROMPT='Travel> '
def run(binary,output=None):
    checks=[];children=[]
    with tempfile.TemporaryDirectory(prefix='faction-travel-') as tmp,socket.socket() as trap:
        root=Path(tmp);root.chmod(0o700);trap.bind(('127.0.0.1',0));trap.listen();trap.settimeout(.2)
        def private(p,text):p.write_text(text);p.chmod(0o600)
        for name in ['identity','known_hosts','identity.pub']:private(root/name,'ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA' if name.endswith('.pub') else 'fixture')
        profile='version=1\nhost="127.0.0.1"\nport='+str(trap.getsockname()[1])+'\nfile_host="127.0.0.1"\nfile_port=2223\nidentity='+json.dumps(str(root/'identity'))+'\npublic_key='+json.dumps(str(root/'identity.pub'))+'\nknown_hosts='+json.dumps(str(root/'known_hosts'))+'\n'
        paths=[root/('board'+str(i)+'.toml') for i in range(16)]
        for p in paths:private(p,profile)
        pack=root/'boards.toml';private(pack,catalog(paths));before={p.name:hashlib.sha256(p.read_bytes()).hexdigest() for p in root.iterdir()}
        def start(extra=(),**kw):c=LauncherConsole([binary,'--boards',pack,*extra],**kw);children.append(c);c.expect(PICK);return c
        def command(c,v):c.send(v);c.expect(PROMPT)
        def visit(c,expected):c.raw(b'\r');c.expect(expected);c.expect(CONFIRM);c.send('n');c.expect(RETURN);c.send('');c.expect(PICK)
        def quit(c):c.raw(b'q');c.finish()
        try:
            c=start(columns=40,rows=18);c.raw(b'\x1b[B');c.expect('> [B] Local Board 2');c.raw(b't');c.expect(PROMPT);page=c.transcript.split(b'LOCAL TRAVEL / Before connecting')[-1];screen=re.sub(rb'\x1b\[[0-9;?]*[ -/]*[@-~]',b'',page).decode();assert len(screen.splitlines())<18 and all(len(v)<40 for v in screen.splitlines());command(c,'c');c.send('b');c.expect('Payload bytes [0-16777216]: ');c.send('1024');c.expect(PROMPT);c.send('s');c.expect('> [B] Local Board 2');c.expect(PICK);visit(c,'Travel: Compact text; per-file ceiling 1024 bytes');quit(c);checks.append('compact_local_panel_apply_preserves_highlight_and_fits_40x18')
            c=start();c.raw(b't');c.expect(PROMPT);command(c,'c');c.send('b');c.expect('Payload bytes [0-16777216]: ');c.send('0');c.expect(PROMPT);c.send('q');c.expect(PICK);visit(c,'Travel: Ordinary terminal; per-file ceiling 16777216 bytes');quit(c);checks.append('cancel_discards_draft_preferences_without_connecting')
            c=start(['--compact','--file-budget','2048']);c.raw(b't');c.expect(PROMPT);c.send('b');c.expect('Payload bytes [0-16777216]: ');c.send('999999999999');c.expect('Invalid ceiling; settings unchanged.');c.expect('Enter to continue: ');c.send('');c.expect(PROMPT);c.send('s');c.expect(PICK);visit(c,'Travel: Compact text; per-file ceiling 2048 bytes');quit(c);checks.append('invalid_limit_is_atomic_and_explicit_cli_preferences_are_reviewable')
            c=start();c.raw(b't');c.expect(PROMPT);command(c,'c');c.raw(b'\x03');c.expect(PICK);visit(c,'Travel: Ordinary terminal; per-file ceiling 16777216 bytes');quit(c);checks.append('ctrl_c_cancels_local_panel_and_preserves_ordinary_default')
            for flags in [['--compact','--kitty'],['--file-budget'],['--file-budget','-1'],['--file-budget','16777217'],['--compact','--compact'],['--file-budget','12','--file-budget','13']]:
                x=subprocess.run([str(binary),'--boards',str(pack),*flags],capture_output=True,timeout=5);assert x.returncode==1 and b'Boards on your device' not in x.stdout
            checks.append('invalid_or_conflicting_options_fail_before_terminal_or_network')
            after={p.name:hashlib.sha256(p.read_bytes()).hexdigest() for p in root.iterdir()};assert before==after
            try:conn,_=trap.accept()
            except TimeoutError:pass
            else:conn.close();raise AssertionError('travel preferences caused a network connection')
            checks.append('preferences_are_memory_only_no_key_profile_history_writes_or_early_connection')
        finally:
            for c in children:c.close()
    result=dict(result='pass',count=len(checks),checks=checks,capacity_claim=False,wire_budget_claim=False,offline_exchange=False,route_change=False)
    if output:output.write_text(json.dumps(result,indent=2,sort_keys=True)+'\n')
    print('Local travel passed:',len(checks),'controls.')
if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('binary',type=Path);p.add_argument('--output',type=Path);a=p.parse_args();run(a.binary.resolve(),a.output)
