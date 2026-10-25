#!/usr/bin/env python3
"""Build original Nightwire CP437 artwork. Python stdlib, offline authoring only.

The angular letterforms, cut-metal sigil and abstract signal cuts are authored here;
no font generator, archival image, image conversion or runtime script is used.
Output is original Codex-assisted BSD-3-Clause artwork, pending owner art review.
"""
import argparse
import struct
from pathlib import Path

ROOT=Path(__file__).resolve().parents[1]
W,H=80,48
# Bespoke angular letter outlines: 18 half-cell pixels tall, cut-back stems,
# pointed terminals and asymmetric counters. These are artwork, not a font.
LETTERS=[
 (9,[(0,3),(3,0),(10,0),(8,3),(4,3),(3,7),(8,7),(6,10),(3,10),(2,16),(0,18)],[]),
 (10,[(0,18),(3,3),(6,0),(10,0),(9,18),(6,15),(6,11),(3,11),(2,16)],[(5,4),(7,3),(6,8),(4,8)]),
 (9,[(0,14),(2,3),(5,0),(10,0),(8,3),(5,4),(3,13),(7,14),(8,12),(7,17),(3,18)],[]),
 (10,[(0,3),(3,0),(11,0),(9,3),(6,4),(4,16),(1,18),(3,4)],[]),
 (6,[(1,3),(3,0),(7,0),(5,3),(4,14),(6,14),(4,17),(0,18),(2,14),(3,3)],[]),
 (10,[(0,14),(2,3),(5,0),(9,0),(11,3),(9,15),(6,18),(2,18)],[(4,5),(7,3),(8,4),(6,13),(3,15)]),
 (11,[(0,18),(2,2),(5,0),(7,10),(9,2),(12,0),(10,16),(7,18),(4,8),(3,15)],[]),
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

def paint_pixels(c,pixels,top):
    """Pack two design pixels into one CP437 cell without changing its width."""
    last=max(y for x,y in pixels)
    for yy in range(0,last+1,2):
        for xx in range(1,79):
            upper=pixels.get((xx,yy),0);lower=pixels.get((xx,yy+1),0)
            if upper and lower:
                c.put(xx,top+yy//2,'█' if upper==lower else '▀',upper,0 if upper==lower else lower)
            elif upper:c.put(xx,top+yy//2,'▀',upper)
            elif lower:c.put(xx,top+yy//2,'▄',lower)

def logo(c):
    face=set();x=3
    for width,shape,hole in LETTERS:
        for yy in range(18):
            for xx in range(width+1):
                if inside(xx+.5,yy+.5,shape) and not (hole and inside(xx+.5,yy+.5,hole)):
                    face.add((x+xx+(17-yy)//6,yy))
        x+=width+1
    pixels={(xx+1,yy+1):1 for xx,yy in face}
    for xx,yy in face:
        # Cold metal at the cut edge, charcoal face, dark red recessed facets.
        edge=(xx-1,yy) not in face or (xx,yy-1) not in face
        recess=(xx+1,yy) not in face and yy>4
        pixels[xx,yy]=7 if edge and yy<12 else 1 if recess else 8
    assert max(xx for xx,yy in pixels)<79
    paint_pixels(c,pixels,3)
    # Fine slashes, teeth and scored facets break up the solid silhouette.
    for x,y in [(6,6),(17,10),(26,9),(40,6),(51,9),(65,8)]:
        c.put(x,y,'/',7)
    for x,y in [(4,9),(15,11),(28,10),(42,4),(54,11),(69,6)]:
        c.put(x,y,'░',8)
    c.put(1,11,'─<',1);c.put(75,4,'>─',1)

def blades(c,top):
    """Original split-wing emblem: diagonal facets, no landscape silhouettes."""
    shapes=[([(4,8),(30,0),(24,7),(36,4),(27,13),(33,13),(22,21),(26,13),(17,16),(23,8)],8),
            ([(76,8),(50,0),(56,7),(44,4),(53,13),(47,13),(58,21),(54,13),(63,16),(57,8)],8),
            ([(40,1),(44,7),(40,19),(36,7)],1)]
    pixels={}
    for points,color in shapes:
        for y in range(22):
            for x in range(1,79):
                if inside(x+.5,y+.5,points):
                    pixels[x,y]=7 if not inside(x+.5,y-.5,points) else color
    paint_pixels(c,pixels,top)

def artwork():
    c=Canvas()
    # Cover: wordmark first. Sparse cuts and microtype frame the steel/red face.
    c.put(3,0,'f / /',1);c.put(11,0,'F A C T I O N',7);c.put(50,0,'NIGHTWIRE / 01 . DARK SIGNAL',8)
    c.put(3,1,'───────\\',8);c.put(12,1,'independent boards / after hours',8)
    c.put(66,1,'/───────',1)
    logo(c)
    c.put(3,13,'▀▀──',1);c.put(9,13,'B B S',7);c.put(60,13,'──░▒▓█▓▒░──',1)
    c.put(9,14,'PRIVATE COMMUNITIES  /  SHARED FREQUENCIES',8)
    c.put(3,15,'[ original cells / faction ]',8);c.put(58,15,'EST. 2026 / STAY LOCAL',8)
    # Middle page: a cut-metal split-wing sigil with a restrained red core.
    c.put(3,16,'NIGHTWIRE / 02 . UNDER THE SURFACE',8);c.put(57,16,'NO ALGORITHM. PEOPLE.',7)
    blades(c,18)
    c.center(29,'R O O M S  /  F I L E S  /  C H A T  /  D O O R S',7)
    c.center(30,'find your people. keep your own frequencies.',8)
    c.put(3,31,'──\\',1);c.put(7,31,'INDEPENDENT BY DESIGN',8);c.put(66,31,'/── // f',1)
    # Last page: asymmetric NFO-style signal cuts, not a simulated object.
    c.put(3,32,'NIGHTWIRE / 03 . KEEP THE CARRIER',8);c.put(55,32,'YOUR BOARD. YOUR RULES.',7)
    for i in range(8):
        x=8+i*2;y=35+i
        c.put(x,y,'\\',7 if i<2 else 8)
        c.put(x+1,y,'▓▒░',1)
        c.put(73-i*2,y,'/',8)
        c.put(70-i*2,y,'░▒▓',4)
    c.put(27,35,'[ f / / ]',1)
    c.put(27,37,'LOCAL ROOMS. REAL PEOPLE.',7)
    c.put(27,39,'FILES WORTH KEEPING.',8)
    c.put(27,41,'LATE NIGHTS. OPEN DOORS.',7)
    c.center(44,'a smaller place on a very large network.',8)
    c.put(3,46,'───\\',1);c.put(9,46,'F A C T I O N  /  B B S',7);c.put(58,46,'/─── NIGHTWIRE //',8)
    c.put(3,47,'original / BSD-3-Clause',8);c.put(54,47,'approved members. always.',8)
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
    body.extend(b'\x1b[0m');record=bytearray(128);record[:7]=b'SAUCE00';record[7:42]=b'Nightwire / original Faction art'.ljust(35);record[42:62]=b'Faction / Codex'.ljust(20);record[62:82]=b'Original / BSD-3'.ljust(20);record[82:90]=b'20261024';record[90:94]=struct.pack('<I',len(body));record[94:96]=bytes([1,1]);record[96:98]=struct.pack('<H',80);record[98:100]=struct.pack('<H',last-first);record[105]=1
    return bytes(body)+b'\x1a'+record

def outputs():
    c=artwork();result={};base=(ROOT/'themes/grid/theme.toml').read_text(encoding='utf-8')
    for folder,season,short,palette in [
        ('nightwire',None,False,{}),('nightwire-short',None,True,{}),
        ('nightwire-seasons/autumn','AUTUMN',False,{1:3,4:1}),
        ('nightwire-seasons/winter','WINTER',False,{1:4,4:1}),
    ]:
        ansi=encoded(c,0,16 if short else 48,palette)
        ascii_card="  / /  F A C T I O N  / /\n  --< N I G H T W I R E >--\n     independent frequencies\n  rooms / files / chat / doors\n"
        if season:ascii_card+=f'          // {season} //\n'
        identity="identity_ansi='login.ans'" if short else "identity_ansi_long='login.ans'"
        if season:
            source=f"version=1\nid='nightwire-{season.lower()}'\nwelcome='{season} / independent frequencies.'\nnormal={{fg=7,bg=0}}\nfocus={{fg=15,bg=4,bold=true}}\nhotkey={{fg={3 if season=='AUTUMN' else 4},bg=0,bold=true}}\ndisabled={{fg=8,bg=0}}\nidentity_ascii='login.txt'\n{identity}\n"
        else:
            source=base.replace("id='faction-grid'",f"id='{folder}'").replace("menu_title='SIGNAL / CONTROL'","menu_title='AFTERHOURS / MAIN'")
            source=source[:source.index("creator=")]+source[source.index('\n[[menu]]'):]
            source=source.replace("identity_ansi='login.ans'",identity)
            source=source.replace('hotkey={fg=13,','hotkey={fg=1,')
        provenance="creator='Faction contributors; Codex-assisted'\nsource='Original Nightwire dark signal; tools/build_nightwire.py'\npermission='BSD-3-Clause; root LICENSE'\n"
        if '\n[[menu]]' in source:source=source.replace('\n[[menu]]','\n'+provenance+'\n[[menu]]',1)
        else:source+=provenance
        for name,data in [('theme.toml',source.encode()),('login.txt',ascii_card.encode()),('login.ans',ansi)]:result['themes/'+folder+'/'+name]=data
    for work,folder in [('nightwire','nightwire'),('autumn','nightwire-seasons/autumn'),('winter','nightwire-seasons/winter')]:
        for ext,name in [('ans','login.ans'),('txt','login.txt')]:
            result[f'gallery/nightwire/{work}.{ext}']=result[f'themes/{folder}/{name}']
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
