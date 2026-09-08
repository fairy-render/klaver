use rand::prelude::*;
use rquickjs::{
    Class, Ctx, FromJs, IntoJs, Object, TypedArray, Value,
    class::{Trace, Tracer},
};

use super::algorithm::{ImportAlgorithm, KeyGenAlgorithm, buffer_bytes};
use super::digest::Algo;
use super::jwk;
use klaver_core::{throw, value::Buffer};

#[cfg(feature = "crypto-asymmetric")]
use crate::dom_exception::DOMException;
#[cfg(feature = "crypto-asymmetric")]
use super::ec;
#[cfg(feature = "crypto-asymmetric")]
use super::rsa as rsa_backend;

/// Which of AES-GCM/CBC/CTR a `CryptoKey`/cipher operation targets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AesVariant {
    Gcm,
    Cbc,
    Ctr,
}

impl AesVariant {
    pub fn from_name(name: &str) -> Option<Self> {
        if name.eq_ignore_ascii_case("AES-GCM") {
            Some(Self::Gcm)
        } else if name.eq_ignore_ascii_case("AES-CBC") {
            Some(Self::Cbc)
        } else if name.eq_ignore_ascii_case("AES-CTR") {
            Some(Self::Ctr)
        } else {
            None
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Gcm => "AES-GCM",
            Self::Cbc => "AES-CBC",
            Self::Ctr => "AES-CTR",
        }
    }

    fn jwk_suffix(self) -> &'static str {
        match self {
            Self::Gcm => "GCM",
            Self::Cbc => "CBC",
            Self::Ctr => "CTR",
        }
    }
}

/// Which RSA-family algorithm a [`CryptoKey`]/subtle-crypto operation targets.
#[cfg(feature = "crypto-asymmetric")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RsaVariant {
    Pkcs1v15,
    Oaep,
    Pss,
}

#[cfg(feature = "crypto-asymmetric")]
impl RsaVariant {
    pub fn from_name(name: &str) -> Option<Self> {
        if name.eq_ignore_ascii_case("RSASSA-PKCS1-v1_5") {
            Some(Self::Pkcs1v15)
        } else if name.eq_ignore_ascii_case("RSA-OAEP") {
            Some(Self::Oaep)
        } else if name.eq_ignore_ascii_case("RSA-PSS") {
            Some(Self::Pss)
        } else {
            None
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Pkcs1v15 => "RSASSA-PKCS1-v1_5",
            Self::Oaep => "RSA-OAEP",
            Self::Pss => "RSA-PSS",
        }
    }
}

/// Which elliptic-curve algorithm a [`CryptoKey`]/subtle-crypto operation targets.
#[cfg(feature = "crypto-asymmetric")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EcVariant {
    Ecdsa,
    Ecdh,
}

#[cfg(feature = "crypto-asymmetric")]
impl EcVariant {
    pub fn from_name(name: &str) -> Option<Self> {
        if name.eq_ignore_ascii_case("ECDSA") {
            Some(Self::Ecdsa)
        } else if name.eq_ignore_ascii_case("ECDH") {
            Some(Self::Ecdh)
        } else {
            None
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Ecdsa => "ECDSA",
            Self::Ecdh => "ECDH",
        }
    }
}

/// A `namedCurve` this crate supports: P-256/P-384/P-521.
#[cfg(feature = "crypto-asymmetric")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EcCurve {
    P256,
    P384,
    P521,
}

#[cfg(feature = "crypto-asymmetric")]
impl EcCurve {
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "P-256" => Some(Self::P256),
            "P-384" => Some(Self::P384),
            "P-521" => Some(Self::P521),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::P256 => "P-256",
            Self::P384 => "P-384",
            Self::P521 => "P-521",
        }
    }

    /// The JWK `crv` field value - identical to [`EcCurve::name`] per RFC 7518 §6.2.1.1, but kept
    /// as a separate method since the two happen to only coincide by spec accident.
    pub fn jwk_crv(self) -> &'static str {
        self.name()
    }
}

/// Which key-derivation function a "derive-only" [`CryptoKey`] (imported straight from raw
/// bytes/a password, never generated/exported) targets. Both HKDF and PBKDF2 take their hash as a
/// per-`deriveBits()`/`deriveKey()`-call parameter rather than fixing it on the key (unlike HMAC),
/// so - unlike [`KeyAlgorithm::Hmac`] - this carries no `hash` field.
#[cfg(feature = "crypto-asymmetric")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeriveKind {
    Hkdf,
    Pbkdf2,
}

#[cfg(feature = "crypto-asymmetric")]
impl DeriveKind {
    pub fn name(self) -> &'static str {
        match self {
            Self::Hkdf => "HKDF",
            Self::Pbkdf2 => "PBKDF2",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyType {
    Secret,
    Private,
    Public,
}

impl KeyType {
    fn as_str(self) -> &'static str {
        match self {
            Self::Secret => "secret",
            Self::Private => "private",
            Self::Public => "public",
        }
    }
}

/// A single entry of `CryptoKey.usages` / the `keyUsages` parameter, per
/// <https://w3c.github.io/webcrypto/#dfn-KeyUsage>.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyUsage {
    Encrypt,
    Decrypt,
    Sign,
    Verify,
    DeriveKey,
    DeriveBits,
    WrapKey,
    UnwrapKey,
}

