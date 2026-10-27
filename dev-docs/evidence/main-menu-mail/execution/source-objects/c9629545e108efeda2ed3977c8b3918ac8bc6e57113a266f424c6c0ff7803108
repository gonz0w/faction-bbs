"""Current service evidence must not accept historical bytes after a source edit."""
import hashlib,tempfile,unittest
from pathlib import Path
from check_integrated_evidence import validate_sources

class CurrentServiceSources(unittest.TestCase):
    def test_frozen_copy_cannot_mask_current_source_drift(self):
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp);p=root/'service.rs';original=b'executed service'
            p.write_bytes(original);digest=hashlib.sha256(original).hexdigest()
            old=root/'dev-docs/evidence/sources/old';old.mkdir(parents=True);(old/'service.rs').write_bytes(original)
            validate_sources({'service.rs':digest},root)
            p.write_bytes(b'changed current service')
            with self.assertRaisesRegex(AssertionError,'current service execution drift'):
                validate_sources({'service.rs':digest},root)

if __name__=='__main__':unittest.main()
