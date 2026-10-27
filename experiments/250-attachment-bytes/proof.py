"""Bounded historical AES-GCM byte proof; crypto is loaded only by explicit CLI."""
import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import selectors
import stat
import subprocess
import sys
import time

MAX_PLAINTEXT = 8192
TAG_BYTES = 16
MAX_CIPHERTEXT = MAX_PLAINTEXT + TAG_BYTES
REFERENCE_SHA256 = "79036e4d5f72cf8696b7ee44a5223ff6f5214a18b4e8809bf9b22167128c10e4"
LENGTHS = (0, 1, 15, 16, 17, 4095, 4096, 4097)
CHUNKS = (1, 15, 16, 17, 4096)
# Source-derived predictions recorded before execution in issue #250.
PREDICTED_DECRYPT_SUCCESS = {
    1: (), 15: (), 16: (0, 16, 4096), 17: (0, 1, 17, 4097),
    4096: (0, 1, 15, 16, 17, 4096, 4097),
}
# NIST CAVS 14.0; pyca/cryptography tag 50.0.2, gcmEncryptExtIV256.rsp.
# Git blob 60da5fae6135187e4d7912a2da62a20bad3c2a48; empty AAD, 128-bit tags.
KATS = (
    ("b52c505a37d78eda5dd34f20c22540ea1b58963cf8e5bf8ffa85f9f2492505b4",
     "516c33929df5a3284ff463d7", "", "bdc1ac884d332457a1d2664f168c76f0"),
    ("31bdadd96698c204aa9ce1448ea94ae1fb4a9a0b3c9d773b51bb1822666b8f22",
     "0d18e06c7c725ac9e362e1ce", "2db5168e932556f8089a0622981d017d",
     "fa4362189661d163fcd6a56d8bf0405ad636ac1bbedd5cc3ee727dc2ab4a9489"),
)


class ProofError(Exception):
    pass


class AuthenticationError(ProofError):
    def __init__(self, message, cause_type=None):
        super().__init__(message)
        self.cause_type = cause_type


def _require(condition, message):
    if not condition:
        raise ProofError(message)


def _validate(mode, key, iv, payload, chunk):
    _require(mode in ("encrypt", "decrypt"), "invalid operation")
    _require(isinstance(key, bytes) and len(key) == 32, "invalid key length")
    _require(isinstance(iv, bytes) and len(iv) == 12, "invalid IV length")
    _require(isinstance(payload, bytes), "payload must be bytes")
    _require(chunk is None or (type(chunk) is int and 1 <= chunk <= MAX_PLAINTEXT), "invalid chunk")
    minimum, maximum = (0, MAX_PLAINTEXT) if mode == "encrypt" else (TAG_BYTES, MAX_CIPHERTEXT)
    _require(minimum <= len(payload) <= maximum, "invalid payload length")


def reference_transform(reference, mode, key, iv, payload, chunk=None):
    """Quarantine every update byte until finalize succeeds; no file or callback."""
    _validate(mode, key, iv, payload, chunk)
    operation = (reference.AESGCMEncryptor if mode == "encrypt" else reference.AESGCMDecryptor)(
        reference.AESKeyData(key, iv)
    )
    pending = bytearray()
    try:
        update = operation.encrypt if mode == "encrypt" else operation.decrypt
        if chunk is None:
            pending.extend(update(payload))
        else:
            for offset in range(0, len(payload), chunk):
                pending.extend(update(payload[offset:offset + chunk]))
        pending.extend(operation.finalize())
        expected = len(payload) + TAG_BYTES if mode == "encrypt" else len(payload) - TAG_BYTES
        _require(len(pending) == expected, "reference output length mismatch")
        return bytes(pending)
    except ProofError:
        raise
    except Exception as error:
        identity = type(error).__module__ + "." + type(error).__qualname__
        if mode == "decrypt":
            raise AuthenticationError("reference decryption rejected", identity) from error
        raise ProofError("reference encryption rejected") from error
    finally:
        pending[:] = bytes(len(pending))


