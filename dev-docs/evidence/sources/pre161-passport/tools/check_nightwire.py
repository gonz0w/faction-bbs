#!/usr/bin/env python3
"""Validate original Nightwire assets through the built product and cell oracle.

Offline Python stdlib. Optional --write-previews writes only the seven declared
new authoring SVGs. These are reference-cell renders, not terminal screenshots.
"""
import argparse
import json
import subprocess
import tomllib
from pathlib import Path
from render_theme_preview import cells,svg
from build_nightwire import outputs

ROOT=Path(__file__).resolve().parents[1]
PACKS=[('long','nightwire',None,3),('short','nightwire-short',None,1),('autumn','nightwire','nightwire-seasons/autumn',3),('winter','nightwire','nightwire-seasons/winter',3)]
PREVIEWS={'nightwire-cover.svg':('long',80,1),'nightwire-city.svg':('long',80,2),'nightwire-receiver.svg':('long',80,3),'nightwire-compact.svg':('long',40,1),'nightwire-autumn.svg':('autumn',80,1),'nightwire-winter.svg':('winter',80,1),'nightwire-menu.svg':('menu',80,1)}

def check(a):
    expected=outputs()
    for name,data in expected.items():assert (ROOT/name).read_bytes()==data,'artwork not regenerated: '+name
    base=tomllib.loads((ROOT/'themes/grid/theme.toml').read_text(encoding='utf-8'))['menu']
    frames={};art={};profiles=[]
    def preview(pack,season,columns=80,page=1,plain=False,login=True):
        args=[str(a.binary),'--theme',str(ROOT/'themes'/pack),'--columns',str(columns)]
        if season:args+=['--season',str(ROOT/'themes'/season)]
        if login:args+=['--login']
        if columns>=80 and not plain and login:args+=['--page',str(page)]
        if plain:args+=['--plain']
        result=subprocess.run(args,capture_output=True,check=True,timeout=10)
        assert len(result.stdout)<=(17408 if login else 4096)
        return result.stdout
    for name,pack,season,count in PACKS:
        theme=tomllib.loads((ROOT/'themes'/pack/'theme.toml').read_text(encoding='utf-8'));assert theme['menu']==base,'art cannot remap menu actions/keys/positions'
        for page in range(1,count+1):
            raw=preview(pack,season,page=page);grid=cells(raw.decode(),80,24,True)
            frames[name,80,page]=grid;art[name,page]=grid[:16]
            visible=[c for row in grid[:16] for c in row if c[0]!=' ']
            assert len(visible)>200 and all(c[1]!=c[2] for c in visible),'empty/invisible artwork'
            assert b'SAUCE' not in raw and b'\x1b[5m' not in raw
            profiles.append({'pack':name,'mode':'ansi','page':page,'bytes':len(raw)})
        raw=preview(pack,season,columns=40);grid=cells(raw.decode(),40,18,True);frames[name,40,1]=grid
        assert b'Art page ' not in raw
        profiles.append({'pack':name,'mode':'compact','bytes':len(raw)})
        raw=preview(pack,season,plain=True);assert b'\x1b' not in raw and b'F A C T I O N' in raw
        assert all(len(line)<40 for line in (ROOT/'themes'/ (season or pack)/'login.txt').read_text(encoding='utf-8').splitlines())
        profiles.append({'pack':name,'mode':'plain','bytes':len(raw)})
    assert art['long',1]==art['short',1],'short identity must be the complete cover'
    for season in ['autumn','winter']:
        for page in range(1,4):
            assert [[c[0] for c in row] for row in art[season,page]]==[[c[0] for c in row] for row in art['long',page]],'season changed composition'
        assert art[season,1]!=art['long',1],'season lacks its coordinated palette'
    frames['menu',80,1]=cells(preview('nightwire',None,login=False).decode(),80)
    for filename,key in PREVIEWS.items():
        document=svg(frames[key],login=key[0]!='menu',paged=key[0]!='menu' and key[1]>=80)
        import re
        document=re.sub(r'<title id="title">.*?</title>','<title id="title">Faction BBS / original Nightwire artwork</title>',document)
        document=re.sub(r'<desc id="description">.*?</desc>','<desc id="description">Original Codex-assisted CP437 artwork. Reference-palette cells; not an emulator/font screenshot or owner acceptance.</desc>',document)
        path=ROOT/'docs/images'/filename
        if a.write_previews:path.write_text(document,encoding='utf-8')
        else:assert path.read_text(encoding='utf-8')==document,'preview not regenerated: '+filename
    if a.output:a.output.write_text(json.dumps({'result':'pass','preview_profiles':profiles,'profile_count':len(profiles),'short_cover_matches':True,'seasonal_geometry_matches':True,'menu_map_unchanged':True,'capacity_claim':False},indent=2)+'\n')
    print(f'Nightwire artwork passed {len(profiles)} product-preview profiles, short-cover/season geometry, menu-map and deterministic source/SVG checks.')

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('binary',type=Path);p.add_argument('--write-previews',action='store_true');p.add_argument('--output',type=Path)
    a=p.parse_args();a.binary=a.binary.resolve(strict=True);check(a)
