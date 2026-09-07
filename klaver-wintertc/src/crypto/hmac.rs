//! Raw HMAC sign/verify over key bytes, dispatching on [`super::digest::Algo`] to pick the
//! underlying hash - mirrors `super::digest::DigestImpl`'s enum-dispatch style.

use hmac::{Hmac, KeyInit, Mac};
use sha1::Sha1;
use sha2::{Sha256, Sha384, Sha512};

use super::digest::Algo;

pub fn sign(hash: Algo, key: &[u8], data: &[u8]) -> Vec<u8> {
    // HMAC accepts a key of any length (RFC 2104 - short keys are zero-padded, long keys are
    // themselves hashed first), so `new_from_slice` never actually fails here.
    match hash {
        Algo::Sha1 => {
            let mut mac = Hmac::<Sha1>::new_from_slice(key).expect("HMAC accepts any key length");
            mac.update(data);
            mac.finalize().into_bytes().to_vec()
        }
        Algo::Sha256 => {
            let mut mac = Hmac::<Sha256>::new_from_slice(key).expect("HMAC accepts any key length");
            mac.update(data);
            mac.finalize().into_bytes().to_vec()
        }
        Algo::Sha384 => {
            let mut mac = Hmac::<Sha384>::new_from_slice(key).expect("HMAC accepts any key length");
            mac.update(data);
            mac.finalize().into_bytes().to_vec()
        }
        Algo::Sha512 => {
            let mut mac = Hmac::<Sha512>::new_from_slice(key).expect("HMAC accepts any key length");
            mac.update(data);
            mac.finalize().into_bytes().to_vec()
        }
    }
}

/// Constant-time tag comparison via [`Mac::verify_slice`] - never throws, per spec `verify()`
/// resolves to `false` rather than rejecting on a mismatched signature.
pub fn verify(hash: Algo, key: &[u8], data: &[u8], signature: &[u8]) -> bool {
    match hash {
        Algo::Sha1 => {
            let Ok(mut mac) = Hmac::<Sha1>::new_from_slice(key) else {
                return false;
            };
            mac.update(data);
            mac.verify_slice(signature).is_ok()
        }
        Algo::Sha256 => {
            let Ok(mut mac) = Hmac::<Sha256>::new_from_slice(key) else {
                return false;
            };
            mac.update(data);
            mac.verify_slice(signature).is_ok()
        }
        Algo::Sha384 => {
            let Ok(mut mac) = Hmac::<Sha384>::new_from_slice(key) else {
                return false;
            };
            mac.update(data);
            mac.verify_slice(signature).is_ok()
        }
        Algo::Sha512 => {
            let Ok(mut mac) = Hmac::<Sha512>::new_from_slice(key) else {
                return false;
            };
            mac.update(data);
            mac.verify_slice(signature).is_ok()
        }
    }
}

/// The default HMAC key length (in bits) per `HmacKeyGenParams` when `length` is omitted: the
/// intrinsic block size of the underlying hash function.
pub fn default_key_length_bits(hash: Algo) -> u32 {
    match hash {
        Algo::Sha1 | Algo::Sha256 => 512,
        Algo::Sha384 | Algo::Sha512 => 1024,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// RFC 4231 test case 1 (HMAC-SHA-256).
    #[test]
    fn matches_rfc4231_test_case_1_hmac_sha256() {
        let key = [0x0bu8; 20];
        let data = b"Hi There";
        let expected = [
            0xb0, 0x34, 0x4c, 0x61, 0xd8, 0xdb, 0x38, 0x53, 0x5c, 0xa8, 0xaf, 0xce, 0xaf, 0x0b,
            0xf1, 0x2b, 0x88, 0x1d, 0xc2, 0x00, 0xc9, 0x83, 0x3d, 0xa7, 0x26, 0xe9, 0x37, 0x6c,
            0x2e, 0x32, 0xcf, 0xf7,
        ];
        assert_eq!(sign(Algo::Sha256, &key, data), expected);
        assert!(verify(Algo::Sha256, &key, data, &expected));
    }

    #[test]
    fn verify_returns_false_rather_than_erroring_on_mismatch() {
        let key = b"key";
        let signature = sign(Algo::Sha256, key, b"data");
        assert!(!verify(Algo::Sha256, key, b"different data", &signature));

        let mut tampered = signature.clone();
        tampered[0] ^= 1;
        assert!(!verify(Algo::Sha256, key, b"data", &tampered));

        assert!(!verify(
            Algo::Sha256,
            key,
            b"data",
            &signature[..signature.len() - 1]
        ));
    }

    #[test]
    fn round_trips_for_every_hash() {
        for hash in [Algo::Sha1, Algo::Sha256, Algo::Sha384, Algo::Sha512] {
            let key = b"a reasonably long test key, longer than any block size";
            let signature = sign(hash, key, b"message");
            assert!(verify(hash, key, b"message", &signature));
        }
    }
}