def _run_bounded(command, frame, stdout_limit, stderr_limit, timeout=5):
    """Bound all pipes and wall time; kill and reap on every incomplete operation."""
    process = subprocess.Popen(command, stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    selection = selectors.DefaultSelector()
    output, error = bytearray(), bytearray()
    deadline, sent = time.monotonic() + timeout, 0
    try:
        for stream in (process.stdin, process.stdout, process.stderr):
            os.set_blocking(stream.fileno(), False)
        if frame:
            selection.register(process.stdin, selectors.EVENT_WRITE, "stdin")
        else:
            process.stdin.close()
        selection.register(process.stdout, selectors.EVENT_READ, "stdout")
        selection.register(process.stderr, selectors.EVENT_READ, "stderr")
        while selection.get_map():
            remaining = deadline - time.monotonic()
            _require(remaining > 0, "subprocess timeout")
            ready = selection.select(remaining)
            _require(bool(ready), "subprocess timeout")
            for selected, events in ready:
                stream, kind = selected.fileobj, selected.data
                if kind == "stdin":
                    try:
                        sent += os.write(stream.fileno(), memoryview(frame)[sent:sent + 4096])
                    except BrokenPipeError:
                        sent = len(frame)
                    if sent == len(frame):
                        selection.unregister(stream)
                        stream.close()
                else:
                    data = os.read(stream.fileno(), 4096)
                    if not data:
                        selection.unregister(stream)
                        stream.close()
                    else:
                        target, limit = (output, stdout_limit) if kind == "stdout" else (error, stderr_limit)
                        _require(len(target) + len(data) <= limit, "subprocess output limit")
                        target.extend(data)
        remaining = deadline - time.monotonic()
        _require(remaining > 0, "subprocess timeout")
        try:
            status = process.wait(timeout=remaining)
        except subprocess.TimeoutExpired:
            raise ProofError("subprocess timeout") from None
        return status, bytes(output), bytes(error)
    finally:
        selection.close()
        if process.poll() is None:
            process.kill()
        process.wait()
        for stream in (process.stdin, process.stdout, process.stderr):
            stream.close()
        output[:] = bytes(len(output))
        error[:] = bytes(len(error))


def native_transform(binary, mode, key, iv, payload, chunk=None):
    _validate(mode, key, iv, payload, chunk)
    command = [str(binary), mode] + ([] if chunk is None else [str(chunk)])
    status, output, error = _run_bounded(command, key + iv + payload, MAX_CIPHERTEXT, 256, 5)
    if status:
        _require(not output, "native failure emitted output")
        if mode == "decrypt" and status == 3:
            raise AuthenticationError("native authentication rejected", "native.exit.3")
        raise ProofError("native request or operation rejected")
    expected = len(payload) + TAG_BYTES if mode == "encrypt" else len(payload) - TAG_BYTES
    _require(len(output) == expected, "native output length mismatch")
    return output


def load_reference(path):
    path = Path(path)
    source = path.read_bytes()
    _require(hashlib.sha256(source).hexdigest() == REFERENCE_SHA256, "reference source identity mismatch")
    specification = importlib.util.spec_from_file_location("faction250_unchanged_reference", path)
    module = importlib.util.module_from_spec(specification)
    sys.modules[specification.name] = module
    specification.loader.exec_module(module)
    _require(hashlib.sha256(path.read_bytes()).hexdigest() == REFERENCE_SHA256, "reference source changed")
    return module


def _object_name(name):
    _require(isinstance(name, str) and name not in ("", ".", "..") and Path(name).name == name, "invalid spool object name")


def write_ciphertext(spool, name, ciphertext):
    _object_name(name)
    _require(len(ciphertext) <= MAX_CIPHERTEXT, "spool object exceeds bound")
    with (Path(spool) / name).open("xb") as output:
        output.write(ciphertext)


def audit_spool(spool, expected):
    spool = Path(spool)
    _require(not spool.is_symlink() and spool.is_dir(), "invalid spool directory")
    entries = {item.name: item for item in spool.iterdir()}
    _require(set(entries) == set(expected), "spool membership mismatch")
    result = {}
    for name, ciphertext in expected.items():
        _object_name(name)
        path = entries[name]
        _require(stat.S_ISREG(path.lstat().st_mode), "spool entry is not regular")
        _require(path.stat().st_size == len(ciphertext) <= MAX_CIPHERTEXT, "spool object length mismatch")
        _require(path.read_bytes() == ciphertext, "spool ciphertext mismatch")
        result[name] = {"bytes": len(ciphertext), "sha256": hashlib.sha256(ciphertext).hexdigest()}
    return result


def mutations(ciphertext):
    altered = bytearray(ciphertext)
    altered[-1] ^= 1
    yield "tag-bit", bytes(altered)
    if len(ciphertext) > TAG_BYTES:
        altered = bytearray(ciphertext)
        altered[0] ^= 1
        yield "cipher-bit", bytes(altered)
    yield "truncate", ciphertext[:-1]
    yield "append", ciphertext + b"\0"
    yield "missing-tag", ciphertext[:-TAG_BYTES]


def run_cases(binary, reference_path, spool, observations):
    reference = load_reference(reference_path)
    binary_before = hashlib.sha256(Path(binary).read_bytes()).hexdigest()
    observations.update({"known_answers": [], "complete_inputs": [], "rejections": [], "chunks": []})
    for number, encoded in enumerate(KATS):
        key, iv, plaintext, expected = map(bytes.fromhex, encoded)
        for transform in (lambda mode, data: native_transform(binary, mode, key, iv, data),
                          lambda mode, data: reference_transform(reference, mode, key, iv, data)):
            _require(transform("encrypt", plaintext) == expected, "known-answer ciphertext mismatch")
            _require(transform("decrypt", expected) == plaintext, "known-answer plaintext mismatch")
        observations["known_answers"].append({"case": number, "plaintext_bytes": len(plaintext), "both_directions_match": True})
    Path(spool).mkdir(mode=0o700)
    expected_spool = {}
    key = bytes(range(32))
    for index, length in enumerate(LENGTHS):
        iv = bytes(11) + bytes([index])
        plaintext = bytes(i % 256 for i in range(length))
        native = native_transform(binary, "encrypt", key, iv, plaintext)
        unchanged = reference_transform(reference, "encrypt", key, iv, plaintext)
        _require(native == unchanged and len(native) == length + TAG_BYTES, "complete ciphertext mismatch")
        _require(native_transform(binary, "decrypt", key, iv, unchanged) == plaintext, "native reciprocal mismatch")
        _require(reference_transform(reference, "decrypt", key, iv, native) == plaintext, "reference reciprocal mismatch")
        name = "blob-" + str(index).zfill(2) + ".bin"
        write_ciphertext(spool, name, native)
        expected_spool[name] = native
        baseline = audit_spool(spool, expected_spool)
        observations["complete_inputs"].append({"case": index, "plaintext_bytes": length, "ciphertext_bytes": len(native), "ciphertext_sha256": hashlib.sha256(native).hexdigest(), "reciprocal": True})
        wrong_key = bytes([key[0] ^ 1]) + key[1:]
        for label, selected_key, changed in [("wrong-key", wrong_key, native)] + [(label, key, data) for label, data in mutations(native)]:
            outcomes = {}
            for implementation, transform in [
                ("native", lambda: native_transform(binary, "decrypt", selected_key, iv, changed)),
                ("reference", lambda: reference_transform(reference, "decrypt", selected_key, iv, changed)),
            ]:
                try:
                    transform()
                except AuthenticationError as error:
                    expected_cause = "cryptography.exceptions.InvalidTag" if implementation == "reference" else "native.exit.3"
                    _require(error.cause_type == expected_cause, "unexpected authentication exception identity")
                    outcomes[implementation] = error.cause_type
                except ProofError:
                    _require(len(changed) < TAG_BYTES, "unexpected request rejection")
                    outcomes[implementation] = "missing authentication framing"
                else:
                    raise ProofError("authentication mutation accepted")
                _require(audit_spool(spool, expected_spool) == baseline, "failed verification changed spool")
            observations["rejections"].append({"case": index, "mutation": label, "accepted_plaintext_bytes": 0, "spool_unchanged": True, "causes": outcomes})
        for chunk in CHUNKS:
            encrypted = reference_transform(reference, "encrypt", key, iv, plaintext, chunk)
            _require(encrypted == native, "reference chunk encryption mismatch")
            _require(native_transform(binary, "encrypt", key, iv, plaintext, chunk) == native, "native chunk encryption mismatch")
            _require(native_transform(binary, "decrypt", key, iv, native, chunk) == plaintext, "native chunk decryption mismatch")
            cause, matched = None, False
            try:
                matched = reference_transform(reference, "decrypt", key, iv, native, chunk) == plaintext
            except AuthenticationError as error:
                cause = error.cause_type
            predicted = length in PREDICTED_DECRYPT_SUCCESS[chunk]
            observations["chunks"].append({"case": index, "plaintext_bytes": length, "chunk_bytes": chunk, "reference_encrypt_match": True, "native_reciprocal_match": True, "reference_decrypt_match": matched, "reference_rejection": cause, "source_predicted_decrypt_match": predicted, "prediction_matches_observation": matched == predicted})
            _require(audit_spool(spool, expected_spool) == baseline, "chunk characterization changed spool")
    observations["spool"] = audit_spool(spool, expected_spool)
    observations["reference_sha256_after"] = hashlib.sha256(Path(reference_path).read_bytes()).hexdigest()
    observations["binary_sha256"] = binary_before
    _require(observations["reference_sha256_after"] == REFERENCE_SHA256, "reference source changed during proof")
    _require(hashlib.sha256(Path(binary).read_bytes()).hexdigest() == binary_before, "native binary changed during proof")
    observations["characterization_prediction_mismatches"] = sum(not row["prediction_matches_observation"] for row in observations["chunks"])


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--native-binary", type=Path, required=True)
    parser.add_argument("--gajim-module", type=Path, required=True)
    parser.add_argument("--spool", type=Path, required=True)
    parser.add_argument("--result", type=Path, required=True)
    args = parser.parse_args()
    if args.result.exists():
        parser.error("result already exists")
    result = {"status": "incomplete", "scope": "local historical AES-256-GCM bytes, empty AAD; no modern descriptor/transport/NetBSD/product claim", "primitive_limit": "native OpenSSL 3.5.7 and reference bundled OpenSSL 4.0.3 share lineage", "laboratory_plaintext_cap": MAX_PLAINTEXT, "laboratory_ciphertext_cap": MAX_CIPHERTEXT, "reference_sha256_before": REFERENCE_SHA256}
    try:
        run_cases(args.native_binary, args.gajim_module, args.spool, result)
        result["status"] = "complete-input-byte-proof-passed"
    except Exception as error:
        result["status"] = "failed"
        result["failure_type"] = type(error).__module__ + "." + type(error).__qualname__
        # Fixed wrapper diagnostics contain no payload, key, IV or upstream error text.
        result["failure"] = str(error) if isinstance(error, ProofError) else "proof setup or execution failed"
    with args.result.open("x") as output:
        json.dump(result, output, indent=2)
        output.write("\n")
    print(json.dumps({"status": result["status"], "known_answers": len(result.get("known_answers", [])), "complete_inputs": len(result.get("complete_inputs", [])), "rejections": len(result.get("rejections", [])), "chunk_observations": len(result.get("chunks", []))}))
    return 0 if result["status"] == "complete-input-byte-proof-passed" else 1


if __name__ == "__main__":
    raise SystemExit(main())
