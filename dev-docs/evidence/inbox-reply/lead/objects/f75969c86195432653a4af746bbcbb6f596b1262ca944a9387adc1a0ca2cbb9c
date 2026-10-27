#!/usr/bin/env python3
"""Original INN offline checks against a temporary loopback refusal fixture.

-D still probes NNTP in INN 2.7.4. The owned fixture sends only a 400 refusal and
checks that the client sends no commands. Never use posting mode or a real board.
"""
import argparse, hashlib, json, os, socket, subprocess, tempfile, threading
from pathlib import Path
from build_inn_reference import SHA256, VERSION, runtime_artifacts

def check(reference, examples, output=None):
    provenance=json.loads((reference/'reference.json').read_text())
    binary=reference/('inn-'+VERSION)/'frontends/inews'
    assert provenance['archive_sha256']==SHA256 and provenance['version']==VERSION
    assert hashlib.sha256(binary.read_bytes()).hexdigest()==provenance['binary_sha256']
    assert runtime_artifacts(binary.parent.parent)==provenance['runtime_artifacts']
    cases=[]
    def canonical(wire):
        r=subprocess.run([str(examples/'canonicalize')],input=wire,capture_output=True,timeout=15)
        assert r.returncode==0,'Faction canonical parse failed'
        return r.stdout
    with tempfile.TemporaryDirectory(prefix='faction-inews-') as temp:
        root=Path(temp);root.chmod(0o700)
        config=root/'inn.conf'
        config.write_text('mta: /usr/bin/false\npathhost: fixture.invalid\ndomain: fixture.invalid\npathnews: '+str(root)+'\norganization: "Synthetic validation"\nserver: 127.0.0.1\nhismethod: hisv6\nenableoverview: false\n')
        env={'PATH':os.environ.get('PATH','/usr/bin:/bin'),'HOME':str(root),'LC_ALL':'C','INNCONF':str(config),'NNTPSERVER':'127.0.0.1','INN_HOSTNAME':'fixture.invalid'}
        def inn(wire):
            assert len(wire)<=24576
            # Keep this owned port alive for the whole child lifetime. Refuse
            # before auth/posting and prove that no application commands arrive.
            with socket.socket(socket.AF_INET,socket.SOCK_STREAM) as guard:
                guard.bind(('127.0.0.1',0));guard.listen(1);guard.settimeout(10)
                port=guard.getsockname()[1];observed=[]
                def refuse():
                    try:
                        connection,_=guard.accept()
                        with connection:
                            connection.settimeout(5)
                            connection.sendall(b'400 Faction synthetic offline refusal\r\n')
                            connection.shutdown(socket.SHUT_WR)
                            assert connection.recv(512)==b'', 'unexpected client command'
                        observed.append('refused_without_commands')
                    except Exception as error:observed.append(type(error).__name__)
                thread=threading.Thread(target=refuse,daemon=True);thread.start()
                try:
                    result=subprocess.run([str(binary),'-h','-D','-P','-S','-O','-p',str(port)],input=wire.replace(b'\r\n',b'\n'),env=env,capture_output=True,timeout=15)
                finally:
                    thread.join(timeout=11)
                assert not thread.is_alive() and observed==['refused_without_commands'], 'loopback refusal fixture failed'
                return result
        for name in ['default','unicode','long','no-newline','whitespace']:
            wire=subprocess.check_output([str(examples/'emit_fixture'),name],timeout=15)
            result=inn(wire);assert result.returncode==0,'INN rejected '+name
            assert canonical(result.stdout.replace(b'\n',b'\r\n'))==wire,'INN roundtrip changed '+name
            cases.append(name+'_roundtrip')
        prefix=b'Date: Wed, 21 Oct 2026 18:00 +0000\r\nFrom: author@fixture.invalid\r\nMessage-ID: <independent@fixture.invalid>\r\nNewsgroups: faction.local.r1\r\nPath: not-for-mail\r\nSubject: plain\r\n'
        inputs={'plain_defaults':prefix+b'\r\nASCII body\r\n',
                'base64_utf8':prefix+b'MIME-Version: 1.0\r\nContent-Type: text/plain; charset="UTF-8"\r\nContent-Transfer-Encoding: base64\r\n\r\nY2Fmw6k=\r\n',
                'encoded_q_subject':prefix.replace(b'Subject: plain',b'Subject: =?UTF-8?Q?caf=C3=A9?=')+b'\r\nASCII body\r\n'}
        for name,wire in inputs.items():
            result=inn(wire);assert result.returncode==0,'INN rejected '+name
            assert canonical(result.stdout.replace(b'\n',b'\r\n'))==canonical(wire),'INN semantic roundtrip changed '+name
            cases.append(name+'_roundtrip')
        ordinary=inputs['plain_defaults']
        bad={'duplicate_subject':ordinary.replace(b'Subject: plain',b'Subject: plain\r\nSubject: duplicate'),
             'invalid_message_id':ordinary.replace(b'<independent@fixture.invalid>',b'not-a-message-id'),
             'empty_body':prefix+b'\r\n'}
        for name,wire in bad.items():
            result=inn(wire);assert result.returncode!=0,'INN did not reject '+name
            cases.append(name+'_rejected')
        # INN inews deliberately delegates some validation to a server. Its
        # acceptance is not evidence of calendar validity; Faction must refuse.
        bad_date=ordinary.replace(b'Wed, 21 Oct 2026 18:00 +0000',b'invalid-date')
        result=inn(bad_date);assert result.returncode==0
        own=subprocess.run([str(examples/'canonicalize')],input=bad_date,capture_output=True,timeout=15)
        assert own.returncode!=0
        cases.append('inn_accepts_invalid_date_faction_rejects')
    result={'result':'pass','count':len(cases),'checks':cases,'version':VERSION,
            'source_sha256':SHA256,'binary_sha256':provenance['binary_sha256'],'runtime_artifacts':provenance['runtime_artifacts'],
            'only_owned_loopback_refusal_fixture':True,'client_commands_observed':0,
            'synthetic_hostname_avoids_dns_lookup':True,
            'service_started':False,'posting_or_external_contact':False}
    if output:output.write_text(json.dumps(result,indent=2,sort_keys=True)+'\n')
    print('Original INN '+VERSION+' passed '+str(len(cases))+' article roundtrip/rejection controls; no posting/service/external contact.')
    return result

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--reference-dir',type=Path,required=True);p.add_argument('--examples',type=Path,required=True);p.add_argument('--output',type=Path)
    a=p.parse_args();check(a.reference_dir.resolve(),a.examples.resolve(),a.output)
