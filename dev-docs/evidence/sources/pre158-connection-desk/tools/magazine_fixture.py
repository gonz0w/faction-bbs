"""Original bounded metadata fixtures; no network, host files or authority."""
import json

def pack(namespace,private=False):
    source='version=1\nenabled=true\nname="Community library"\ncompact=false\nmagazine_name="Nightwire magazine"\nshelf=[]\nbook=[]\n'
    for n,state in [(1,'published'),(2,'draft'),(3,'withdrawn')]:
        source+='\n[[magazine]]\nid="signal-'+str(n)+'"\nnumber='+str(n)+'\nrevision=1\nstate="'+state+'"\ntitle="'+('Visible local edition' if n==1 else 'HIDDEN-EDITION-'+str(n))+'"\neditor="Synthetic editor"\nreleased="Autumn 2026"\nschedule="Whenever ready"\n'
        if n==1:source+='room=1\ndownload="faction-file:'+namespace+'/'+f'{7 if private else 2:032x}'+'"\n'
        for i in range(1,6):
            body=('ORIGINAL-ARTICLE-START\n'+''.join('Line '+str(j)+' brings the carrier home.\n' for j in range(1,40))+'ORIGINAL-ARTICLE-END') if n==1 else 'HIDDEN-ARTICLE-BODY'
            values=dict(id='article-'+str(i),title='Original article '+str(i),section='Field notes',author='Fixture author',rights='BSD-3-Clause original synthetic text',provenance='Written by fixture author',consent='board-local' if state=='published' else 'pending',body=body)
            source+='\n[[magazine.article]]\n'+''.join(k+'='+json.dumps(v)+'\n' for k,v in values.items())
    return source
