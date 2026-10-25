"""Palette and edition contracts; artistic acceptance belongs to the owner."""
import unittest
import tomllib
from collections import Counter

from build_nightwire import artwork, encoded, outputs
from render_theme_preview import cells


class NightwireTests(unittest.TestCase):
    def test_restrained_dark_ink_on_each_page_and_season(self):
        generated = outputs()
        for folder in ['nightwire', 'nightwire-seasons/autumn', 'nightwire-seasons/winter']:
            raw = generated[f'themes/{folder}/login.ans']
            grid = cells(raw[:-129].decode('cp437'), 80, 48)
            for page in range(3):
                with self.subTest(folder=folder, page=page + 1):
                    ink = Counter(fg for row in grid[page*16:(page+1)*16]
                                  for glyph, fg, _, _ in row if glyph != ' ')
                    self.assertGreater(sum(ink.values()), 200)
                    self.assertEqual(ink[13] + ink[14], 0, 'no bright magenta/cyan bands')
                    dark = sum(ink[c] for c in [1, 3, 4, 8])
                    self.assertGreaterEqual(dark / sum(ink.values()), .55)
                    self.assertLessEqual(ink[15] / sum(ink.values()), .05,
                                         'white is a sparse edge highlight')

    def test_generator_owns_matching_gallery_editions(self):
        generated = outputs()
        for work, folder in [('nightwire', 'nightwire'),
                             ('autumn', 'nightwire-seasons/autumn'),
                             ('winter', 'nightwire-seasons/winter')]:
            for ext, source in [('ans', 'login.ans'), ('txt', 'login.txt')]:
                with self.subTest(work=work, ext=ext):
                    self.assertEqual(generated.get(f'gallery/nightwire/{work}.{ext}'),
                                     generated[f'themes/{folder}/{source}'])

    def test_menu_accents_follow_dark_identity(self):
        for name, raw in outputs().items():
            if name.endswith('theme.toml'):
                with self.subTest(pack=name):
                    theme = tomllib.loads(raw.decode())
                    self.assertNotIn(theme['hotkey']['fg'], [13, 14])
                    self.assertEqual(theme['normal']['bg'], 0)

    def test_cp437_extent_and_original_rights_record(self):
        c = artwork()
        self.assertEqual((len(c.cells), {len(row) for row in c.cells}), (48, {80}))
        for height in [16, 48]:
            raw = encoded(c, 0, height, {})
            record = raw[-128:]
            self.assertEqual(record[:7], b'SAUCE00')
            self.assertEqual(int.from_bytes(record[96:98], 'little'), 80)
            self.assertEqual(int.from_bytes(record[98:100], 'little'), height)
            self.assertIn(b'Original / BSD-3', record)
            self.assertEqual(record[105], 1)
            cells(raw[:-129].decode('cp437'), 80, height)

    def test_printable_compact_fallback_and_determinism(self):
        generated = outputs()
        self.assertEqual(generated, outputs())
        for name, raw in generated.items():
            if name.endswith('.txt'):
                self.assertIn(b'F A C T I O N', raw)
                self.assertTrue(all(32 <= c <= 126 or c == 10 for c in raw))
                self.assertTrue(all(len(line) < 40 for line in raw.splitlines()))


if __name__ == '__main__':
    unittest.main()
