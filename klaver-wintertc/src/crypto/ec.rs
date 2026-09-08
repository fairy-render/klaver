//! Low-level ECDSA sign/verify, ECDH shared-secret derivation, and PKCS#8/SPKI/raw/JWK-component
//! access for P-256/P-384/P-521, with no JS/rquickjs types in sight - callers (`crypto::key`,
//! `crypto::module`) own translating [`EcError`] into the right named `DOMException`. Mirrors
//! `rsa.rs`'s split (and, like it, hashes the message itself via `digest::Algo` and only ever
//! passes already-hashed bytes into the underlying crate - `PrehashSigner`/`PrehashVerifier`, not
//! the convenience `Signer`/`Verifier` path - since WebCrypto's ECDSA hash is a per-call parameter,
//! not fixed to the curve's "native" digest the way `signature::Signer::sign()` would assume).
//!
//! Unlike `rsa.rs`, there's no `rand_core`-version landmine here: `p256`/`p384`/`p521` 0.14 (via
//! `elliptic-curve` 0.14) depend on `rand_core = "0.10"`, the same major version as this
//! workspace's own `rand = "0.10"` - `rand::rng()` is used directly for keygen.
//!
//! `p256`/`p384`/`p521` all depend on the exact same `pkcs8`/`spki` (0.11/0.8) versions as each
//! other (confirmed via `Cargo.lock` - only one copy of each resolves), so a single
//! `use p256::pkcs8::…` import's traits apply equally to `p384`'s/`p521`'s types; no per-curve
//! aliasing needed (contrast `rsa.rs`, which must never mix its pkcs8 0.10 with this 0.11).

use p256::ecdsa::signature::hazmat::{PrehashSigner, PrehashVerifier};
use p256::pkcs8::spki::{DecodePublicKey, EncodePublicKey};
use p256::pkcs8::{DecodePrivateKey, EncodePrivateKey};

use super::key::EcCurve;

#[derive(Clone)]
pub enum EcKeyPair {
    P256Private(p256::SecretKey),
    P256Public(p256::PublicKey),
    P384Private(p384::SecretKey),
    P384Public(p384::PublicKey),
    P521Private(p521::SecretKey),
    P521Public(p521::PublicKey),
}

impl EcKeyPair {
    pub fn curve(&self) -> EcCurve {
        match self {
            Self::P256Private(_) | Self::P256Public(_) => EcCurve::P256,
            Self::P384Private(_) | Self::P384Public(_) => EcCurve::P384,
            Self::P521Private(_) | Self::P521Public(_) => EcCurve::P521,
        }
    }

