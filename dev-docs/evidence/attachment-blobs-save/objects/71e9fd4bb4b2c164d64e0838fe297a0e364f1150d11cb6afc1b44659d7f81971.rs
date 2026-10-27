#![cfg(feature = "native")]
use faction_omemo::{attachment::Descriptor, blob};
use zeroize::Zeroizing;

fn descriptor(encrypted: &blob::EncryptedBlob) -> Descriptor {
    Descriptor {
        name: Zeroizing::new("private.txt".into()),
        media_type: Zeroizing::new("text/plain".into()),
        size: encrypted.size,
        hash: encrypted.hash.clone(),
        key: encrypted.key.clone(),
        iv: encrypted.iv.clone(),
        locator: Zeroizing::new("https://blob.test/opaque".into()),
    }
}

#[test]
fn complete_buffers_have_exact_framing_and_verified_plaintext() {
    assert_eq!(blob::MAX_BLOB_BYTES, 1_048_576);
    for len in [0, 1, 15, 16, 17, 4095, 4096, 4097, blob::MAX_BLOB_BYTES] {
        let plain = Zeroizing::new((0..len).map(|i| (i % 256) as u8).collect::<Vec<_>>());
        let encrypted = blob::encrypt(&plain).unwrap();
        assert_eq!(encrypted.size, len as u64);
        assert_eq!(encrypted.ciphertext.len(), len + 16);
        let verified = blob::verify(&encrypted.ciphertext, &descriptor(&encrypted)).unwrap();
        assert_eq!(verified.as_bytes(), plain.as_slice());
    }
}

#[test]
fn generated_material_differs_and_hash_matches_independent_sha256_values() {
    let a = blob::encrypt(b"abc").unwrap();
    let b = blob::encrypt(b"abc").unwrap();
    assert!(*a.key != *b.key);
    assert!(*a.iv != *b.iv);
    assert_ne!(a.ciphertext, b.ciphertext);
    assert_eq!(
        *a.hash,
        [0xba, 0x78, 0x16, 0xbf, 0x8f, 0x01, 0xcf, 0xea,
         0x41, 0x41, 0x40, 0xde, 0x5d, 0xae, 0x22, 0x23,
         0xb0, 0x03, 0x61, 0xa3, 0x96, 0x17, 0x7a, 0x9c,
         0xb4, 0x10, 0xff, 0x61, 0xf2, 0x00, 0x15, 0xad]
    );
    assert_eq!(
        *blob::encrypt(b"").unwrap().hash,
        [0xe3, 0xb0, 0xc4, 0x42, 0x98, 0xfc, 0x1c, 0x14,
         0x9a, 0xfb, 0xf4, 0xc8, 0x99, 0x6f, 0xb9, 0x24,
         0x27, 0xae, 0x41, 0xe4, 0x64, 0x9b, 0x93, 0x4c,
         0xa4, 0x95, 0x99, 0x1b, 0x78, 0x52, 0xb8, 0x55]
    );
}

#[test]
fn corrupted_bytes_and_incorrect_assertions_never_return_verified_values() {
    let encrypted = blob::encrypt(b"private payload <&>\0").unwrap();
    for change in ["key", "iv", "hash", "size", "overflow", "name", "media", "locator"] {
        let mut d = descriptor(&encrypted);
        match change {
            "key" => d.key[0] ^= 1,
            "iv" => d.iv[0] ^= 1,
            "hash" => d.hash[0] ^= 1,
            "size" => d.size += 1,
            "overflow" => d.size = u64::MAX,
            "name" => *d.name = "../escape".into(),
            "media" => *d.media_type = "text/plain;active".into(),
            _ => *d.locator = "https://blob.test/key?secret".into(),
        }
        assert!(blob::verify(&encrypted.ciphertext, &d).is_err(), "{change}");
    }
    for change in ["tag", "ciphertext", "truncate", "append", "missing-tag", "empty"] {
        let mut bad = encrypted.ciphertext.clone();
        match change {
            "tag" => *bad.last_mut().unwrap() ^= 1,
            "ciphertext" => bad[0] ^= 1,
            "truncate" => { bad.pop(); }
            "append" => bad.push(0),
            "missing-tag" => bad.truncate(bad.len() - 16),
            _ => bad.clear(),
        }
        assert!(blob::verify(&bad, &descriptor(&encrypted)).is_err(), "{change}");
    }
    assert_eq!(
        blob::verify(&encrypted.ciphertext, &descriptor(&encrypted)).unwrap().as_bytes(),
        b"private payload <&>\0"
    );
}

#[test]
fn supplied_and_declared_oversize_are_refused() {
    assert!(blob::encrypt(&vec![0; blob::MAX_BLOB_BYTES + 1]).is_err());
    let encrypted = blob::encrypt(b"valid").unwrap();
    let mut d = descriptor(&encrypted);
    d.size = blob::MAX_BLOB_BYTES as u64 + 1;
    assert!(blob::verify(&encrypted.ciphertext, &d).is_err());
    assert!(blob::verify(&vec![0; blob::MAX_BLOB_BYTES + 17], &descriptor(&encrypted)).is_err());
}

#[test]
fn bounded_cap_operation_observation() {
    let plain = Zeroizing::new(vec![0x53; blob::MAX_BLOB_BYTES]);
    let started = std::time::Instant::now();
    let encrypted = blob::encrypt(&plain).unwrap();
    let encrypt_elapsed = started.elapsed();
    let started = std::time::Instant::now();
    let verified = blob::verify(&encrypted.ciphertext, &descriptor(&encrypted)).unwrap();
    let verify_elapsed = started.elapsed();
    assert_eq!(verified.as_bytes(), plain.as_slice());
    println!("bounded synthetic complete-buffer observation: plaintext_bytes={} ciphertext_bytes={} encrypt_seconds={:.6} verify_seconds={:.6}; one host test process, not throughput/capacity qualification", plain.len(), encrypted.ciphertext.len(), encrypt_elapsed.as_secs_f64(), verify_elapsed.as_secs_f64());
}
