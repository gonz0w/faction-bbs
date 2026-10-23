#!/usr/bin/env python3
"""Build original Nightwire CP437 artwork. Python stdlib, offline authoring only.

The skyline, radio/receiver and beveled/slanted letter polygons are authored here;
no font generator, archival image, image conversion or runtime script is used.
Output is original Codex-assisted BSD-3-Clause artwork, pending owner art review.
"""
import argparse
import struct
from pathlib import Path

ROOT=Path(__file__).resolve().parents[1]
W,H=80,48
# Original glyph outlines, in a 12-pixel-tall design space. Each tuple contains
# width, silhouette and counter (negative-space cutout), not a reusable font.
LETTERS=[
 (8,[(0,0),(8,0),(7,3),(3,3),(3,5),(7,5),(6,7),(3,7),(3,11),(0,12)],[]),
 (8,[(0,12),(2,2),(4,0),(8,0),(8,12),(5,12),(5,8),(3,8),(2,12)],[(4,3),(5,3),(5,6),(3,6)]),
 (8,[(2,0),(8,0),(7,3),(4,3),(3,9),(7,9),(6,12),(1,12),(0,10),(1,2)],[]),
 (9,[(0,0),(9,0),(8,3),(6,3),(5,12),(2,12),(3,3),(0,3)],[]),
 (7,[(1,0),(7,0),(6,2),(5,2),(4,10),(6,10),(5,12),(0,12),(1,10),(2,10),(3,2),(1,2)],[]),
 (9,[(2,0),(7,0),(9,2),(8,10),(6,12),(1,12),(0,10),(1,2)],[(4,3),(6,3),(5,9),(3,9)]),
 (10,[(0,12),(1,0),(4,0),(6,7),(7,0),(10,0),(8,12),(5,12),(3,5),(2,12)],[]),
]

def inside(x,y,points):
    result=False
    for (ax,ay),(bx,by) in zip(points,points[1:]+points[:1]):
        if (ay>y)!=(by>y) and x<(bx-ax)*(y-ay)/(by-ay)+ax:result=not result
    return result

