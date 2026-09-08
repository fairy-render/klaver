//! Low-level RSA sign/verify (PKCS#1 v1.5), encrypt/decrypt (OAEP), and PKCS#8/SPKI/JWK-component
//! access, with no JS/rquickjs types in sight - callers (`crypto::key`, `crypto::module`) own
//! translating [`RsaError`] into the right named `DOMException`. Mirrors `aes.rs`'s split.
//!
//! Deliberately never goes through `rsa`'s own generic `Digest`-parametrized sign/verify path
//! (`pkcs1v15::SigningKey<D>`): this crate's own `sha1`/`sha2` (used by `digest.rs`) are a
//! different major version of the `digest` trait than what `rsa` 0.9 depends on internally, so the
//! two aren't interchangeable as a `D: Digest` type parameter. Instead, PKCS#1 v1.5 sign/verify
//! hashes with `digest::Algo`/`Digest` and builds `Pkcs1v15Sign` directly from its public
//! `hash_len`/`prefix` fields, with the ASN.1 DigestInfo prefix (RFC 8017 Appendix B.1 - the same
//! well-known constant bytes every RSA implementation hardcodes, e.g. Go's `crypto/rsa`) computed
//! by hand in [`digest_info_prefix`]. This works uniformly for all four hashes, including SHA-1
//! (which `rsa` doesn't re-export a compatible digest type for anyway).
//!
//! OAEP has no such manual-construction escape hatch (its digest usage runs deeper than a fixed
//! prefix), so it goes through `rsa`'s own re-exported `sha2` types instead (this crate's `sha2`
//! Cargo feature) - limited to SHA-256/384/512, since `rsa` has no compatible SHA-1 re-export.
//!
//! RSA keygen/encrypt/sign also need an RNG satisfying `rsa`'s `CryptoRngCore` bound, which is
//! `rand_core` 0.6 - a different major version than this workspace's `rand = "0.10"` (rand_core
//! 0.10) used by `ec.rs`. Always use `rsa::rand_core::OsRng` (via `rsa`'s `getrandom` feature) here,
//! never `rand::rng()`.

use rsa::rand_core::OsRng;
use rsa::sha2::{Sha256, Sha384, Sha512};
use rsa::traits::{PrivateKeyParts, PublicKeyParts};
use rsa::{BigUint, Oaep, Pkcs1v15Sign, RsaPrivateKey, RsaPublicKey};

use super::digest::Algo;

#[derive(Clone)]
pub enum RsaKeyPair {
    Private(RsaPrivateKey),
    Public(RsaPublicKey),
}

impl RsaKeyPair {
    pub fn public_key(&self) -> RsaPublicKey {
        match self {
            Self::Private(k) => k.to_public_key(),
            Self::Public(k) => k.clone(),
        }
    }
}

/// `(modulusLength, publicExponent)` for `CryptoKey.algorithm`, derived from the key itself -
/// used right after import (`"pkcs8"`/`"spki"`/`"jwk"` don't otherwise carry these as separate,
/// pre-validated fields the way `generateKey`'s `RsaHashedKeyGenParams` does).
pub fn key_algorithm_params(pair: &RsaKeyPair) -> (u32, Vec<u8>) {
    let public = pair.public_key();
    (public.n().bits() as u32, public.e().to_bytes_be())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RsaError {
    /// Key material is malformed (bad `n`/`e`/`d`/primes, or undecodable PKCS#8/SPKI DER).
    InvalidKey,
    /// A sign/encrypt/decrypt operation itself failed (includes AEAD-style ciphertext/signature
    /// rejection - spec-wise this and [`Self::InvalidKey`] both become an `OperationError`/
    /// `DataError` depending on which operation raised it; see `module.rs`'s mapping).
    OperationFailed,
    /// The requested hash isn't usable with this operation (OAEP only supports SHA-256/384/512).
    UnsupportedHash,
}

impl From<rsa::Error> for RsaError {
    fn from(_: rsa::Error) -> Self {
        RsaError::OperationFailed
    }
}

impl From<rsa::pkcs8::Error> for RsaError {
    fn from(_: rsa::pkcs8::Error) -> Self {
        RsaError::InvalidKey
    }
}

impl From<rsa::pkcs8::spki::Error> for RsaError {
    fn from(_: rsa::pkcs8::spki::Error) -> Self {
        RsaError::InvalidKey
    }
}

/// Generates an RSA keypair with the given modulus length (bits) and public exponent (big-endian
/// bytes, e.g. `[1, 0, 1]` for 65537).
pub fn generate_keypair(
    modulus_length_bits: u32,
    public_exponent: &[u8],
) -> Result<(RsaPrivateKey, RsaPublicKey), RsaError> {
    let exponent = BigUint::from_bytes_be(public_exponent);
    let private = RsaPrivateKey::new_with_exp(&mut OsRng, modulus_length_bits as usize, &exponent)
        .map_err(|_| RsaError::InvalidKey)?;
    let public = private.to_public_key();
    Ok((private, public))
}

/// The ASN.1 DigestInfo prefix each PKCS#1 v1.5 signature's hash is wrapped in, per RFC 8017
/// Appendix B.1. Stable, well-known constants - not computed from any crate's OID machinery.
fn digest_info_prefix(hash: Algo) -> &'static [u8] {
    match hash {
        Algo::Sha1 => &[
            0x30, 0x21, 0x30, 0x09, 0x06, 0x05, 0x2b, 0x0e, 0x03, 0x02, 0x1a, 0x05, 0x00, 0x04,
            0x14,
        ],
        Algo::Sha256 => &[
            0x30, 0x31, 0x30, 0x0d, 0x06, 0x09, 0x60, 0x86, 0x48, 0x01, 0x65, 0x03, 0x04, 0x02,
            0x01, 0x05, 0x00, 0x04, 0x20,
        ],
        Algo::Sha384 => &[
            0x30, 0x41, 0x30, 0x0d, 0x06, 0x09, 0x60, 0x86, 0x48, 0x01, 0x65, 0x03, 0x04, 0x02,
            0x02, 0x05, 0x00, 0x04, 0x30,
        ],
        Algo::Sha512 => &[
            0x30, 0x51, 0x30, 0x0d, 0x06, 0x09, 0x60, 0x86, 0x48, 0x01, 0x65, 0x03, 0x04, 0x02,
            0x03, 0x05, 0x00, 0x04, 0x40,
        ],
    }
}