impl KeyUsage {
    fn as_str(self) -> &'static str {
        match self {
            Self::Encrypt => "encrypt",
            Self::Decrypt => "decrypt",
            Self::Sign => "sign",
            Self::Verify => "verify",
            Self::DeriveKey => "deriveKey",
            Self::DeriveBits => "deriveBits",
            Self::WrapKey => "wrapKey",
            Self::UnwrapKey => "unwrapKey",
        }
    }
}

impl<'js> FromJs<'js> for KeyUsage {
    fn from_js(ctx: &Ctx<'js>, value: Value<'js>) -> rquickjs::Result<Self> {
        let s = std::string::String::from_js(ctx, value)?;
        Ok(match s.as_str() {
            "encrypt" => Self::Encrypt,
            "decrypt" => Self::Decrypt,
            "sign" => Self::Sign,
            "verify" => Self::Verify,
            "deriveKey" => Self::DeriveKey,
            "deriveBits" => Self::DeriveBits,
            "wrapKey" => Self::WrapKey,
            "unwrapKey" => Self::UnwrapKey,
            _ => throw_dom!(
                ctx,
                "SyntaxError",
                format!("unrecognized key usage \"{s}\"")
            ),
        })
    }
}

impl<'js> IntoJs<'js> for KeyUsage {
    fn into_js(self, ctx: &Ctx<'js>) -> rquickjs::Result<Value<'js>> {
        self.as_str().into_js(ctx)
    }
}

/// The algorithm a [`CryptoKey`] was generated/imported under - mirrors the spec's
/// `KeyAlgorithm`/`AesKeyAlgorithm`/`HmacKeyAlgorithm` dictionaries. Reconstructed as a fresh JS
/// object on every `CryptoKey.algorithm` access (see [`CryptoKey::algorithm`]), never cached as a
/// live JS value, so this holds no `'js` state.
#[derive(Debug, Clone)]
pub enum KeyAlgorithm {
    Aes {
        name: AesVariant,
        length: u16,
    },
    Hmac {
        hash: Algo,
        length: u32,
    },
    #[cfg(feature = "crypto-asymmetric")]
    RsaHashed {
        variant: RsaVariant,
        modulus_length: u32,
        public_exponent: Vec<u8>,
        hash: Algo,
    },
    #[cfg(feature = "crypto-asymmetric")]
    Ec {
        variant: EcVariant,
        named_curve: EcCurve,
    },
    /// HKDF/PBKDF2 "derive-only" keys - just `{name}` per spec, no further fields (see
    /// [`DeriveKind`]'s doc comment for why there's no `hash`).
    #[cfg(feature = "crypto-asymmetric")]
    DeriveOnly(DeriveKind),
}

impl KeyAlgorithm {
    fn into_object<'js>(self, ctx: &Ctx<'js>) -> rquickjs::Result<Object<'js>> {
        let obj = Object::new(ctx.clone())?;
        match self {
            KeyAlgorithm::Aes { name, length } => {
                obj.set("name", name.name())?;
                obj.set("length", length)?;
            }
            KeyAlgorithm::Hmac { hash, length } => {
                obj.set("name", "HMAC")?;
                let hash_obj = Object::new(ctx.clone())?;
                hash_obj.set("name", hash.spec_name())?;
                obj.set("hash", hash_obj)?;
                obj.set("length", length)?;
            }
            #[cfg(feature = "crypto-asymmetric")]
            KeyAlgorithm::RsaHashed {
                variant,
                modulus_length,
                public_exponent,
                hash,
            } => {
                obj.set("name", variant.name())?;
                obj.set("modulusLength", modulus_length)?;
                obj.set(
                    "publicExponent",
                    TypedArray::<u8>::new(ctx.clone(), public_exponent)?,
                )?;
                let hash_obj = Object::new(ctx.clone())?;
                hash_obj.set("name", hash.spec_name())?;
                obj.set("hash", hash_obj)?;
            }
            #[cfg(feature = "crypto-asymmetric")]
            KeyAlgorithm::Ec {
                variant,
                named_curve,
            } => {
                obj.set("name", variant.name())?;
                obj.set("namedCurve", named_curve.name())?;
            }
            #[cfg(feature = "crypto-asymmetric")]
            KeyAlgorithm::DeriveOnly(kind) => {
                obj.set("name", kind.name())?;
            }
        }
        Ok(obj)
    }

    /// The `alg` tag WebCrypto's JWK export/import uses for this algorithm, per
    /// <https://w3c.github.io/webcrypto/#jwk-mapping-algorithm-2> (e.g. `"A128GCM"`, `"HS256"`).
    /// `None` for EC keys - the spec's JWK mapping table has no entry for ECDSA/ECDH (the curve,
    /// in `crv`, already identifies the key; `sign()`/`verify()` supply the hash per call rather
    /// than storing it on the key), so those JWKs are exported/imported with no `alg` field.
    fn jwk_alg_tag(&self) -> Option<std::string::String> {
        match self {
            KeyAlgorithm::Aes { name, length } => {
                let prefix = match length {
                    128 => "A128",
                    192 => "A192",
                    256 => "A256",
                    _ => "A???",
                };
                Some(format!("{prefix}{}", name.jwk_suffix()))
            }
            KeyAlgorithm::Hmac { hash, .. } => Some(
                match hash {
                    Algo::Sha1 => "HS1",
                    Algo::Sha256 => "HS256",
                    Algo::Sha384 => "HS384",
                    Algo::Sha512 => "HS512",
                }
                .to_string(),
            ),
            #[cfg(feature = "crypto-asymmetric")]
            KeyAlgorithm::RsaHashed { variant, hash, .. } => Some(
                match (variant, hash) {
                    (RsaVariant::Pkcs1v15, Algo::Sha1) => "RS1",
                    (RsaVariant::Pkcs1v15, Algo::Sha256) => "RS256",
                    (RsaVariant::Pkcs1v15, Algo::Sha384) => "RS384",
                    (RsaVariant::Pkcs1v15, Algo::Sha512) => "RS512",
                    (RsaVariant::Oaep, Algo::Sha1) => "RSA-OAEP",
                    (RsaVariant::Oaep, Algo::Sha256) => "RSA-OAEP-256",
                    (RsaVariant::Oaep, Algo::Sha384) => "RSA-OAEP-384",
                    (RsaVariant::Oaep, Algo::Sha512) => "RSA-OAEP-512",
                    (RsaVariant::Pss, Algo::Sha1) => "PS1",
                    (RsaVariant::Pss, Algo::Sha256) => "PS256",
                    (RsaVariant::Pss, Algo::Sha384) => "PS384",
                    (RsaVariant::Pss, Algo::Sha512) => "PS512",
                }
                .to_string(),
            ),
            #[cfg(feature = "crypto-asymmetric")]
            KeyAlgorithm::Ec { .. } => None,
            // Not reachable in practice - HKDF/PBKDF2 keys are never extractable (see
            // `import_derive_only_key`), so `export_key`'s JWK branch never runs for them, but the
            // mapping is still total over `KeyAlgorithm`.
            #[cfg(feature = "crypto-asymmetric")]
            KeyAlgorithm::DeriveOnly(_) => None,
        }
    }
}

