"""Guards for the main-menu mail acceptance probe's observable contract."""
import unittest
import probe_menu_mail as proof

class MenuMailOracle(unittest.TestCase):
    def test_files_disabled_requires_explicit_mail_and_no_transfer_capability(self):
        proof.require_menu_mail(b'[M] Home mail\r\n', b'\x1b]7770;FACTION-MAIL1;offer\x07')
        for menu,offer in [(b'[F] Files', b''), (b'[M] Home mail',b'\x1b]7770;FACTION1;files;key\x07'),(b'[M] Home mail', b'\x1b]7770;FACTION-MAIL1;offer\x07\x1b]7770;FACTION1;namespace;x\x07')]:
            with self.assertRaises(AssertionError):proof.require_menu_mail(menu,offer)

    def test_same_connected_visit_requires_same_client_and_ssh(self):
        proof.assert_same_visit({'client':41,'ssh':42},{'client':41,'ssh':42})
        with self.assertRaises(AssertionError):proof.assert_same_visit({'client':41,'ssh':42},{'client':41,'ssh':43})

    def test_private_capture_checks_each_board_bound_byte_and_marker(self):
        proof.require_board_bound_private_absent(b'login\n@MAIL1 park '+b'a'*32+b'\n',[b'private body',b'home password'])
        for data in [b'private body\n', b'home password\n']:
            with self.assertRaises(AssertionError):proof.require_board_bound_private_absent(data,[b'private body',b'home password'])
        with self.assertRaises(AssertionError):proof.require_board_bound_private_absent(b'', [b'private body'])

    def test_only_versioned_bounded_nonce_controls_are_public(self):
        proof.require_mail_control(b'@MAIL1 park '+b'a'*32+b'\n', 'park')
        for value in [b'@MAIL1 park '+b'A'*32+b'\n', b'@MAIL1 park '+b'a'*33+b'\n', b'@MAIL1 park '+b'a'*32+b' extra\n',b'@FACTION1 key secret\n']:
            with self.assertRaises(AssertionError):proof.require_mail_control(value,'park')

if __name__=='__main__':unittest.main()