fn digest_len(hash: Algo) -> usize {
    match hash {
        Algo::Sha1 => 20,
        Algo::Sha256 => 32,
        Algo::Sha384 => 48,
        Algo::Sha512 => 64,
    }
}

fn pkcs1v15_padding(hash: Algo) -> Pkcs1v15Sign {
    Pkcs1v15Sign {
        hash_len: Some(digest_len(hash)),
        prefix: digest_info_prefix(hash).to_vec().into_boxed_slice(),
    }
}

/// `hashed` must already be the result of hashing the message with `hash` (callers hash via
/// `digest::Digest`, matching WebCrypto's `sign()` contract of hashing internally before padding).
pub fn pkcs1v15_sign(key: &RsaPrivateKey, hash: Algo, hashed: &[u8]) -> Result<Vec<u8>, RsaError> {
    Ok(key.sign_with_rng(&mut OsRng, pkcs1v15_padding(hash), hashed)?)
}

/// Never errors - constant-time-ish rejection via `Result::is_ok()`, matching this crate's other
/// `verify()` conventions (see `hmac::verify`) of resolving to `false` rather than throwing.
pub fn pkcs1v15_verify(key: &RsaPublicKey, hash: Algo, hashed: &[u8], signature: &[u8]) -> bool {
    key.verify(pkcs1v15_padding(hash), hashed, signature).is_ok()
}

enum OaepDigest {
    Sha256,
    Sha384,
    Sha512,
}

fn oaep_digest(hash: Algo) -> Result<OaepDigest, RsaError> {
    match hash {
        Algo::Sha256 => Ok(OaepDigest::Sha256),
        Algo::Sha384 => Ok(OaepDigest::Sha384),
        Algo::Sha512 => Ok(OaepDigest::Sha512),
        Algo::Sha1 => Err(RsaError::UnsupportedHash),
    }
}

fn oaep_padding(hash: Algo, label: Option<&[u8]>) -> Result<Oaep, RsaError> {
    // `label` is spec'd as a `BufferSource` (arbitrary bytes), but `Oaep::new_with_label` takes
    // `impl AsRef<str>` - WebCrypto's own `RsaOaepParams.label` is virtually always ASCII/empty in
    // practice (it's an application-chosen context tag, not attacker/ciphertext-derived), so a
    // lossy UTF-8 decode here is an acceptable, spec-compatible-in-practice trade-off.
    let label = label.map(|bytes| std::string::String::from_utf8_lossy(bytes).into_owned());
    Ok(match (oaep_digest(hash)?, label) {
        (OaepDigest::Sha256, Some(label)) => Oaep::new_with_label::<Sha256, _>(label),
        (OaepDigest::Sha256, None) => Oaep::new::<Sha256>(),
        (OaepDigest::Sha384, Some(label)) => Oaep::new_with_label::<Sha384, _>(label),
        (OaepDigest::Sha384, None) => Oaep::new::<Sha384>(),
        (OaepDigest::Sha512, Some(label)) => Oaep::new_with_label::<Sha512, _>(label),
        (OaepDigest::Sha512, None) => Oaep::new::<Sha512>(),
    })
}

