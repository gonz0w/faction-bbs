"""Authoring previews must remain inert and respect fixed terminal dimensions."""
import unittest
from xml.etree import ElementTree
from render_theme_preview import cells, svg

class PreviewSafety(unittest.TestCase):
    def test_rejects_unsupported_controls_and_out_of_screen_output(self):
        for frame in ['\x1b]52;c;payload\x07', '\x1b_Gi=1;data\x1b\\', '\x1b[19;1Hhidden', 'x'*41, '\x1b[38;2;1;2;3m', '界']:
            with self.subTest(frame=frame), self.assertRaises(ValueError): cells(frame, 40)

    def test_text_is_escaped_and_focus_background_is_retained(self):
        grid = cells('\x1b[2J\x1b[1;1H\x1b[0;1;97;44m<script>&', 40)
        self.assertEqual(grid[0][0], ('<', 15, 4, True))
        document = svg(grid)
        root = ElementTree.fromstring(document)
        self.assertNotIn('<script>', document)
        self.assertIn('&lt;script&gt;&amp;', document)
        self.assertTrue(any(node.tag.endswith('text') and node.text == '<script>&' for node in root))

if __name__ == '__main__': unittest.main()
