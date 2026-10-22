#!/usr/bin/env python3
"""Independent stdlib mail/MIME parser control, not news-server interoperability."""
import argparse, email.policy, email.parser, email.utils, subprocess
from pathlib import Path

def check(binary):
    wire=subprocess.check_output([str(binary)],timeout=15)
    assert len(wire)<=24576
    article=email.parser.BytesParser(policy=email.policy.default).parsebytes(wire)
    assert not article.defects and all(not field.defects for field in article.values())
    assert str(article['Message-ID'])=='<synthetic@fixture.invalid>'
    assert str(article['Subject'])=='Midnight signal'
    assert email.utils.parseaddr(str(article['From']))[1]=='author@fixture.invalid'
    assert email.utils.parsedate_to_datetime(str(article['Date'])).isoformat()=='2026-10-21T18:00:00+00:00'
    assert str(article['Newsgroups'])=='faction.local.r1' and str(article['Path'])=='not-for-mail'
    assert str(article['References']).split()==['<root@fixture.invalid>','<parent@fixture.invalid>']
    assert article.get_content_type()=='text/plain' and article.get_content_charset()=='utf-8'
    assert article.get_payload(decode=True).decode('utf-8').removesuffix('\r\n').replace('\r\n','\n')=='Unicode: café ☃\n> reply\n'
    print('Independent Python email/MIME parser accepted synthetic headers, date, references and UTF-8 body; no news-server claim.')

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('binary',type=Path)
    check(p.parse_args().binary.resolve())
