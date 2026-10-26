"""Published #214 native source objects must retain protocol CRLF bytes."""
import subprocess,tempfile,unittest
from pathlib import Path

class PickerEvidenceBytes(unittest.TestCase):
    def test_git_preserves_native_protocol_input(self):
        with tempfile.TemporaryDirectory(prefix='faction214-byte-test-') as tmp:
            root=Path(tmp)
            def git(*args):return subprocess.check_output(['git','-c','core.autocrlf=false','-c','core.attributesfile=/dev/null','-C',str(root),*args],stderr=subprocess.PIPE)
            git('init','-q');(root/'.gitattributes').write_bytes((Path(__file__).resolve().parents[1]/'.gitattributes').read_bytes())
            for folder in ['picker-mail/native-source-objects','local-unread-mail/source-objects']:
                with self.subTest(folder=folder):
                    name='dev-docs/evidence/'+folder+'/'+'a'*64
                    path=root/name;path.parent.mkdir(parents=True);raw=b'public\r\nprotocol fixture\r\n';path.write_bytes(raw)
                    git('add','--',name);self.assertEqual(git('show',':'+name),raw)
