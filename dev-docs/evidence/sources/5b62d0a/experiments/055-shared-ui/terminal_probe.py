#!/usr/bin/env python3
"""Guest-only actual descriptor/slow-reader/connection-frame oracles. Private fixture."""
import array
import fcntl
import json
import os
import pty
import select
import socket
import struct
import sys
import termios
import time
from pathlib import Path


assert Path('/etc/faction-disposable-test').is_file()
secret=sys.stdin.buffer.read(129).decode();assert 24<=len(secret)<=128
import pwd
uid=pwd.getpwnam('factionterm').pw_uid
def connect():
    master,slave=pty.openpty();os.fchown(slave,uid,-1)
    fcntl.ioctl(slave,termios.TIOCSWINSZ,struct.pack('HHHH',24,80,0,0))
    control=socket.socket(socket.AF_UNIX);control.connect('/var/run/faction/terminal.sock')
    packet=b'FBSTTY01'+bytes([5])+b'xterm'+bytes(58)
    assert len(packet)==72
    control.sendmsg([packet],[(socket.SOL_SOCKET,socket.SCM_RIGHTS,array.array('i',[slave]))]);os.close(slave)
    return master,control
def expect(master,text,timeout=5):
    data=b'';end=time.monotonic()+timeout
    while text not in data:
        assert time.monotonic()<end,'terminal probe prompt timed out'
        if select.select([master],[],[],.1)[0]:
            part=os.read(master,8192);assert part,'terminal probe disconnected';data+=part
            assert len(data)<256*1024
    return data
def login(master):
    expect(master,b'[L]ogin [R]egister [Q]uit: ');os.write(master,b'l\n');expect(master,b'Handle: ')
    os.write(master,b'profile0000\n');expect(master,b'Password (hidden): ');os.write(master,secret.encode()+b'\n')
    data=expect(master,b'Press Enter to continue: ');assert secret.encode() not in data
    os.write(master,b'\n');expect(master,b'Ctrl-C exits; Esc: Enter to recover.')
results=[]
master,control=connect()
try:
    expect(master,b'[L]ogin [R]egister [Q]uit: ')
    control.sendall(b'{"session":1,"member":1,"role":"sysop","command":"export"}')
    control.settimeout(3);assert control.recv(1) in [b'',b'\x01']
    results.append({'oracle':'control bytes cannot import a session or owner command','result':'pass'})
finally:os.close(master);control.close()
time.sleep(.2)
master,control=connect()
try:
    login(master)
    # No reads from this master after login. Repeated legitimate room listings
    # fill the actual guest PTY output queue; input/output stay independently bounded.
    began=time.monotonic();control.setblocking(False);closed=False
    for _ in range(180):
        try:os.write(master,b'\x1b[B\x1b[A'*4)
        except OSError as e:
            # NetBSD may report terminal hangup before the control EOF becomes
            # readable. Require that independent close result before accepting it.
            assert e.errno==5
            control.setblocking(True);control.settimeout(2)
            assert control.recv(1) in [b'',b'\x01'];closed=True;break
        try:
            if control.recv(1) in [b'',b'\x01']:closed=True;break
        except BlockingIOError:pass
        time.sleep(.05)
    assert closed,'slow reader not disconnected by output deadline'
    elapsed=time.monotonic()-began;assert elapsed<12
    results.append({'oracle':'non-reading guest PTY fills and disconnects within 12 seconds','result':'pass','seconds':round(elapsed,3)})
finally:os.close(master);control.close()
master,control=connect()
try:
    login(master);os.write(master,b'q');expect(master,b'Logged out.')
    control.settimeout(3);assert control.recv(1)==b'\x00'
    results.append({'oracle':'fresh approved connection after forged control and slow reader','result':'pass'})
finally:os.close(master);control.close()
print(json.dumps({'results':results}))