pub fn oaep_encrypt(
    key: &RsaPublicKey,
    hash: Algo,
    label: Option<&[u8]>,
    plaintext: &[u8],
) -> Result<Vec<u8>, RsaError> {
    let padding = oaep_padding(hash, label)?;
    Ok(key.encrypt(&mut OsRng, padding, plaintext)?)
}

pub fn oaep_decrypt(
    key: &RsaPrivateKey,
    hash: Algo,
    label: Option<&[u8]>,
    ciphertext: &[u8],
) -> Result<Vec<u8>, RsaError> {
    let padding = oaep_padding(hash, label)?;
    Ok(key.decrypt(padding, ciphertext)?)
}

pub fn to_pkcs8_der(key: &RsaPrivateKey) -> Result<Vec<u8>, RsaError> {
    use rsa::pkcs8::EncodePrivateKey;
    Ok(key.to_pkcs8_der()?.as_bytes().to_vec())
}

pub fn from_pkcs8_der(bytes: &[u8]) -> Result<RsaPrivateKey, RsaError> {
    use rsa::pkcs8::DecodePrivateKey;
    Ok(RsaPrivateKey::from_pkcs8_der(bytes)?)
}

pub fn to_spki_der(key: &RsaPublicKey) -> Result<Vec<u8>, RsaError> {
    use rsa::pkcs8::EncodePublicKey;
    Ok(key.to_public_key_der()?.as_bytes().to_vec())
}

pub fn from_spki_der(bytes: &[u8]) -> Result<RsaPublicKey, RsaError> {
    use rsa::pkcs8::DecodePublicKey;
    Ok(RsaPublicKey::from_public_key_der(bytes)?)
}

/// `n`/`e`/`d`/`p`/`q`/`dp`/`dq`/`qi`, each as raw big-endian bytes, for JWK export. `d`/`p`/`q`/
/// `dp`/`dq`/`qi` are only present for a private key.
pub struct RsaComponents {
    pub n: Vec<u8>,
    pub e: Vec<u8>,
    pub d: Option<Vec<u8>>,
    pub p: Option<Vec<u8>>,
    pub q: Option<Vec<u8>>,
    pub dp: Option<Vec<u8>>,
    pub dq: Option<Vec<u8>>,
    pub qi: Option<Vec<u8>>,
}

pub fn public_components(key: &RsaPublicKey) -> RsaComponents {
    RsaComponents {
        n: key.n().to_bytes_be(),
        e: key.e().to_bytes_be(),
        d: None,
        p: None,
        q: None,
        dp: None,
        dq: None,
        qi: None,
    }
}

/// Only two-prime RSA keys are supported for JWK export (returns [`RsaError::InvalidKey`]
/// otherwise) - the same restriction WebCrypto itself imposes (`RsaPrivateKey.d/p/q` implies a
/// two-prime key; multi-prime `otherPrimeInfos` is a JWK extension no browser actually exports).
pub fn private_components(key: &RsaPrivateKey) -> Result<RsaComponents, RsaError> {
    let primes = key.primes();
    if primes.len() != 2 {
        return Err(RsaError::InvalidKey);
    }
    let (dp, dq) = (key.dp(), key.dq());
    let qi = key.crt_coefficient();
    Ok(RsaComponents {
        n: key.n().to_bytes_be(),
        e: key.e().to_bytes_be(),
        d: Some(key.d().to_bytes_be()),
        p: Some(primes[0].to_bytes_be()),
        q: Some(primes[1].to_bytes_be()),
        dp: dp.map(|v| v.to_bytes_be()),
        dq: dq.map(|v| v.to_bytes_be()),
        qi: qi.map(|v| v.to_bytes_be()),
    })
}

pub fn public_key_from_components(n: &[u8], e: &[u8]) -> Result<RsaPublicKey, RsaError> {
    RsaPublicKey::new(BigUint::from_bytes_be(n), BigUint::from_bytes_be(e))
        .map_err(|_| RsaError::InvalidKey)
}

pub fn private_key_from_components(
    n: &[u8],
    e: &[u8],
    d: &[u8],
    p: &[u8],
    q: &[u8],
) -> Result<RsaPrivateKey, RsaError> {
    let mut key = RsaPrivateKey::from_components(
        BigUint::from_bytes_be(n),
        BigUint::from_bytes_be(e),
        BigUint::from_bytes_be(d),
        vec![BigUint::from_bytes_be(p), BigUint::from_bytes_be(q)],
    )
    .map_err(|_| RsaError::InvalidKey)?;
    key.precompute().map_err(|_| RsaError::InvalidKey)?;
    Ok(key)
}