/// Raw key bytes, tagged by which kind of key they are.
enum KeyMaterial {
    Aes(Vec<u8>),
    Hmac(Vec<u8>),
    #[cfg(feature = "crypto-asymmetric")]
    Rsa(rsa_backend::RsaKeyPair),
    #[cfg(feature = "crypto-asymmetric")]
    Ec(ec::EcKeyPair),
    /// Raw bytes (or a password, for PBKDF2) behind an HKDF/PBKDF2 "derive-only" key - see
    /// [`DeriveKind`].
    #[cfg(feature = "crypto-asymmetric")]
    Derive(Vec<u8>),
}

impl KeyMaterial {
    /// Only ever called for symmetric (AES/HMAC/derive-only) material - callers already know the
    /// kind from the `KeyAlgorithm`/`ImportAlgorithm` they dispatched on, same invariant
    /// `require_aes_variant`/`require_hmac` establish in `module.rs` before reaching for key
    /// bytes.
    fn bytes(&self) -> &[u8] {
        match self {
            Self::Aes(b) | Self::Hmac(b) => b,
            #[cfg(feature = "crypto-asymmetric")]
            Self::Derive(b) => b,
            #[cfg(feature = "crypto-asymmetric")]
            Self::Rsa(_) | Self::Ec(_) => {
                unreachable!("bytes() is only called for symmetric key material")
            }
        }
    }
}

/// `CryptoKey`, per <https://w3c.github.io/webcrypto/#cryptokey-interface>. Per spec there is no
/// constructor - the only way to get an instance is `generateKey`/`importKey`/`unwrapKey`, so
/// (like `Performance`, see `performance.rs`) the JS-visible constructor just throws, and
/// `CryptoKey::create` is the real, Rust-only way to build one.
#[derive(rquickjs::JsLifetime)]
#[rquickjs::class]
pub struct CryptoKey {
    ty: KeyType,
    extractable: bool,
    usages: Vec<KeyUsage>,
    algorithm: KeyAlgorithm,
    material: KeyMaterial,
}

impl<'js> Trace<'js> for CryptoKey {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

impl CryptoKey {
    fn create(
        ty: KeyType,
        extractable: bool,
        usages: Vec<KeyUsage>,
        algorithm: KeyAlgorithm,
        material: KeyMaterial,
    ) -> Self {
        Self {
            ty,
            extractable,
            usages,
            algorithm,
            material,
        }
    }

