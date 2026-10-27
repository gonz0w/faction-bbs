use super::*;

fn bytes(raw: &str) -> Vec<u8> {
    raw.as_bytes().chunks_exact(2).map(|p| {
        u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap()
    }).collect()
}

#[test]
fn retained_nist_cavs_vectors_match_exact_ciphertext_and_tag() {
    // Same independent CAVS14 public vectors retained by issue250/proof.py.
    for (key, iv, plain, expected) in [
        ("b52c505a37d78eda5dd34f20c22540ea1b58963cf8e5bf8ffa85f9f2492505b4",
         "516c33929df5a3284ff463d7", "", "bdc1ac884d332457a1d2664f168c76f0"),
        ("31bdadd96698c204aa9ce1448ea94ae1fb4a9a0b3c9d773b51bb1822666b8f22",
         "0d18e06c7c725ac9e362e1ce", "2db5168e932556f8089a0622981d017d",
         "fa4362189661d163fcd6a56d8bf0405ad636ac1bbedd5cc3ee727dc2ab4a9489"),
    ] {
        let key: [u8; 32] = bytes(key).try_into().unwrap();
        let iv: [u8; 12] = bytes(iv).try_into().unwrap();
        let plain = bytes(plain);
        let expected = bytes(expected);
        let encrypted = crypt(true, &plain, &key, &iv).unwrap();
        assert_eq!(*encrypted, expected);
        assert_eq!(*crypt(false, &expected, &key, &iv).unwrap(), plain);
        let mut corrupt = expected;
        *corrupt.last_mut().unwrap() ^= 1;
        assert!(crypt(false, &corrupt, &key, &iv).is_err());
    }
}

#[test]
fn native_authentication_failure_wipes_entire_provisional_output() {
    let key = [7; 32];
    let iv = [8; 12];
    let mut ciphertext = crypt(true, b"secret provisional bytes", &key, &iv).unwrap();
    *ciphertext.last_mut().unwrap() ^= 1;
    let mut out = vec![0xa5; ciphertext.len()];
    let mut written = 99;
    assert_ne!(unsafe {
        faction_blob_crypt(0, key.as_ptr(), iv.as_ptr(), ciphertext.as_ptr(), ciphertext.len(), out.as_mut_ptr(), out.len(), &mut written)
    }, 0);
    assert_eq!(written, 0);
    assert!(out.iter().all(|byte| *byte == 0));
}
