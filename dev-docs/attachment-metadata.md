# Experimental attachment metadata in native SCE

The native-only [`attachment` codec](../crates/faction-omemo/src/attachment.rs) encodes and decodes one bounded candidate descriptor alongside subject and body inside SCE. It is experimental metadata handling, not attachment delivery or adoption of a complete XMPP file-sharing profile. The default feature set remains empty. Existing text-only SCE APIs reject this extension, and the current mail checkpoint does not accept or persist it.

## Candidate contract

`attachment::encode` returns private, zeroizing XML bytes. `attachment::decode` validates already authenticated XML against caller-supplied sender, recipient and enclosing sending time. `attachment::decode_network` validates the addresses and returns the inner sending time; the caller must establish the appropriate trusted time relationship. Parsing alone does not authenticate a sender or route a device. Existing SCE limits apply: at most 8192 plaintext bytes and 128 XML nodes, exact address bindings, bounded subject and body, and the existing UTC time and padding rules. `decode` requires the inner time to agree within 300 seconds of the trusted enclosing sending time.

SCE content contains exactly one `urn:xmpp:sfs:0` file-sharing element with `disposition='attachment'`. Its `urn:xmpp:file:metadata:0` file contains a name, media type, original size and one `urn:xmpp:hashes:2` SHA-256 hash. One `urn:xmpp:esfs:0` encrypted source names the cipher `urn:xmpp:ciphers:aes-256-gcm-nopadding:0`, a key and IV, and one nested SFS source with a `http://jabber.org/protocol/url-data` target. The hash and key are 32 bytes each; the IV is 12 bytes. Their XML representations use canonical standard Base64. Unknown fields, duplicates, unsupported namespaces and algorithms, active XML and malformed or trailing content reject.

| Field | Accepted candidate bounds |
| --- | --- |
| Name | 1–255 UTF-8 bytes; no controls, slash or backslash, and not `.` or `..`. It is not a filesystem path. |
| Media type | 1–127 ASCII bytes; one nonempty token on each side of `/`, without parameters. |
| Size | Canonical unsigned decimal `u64`, including zero; `size + 16` must not overflow. This is an assertion, not an allocation or blob-size limit. |
| Locator | 1–512 printable ASCII bytes, beginning `https://` with a nonempty authority; no credentials, whitespace, backslash, query or fragment. It is inert and does not authorize a destination. |

These restrictions are a narrow candidate subset, not complete filename, MIME or URL validation. The codec neither opens the locator nor saves a file. Owned XML, keys and private metadata use zeroizing buffers; parser normalization and OS copies are not guaranteed erased. Errors do not expose descriptor contents.

## Authentication and checkpoint boundary

A native caller authenticates the encrypted message with the existing verified peer and `open_pending` before decoding its candidate plaintext. After validating the descriptor and SCE bindings, the caller explicitly commits the pending state; on rejection it aborts or drops the candidate. The codec does not make that transaction decision. The current checkpoint continues to reject an authentic candidate extension without changing its canonical ciphertext or history.

On macOS 27.2 ARM64 with Rust 1.96, Apple Clang 21 and the existing OpenSSL 3.6.3, focused tests passed for descriptor contracts, native encrypted exchange and checkpoint rejection. They exercised valid commit, malformed authentic input abort with unchanged committed snapshots, subsequent valid acceptance, untrusted or wrong peers, and modified or truncated messages. These host observations do not establish NetBSD support or interoperability with a modern client.

The [saved host evidence](evidence/attachment-metadata/README.md) preserves the executed variants, original failures, final commands and separate review. It does not rerun the tests.

The codec does not encrypt or verify blob bytes, verify the asserted size or hash, prove caller-supplied key freshness, route numeric device IDs, fetch or upload, store keys or blobs durably, or deliver attachments. Private storage, recipient authorization, recovery and delivery remain separate work under [#88](https://github.com/gonz0w/faction-bbs/issues/88). The earlier [historical byte proof](research/0059-attachment-byte-proof.md) is a separate bounded observation and does not establish this descriptor's interoperability.