    pub(crate) fn usage_list(&self) -> &[KeyUsage] {
        &self.usages
    }

    pub(crate) fn algorithm_variant(&self) -> KeyAlgorithm {
        self.algorithm.clone()
    }

    #[cfg(feature = "crypto-asymmetric")]
    pub(crate) fn key_type(&self) -> KeyType {
        self.ty
    }

    #[cfg(feature = "crypto-asymmetric")]
    pub(crate) fn rsa_key_pair(&self) -> Option<&rsa_backend::RsaKeyPair> {
        match &self.material {
            KeyMaterial::Rsa(kp) => Some(kp),
            _ => None,
        }
    }

    #[cfg(feature = "crypto-asymmetric")]
    pub(crate) fn ec_key_pair(&self) -> Option<&ec::EcKeyPair> {
        match &self.material {
            KeyMaterial::Ec(kp) => Some(kp),
            _ => None,
        }
    }

    pub(crate) fn key_bytes(&self) -> &[u8] {
        self.material.bytes()
    }
}

#[rquickjs::methods]
impl CryptoKey {
    #[qjs(constructor)]
    pub fn ctor<'js>(ctx: Ctx<'js>) -> rquickjs::Result<Self> {
        throw!(@type ctx, "Illegal constructor")
    }

    #[qjs(get, rename = "type")]
    pub fn get_type(&self) -> &'static str {
        self.ty.as_str()
    }

    #[qjs(get)]
    pub fn extractable(&self) -> bool {
        self.extractable
    }

    #[qjs(get)]
    pub fn usages(&self) -> Vec<KeyUsage> {
        self.usages.clone()
    }

    #[qjs(get)]
    pub fn algorithm<'js>(&self, ctx: Ctx<'js>) -> rquickjs::Result<Object<'js>> {
        self.algorithm.clone().into_object(&ctx)
    }
}

klaver_core::create_export!(CryptoKey);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyFormat {
    Raw,
    Pkcs8,
    Spki,
    Jwk,
}

impl<'js> FromJs<'js> for KeyFormat {
    fn from_js(ctx: &Ctx<'js>, value: Value<'js>) -> rquickjs::Result<Self> {
        let s = std::string::String::from_js(ctx, value)?;
        Ok(match s.as_str() {
            "raw" => Self::Raw,
            "pkcs8" => Self::Pkcs8,
            "spki" => Self::Spki,
            "jwk" => Self::Jwk,
            _ => throw!(@type ctx, format!("unrecognized key format \"{s}\"")),
        })
    }
}

fn not_supported_format<'js, T>(ctx: &Ctx<'js>, format: KeyFormat) -> rquickjs::Result<T> {
    throw_dom!(
        ctx,
        "NotSupportedError",
        format!("key format {format:?} is not supported for this key type")
    )
}

#[cfg(feature = "crypto-asymmetric")]
pub(crate) fn rsa_error<'js>(ctx: &Ctx<'js>, err: rsa_backend::RsaError) -> rquickjs::Error {
    match err {
        rsa_backend::RsaError::InvalidKey => {
            DOMException::throw_named(ctx, "DataError", "invalid RSA key material")
        }
        rsa_backend::RsaError::OperationFailed => {
            DOMException::throw_named(ctx, "OperationError", "RSA operation failed")
        }
    }
}

#[cfg(feature = "crypto-asymmetric")]
pub(crate) fn ec_error<'js>(ctx: &Ctx<'js>, err: ec::EcError) -> rquickjs::Error {
    match err {
        ec::EcError::InvalidKey => {
            DOMException::throw_named(ctx, "DataError", "invalid EC key material")
        }
        ec::EcError::OperationFailed => {
            DOMException::throw_named(ctx, "OperationError", "EC operation failed")
        }
    }
}

/// Splits `generateKey`'s requested `keyUsages` between the private/public halves of a
/// [`crate::crypto::key::CryptoKeyPair`]-shaped result, throwing `SyntaxError` for any usage not
/// valid for `priv_allowed ∪ pub_allowed` - per spec, every requested usage must be valid for *some*
/// side of the pair.
#[cfg(feature = "crypto-asymmetric")]
fn split_usages<'js>(
    ctx: &Ctx<'js>,
    usages: &[KeyUsage],
    priv_allowed: &[KeyUsage],
    pub_allowed: &[KeyUsage],
) -> rquickjs::Result<(Vec<KeyUsage>, Vec<KeyUsage>)> {
    for usage in usages {
        if !priv_allowed.contains(usage) && !pub_allowed.contains(usage) {
            throw_dom!(
                ctx,
                "SyntaxError",
                format!("usage {usage:?} is not valid for this algorithm")
            );
        }
    }
    Ok((
        usages
            .iter()
            .copied()
            .filter(|u| priv_allowed.contains(u))
            .collect(),
        usages
            .iter()
            .copied()
            .filter(|u| pub_allowed.contains(u))
            .collect(),
    ))
}

