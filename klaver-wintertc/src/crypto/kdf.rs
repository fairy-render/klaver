//! Low-level HKDF (RFC 5869)/PBKDF2 (RFC 8018) byte derivation, with no JS/rquickjs types in
//! sight - mirrors `hmac.rs`'s enum-dispatch-on-`Algo` style. Unlike `rsa.rs`/`ec.rs`, there's no
//! separate-dependency-tree landmine here: the `hkdf`/`pbkdf2` crates both depend on the exact
//! same `hmac = "0.13"`/`sha1 = "0.11"`/`sha2 = "0.11"` line already used elsewhere in this crate
//! (see `Cargo.toml`), so this crate's own `Sha1`/`Sha256`/`Sha384`/`Sha512` types are used
//! directly - no `rsa`/`p256`-style reexport indirection needed.

use hkdf::Hkdf;
use sha1::Sha1;
use sha2::{Sha256, Sha384, Sha512};

use super::digest::Algo;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KdfError {
    /// HKDF-Expand's `length` exceeds `255 * hash output size` (RFC 5869 §2.3).
    InvalidLength,
}

/// `length` is in bytes (already validated/converted from `deriveBits`'s bit length by the
/// caller). HKDF has no notion of a "verify"/round-trip check - it's a one-shot KDF.
pub fn hkdf_derive_bits(
    hash: Algo,
    salt: &[u8],
    ikm: &[u8],
    info: &[u8],
    length: usize,
) -> Result<Vec<u8>, KdfError> {
    let mut okm = vec![0u8; length];
    let result = match hash {
        Algo::Sha1 => Hkdf::<Sha1>::new(Some(salt), ikm).expand(info, &mut okm),
        Algo::Sha256 => Hkdf::<Sha256>::new(Some(salt), ikm).expand(info, &mut okm),
        Algo::Sha384 => Hkdf::<Sha384>::new(Some(salt), ikm).expand(info, &mut okm),
        Algo::Sha512 => Hkdf::<Sha512>::new(Some(salt), ikm).expand(info, &mut okm),
    };
    result.map_err(|_| KdfError::InvalidLength)?;
    Ok(okm)
}

/// `length` is in bytes. PBKDF2 has no failure mode of its own (any `iterations`/`length`
/// combination is valid) - `pbkdf2_hmac` never errors.
pub fn pbkdf2_derive_bits(
    hash: Algo,
    password: &[u8],
    salt: &[u8],
    iterations: u32,
    length: usize,
) -> Vec<u8> {
    let mut out = vec![0u8; length];
    match hash {
        Algo::Sha1 => pbkdf2::pbkdf2_hmac::<Sha1>(password, salt, iterations, &mut out),
        Algo::Sha256 => pbkdf2::pbkdf2_hmac::<Sha256>(password, salt, iterations, &mut out),
        Algo::Sha384 => pbkdf2::pbkdf2_hmac::<Sha384>(password, salt, iterations, &mut out),
        Algo::Sha512 => pbkdf2::pbkdf2_hmac::<Sha512>(password, salt, iterations, &mut out),
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// RFC 5869 Appendix A.1 (HKDF-SHA-256) test vector.
    #[test]
    fn matches_rfc5869_test_case_1() {
        let ikm = [0x0bu8; 22];
        let salt: [u8; 13] = [
            0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c,
        ];
        let info: [u8; 10] = [0xf0, 0xf1, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7, 0xf8, 0xf9];
        let expected: [u8; 42] = [
            0x3c, 0xb2, 0x5f, 0x25, 0xfa, 0xac, 0xd5, 0x7a, 0x90, 0x43, 0x4f, 0x64, 0xd0, 0x36,
            0x2f, 0x2a, 0x2d, 0x2d, 0x0a, 0x90, 0xcf, 0x1a, 0x5a, 0x4c, 0x5d, 0xb0, 0x2d, 0x56,
            0xec, 0xc4, 0xc5, 0xbf, 0x34, 0x00, 0x72, 0x08, 0xd5, 0xb8, 0x87, 0x18, 0x58, 0x65,
        ];
        let okm = hkdf_derive_bits(Algo::Sha256, &salt, &ikm, &info, 42).unwrap();
        assert_eq!(okm, expected);
    }

    #[test]
    fn pbkdf2_matches_rfc6070_test_case_1() {
        let expected = [
            0x0c, 0x60, 0xc8, 0x0f, 0x96, 0x1f, 0x0e, 0x71, 0xf3, 0xa9, 0xb5, 0x24, 0xaf, 0x60,
            0x12, 0x06, 0x2f, 0xe0, 0x37, 0xa6,
        ];
        let out = pbkdf2_derive_bits(Algo::Sha1, b"password", b"salt", 1, 20);
        assert_eq!(out, expected);
    }

    #[test]
    fn hkdf_output_length_matches_request() {
        let out = hkdf_derive_bits(Algo::Sha256, b"salt", b"ikm", b"info", 64).unwrap();
        assert_eq!(out.len(), 64);
    }
}
