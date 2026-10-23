import unittest
from probe_terminal_profile import ReplyCollector

class Reports(unittest.TestCase):
    def test_fragmented_reports_are_exact_and_never_include_keyboard_text(self):
        frame=b"\x1b_Gi=17392;OK\x1b\\";r=ReplyCollector('kitty')
        for byte in b'private typing'+frame+b' more typing':r.feed(bytes([byte]))
        self.assertEqual(r.finish(),frame)
        self.assertEqual(r.unexpected,len(b'private typing more typing'))
        self.assertEqual(r.pending,b'')

    def test_oversized_incomplete_wrong_id_and_duplicate_reports_fail_clean_measurement(self):
        for raw in [b'\x1b_Gi=99;OK\x1b\\',b'\x1b_Gi=17392;OK',b'x'*1024]:
            r=ReplyCollector('kitty');r.feed(raw);self.assertIsNone(r.finish());self.assertGreater(r.unexpected,0);self.assertEqual(r.pending,b'')
        r=ReplyCollector('sync');frame=b'\x1b[?2026;2$y';r.feed(frame);r.feed(frame);self.assertEqual(r.finish(),frame);self.assertEqual(r.unexpected,len(frame))

    def test_capabilities_have_independent_report_grammars(self):
        cases={'cursor':b'\x1b[18;40R','sgr':b'\x1bP1$r0;38:2::17:34:51m\x1b\\','image':b'\x1b_Gi=17391;ENOENT:No image\x1b\\'}
        cases['sgr_refusal']=b'\x1bP0$r\x1b\\'
        for kind,frame in cases.items():
            kind='sgr' if kind=='sgr_refusal' else kind
            r=ReplyCollector(kind)
            for byte in frame:r.feed(bytes([byte]))
            self.assertEqual(r.finish(),frame);self.assertEqual(r.unexpected,0)
            wrong=ReplyCollector('sync');wrong.feed(frame);self.assertIsNone(wrong.finish())

if __name__=='__main__':unittest.main()