    pub fn is_private(&self) -> bool {
        matches!(
            self,
            Self::P256Private(_) | Self::P384Private(_) | Self::P521Private(_)
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EcError {
    /// Key material is malformed (bad SEC1 point, bad scalar, undecodable PKCS#8/SPKI DER, or a
    /// curve/key-kind mismatch between the two sides of an operation).
    InvalidKey,
    /// A sign/derive operation itself failed.
    OperationFailed,
}

pub fn generate_keypair(curve: EcCurve) -> (EcKeyPair, EcKeyPair) {
    let mut rng = rand::rng();
    match curve {
        EcCurve::P256 => {
            let secret = p256::SecretKey::random(&mut rng);
            let public = secret.public_key();
            (EcKeyPair::P256Private(secret), EcKeyPair::P256Public(public))
        }
        EcCurve::P384 => {
            let secret = p384::SecretKey::random(&mut rng);
            let public = secret.public_key();
            (EcKeyPair::P384Private(secret), EcKeyPair::P384Public(public))
        }
        EcCurve::P521 => {
            let secret = p521::SecretKey::random(&mut rng);
            let public = secret.public_key();
            (EcKeyPair::P521Private(secret), EcKeyPair::P521Public(public))
        }
    }
}

/// `prehash` must already be the result of hashing the message with the caller-chosen hash
/// (WebCrypto's ECDSA hash is a per-`sign()`-call parameter, not fixed to the key/curve).
pub fn ecdsa_sign(key: &EcKeyPair, prehash: &[u8]) -> Result<Vec<u8>, EcError> {
    match key {
        EcKeyPair::P256Private(sk) => {
            let signing_key: p256::ecdsa::SigningKey = sk.clone().into();
            let sig: p256::ecdsa::Signature = signing_key
                .sign_prehash(prehash)
                .map_err(|_| EcError::OperationFailed)?;
            Ok(sig.to_vec())
        }
        EcKeyPair::P384Private(sk) => {
            let signing_key: p384::ecdsa::SigningKey = sk.clone().into();
            let sig: p384::ecdsa::Signature = signing_key
                .sign_prehash(prehash)
                .map_err(|_| EcError::OperationFailed)?;
            Ok(sig.to_vec())
        }
        EcKeyPair::P521Private(sk) => {
            let signing_key: p521::ecdsa::SigningKey = sk.clone().into();
            let sig: p521::ecdsa::Signature = signing_key
                .sign_prehash(prehash)
                .map_err(|_| EcError::OperationFailed)?;
            Ok(sig.to_vec())
        }
        EcKeyPair::P256Public(_) | EcKeyPair::P384Public(_) | EcKeyPair::P521Public(_) => {
            Err(EcError::InvalidKey)
        }
    }
}

/// Never errors - constant-time-ish rejection via `Result::is_ok()`, matching this crate's other
/// `verify()` conventions (see `hmac::verify`, `rsa::pkcs1v15_verify`).
pub fn ecdsa_verify(key: &EcKeyPair, prehash: &[u8], signature: &[u8]) -> bool {
    match key {
        EcKeyPair::P256Public(pk) => {
            let Ok(sig) = p256::ecdsa::Signature::from_slice(signature) else {
                return false;
            };
            let verifying_key: p256::ecdsa::VerifyingKey = pk.clone().into();
            verifying_key.verify_prehash(prehash, &sig).is_ok()
        }
        EcKeyPair::P384Public(pk) => {
            let Ok(sig) = p384::ecdsa::Signature::from_slice(signature) else {
                return false;
            };
            let verifying_key: p384::ecdsa::VerifyingKey = pk.clone().into();
            verifying_key.verify_prehash(prehash, &sig).is_ok()
        }
        EcKeyPair::P521Public(pk) => {
            let Ok(sig) = p521::ecdsa::Signature::from_slice(signature) else {
                return false;
            };
            let verifying_key: p521::ecdsa::VerifyingKey = pk.clone().into();
            verifying_key.verify_prehash(prehash, &sig).is_ok()
        }
        EcKeyPair::P256Private(_) | EcKeyPair::P384Private(_) | EcKeyPair::P521Private(_) => false,
    }
}

/// Raw ECDH shared secret (no KDF applied - matches this milestone's "straight ECDH" scope, see
/// `module.rs`'s `derive_bits`/`derive_key`). Errors on a curve mismatch or if either side isn't
/// the expected key kind (private/public respectively) - callers should have already validated
/// `KeyUsage`/`KeyType` before reaching here (see `module.rs`'s `require_key_type`).
pub fn ecdh_derive_bits(private: &EcKeyPair, public: &EcKeyPair) -> Result<Vec<u8>, EcError> {
    match (private, public) {
        (EcKeyPair::P256Private(sk), EcKeyPair::P256Public(pk)) => {
            Ok(sk.diffie_hellman(pk).raw_secret_bytes().to_vec())
        }
        (EcKeyPair::P384Private(sk), EcKeyPair::P384Public(pk)) => {
            Ok(sk.diffie_hellman(pk).raw_secret_bytes().to_vec())
        }
        (EcKeyPair::P521Private(sk), EcKeyPair::P521Public(pk)) => {
            Ok(sk.diffie_hellman(pk).raw_secret_bytes().to_vec())
        }
        _ => Err(EcError::InvalidKey),
    }
}

/// The uncompressed SEC1 point (`0x04 || x || y`) - WebCrypto's `"raw"` format for EC public keys.
pub fn public_key_to_raw(key: &EcKeyPair) -> Result<Vec<u8>, EcError> {
    match key {
        EcKeyPair::P256Public(pk) => Ok(pk.to_sec1_bytes().to_vec()),
        EcKeyPair::P384Public(pk) => Ok(pk.to_sec1_bytes().to_vec()),
        EcKeyPair::P521Public(pk) => Ok(pk.to_sec1_bytes().to_vec()),
        _ => Err(EcError::InvalidKey),
    }
}

pub fn public_key_from_raw(curve: EcCurve, bytes: &[u8]) -> Result<EcKeyPair, EcError> {
    match curve {
        EcCurve::P256 => Ok(EcKeyPair::P256Public(
            p256::PublicKey::from_sec1_bytes(bytes).map_err(|_| EcError::InvalidKey)?,
        )),
        EcCurve::P384 => Ok(EcKeyPair::P384Public(
            p384::PublicKey::from_sec1_bytes(bytes).map_err(|_| EcError::InvalidKey)?,
        )),
        EcCurve::P521 => Ok(EcKeyPair::P521Public(
            p521::PublicKey::from_sec1_bytes(bytes).map_err(|_| EcError::InvalidKey)?,
        )),
    }
}

pub fn to_pkcs8_der(key: &EcKeyPair) -> Result<Vec<u8>, EcError> {
    match key {
        EcKeyPair::P256Private(sk) => Ok(sk
            .to_pkcs8_der()
            .map_err(|_| EcError::InvalidKey)?
            .as_bytes()
            .to_vec()),
        EcKeyPair::P384Private(sk) => Ok(sk
            .to_pkcs8_der()
            .map_err(|_| EcError::InvalidKey)?
            .as_bytes()
            .to_vec()),
        EcKeyPair::P521Private(sk) => Ok(sk
            .to_pkcs8_der()
            .map_err(|_| EcError::InvalidKey)?
            .as_bytes()
            .to_vec()),
        _ => Err(EcError::InvalidKey),
    }
}

pub fn from_pkcs8_der(curve: EcCurve, bytes: &[u8]) -> Result<EcKeyPair, EcError> {
    match curve {
        EcCurve::P256 => Ok(EcKeyPair::P256Private(
            p256::SecretKey::from_pkcs8_der(bytes).map_err(|_| EcError::InvalidKey)?,
        )),
        EcCurve::P384 => Ok(EcKeyPair::P384Private(
            p384::SecretKey::from_pkcs8_der(bytes).map_err(|_| EcError::InvalidKey)?,
        )),
        EcCurve::P521 => Ok(EcKeyPair::P521Private(
            p521::SecretKey::from_pkcs8_der(bytes).map_err(|_| EcError::InvalidKey)?,
        )),
    }
}

pub fn to_spki_der(key: &EcKeyPair) -> Result<Vec<u8>, EcError> {
    match key {
        EcKeyPair::P256Public(pk) => Ok(pk
            .to_public_key_der()
            .map_err(|_| EcError::InvalidKey)?
            .as_bytes()
            .to_vec()),
        EcKeyPair::P384Public(pk) => Ok(pk
            .to_public_key_der()
            .map_err(|_| EcError::InvalidKey)?
            .as_bytes()
            .to_vec()),
        EcKeyPair::P521Public(pk) => Ok(pk
            .to_public_key_der()
            .map_err(|_| EcError::InvalidKey)?
            .as_bytes()
            .to_vec()),
        _ => Err(EcError::InvalidKey),
    }
}

pub fn from_spki_der(curve: EcCurve, bytes: &[u8]) -> Result<EcKeyPair, EcError> {
    match curve {
        EcCurve::P256 => Ok(EcKeyPair::P256Public(
            p256::PublicKey::from_public_key_der(bytes).map_err(|_| EcError::InvalidKey)?,
        )),
        EcCurve::P384 => Ok(EcKeyPair::P384Public(
            p384::PublicKey::from_public_key_der(bytes).map_err(|_| EcError::InvalidKey)?,
        )),
        EcCurve::P521 => Ok(EcKeyPair::P521Public(
            p521::PublicKey::from_public_key_der(bytes).map_err(|_| EcError::InvalidKey)?,
        )),
    }
}

/// `crv`/`x`/`y` for public, plus `d` for private, all fixed-width big-endian per curve, for JWK
/// export (RFC 7518 §6.2).
pub struct EcComponents {
    pub curve: EcCurve,
    pub x: Vec<u8>,
    pub y: Vec<u8>,
    pub d: Option<Vec<u8>>,
}

/// Splits an uncompressed SEC1 point (`0x04 || x || y`) into its two equal-length coordinates.
fn split_sec1_point(bytes: &[u8]) -> (Vec<u8>, Vec<u8>) {
    let coord_len = (bytes.len() - 1) / 2;
    (
        bytes[1..1 + coord_len].to_vec(),
        bytes[1 + coord_len..].to_vec(),
    )
}

pub fn components(key: &EcKeyPair) -> EcComponents {
    match key {
        EcKeyPair::P256Public(pk) => {
            let (x, y) = split_sec1_point(&pk.to_sec1_bytes());
            EcComponents {
                curve: EcCurve::P256,
                x,
                y,
                d: None,
            }
        }
        EcKeyPair::P256Private(sk) => {
            let (x, y) = split_sec1_point(&sk.public_key().to_sec1_bytes());
            EcComponents {
                curve: EcCurve::P256,
                x,
                y,
                d: Some(sk.to_bytes().as_slice().to_vec()),
            }
        }
        EcKeyPair::P384Public(pk) => {
            let (x, y) = split_sec1_point(&pk.to_sec1_bytes());
            EcComponents {
                curve: EcCurve::P384,
                x,
                y,
                d: None,
            }
        }
        EcKeyPair::P384Private(sk) => {
            let (x, y) = split_sec1_point(&sk.public_key().to_sec1_bytes());
            EcComponents {
                curve: EcCurve::P384,
                x,
                y,
                d: Some(sk.to_bytes().as_slice().to_vec()),
            }
        }
        EcKeyPair::P521Public(pk) => {
            let (x, y) = split_sec1_point(&pk.to_sec1_bytes());
            EcComponents {
                curve: EcCurve::P521,
                x,
                y,
                d: None,
            }
        }
        EcKeyPair::P521Private(sk) => {
            let (x, y) = split_sec1_point(&sk.public_key().to_sec1_bytes());
            EcComponents {
                curve: EcCurve::P521,
                x,
                y,
                d: Some(sk.to_bytes().as_slice().to_vec()),
            }
        }
    }
}

/// Builds a public key from raw `x`/`y` JWK coordinates.
pub fn public_key_from_components(curve: EcCurve, x: &[u8], y: &[u8]) -> Result<EcKeyPair, EcError> {
    let mut point = Vec::with_capacity(1 + x.len() + y.len());
    point.push(0x04);
    point.extend_from_slice(x);
    point.extend_from_slice(y);
    public_key_from_raw(curve, &point)
}

/// Builds a private key from a raw `d` JWK scalar (the `x`/`y` coordinates are redundant with `d`
/// and aren't separately validated here, matching WebCrypto implementations' common practice).
pub fn private_key_from_components(curve: EcCurve, d: &[u8]) -> Result<EcKeyPair, EcError> {
    match curve {
        EcCurve::P256 => Ok(EcKeyPair::P256Private(
            p256::SecretKey::from_slice(d).map_err(|_| EcError::InvalidKey)?,
        )),
        EcCurve::P384 => Ok(EcKeyPair::P384Private(
            p384::SecretKey::from_slice(d).map_err(|_| EcError::InvalidKey)?,
        )),
        EcCurve::P521 => Ok(EcKeyPair::P521Private(
            p521::SecretKey::from_slice(d).map_err(|_| EcError::InvalidKey)?,
        )),
    }
}
