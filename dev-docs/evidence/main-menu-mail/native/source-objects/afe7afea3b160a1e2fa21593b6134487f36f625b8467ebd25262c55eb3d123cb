#!/usr/bin/env python3
"""Compare ARM64 Mach-O machine code across private fixture packaging/signing."""
import argparse,hashlib,json,struct
from pathlib import Path

def arm64_text(path):
    data=path.read_bytes();base=0
    if data[:4]==bytes.fromhex('cafebabe'):
        count=struct.unpack_from('>I',data,4)[0]
        for i in range(count):
            cpu,_,offset,_,_=struct.unpack_from('>IIIII',data,8+20*i)
            if cpu==0x0100000c:base=offset;break
        else:raise ValueError('no ARM64 executable')
    if data[base:base+4]!=bytes.fromhex('cffaedfe'):raise ValueError('expected ARM64 Mach-O')
    ncmd=struct.unpack_from('<I',data,base+16)[0];cursor=base+32
    for _ in range(ncmd):
        command,size=struct.unpack_from('<II',data,cursor)
        if command==0x19:
            sections=struct.unpack_from('<I',data,cursor+64)[0]
            for j in range(sections):
                section=cursor+72+80*j;name=data[section:section+16].split(b'\0')[0]
                if name==b'__text':
                    length=struct.unpack_from('<Q',data,section+40)[0];offset=struct.unpack_from('<I',data,section+48)[0]
                    return hashlib.sha256(data[base+offset:base+offset+length]).hexdigest()
        cursor+=size
    raise ValueError('no executable text section')
def compare(original,fixture):
    left=arm64_text(original);right=arm64_text(fixture)
    if left!=right:raise ValueError('fixture machine code differs')
    return {'arm64_text_sha256':left,'original_file_sha256':hashlib.sha256(original.read_bytes()).hexdigest(),'fixture_file_sha256':hashlib.sha256(fixture.read_bytes()).hexdigest(),'text_identical':True,'packaging_modified':True}
if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('original',type=Path);p.add_argument('fixture',type=Path);a=p.parse_args();print(json.dumps(compare(a.original,a.fixture),indent=2,sort_keys=True))
