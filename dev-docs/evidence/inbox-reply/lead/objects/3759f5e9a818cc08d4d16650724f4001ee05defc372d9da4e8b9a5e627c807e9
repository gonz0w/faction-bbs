#!/usr/bin/env python3
"""Owned disposable Telnet fixture, not a classic game. Binds loopback only.
No transcripts/credentials; --info contains ephemeral ports and check counters.
"""
import argparse
import json
import os
from pathlib import Path
import signal
import socket
import threading
import time


def run(info):
    listener=socket.socket();listener.bind(('127.0.0.1',0));listener.listen(8)
    refused=socket.socket();refused.bind(('127.0.0.1',0))
    lock=threading.Lock();state={'port':listener.getsockname()[1],'refused_port':refused.getsockname()[1],
        'pid':os.getpid(),'accepted':0,'closed':0,'terminal_ansi':0,'terminal_dumb':0,'naws':0,'queries_received':0}
    def save():
        tmp=info.with_suffix('.tmp');tmp.write_text(json.dumps(state));tmp.chmod(0o600);tmp.replace(info)
    def count(key):
        with lock:state[key]+=1;save()
    save()
    def serve(peer):
        count('accepted');peer.settimeout(120)
        try:
            for part in [b'\xff',b'\xfd\x18\xff\xfd',b'\x1f\xff\xfb\x01',b'\xff\xfa\x18\x01\xff\xf0']:
                peer.sendall(part);time.sleep(.005)
            for part in [b'\x1b[1;',b'35mOwned classic link fixture\x1b[0m\r\n',b'\xb0\xb1\xb2\xdb\r\n',
                b'\x1b]52;c;do-not-copy\x07\x1bPhidden-payload\x1b\\\x1b[6n',b'Game> ']:
                peer.sendall(part);time.sleep(.005)
            mode='data';command=0;sub=bytearray();line=bytearray()
            while True:
                data=peer.recv(256)
                if not data:return
                for b in data:
                    if mode=='iac':
                        if b in (251,252,253,254):mode='command';command=b
                        elif b==250:mode='sub';sub.clear()
                        else:mode='data'
                    elif mode=='command':mode='data'
                    elif mode=='sub':
                        if b==255:mode='sub-iac'
                        else:sub.append(b)
                        assert len(sub)<=64
                    elif mode=='sub-iac':
                        if b==240:
                            if sub==b'\x18\x00ANSI':count('terminal_ansi')
                            elif sub==b'\x18\x00DUMB':count('terminal_dumb')
                            elif sub and sub[0]==31:count('naws')
                            mode='data'
                        elif b==255:sub.append(b);mode='sub'
                        else:return
                    elif b==255:mode='iac'
                    elif b in (0,13):pass
                    elif b==10:
                        value=bytes(line).lower();line.clear()
                        if value==b'q':peer.sendall(b'Game goodbye.\r\n');return
                        if value==b'z':peer.sendall(b'\x1b]'+b'x'*1100);continue
                        if b'\x1b[' in value:count('queries_received')
                        peer.sendall(b'You made a move. No local BBS scores are submitted.\r\nGame> ')
                    elif len(line)<128:line.append(b)
                    else:return
        except (OSError,AssertionError):pass
        finally:peer.close();count('closed')
    signal.signal(signal.SIGTERM,lambda *_:os._exit(0))
    while True:
        peer,_=listener.accept();threading.Thread(target=serve,args=(peer,),daemon=True).start()


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--info',type=Path,required=True)
    a=p.parse_args();run(a.info)
