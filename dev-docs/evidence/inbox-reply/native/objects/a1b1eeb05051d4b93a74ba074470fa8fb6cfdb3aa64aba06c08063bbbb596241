"""Newline-compliant JSON views must preserve all original measured bytes."""
import copy,hashlib,json,unittest
import check_unread_mail_evidence as check

class PublicationBytes(unittest.TestCase):
    def fixture(self):
        sha=lambda b:hashlib.sha256(b).hexdigest()
        raw=b'{"observed": true}'
        prior=b'{"immutable": "previous complete record"}\n'
        name='dev-docs/evidence/local-unread-mail/qualification/control.json'
        blob='dev-docs/evidence/local-unread-mail/source-objects/'+sha(raw)
        previous='dev-docs/evidence/local-unread-mail/source-objects/'+sha(prior)
        files={name:raw+b'\n',blob:raw,previous:prior}
        mapping=dict(schema=1,prior_record=dict(path=previous,sha256=sha(prior)),files={name:dict(raw_path=blob,raw_sha256=sha(raw),view_sha256=sha(raw+b'\n'))})
        return mapping,files,name,raw,prior
    def test_original_record_and_original_bytes_remain_resolvable(self):
        m,files,name,raw,prior=self.fixture()
        old,resolve=check.publication_resolver(m,lambda n,h:files[n])
        self.assertEqual(old,prior)
        self.assertEqual(resolve(name,hashlib.sha256(raw).hexdigest()),raw)
        self.assertEqual(resolve(name,hashlib.sha256(raw+b'\n').hexdigest()),raw+b'\n')
    def test_only_one_appended_newline_is_permitted(self):
        for view in [b'{"observed": false}\n',b'{"observed": true} \n',b'{"observed": true}\n\n']:
            m,files,name,raw,prior=self.fixture();files[name]=view
            m['files'][name]['view_sha256']=hashlib.sha256(view).hexdigest()
            with self.assertRaises(AssertionError):check.publication_resolver(m,lambda n,h:files[n])
    def test_raw_substitution_and_path_escape_fail_closed(self):
        m,files,name,raw,prior=self.fixture();files[m['files'][name]['raw_path']]=b'wrong'
        with self.assertRaises(AssertionError):check.publication_resolver(m,lambda n,h:files[n])
        m,files,name,raw,prior=self.fixture();m['files']['../outside.json']=m['files'].pop(name)
        with self.assertRaises(AssertionError):check.publication_resolver(m,lambda n,h:files[n])
if __name__=='__main__':unittest.main()
