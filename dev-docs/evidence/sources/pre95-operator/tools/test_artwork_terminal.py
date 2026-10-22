import unittest
from probe_artwork_terminal import checkpoints


class CanonicalArtworkProbe(unittest.TestCase):
    def test_rows_preserve_original_bytes_and_pending_wrap(self):
        frame = b'\x1b[0m\x1b[2J\x1b[H' + b'A' * 80 + b'\x1b[2;1H' + '▄██▄'.encode() + b'\x1b[0m'
        chunks = checkpoints(frame, 80, 24)
        self.assertEqual(b''.join(raw for raw, _ in chunks), frame)
        self.assertEqual([pos for _, pos in chunks], [[1, 80], [2, 5]])

    def test_cp437_is_one_cell_and_crlf_has_one_line_advance(self):
        chunks = checkpoints('▀▄█─┼·│\r\nASCII'.encode(), 80, 24)
        self.assertEqual(chunks[-1][1], [2, 6])

    def test_untrusted_controls_geometry_and_scroll_refuse(self):
        for frame in [b'\x1b]0;title\x07', b'\x1b[25;1Htext', b'A' * (80 * 24 + 1),
                      b'\x1b[5mtext', b'\x1b[1J', b'\x1bPsecret\x1b\\', b'\x1b[1;0Htext']:
            with self.assertRaises(ValueError):
                checkpoints(frame, 80, 24)


if __name__ == '__main__':
    unittest.main()