#[cfg(feature = "crypto-asymmetric")]
fn key_pair_object<'js>(
    ctx: Ctx<'js>,
    public_key: Class<'js, CryptoKey>,
    private_key: Class<'js, CryptoKey>,
) -> rquickjs::Result<Value<'js>> {
    let obj = Object::new(ctx)?;
    obj.set("publicKey", public_key)?;
    obj.set("privateKey", private_key)?;
    Ok(obj.into_value())
}

pub async fn generate_key<'js>(
    ctx: Ctx<'js>,
    algorithm: KeyGenAlgorithm,
    extractable: bool,
    usages: Vec<KeyUsage>,
) -> rquickjs::Result<Value<'js>> {
    match algorithm {
        KeyGenAlgorithm::Aes { variant, length } => {
            if !matches!(length, 128 | 192 | 256) {
                throw_dom!(
                    ctx,
                    "OperationError",
                    format!("invalid AES key length {length}")
                );
            }
            let mut bytes = vec![0u8; (length / 8) as usize];
            rand::rng().fill_bytes(&mut bytes);
            let key = CryptoKey::create(
                KeyType::Secret,
                extractable,
                usages,
                KeyAlgorithm::Aes {
                    name: variant,
                    length,
                },
                KeyMaterial::Aes(bytes),
            );
            Ok(Class::instance(ctx, key)?.into_value())
        }
        KeyGenAlgorithm::Hmac { hash, length } => {
            let length_bits = length.unwrap_or_else(|| super::hmac::default_key_length_bits(hash));
            let mut bytes = vec![0u8; length_bits.div_ceil(8) as usize];
            rand::rng().fill_bytes(&mut bytes);
            let key = CryptoKey::create(
                KeyType::Secret,
                extractable,
                usages,
                KeyAlgorithm::Hmac {
                    hash,
                    length: length_bits,
                },
                KeyMaterial::Hmac(bytes),
            );
            Ok(Class::instance(ctx, key)?.into_value())
        }
        #[cfg(feature = "crypto-asymmetric")]
        KeyGenAlgorithm::RsaHashed {
            variant,
            modulus_length,
            public_exponent,
            hash,
        } => {
            let (private, public) =
                rsa_backend::generate_keypair(modulus_length, &public_exponent)
                    .map_err(|e| rsa_error(&ctx, e))?;
            let algorithm = KeyAlgorithm::RsaHashed {
                variant,
                modulus_length,
                public_exponent,
                hash,
            };
            let (priv_allowed, pub_allowed): (&[KeyUsage], &[KeyUsage]) = match variant {
                RsaVariant::Pkcs1v15 | RsaVariant::Pss => (&[KeyUsage::Sign], &[KeyUsage::Verify]),
                RsaVariant::Oaep => (
                    &[KeyUsage::Decrypt, KeyUsage::UnwrapKey],
                    &[KeyUsage::Encrypt, KeyUsage::WrapKey],
                ),
            };
            let (priv_usages, pub_usages) =
                split_usages(&ctx, &usages, priv_allowed, pub_allowed)?;
            let private_key = Class::instance(
                ctx.clone(),
                CryptoKey::create(
                    KeyType::Private,
                    extractable,
                    priv_usages,
                    algorithm.clone(),
                    KeyMaterial::Rsa(rsa_backend::RsaKeyPair::Private(private)),
                ),
            )?;
            let public_key = Class::instance(
                ctx.clone(),
                CryptoKey::create(
                    KeyType::Public,
                    true,
                    pub_usages,
                    algorithm,
                    KeyMaterial::Rsa(rsa_backend::RsaKeyPair::Public(public)),
                ),
            )?;
            key_pair_object(ctx, public_key, private_key)
        }
        #[cfg(feature = "crypto-asymmetric")]
        KeyGenAlgorithm::Ec {
            variant,
            named_curve,
        } => {
            let (private, public) = ec::generate_keypair(named_curve);
            let algorithm = KeyAlgorithm::Ec {
                variant,
                named_curve,
            };
            let (priv_allowed, pub_allowed): (&[KeyUsage], &[KeyUsage]) = match variant {
                EcVariant::Ecdsa => (&[KeyUsage::Sign], &[KeyUsage::Verify]),
                // Per spec, ECDH's public key always ends up with an empty `usages` list - only
                // the private/base key is ever passed to `deriveBits`/`deriveKey`.
                EcVariant::Ecdh => (
                    &[KeyUsage::DeriveKey, KeyUsage::DeriveBits],
                    &[],
                ),
            };
            let (priv_usages, pub_usages) =
                split_usages(&ctx, &usages, priv_allowed, pub_allowed)?;
            let private_key = Class::instance(
                ctx.clone(),
                CryptoKey::create(
                    KeyType::Private,
                    extractable,
                    priv_usages,
                    algorithm.clone(),
                    KeyMaterial::Ec(private),
                ),
            )?;
            let public_key = Class::instance(
                ctx.clone(),
                CryptoKey::create(
                    KeyType::Public,
                    true,
                    pub_usages,
                    algorithm,
                    KeyMaterial::Ec(public),
                ),
            )?;
            key_pair_object(ctx, public_key, private_key)
        }
    }
}

