"""Stdlib unit contracts; native controls run only with --native-binary PATH."""
import argparse
import contextlib
import io
import os
from pathlib import Path
import sys
import tempfile
import unittest
from unittest import mock

import proof


class FakeReference:
    events = []
    fail_final = False

    class AESKeyData:
        def __init__(self, key, iv):
            self.key, self.iv = key, iv

    class AESGCMEncryptor:
        def __init__(self, data):
            pass

        def encrypt(self, data):
            return b"C" * len(data)

        def finalize(self):
            return b"T" * 16

    class AESGCMDecryptor:
        def __init__(self, data):
            pass

        def decrypt(self, data):
            FakeReference.events.append("update")
            return b"P" * max(0, len(data) - 16)

        def finalize(self):
            FakeReference.events.append("finalize")
            if FakeReference.fail_final:
                raise ValueError("injected final authentication failure")
            return b""


class HarnessContracts(unittest.TestCase):
    def setUp(self):
        FakeReference.events = []
        FakeReference.fail_final = False
        self.key = bytes(range(32))
        self.iv = bytes(12)

    def test_reference_returns_only_after_successful_finalize(self):
        result = proof.reference_transform(
            FakeReference, "decrypt", self.key, self.iv, b"C" * 17
        )
        self.assertTrue(result == b"P", "authenticated output mismatch")
        self.assertEqual(FakeReference.events, ["update", "finalize"])

    def test_injected_final_failure_releases_no_output_or_file(self):
        FakeReference.fail_final = True
        stdout, stderr = io.StringIO(), io.StringIO()
        with contextlib.redirect_stdout(stdout), contextlib.redirect_stderr(stderr):
            with mock.patch("builtins.open") as opened:
                with self.assertRaises(proof.AuthenticationError):
                    proof.reference_transform(
                        FakeReference, "decrypt", self.key, self.iv, b"C" * 32
                    )
                opened.assert_not_called()
        self.assertEqual(FakeReference.events, ["update", "finalize"])
        self.assertEqual(stdout.getvalue(), "")
        self.assertEqual(stderr.getvalue(), "")

    def test_reference_encoding_bounds_are_closed_under_tag_append(self):
        cipher = proof.reference_transform(
            FakeReference, "encrypt", self.key, self.iv, b"P" * 8192
        )
        self.assertEqual(len(cipher), 8208)
        plain = proof.reference_transform(
            FakeReference, "decrypt", self.key, self.iv, cipher
        )
        self.assertEqual(len(plain), 8192)

    def test_invalid_requests_fail_before_reference_instantiation(self):
        cases = [
            ("encrypt", self.key[:-1], self.iv, b"", None),
            ("encrypt", self.key, self.iv[:-1], b"", None),
            ("invalid", self.key, self.iv, b"", None),
            ("encrypt", self.key, self.iv, b"P" * 8193, None),
            ("decrypt", self.key, self.iv, b"C" * 8209, None),
            ("decrypt", self.key, self.iv, b"C" * 15, None),
            ("encrypt", self.key, self.iv, b"", 0),
            ("encrypt", self.key, self.iv, b"", -1),
            ("encrypt", self.key, self.iv, b"", 8193),
            ("encrypt", self.key, self.iv, b"", True),
        ]
        for mode, key, iv, payload, chunk in cases:
            with self.subTest(mode=mode, payload_length=len(payload), chunk=chunk):
                with self.assertRaises(proof.ProofError):
                    proof.reference_transform(object(), mode, key, iv, payload, chunk)

    def test_native_failure_stdout_is_never_accepted(self):
        with mock.patch.object(proof, "_run_bounded", return_value=(3, b"bad", b"")):
            with self.assertRaises(proof.ProofError):
                proof.native_transform("unused", "decrypt", self.key, self.iv, b"C" * 16)

    def test_native_rejection_and_wrong_length_are_errors(self):
        with mock.patch.object(proof, "_run_bounded", return_value=(3, b"", b"rejected")):
            with self.assertRaises(proof.AuthenticationError):
                proof.native_transform("unused", "decrypt", self.key, self.iv, b"C" * 16)
        with mock.patch.object(proof, "_run_bounded", return_value=(0, b"", b"")):
            with self.assertRaises(proof.ProofError):
                proof.native_transform("unused", "encrypt", self.key, self.iv, b"")

    def test_malformed_source_rejected_before_import(self):
        with tempfile.TemporaryDirectory(prefix="faction250-unit-") as folder:
            path = Path(folder) / "reference.py"
            path.write_text('raise RuntimeError("must not execute")\n')
            with self.assertRaises(proof.ProofError):
                proof.load_reference(path)

    def test_spool_exact_membership_bytes_and_regular_files(self):
        with tempfile.TemporaryDirectory(prefix="faction250-unit-") as folder:
            spool = Path(folder) / "spool"
            spool.mkdir(mode=0o700)
            expected = {"blob-00.bin": b"opaque ciphertext"}
            proof.write_ciphertext(spool, "blob-00.bin", expected["blob-00.bin"])
            first = proof.audit_spool(spool, expected)
            self.assertEqual(set(first), set(expected))
            with self.assertRaises(FileExistsError):
                proof.write_ciphertext(spool, "blob-00.bin", b"replacement")
            self.assertTrue(proof.audit_spool(spool, expected) == first)
            (spool / "extra").write_bytes(b"unaccepted")
            with self.assertRaises(proof.ProofError):
                proof.audit_spool(spool, expected)
            (spool / "extra").unlink()
            (spool / "blob-00.bin").write_bytes(b"modified")
            with self.assertRaises(proof.ProofError):
                proof.audit_spool(spool, expected)

    def test_spool_rejects_symlink_and_traversal(self):
        with tempfile.TemporaryDirectory(prefix="faction250-unit-") as folder:
            spool = Path(folder) / "spool"
            spool.mkdir(mode=0o700)
            (spool / "blob").symlink_to(Path(folder) / "outside")
            with self.assertRaises(proof.ProofError):
                proof.audit_spool(spool, {"blob": b"cipher"})
            for name in ["../outside", "/absolute", "nested/blob", ""]:
                with self.subTest(name=name):
                    with self.assertRaises(proof.ProofError):
                        proof.write_ciphertext(spool, name, b"cipher")

    def test_failed_reference_does_not_mutate_spool(self):
        with tempfile.TemporaryDirectory(prefix="faction250-unit-") as folder:
            spool = Path(folder)
            expected = {"blob": b"cipher"}
            proof.write_ciphertext(spool, "blob", expected["blob"])
            before = proof.audit_spool(spool, expected)
            FakeReference.fail_final = True
            with self.assertRaises(proof.AuthenticationError):
                proof.reference_transform(
                    FakeReference, "decrypt", self.key, self.iv, b"C" * 32
                )
            self.assertTrue(proof.audit_spool(spool, expected) == before)

    def test_mutations_include_tag_ciphertext_truncation_append_and_missing_tag(self):
        mutations = dict(proof.mutations(bytes(range(33))))
        self.assertEqual(set(mutations), {"tag-bit", "cipher-bit", "truncate", "append", "missing-tag"})
        self.assertEqual(len(mutations["append"]), 34)
        self.assertEqual(len(mutations["missing-tag"]), 17)
        self.assertNotIn("cipher-bit", dict(proof.mutations(bytes(16))))


