"""Contracts for the new private writing probe; no VM or network service."""
import unittest
from probe_mail_writing import require_body_pages, seed_input

class WritingProbeTests(unittest.TestCase):
    def test_body_pages_refuse_missing_duplicate_unsafe_oversized_and_premature_actions(self):
        rows=['row'+str(n) for n in range(12)]
        pages=['[LOCAL BODY] Page 1/2\r\n'+'\r\n'.join('| '+r for r in rows[:10])+'\r\n[Enter] Next page  [B] Return\r\n', '[LOCAL BODY] Page 2/2\r\n'+'\r\n'.join('| '+r for r in rows[10:])+'\r\n']
        require_body_pages(pages,rows)
        for bad in [["\u0085"+pages[0],pages[1]],["\v"+pages[0],pages[1]],pages[:1],pages[::-1],[pages[0].replace('row0','row1'),pages[1]],[pages[0]+'Type read to acknowledge locally',pages[1]],[pages[0].replace('row0','x'*38),pages[1]],[pages[0].replace('row0','evil\x1b]52;c;x'),pages[1]],[pages[0],pages[1].replace('row11','row11\u202e')]]:
            with self.assertRaises(AssertionError):require_body_pages(bad,rows)

    def test_fixture_input_preserves_lf_and_byte_boundaries_and_refuses_bad_headers(self):
        self.assertEqual(seed_input('subject','bob@away.test','first\n\nlast\n'),b'subject\nbob@away.test\nfirst\n\nlast\n')
        self.assertTrue(seed_input('subject','bob@away.test','λ'*512).endswith(('λ'*512).encode()))
        for subject,peer,body in [('bad\nheader','bob@away.test','a'),('subject','bad\npeer','a'),('subject','bob@away.test','x'*1025)]:
            with self.assertRaises(AssertionError):seed_input(subject,peer,body)

if __name__=='__main__':unittest.main()