fn symmetric_bytes_from_format<'js>(
    ctx: &Ctx<'js>,
    format: KeyFormat,
    key_data: Value<'js>,
) -> rquickjs::Result<Vec<u8>> {
    match format {
        KeyFormat::Raw => {
            let buffer = Buffer::from_js(ctx, key_data)?;
            buffer_bytes(ctx, buffer)
        }
        KeyFormat::Jwk => {
            let obj = Object::from_js(ctx, key_data)?;
            jwk::oct_from_jwk(ctx, &obj)
        }
        KeyFormat::Pkcs8 | KeyFormat::Spki => not_supported_format(ctx, format),
    }
}

#[cfg(feature = "crypto-asymmetric")]
fn import_rsa_key<'js>(
    ctx: &Ctx<'js>,
    format: KeyFormat,
    key_data: Value<'js>,
    variant: RsaVariant,
    hash: Algo,
    extractable: bool,
    usages: Vec<KeyUsage>,
) -> rquickjs::Result<Class<'js, CryptoKey>> {
    let (ty, material) = match format {
        KeyFormat::Spki => {
            let buffer = Buffer::from_js(ctx, key_data)?;
            let bytes = buffer_bytes(ctx, buffer)?;
            let pub_key = rsa_backend::from_spki_der(&bytes).map_err(|e| rsa_error(ctx, e))?;
            (KeyType::Public, rsa_backend::RsaKeyPair::Public(pub_key))
        }
        KeyFormat::Pkcs8 => {
            let buffer = Buffer::from_js(ctx, key_data)?;
            let bytes = buffer_bytes(ctx, buffer)?;
            let priv_key = rsa_backend::from_pkcs8_der(&bytes).map_err(|e| rsa_error(ctx, e))?;
            (KeyType::Private, rsa_backend::RsaKeyPair::Private(priv_key))
        }
        KeyFormat::Jwk => {
            let obj = Object::from_js(ctx, key_data)?;
            let fields = jwk::rsa_from_jwk(ctx, &obj)?;
            match fields.private {
                Some(priv_fields) => {
                    let priv_key = rsa_backend::private_key_from_components(
                        &fields.n,
                        &fields.e,
                        &priv_fields.d,
                        &priv_fields.p,
                        &priv_fields.q,
                    )
                    .map_err(|e| rsa_error(ctx, e))?;
                    (KeyType::Private, rsa_backend::RsaKeyPair::Private(priv_key))
                }
                None => {
                    let pub_key = rsa_backend::public_key_from_components(&fields.n, &fields.e)
                        .map_err(|e| rsa_error(ctx, e))?;
                    (KeyType::Public, rsa_backend::RsaKeyPair::Public(pub_key))
                }
            }
        }
        KeyFormat::Raw => return not_supported_format(ctx, format),
    };

    let (modulus_length, public_exponent) = rsa_backend::key_algorithm_params(&material);
    Class::instance(
        ctx.clone(),
        CryptoKey::create(
            ty,
            extractable,
            usages,
            KeyAlgorithm::RsaHashed {
                variant,
                modulus_length,
                public_exponent,
                hash,
            },
            KeyMaterial::Rsa(material),
        ),
    )
}

#[cfg(feature = "crypto-asymmetric")]
fn import_ec_key<'js>(
    ctx: &Ctx<'js>,
    format: KeyFormat,
    key_data: Value<'js>,
    variant: EcVariant,
    named_curve: EcCurve,
    extractable: bool,
    usages: Vec<KeyUsage>,
) -> rquickjs::Result<Class<'js, CryptoKey>> {
    let (ty, material) = match format {
        KeyFormat::Raw => {
            let buffer = Buffer::from_js(ctx, key_data)?;
            let bytes = buffer_bytes(ctx, buffer)?;
            let pair =
                ec::public_key_from_raw(named_curve, &bytes).map_err(|e| ec_error(ctx, e))?;
            (KeyType::Public, pair)
        }
        KeyFormat::Spki => {
            let buffer = Buffer::from_js(ctx, key_data)?;
            let bytes = buffer_bytes(ctx, buffer)?;
            let pair = ec::from_spki_der(named_curve, &bytes).map_err(|e| ec_error(ctx, e))?;
            (KeyType::Public, pair)
        }
        KeyFormat::Pkcs8 => {
            let buffer = Buffer::from_js(ctx, key_data)?;
            let bytes = buffer_bytes(ctx, buffer)?;
            let pair = ec::from_pkcs8_der(named_curve, &bytes).map_err(|e| ec_error(ctx, e))?;
            (KeyType::Private, pair)
        }
        KeyFormat::Jwk => {
            let obj = Object::from_js(ctx, key_data)?;
            let fields = jwk::ec_from_jwk(ctx, &obj)?;
            let jwk_curve = match EcCurve::from_name(&fields.crv) {
                Some(c) => c,
                None => throw_dom!(
                    ctx,
                    "DataError",
                    format!("unrecognized crv \"{}\"", fields.crv)
                ),
            };
            if jwk_curve != named_curve {
                throw_dom!(
                    ctx,
                    "DataError",
                    "JWK \"crv\" does not match the requested namedCurve"
                );
            }
            match fields.d {
                Some(d) => {
                    let pair = ec::private_key_from_components(named_curve, &d)
                        .map_err(|e| ec_error(ctx, e))?;
                    (KeyType::Private, pair)
                }
                None => {
                    let pair =
                        ec::public_key_from_components(named_curve, &fields.x, &fields.y)
                            .map_err(|e| ec_error(ctx, e))?;
                    (KeyType::Public, pair)
                }
            }
        }
    };

    Class::instance(
        ctx.clone(),
        CryptoKey::create(
            ty,
            extractable,
            usages,
            KeyAlgorithm::Ec {
                variant,
                named_curve,
            },
            KeyMaterial::Ec(material),
        ),
    )
}