class ProcessBounds(unittest.TestCase):
    def command(self, source):
        return [sys.executable, "-I", "-B", "-c", source]

    def test_output_and_exit_are_captured_within_bounds(self):
        status, output, error = proof._run_bounded(
            self.command("import sys; d=sys.stdin.buffer.read(); sys.stdout.buffer.write(d); sys.stderr.write('x')"),
            b"public", 6, 1, 2,
        )
        self.assertEqual(status, 0)
        self.assertTrue(output == b"public")
        self.assertTrue(error == b"x")

    def test_stdout_overflow_kills_and_reaps(self):
        with self.assertRaises(proof.ProofError):
            proof._run_bounded(self.command("import sys; sys.stdout.write('x'*10000)"), b"", 8, 8, 2)

    def test_stderr_overflow_kills_and_reaps(self):
        with self.assertRaises(proof.ProofError):
            proof._run_bounded(self.command("import sys; sys.stderr.write('x'*10000)"), b"", 8, 8, 2)

    def test_timeout_kills_and_reaps(self):
        with self.assertRaises(proof.ProofError):
            proof._run_bounded(self.command("import time; time.sleep(5)"), b"", 8, 8, 0.1)

    def test_nonreading_stdin_remains_time_bounded(self):
        with self.assertRaises(proof.ProofError):
            proof._run_bounded(self.command("import time; time.sleep(5)"), b"x" * 1000000, 8, 8, 0.1)