class Canvas:
    def __init__(self):self.cells=[[(' ',7,0) for _ in range(W)] for _ in range(H)]
    def put(self,x,y,text,fg=7,bg=0):
        assert 0<=y<H and 0<=x and x+len(text)<=W
        text.encode('cp437')
        for i,c in enumerate(text):self.cells[y][x+i]=(c,fg,bg)
    def line(self,x1,y1,x2,y2,glyph,fg):
        steps=max(abs(x2-x1),abs(y2-y1))
        for n in range(steps+1):
            self.put(x1+(x2-x1)*n//max(1,steps),y1+(y2-y1)*n//max(1,steps),glyph,fg)
    def center(self,y,text,fg):self.put((W-len(text))//2,y,text,fg)

def skyline(c,y,deep=False):
    # Staggered silhouettes, sparse windows and roof hardware rather than a frame.
    buildings=[(3,7,4),(11,8,6),(20,6,3),(28,9,5),(39,7,7),(48,9,4),(59,6,6),(68,9,3)]
    for x,width,height in buildings:
        top=y+7-height
        for yy in range(top,y+8):
            c.put(x,yy,('▒' if deep else '░')*width,4)
            c.put(x+width-1,yy,'▐',8)
            if yy>top:
                for xx in range(x+1,x+width-1,3):
                    if (xx+yy)%3:c.put(xx,yy,'■',11 if (xx+yy)%4==0 else 6)
        c.put(x,top,'▄'*width,8)
        if height>4:c.put(x+width//2,top-1,'┴',8)
    c.line(1,y+8,78,y+8,'▄',4)

def logo(c):
    pixels={};x=4
    for width,shape,hole in LETTERS:
        for yy in range(12):
            for xx in range(width):
                if inside(xx+.5,yy+.5,shape) and not (hole and inside(xx+.5,yy+.5,hole)):
                    pixels[x+xx+(11-yy)//4,yy]=15 if yy<2 else 14 if yy<6 else 6 if yy<9 else 13
        x+=width+2
    shadow={(xx+1,yy+2):4 for xx,yy in pixels};shadow.update(pixels)
    assert max(xx for xx,yy in shadow)<79
    for yy in range(0,14,2):
        for xx in range(1,79):
            upper=shadow.get((xx,yy),0);lower=shadow.get((xx,yy+1),0)
            if upper and lower:
                c.put(xx,7+yy//2,'█' if upper==lower else '▀',upper,0 if upper==lower else lower)
            elif upper:c.put(xx,7+yy//2,'▀',upper)
            elif lower:c.put(xx,7+yy//2,'▄',lower)

def artwork():
    c=Canvas();c.put(3,0,'NIGHTWIRE / 01',8);c.put(52,0,'AFTER HOURS. ALWAYS LOCAL.',8)
    # Cover: distant rooftops, moon and radio mast, with room for a custom wordmark.
    for x,y in [(6,1),(24,2),(40,1),(52,3),(75,2)]:c.put(x,y,'·',8)
    c.put(65,1,'▄██▄',8);c.put(64,2,'▐████▌',7);c.put(65,3,'▀██▀',8)
    for x,width,top in [(4,8,4),(14,7,3),(23,10,4),(36,6,2),(44,9,4),(57,5,3),(70,6,4)]:
        for y in range(top,6):
            c.put(x,y,'▒'*width,4)
            if y>top:
                for wx in range(x+1,x+width-1,3):c.put(wx,y,'■',6)
        c.put(x,top,'▄'*width,8)
    c.line(38,0,38,5,'│',6);c.put(37,0,'─┼─',14);c.put(34,1,'·  │  ·',6)
    c.line(2,6,77,6,'─',4);logo(c)
    c.center(14,'PRIVATE COMMUNITIES / SHARED FREQUENCIES',7)
    c.put(3,15,'// FACTION BBS',13);c.put(43,15,'original cells / faction 2026',8)
    # Middle page: layered city, cables and a descending perspective street.
    c.put(3,16,'NIGHTWIRE / 02',8);c.put(51,16,'FIND YOUR PEOPLE.',14)
    for x,y in [(5,18),(24,17),(46,18),(71,18)]:c.put(x,y,'·',8)
    skyline(c,18,True)
    c.line(8,17,50,22,'─',8);c.line(51,22,75,19,'─',8)
    c.put(17,21,'o',13);c.put(44,23,'o',13)
    for y in range(27,30):
        spread=(y-26)*5;c.put(40-spread,y,'/',8);c.put(40+spread,y,'\\',8)
        c.put(39,y,'│',5);c.put(41,y,'│',5)
    c.center(30,'ROOMS / FILES / LATE-NIGHT CHAT / DOORS',7)
    c.put(3,31,'// A SMALLER PLACE ON A VERY LARGE NETWORK',6)
    # Last page: original receiver silhouette, hand-set dials and signal trace.
    c.put(3,32,'NIGHTWIRE / 03',8);c.put(47,32,'LEAVE A SIGNAL.',14)
    c.put(70,33,'·',14);c.put(68,34,'/',8)
    c.put(8,35,'▄'*63,8)
    for y in range(36,43):c.put(7,y,'▐',8);c.put(71,y,'▌',8)
    c.put(10,37,'FACTION   /   COMMUNITY RECEIVER',7)
    c.put(10,38,'┌'+'─'*38+'┐',6);c.put(10,39,'│'+' '*38+'│',6);c.put(10,40,'└'+'─'*38+'┘',6)
    c.put(12,39,'_ /\\ __ /\\____/\\_ /\\___/\\__ /\\_',14)
    for x,color in [(55,13),(63,6)]:
        c.put(x+1,37,'▄██▄',8);c.put(x,38,'▐░▒▒░▌',color);c.put(x+1,39,'▀██▀',8)
    c.put(56,41,'TUNE',8);c.put(64,41,'GAIN',8)
    c.put(10,42,'APPROVED MEMBERS. INDEPENDENT BOARDS.',7)
    c.put(7,43,'▀'*65,8)
    c.center(45,'KEEP THE CARRIER.',13)
    c.put(3,47,'// YOUR BOARD. YOUR RULES.',6);c.put(44,47,'original cells / faction 2026',8)
    return c

def encoded(c,first,last,palette):
    body=bytearray();current=None
    for row in range(first,last):
        cells=c.cells[row];end=max((x+1 for x,cell in enumerate(cells) if cell!=(' ',7,0)),default=0)
        if not end:continue
        body.extend(f'\x1b[{row-first+1};1H'.encode())
        for glyph,fg,bg in cells[:end]:
            style=palette.get(fg,fg),palette.get(bg,bg)
            if style!=current:
                f,b=style;body.extend(f'\x1b[{30+f if f<8 else 90+f-8};{40+b if b<8 else 100+b-8}m'.encode());current=style
            body.extend(glyph.encode('cp437'))
    body.extend(b'\x1b[0m');record=bytearray(128);record[:7]=b'SAUCE00';record[7:42]=b'Nightwire / original Faction art'.ljust(35);record[42:62]=b'Faction / Codex'.ljust(20);record[62:82]=b'Original / BSD-3'.ljust(20);record[82:90]=b'20261022';record[90:94]=struct.pack('<I',len(body));record[94:96]=bytes([1,1]);record[96:98]=struct.pack('<H',80);record[98:100]=struct.pack('<H',last-first);record[105]=1
    return bytes(body)+b'\x1a'+record

def outputs():
    c=artwork();result={};base=(ROOT/'themes/grid/theme.toml').read_text(encoding='utf-8')
    for folder,season,short,palette in [
        ('nightwire',None,False,{}),('nightwire-short',None,True,{}),
        ('nightwire-seasons/autumn','AUTUMN',False,{14:11,6:3,13:9,5:1}),
        ('nightwire-seasons/winter','WINTER',False,{13:12,5:4,6:6,14:14}),
    ]:
        ansi=encoded(c,0,16 if short else 48,palette)
        ascii_card="    __ / F A C T I O N / __\n .-'/  N I G H T W I R E  /-.\n  '  private frequencies   '\n  rooms / files / chat / doors\n"
        if season:ascii_card+=f'          // {season} //\n'
        identity="identity_ansi='login.ans'" if short else "identity_ansi_long='login.ans'"
        if season:
            source=f"version=1\nid='nightwire-{season.lower()}'\nwelcome='{season} / independent frequencies.'\nnormal={{fg=7,bg=0}}\nfocus={{fg=15,bg=4,bold=true}}\nhotkey={{fg={9 if season=='AUTUMN' else 14},bg=0,bold=true}}\ndisabled={{fg=8,bg=0}}\nidentity_ascii='login.txt'\n{identity}\n"
        else:
            source=base.replace("id='faction-grid'",f"id='{folder}'").replace("menu_title='SIGNAL / CONTROL'","menu_title='AFTERHOURS / MAIN'")
            source=source[:source.index("creator=")]+source[source.index('\n[[menu]]'):]
            source=source.replace("identity_ansi='login.ans'",identity)
        provenance="creator='Faction contributors; Codex-assisted'\nsource='Original Nightwire scene; tools/build_nightwire.py'\npermission='BSD-3-Clause; root LICENSE'\n"
        if '\n[[menu]]' in source:source=source.replace('\n[[menu]]','\n'+provenance+'\n[[menu]]',1)
        else:source+=provenance
        for name,data in [('theme.toml',source.encode()),('login.txt',ascii_card.encode()),('login.ans',ansi)]:result['themes/'+folder+'/'+name]=data
    return result

def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--check',action='store_true');a=p.parse_args()
    result=outputs()
    for name,data in result.items():
        path=ROOT/name
        if a.check:
            if not path.exists() or path.read_bytes()!=data:raise SystemExit('Original artwork needs regeneration: '+name)
        else:path.parent.mkdir(parents=True,exist_ok=True);path.write_bytes(data)
    print(f'Original Nightwire artwork {"matches" if a.check else "built"}: {len(result)} files; no runtime generator or borrowed art.')

if __name__=='__main__':main()