/// Imports an HKDF/PBKDF2 base key - always from raw bytes (a password, for PBKDF2), and per
/// spec always non-extractable (`SyntaxError` otherwise, since there'd be no way to ever get the
/// bytes back out - `exportKey`/`wrapKey` on one of these always fail the `extractable` check
/// before even reaching `export_key`'s per-material match).
#[cfg(feature = "crypto-asymmetric")]
fn import_derive_only_key<'js>(
    ctx: &Ctx<'js>,
    format: KeyFormat,
    key_data: Value<'js>,
    kind: DeriveKind,
    extractable: bool,
    usages: Vec<KeyUsage>,
) -> rquickjs::Result<Class<'js, CryptoKey>> {
    if format != KeyFormat::Raw {
        return not_supported_format(ctx, format);
    }
    if extractable {
        throw_dom!(
            ctx,
            "SyntaxError",
            format!("{} keys must not be extractable", kind.name())
        );
    }
    let buffer = Buffer::from_js(ctx, key_data)?;
    let bytes = buffer_bytes(ctx, buffer)?;
    Class::instance(
        ctx.clone(),
        CryptoKey::create(
            KeyType::Secret,
            extractable,
            usages,
            KeyAlgorithm::DeriveOnly(kind),
            KeyMaterial::Derive(bytes),
        ),
    )
}

pub async fn import_key<'js>(
    ctx: Ctx<'js>,
    format: KeyFormat,
    key_data: Value<'js>,
    algorithm: ImportAlgorithm,
    extractable: bool,
    usages: Vec<KeyUsage>,
) -> rquickjs::Result<Class<'js, CryptoKey>> {
    match algorithm {
        ImportAlgorithm::Aes(variant) => {
            let raw_bytes = symmetric_bytes_from_format(&ctx, format, key_data)?;
            let length = (raw_bytes.len() * 8) as u16;
            if !matches!(length, 128 | 192 | 256) {
                throw_dom!(ctx, "DataError", format!("invalid AES key length {length}"));
            }
            Class::instance(
                ctx,
                CryptoKey::create(
                    KeyType::Secret,
                    extractable,
                    usages,
                    KeyAlgorithm::Aes {
                        name: variant,
                        length,
                    },
                    KeyMaterial::Aes(raw_bytes),
                ),
            )
        }
        ImportAlgorithm::Hmac { hash } => {
            let raw_bytes = symmetric_bytes_from_format(&ctx, format, key_data)?;
            let length = (raw_bytes.len() * 8) as u32;
            Class::instance(
                ctx,
                CryptoKey::create(
                    KeyType::Secret,
                    extractable,
                    usages,
                    KeyAlgorithm::Hmac { hash, length },
                    KeyMaterial::Hmac(raw_bytes),
                ),
            )
        }
        #[cfg(feature = "crypto-asymmetric")]
        ImportAlgorithm::RsaHashed { variant, hash } => {
            import_rsa_key(&ctx, format, key_data, variant, hash, extractable, usages)
        }
        #[cfg(feature = "crypto-asymmetric")]
        ImportAlgorithm::Ec {
            variant,
            named_curve,
        } => import_ec_key(
            &ctx,
            format,
            key_data,
            variant,
            named_curve,
            extractable,
            usages,
        ),
        #[cfg(feature = "crypto-asymmetric")]
        ImportAlgorithm::Hkdf => {
            import_derive_only_key(&ctx, format, key_data, DeriveKind::Hkdf, extractable, usages)
        }
        #[cfg(feature = "crypto-asymmetric")]
        ImportAlgorithm::Pbkdf2 => import_derive_only_key(
            &ctx,
            format,
            key_data,
            DeriveKind::Pbkdf2,
            extractable,
            usages,
        ),
    }
}