def native_suite(binary):
    class NativeContract(unittest.TestCase):
        key = bytes.fromhex("31bdadd96698c204aa9ce1448ea94ae1fb4a9a0b3c9d773b51bb1822666b8f22")
        iv = bytes.fromhex("0d18e06c7c725ac9e362e1ce")
        plain = bytes.fromhex("2db5168e932556f8089a0622981d017d")
        cipher = bytes.fromhex("fa4362189661d163fcd6a56d8bf0405ad636ac1bbedd5cc3ee727dc2ab4a9489")

        def raw(self, mode, payload, chunk=None):
            args = [str(binary), mode] + ([] if chunk is None else [str(chunk)])
            return proof._run_bounded(args, self.key + self.iv + payload, 8208, 256, 5)

        def test_independent_nist_known_answer_reciprocal(self):
            for chunk in [None, 1, 15, 16, 17, 4096]:
                with self.subTest(chunk=chunk):
                    status, output, error = self.raw("encrypt", self.plain, chunk)
                    self.assertEqual(status, 0)
                    self.assertTrue(output == self.cipher, "NIST encryption mismatch")
                    status, output, error = self.raw("decrypt", self.cipher, chunk)
                    self.assertEqual(status, 0)
                    self.assertTrue(output == self.plain, "NIST decryption mismatch")

        def test_auth_failures_have_empty_stdout(self):
            for label, payload in proof.mutations(self.cipher):
                with self.subTest(mutation=label):
                    status, output, error = self.raw("decrypt", payload)
                    self.assertNotEqual(status, 0)
                    self.assertTrue(output == b"", "unauthenticated output escaped")
            wrong = bytes([self.key[0] ^ 1]) + self.key[1:]
            status, output, error = proof._run_bounded(
                [str(binary), "decrypt"], wrong + self.iv + self.cipher, 8208, 256, 5
            )
            self.assertNotEqual(status, 0)
            self.assertTrue(output == b"")

        def test_input_and_argv_rejections_have_empty_stdout(self):
            for args, frame in [
                (["encrypt"], b""), (["decrypt"], self.key + self.iv + bytes(15)),
                (["encrypt"], self.key + self.iv + bytes(8193)),
                (["decrypt"], self.key + self.iv + bytes(8209)),
                (["invalid"], self.key + self.iv),
                (["encrypt", "0"], self.key + self.iv),
                (["encrypt", "-1"], self.key + self.iv),
                (["encrypt", "1x"], self.key + self.iv),
                (["encrypt", "999999999999999999999999999999"], self.key + self.iv),
                (["encrypt", "1", "extra"], self.key + self.iv),
            ]:
                with self.subTest(argv=args, frame_length=len(frame)):
                    status, output, error = proof._run_bounded(
                        [str(binary)] + args, frame, 8208, 256, 5
                    )
                    self.assertNotEqual(status, 0)
                    self.assertTrue(output == b"")

        def test_cap_roundtrip_and_largest_fixture_append_authentication(self):
            plain = bytes(i % 256 for i in range(8192))
            status, cipher, error = self.raw("encrypt", plain)
            self.assertEqual(status, 0)
            self.assertEqual(len(cipher), 8208)
            status, output, error = self.raw("decrypt", cipher)
            self.assertEqual(status, 0)
            self.assertTrue(output == plain)
            status, cipher, error = self.raw("encrypt", plain[:4097])
            self.assertEqual(status, 0)
            status, output, error = self.raw("decrypt", cipher + b"\0")
            self.assertEqual(status, 3, "appended bytes must reach authentication failure")
            self.assertTrue(output == b"")

    return unittest.defaultTestLoader.loadTestsFromTestCase(NativeContract)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--native-binary", type=Path)
    args = parser.parse_args()
    suite = unittest.TestSuite([
        unittest.defaultTestLoader.loadTestsFromTestCase(HarnessContracts),
        unittest.defaultTestLoader.loadTestsFromTestCase(ProcessBounds),
    ])
    if args.native_binary is not None:
        suite.addTests(native_suite(args.native_binary))
    result = unittest.TextTestRunner(verbosity=2).run(suite)
    raise SystemExit(0 if result.wasSuccessful() else 1)
