"""Guards for the main-menu mail acceptance probe's observable contract."""
import unittest
import probe_menu_mail as proof

def dump(payload,fd=6,direction='read',pid=42,declared=None):
    header=f'{pid} {pid} faction-service GIO fd {fd} {direction} {len(payload) if declared is None else declared} bytes\n'
    lines=[]
    for offset in range(0,len(payload),16):
        chunk=payload[offset:offset+16]
        field=' '.join(f'{b:02x}' for b in chunk[:8])
        if len(chunk)>8:field+='  '+' '.join(f'{b:02x}' for b in chunk[8:])
        ascii=''.join(chr(b) if 33<=b<=126 else '.' for b in chunk)
        lines.append(f'\t{offset:03x}   {field:<48} {ascii}\n')
    return (header+''.join(lines)).encode()

class MenuMailOracle(unittest.TestCase):
    def test_files_disabled_requires_explicit_mail_and_no_transfer_capability(self):
        proof.require_menu_mail(b'[M] Home mail\r\n', b'\x1b]7770;FACTION-MAIL1;offer\x07')
        for menu,offer in [(b'[F] Files', b''), (b'[M] Home mail',b'\x1b]7770;FACTION1;files;key\x07'),(b'[M] Home mail', b'\x1b]7770;FACTION-MAIL1;offer\x07\x1b]7770;FACTION1;namespace;x\x07')]:
            with self.assertRaises(AssertionError):proof.require_menu_mail(menu,offer)

    def test_same_connected_visit_requires_same_client_and_ssh(self):
        proof.assert_same_visit({'client':41,'ssh':42},{'client':41,'ssh':42})
        with self.assertRaises(AssertionError):proof.assert_same_visit({'client':41,'ssh':42},{'client':41,'ssh':43})

    def test_private_capture_checks_each_board_bound_byte_and_marker(self):
        proof.require_board_bound_private_absent(dump(b'login\n@MAIL1 park '+b'a'*32+b'\n'),[b'private body',b'home password'])
        for data in [b'private body\n', b'home password\n']:
            with self.assertRaises(AssertionError):proof.require_board_bound_private_absent(dump(data),[b'private body',b'home password'])
        with self.assertRaises(AssertionError):proof.require_board_bound_private_absent(b'', [b'private body'])

    def test_bytewise_and_interleaved_gio_records_cannot_hide_private_marker(self):
        marker=b'private body'
        for trace in [b''.join(dump(bytes([b])) for b in marker),dump(marker[:5])+dump(b'noise',fd=7)+dump(marker[5:])]:
            with self.assertRaises(AssertionError):proof.require_board_bound_private_absent(trace,[marker])

    def test_hex_bytes_are_scanned_after_decoding_including_nonprintable_bytes(self):
        for marker in [b'private body',b'private\x00body\xff']:
            with self.assertRaises(AssertionError):proof.require_board_bound_private_absent(dump(marker),[marker])

    def test_unrelated_fd_pid_and_opposite_directions_are_separate_streams(self):
        for trace in [dump(b'private')+dump(b' body',fd=7),dump(b'private')+dump(b' body',pid=43),dump(b'private')+dump(b' body',direction='wrote')]:
            proof.require_board_bound_private_absent(trace,[b'private body'])

    def test_malformed_truncated_or_unrecognized_dump_never_means_absence(self):
        good=dump(b'safe')
        for trace in [dump(b'x',declared=2),dump(b'x',declared=0),b'42 faction-service GIO fd 6 read 1 bytes\n',good.replace(b'000   ',b'010   '),good.replace(b'73 61',b'zz 61'),b'42 faction-service GIO fd 6 read 4 bytes\n       "safe"\n',b'unknown header\n'+good]:
            with self.assertRaises(AssertionError):proof.require_board_bound_private_absent(trace,[b'private body'])

    def test_kdump_repeated_full_rows_expand_to_next_explicit_offset(self):
        row=b'0123456789abcdef'
        payload=row*3+b'final row'
        lines=dump(payload).splitlines(keepends=True)
        compressed=b''.join(lines[:2])+b'\t***\n'+b''.join(lines[4:])
        streams,records=proof.reconstruct_kdump_streams(compressed)
        self.assertEqual(streams,{(42,6,'read'):payload})
        self.assertEqual(records,1)
        # A marker spanning a repeated row boundary must still be found.
        with self.assertRaises(AssertionError):
            proof.require_board_bound_private_absent(compressed,[b'abcdef0123456789'])

    def test_kdump_repeats_reject_unverified_ambiguous_and_out_of_bounds_forms(self):
        full=dump(b'x'*16+b'end').splitlines(keepends=True)
        partial=dump(b'x'*7).splitlines(keepends=True)
        for trace in [
            b''.join(full[:1])+b'\t***\n'+b''.join(full[1:]),
            b''.join(full[:2])+b'\t***\n',
            b''.join(full[:2])+b'\t***\n\t***\n'+b''.join(full[2:]),
            b''.join(full[:2])+b'\t***\n'+full[2],
            b''.join(full[:2])+b'\t***\n'+full[2].replace(b'010   ',b'020   '),
            b''.join(full[:2])+b'\t***\n'+full[2].replace(b'010   ',b'011   '),
            b''.join(partial)+b'\t***\n',
        ]:
            with self.subTest(trace_length=len(trace)):
                with self.assertRaises(AssertionError):
                    proof.reconstruct_kdump_streams(trace)

    def test_kdump_single_repeated_row_has_verified_blank_hex_and_prior_ascii(self):
        row=b'0123456789abcdef'
        payload=row*2+b'next'
        lines=dump(payload).splitlines(keepends=True)
        blank=f'\t010   {"":48} {row.decode()}\n'.encode()
        compressed=b''.join(lines[:2])+blank+b''.join(lines[3:])
        streams,records=proof.reconstruct_kdump_streams(compressed)
        self.assertEqual(streams,{(42,6,'read'):payload})
        self.assertEqual(records,1)
        with self.assertRaises(AssertionError):
            proof.require_board_bound_private_absent(compressed,[b'abcdef0123456789'])

    def test_kdump_blank_hex_repeat_requires_prior_full_row_offset_ascii_and_extent(self):
        row=b'0123456789abcdef'
        lines=dump(row*2+b'next').splitlines(keepends=True)
        blank=f'\t010   {"":48} {row.decode()}\n'.encode()
        for trace in [
            b''.join(lines[:1])+blank+b''.join(lines[3:]),
            b''.join(lines[:2])+blank.replace(b'010   ',b'020   ')+b''.join(lines[3:]),
            b''.join(lines[:2])+blank.replace(row,b'fedcba9876543210')+b''.join(lines[3:]),
            b''.join(dump(row*2).splitlines(keepends=True)[:2])+blank,
            dump(b'partial')+blank,
        ]:
            with self.subTest(trace_length=len(trace)):
                with self.assertRaises(AssertionError):proof.reconstruct_kdump_streams(trace)

    def test_only_versioned_bounded_nonce_controls_are_public(self):
        proof.require_mail_control(b'@MAIL1 park '+b'a'*32+b'\n', 'park')
        for value in [b'@MAIL1 park '+b'A'*32+b'\n', b'@MAIL1 park '+b'a'*33+b'\n', b'@MAIL1 park '+b'a'*32+b' extra\n',b'@FACTION1 key secret\n']:
            with self.assertRaises(AssertionError):proof.require_mail_control(value,'park')

if __name__=='__main__':unittest.main()