#[cfg(feature = "crypto-asymmetric")]
fn export_rsa_key<'js>(
    ctx: &Ctx<'js>,
    format: KeyFormat,
    pair: &rsa_backend::RsaKeyPair,
    key_ref: &CryptoKey,
) -> rquickjs::Result<Value<'js>> {
    match format {
        KeyFormat::Spki => {
            let rsa_backend::RsaKeyPair::Public(pub_key) = pair else {
                throw_dom!(ctx, "InvalidAccessError", "\"spki\" export requires a public key");
            };
            let der = rsa_backend::to_spki_der(pub_key).map_err(|e| rsa_error(ctx, e))?;
            Ok(rquickjs::ArrayBuffer::new(ctx.clone(), der)?.into_value())
        }
        KeyFormat::Pkcs8 => {
            let rsa_backend::RsaKeyPair::Private(priv_key) = pair else {
                throw_dom!(ctx, "InvalidAccessError", "\"pkcs8\" export requires a private key");
            };
            let der = rsa_backend::to_pkcs8_der(priv_key).map_err(|e| rsa_error(ctx, e))?;
            Ok(rquickjs::ArrayBuffer::new(ctx.clone(), der)?.into_value())
        }
        KeyFormat::Jwk => {
            let components = match pair {
                rsa_backend::RsaKeyPair::Public(k) => rsa_backend::public_components(k),
                rsa_backend::RsaKeyPair::Private(k) => {
                    rsa_backend::private_components(k).map_err(|e| rsa_error(ctx, e))?
                }
            };
            let alg_tag = key_ref.algorithm.jwk_alg_tag();
            let obj = jwk::rsa_to_jwk(ctx, &components, alg_tag.as_deref(), key_ref.extractable)?;
            Ok(obj.into_value())
        }
        KeyFormat::Raw => not_supported_format(ctx, format),
    }
}

#[cfg(feature = "crypto-asymmetric")]
fn export_ec_key<'js>(
    ctx: &Ctx<'js>,
    format: KeyFormat,
    pair: &ec::EcKeyPair,
    key_ref: &CryptoKey,
) -> rquickjs::Result<Value<'js>> {
    match format {
        KeyFormat::Raw => {
            let bytes = ec::public_key_to_raw(pair).map_err(|_| {
                DOMException::throw_named(ctx, "InvalidAccessError", "\"raw\" export requires a public key")
            })?;
            Ok(rquickjs::ArrayBuffer::new(ctx.clone(), bytes)?.into_value())
        }
        KeyFormat::Spki => {
            let der = ec::to_spki_der(pair).map_err(|_| {
                DOMException::throw_named(ctx, "InvalidAccessError", "\"spki\" export requires a public key")
            })?;
            Ok(rquickjs::ArrayBuffer::new(ctx.clone(), der)?.into_value())
        }
        KeyFormat::Pkcs8 => {
            let der = ec::to_pkcs8_der(pair).map_err(|_| {
                DOMException::throw_named(ctx, "InvalidAccessError", "\"pkcs8\" export requires a private key")
            })?;
            Ok(rquickjs::ArrayBuffer::new(ctx.clone(), der)?.into_value())
        }
        KeyFormat::Jwk => {
            let components = ec::components(pair);
            let obj = jwk::ec_to_jwk(ctx, &components, key_ref.extractable)?;
            Ok(obj.into_value())
        }
    }
}

pub async fn export_key<'js>(
    ctx: Ctx<'js>,
    format: KeyFormat,
    key: Class<'js, CryptoKey>,
) -> rquickjs::Result<Value<'js>> {
    let key_ref = key.borrow();
    if !key_ref.extractable {
        throw_dom!(ctx, "InvalidAccessError", "key is not extractable");
    }

    match &key_ref.material {
        KeyMaterial::Aes(_) | KeyMaterial::Hmac(_) => match format {
            KeyFormat::Raw => {
                let buf = rquickjs::ArrayBuffer::new(ctx, key_ref.material.bytes().to_vec())?;
                Ok(buf.into_value())
            }
            KeyFormat::Jwk => {
                let alg_tag = key_ref.algorithm.jwk_alg_tag();
                let obj = jwk::oct_to_jwk(
                    &ctx,
                    key_ref.material.bytes(),
                    alg_tag.as_deref(),
                    key_ref.extractable,
                )?;
                Ok(obj.into_value())
            }
            KeyFormat::Pkcs8 | KeyFormat::Spki => not_supported_format(&ctx, format),
        },
        #[cfg(feature = "crypto-asymmetric")]
        KeyMaterial::Rsa(pair) => export_rsa_key(&ctx, format, pair, &key_ref),
        #[cfg(feature = "crypto-asymmetric")]
        KeyMaterial::Ec(pair) => export_ec_key(&ctx, format, pair, &key_ref),
        // Unreachable - HKDF/PBKDF2 keys are always imported with `extractable: false` (see
        // `import_derive_only_key`), so the `extractable` check above already threw.
        #[cfg(feature = "crypto-asymmetric")]
        KeyMaterial::Derive(_) => unreachable!("derive-only keys are never extractable"),
    }
}
