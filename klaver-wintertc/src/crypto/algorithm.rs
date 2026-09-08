//! `AlgorithmIdentifier`-style (`DOMString or object`) argument parsing for `subtle.*` methods.
//! Combines `fetch::fetch::FetchInit`'s "try several JS shapes in order" structure with
//! `blob::BlobOptions`'s "read named fields off a dict object" structure.

use klaver_core::{throw, value::Buffer};
use rquickjs::{Ctx, FromJs, Object, String as JsString, Value};

use super::digest::Algo;
use super::key::AesVariant;
#[cfg(feature = "crypto-asymmetric")]
use super::key::{EcCurve, EcVariant, RsaVariant};

/// Extracts `{name, params}` from either a bare string (`params: None`) or an object with a
/// `name` field (`params: Some(the object itself)`, so callers can read further fields off it).
struct RawAlgorithm<'js> {
    name: std::string::String,
    params: Option<Object<'js>>,
}

impl<'js> FromJs<'js> for RawAlgorithm<'js> {
    fn from_js(ctx: &Ctx<'js>, value: Value<'js>) -> rquickjs::Result<Self> {
        if let Ok(s) = JsString::from_value(value.clone()) {
            return Ok(RawAlgorithm {
                name: s.to_string()?,
                params: None,
            });
        }

        let obj = Object::from_js(ctx, value)?;
        let name: std::string::String = obj.get("name")?;
        Ok(RawAlgorithm {
            name,
            params: Some(obj),
        })
    }
}

fn require_params<'js>(
    ctx: &Ctx<'js>,
    raw: &RawAlgorithm<'js>,
    what: &str,
) -> rquickjs::Result<Object<'js>> {
    match &raw.params {
        Some(params) => Ok(params.clone()),
        None => {
            throw!(@type ctx, format!("{} requires an algorithm object, not a bare string", what))
        }
    }
}

fn unrecognized_algorithm<'js, T>(ctx: &Ctx<'js>, name: &str) -> rquickjs::Result<T> {
    throw_dom!(
        ctx,
        "NotSupportedError",
        format!("unrecognized algorithm \"{name}\"")
    )
}

#[cfg(feature = "crypto-asymmetric")]
fn parse_named_curve<'js>(ctx: &Ctx<'js>, params: &Object<'js>) -> rquickjs::Result<EcCurve> {
    let named_curve: std::string::String = params.get("namedCurve")?;
    match EcCurve::from_name(&named_curve) {
        Some(curve) => Ok(curve),
        None => throw_dom!(
            ctx,
            "NotSupportedError",
            format!("unrecognized named curve \"{named_curve}\"")
        ),
    }
}

/// A hash `AlgorithmIdentifier` (e.g. HMAC's `hash` field): either a bare digest name or
/// `{name: "SHA-256"}`.
pub struct HashAlgorithm(pub Algo);

impl<'js> FromJs<'js> for HashAlgorithm {
    fn from_js(ctx: &Ctx<'js>, value: Value<'js>) -> rquickjs::Result<Self> {
        let raw = RawAlgorithm::from_js(ctx, value)?;
        match Algo::from_name(&raw.name) {
            Some(algo) => Ok(HashAlgorithm(algo)),
            None => unrecognized_algorithm(ctx, &raw.name),
        }
    }
}

pub(crate) fn buffer_bytes<'js>(ctx: &Ctx<'js>, buffer: Buffer<'js>) -> rquickjs::Result<Vec<u8>> {
    let Some(raw) = buffer.as_raw() else {
        throw!(@type ctx, "buffer is detached")
    };
    Ok(raw.slice().to_vec())
}

/// `encrypt()`/`decrypt()`'s algorithm argument - `AesGcmParams`/`AesCbcParams`/`AesCtrParams`.
/// All three always require an algorithm object (never a bare string): every one of them has at
/// least one mandatory field (`iv`/`counter`) `encrypt`/`decrypt` can't default.
pub enum CipherAlgorithm {
    AesGcm {
        iv: Vec<u8>,
        additional_data: Option<Vec<u8>>,
        tag_length_bits: u16,
    },
    AesCbc {
        iv: Vec<u8>,
    },
    AesCtr {
        counter: Vec<u8>,
        length_bits: u8,
    },
    /// The OAEP hash itself comes from the key (like `SignAlgorithm::RsaSsaPkcs1`), not from
    /// `RsaOaepParams` - only the optional `label` is a per-call parameter.
    #[cfg(feature = "crypto-asymmetric")]
    RsaOaep { label: Option<Vec<u8>> },
}

impl<'js> FromJs<'js> for CipherAlgorithm {
    fn from_js(ctx: &Ctx<'js>, value: Value<'js>) -> rquickjs::Result<Self> {
        let raw = RawAlgorithm::from_js(ctx, value)?;

        #[cfg(feature = "crypto-asymmetric")]
        if raw.name.eq_ignore_ascii_case("RSA-OAEP") {
            // Unlike AES, `RsaOaepParams.label` is optional and there's no other mandatory
            // field, so a bare `"RSA-OAEP"` string (no params object at all) is valid.
            let label = match &raw.params {
                Some(params) => {
                    let label: Option<Buffer> = params.get("label")?;
                    label.map(|b| buffer_bytes(ctx, b)).transpose()?
                }
                None => None,
            };
            return Ok(CipherAlgorithm::RsaOaep { label });
        }

        let Some(variant) = AesVariant::from_name(&raw.name) else {
            return unrecognized_algorithm(ctx, &raw.name);
        };
        let params = require_params(ctx, &raw, "AES encrypt/decrypt")?;

        Ok(match variant {
            AesVariant::Gcm => {
                let iv: Buffer = params.get("iv")?;
                let additional_data: Option<Buffer> = params.get("additionalData")?;
                let tag_length_bits: Option<u16> = params.get("tagLength")?;
                CipherAlgorithm::AesGcm {
                    iv: buffer_bytes(ctx, iv)?,
                    additional_data: additional_data.map(|b| buffer_bytes(ctx, b)).transpose()?,
                    tag_length_bits: tag_length_bits.unwrap_or(128),
                }
            }
            AesVariant::Cbc => {
                let iv: Buffer = params.get("iv")?;
                CipherAlgorithm::AesCbc {
                    iv: buffer_bytes(ctx, iv)?,
                }
            }
            AesVariant::Ctr => {
                let counter: Buffer = params.get("counter")?;
                let length_bits: u8 = params.get("length")?;
                CipherAlgorithm::AesCtr {
                    counter: buffer_bytes(ctx, counter)?,
                    length_bits,
                }
            }
        })
    }
}

/// `generateKey()`'s algorithm argument - `AesKeyGenParams` (always needs `length`) or
/// `HmacKeyGenParams` (always needs `hash`; `length` is optional, defaulting to the hash's
/// block size).
pub enum KeyGenAlgorithm {
    Aes {
        variant: AesVariant,
        length: u16,
    },
    Hmac {
        hash: Algo,
        length: Option<u32>,
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
}

impl<'js> FromJs<'js> for KeyGenAlgorithm {
    fn from_js(ctx: &Ctx<'js>, value: Value<'js>) -> rquickjs::Result<Self> {
        let raw = RawAlgorithm::from_js(ctx, value)?;

        if raw.name.eq_ignore_ascii_case("HMAC") {
            let params = require_params(ctx, &raw, "HmacKeyGenParams")?;
            let hash: HashAlgorithm = params.get("hash")?;
            let length: Option<u32> = params.get("length")?;
            return Ok(KeyGenAlgorithm::Hmac {
                hash: hash.0,
                length,
            });
        }

        #[cfg(feature = "crypto-asymmetric")]
        if let Some(variant) = RsaVariant::from_name(&raw.name) {
            let params = require_params(ctx, &raw, "RsaHashedKeyGenParams")?;
            let modulus_length: u32 = params.get("modulusLength")?;
            let public_exponent: Buffer = params.get("publicExponent")?;
            let hash: HashAlgorithm = params.get("hash")?;
            return Ok(KeyGenAlgorithm::RsaHashed {
                variant,
                modulus_length,
                public_exponent: buffer_bytes(ctx, public_exponent)?,
                hash: hash.0,
            });
        }

        #[cfg(feature = "crypto-asymmetric")]
        if let Some(variant) = EcVariant::from_name(&raw.name) {
            let params = require_params(ctx, &raw, "EcKeyGenParams")?;
            let named_curve = parse_named_curve(ctx, &params)?;
            return Ok(KeyGenAlgorithm::Ec {
                variant,
                named_curve,
            });
        }

        let Some(variant) = AesVariant::from_name(&raw.name) else {
            return unrecognized_algorithm(ctx, &raw.name);
        };
        let params = require_params(ctx, &raw, "AesKeyGenParams")?;
        let length: u16 = params.get("length")?;
        Ok(KeyGenAlgorithm::Aes { variant, length })
    }
}

/// `importKey()`'s algorithm argument - a bare AES algorithm name is enough (key length comes
/// from the key data itself), but HMAC always needs `{hash}`.
pub enum ImportAlgorithm {
    Aes(AesVariant),
    Hmac {
        hash: Algo,
    },
    #[cfg(feature = "crypto-asymmetric")]
    RsaHashed {
        variant: RsaVariant,
        hash: Algo,
    },
    #[cfg(feature = "crypto-asymmetric")]
    Ec {
        variant: EcVariant,
        named_curve: EcCurve,
    },
    /// `{name: "HKDF"}` - no hash/length here, since `HkdfParams.hash` is a per-`deriveBits()`/
    /// `deriveKey()`-call parameter, not fixed to the key (unlike HMAC).
    #[cfg(feature = "crypto-asymmetric")]
    Hkdf,
    /// `{name: "PBKDF2"}` - same story as `Hkdf` (`Pbkdf2Params.hash`/`iterations` are per-call).
    #[cfg(feature = "crypto-asymmetric")]
    Pbkdf2,
}

impl<'js> FromJs<'js> for ImportAlgorithm {
    fn from_js(ctx: &Ctx<'js>, value: Value<'js>) -> rquickjs::Result<Self> {
        let raw = RawAlgorithm::from_js(ctx, value)?;

        if raw.name.eq_ignore_ascii_case("HMAC") {
            let params = require_params(ctx, &raw, "HmacImportParams")?;
            let hash: HashAlgorithm = params.get("hash")?;
            return Ok(ImportAlgorithm::Hmac { hash: hash.0 });
        }

        #[cfg(feature = "crypto-asymmetric")]
        if raw.name.eq_ignore_ascii_case("HKDF") {
            return Ok(ImportAlgorithm::Hkdf);
        }

        #[cfg(feature = "crypto-asymmetric")]
        if raw.name.eq_ignore_ascii_case("PBKDF2") {
            return Ok(ImportAlgorithm::Pbkdf2);
        }

        #[cfg(feature = "crypto-asymmetric")]
        if let Some(variant) = RsaVariant::from_name(&raw.name) {
            let params = require_params(ctx, &raw, "RsaHashedImportParams")?;
            let hash: HashAlgorithm = params.get("hash")?;
            return Ok(ImportAlgorithm::RsaHashed {
                variant,
                hash: hash.0,
            });
        }

        #[cfg(feature = "crypto-asymmetric")]
        if let Some(variant) = EcVariant::from_name(&raw.name) {
            let params = require_params(ctx, &raw, "EcKeyImportParams")?;
            let named_curve = parse_named_curve(ctx, &params)?;
            return Ok(ImportAlgorithm::Ec {
                variant,
                named_curve,
            });
        }

        match AesVariant::from_name(&raw.name) {
            Some(variant) => Ok(ImportAlgorithm::Aes(variant)),
            None => unrecognized_algorithm(ctx, &raw.name),
        }
    }
}

/// `sign()`/`verify()`'s algorithm argument. HMAC and RSASSA-PKCS1-v1_5 carry no
/// operation-specific parameters (their hash lives on the key), but ECDSA's hash and RSA-PSS's
/// salt length are supplied per-call via `EcdsaParams`/`RsaPssParams` - the one place RSA and EC
/// diverge here.
pub enum SignAlgorithm {
    Hmac,
    #[cfg(feature = "crypto-asymmetric")]
    RsaSsaPkcs1,
    #[cfg(feature = "crypto-asymmetric")]
    RsaPss { salt_length: usize },
    #[cfg(feature = "crypto-asymmetric")]
    Ecdsa { hash: Algo },
}

impl<'js> FromJs<'js> for SignAlgorithm {
    fn from_js(ctx: &Ctx<'js>, value: Value<'js>) -> rquickjs::Result<Self> {
        let raw = RawAlgorithm::from_js(ctx, value)?;
        if raw.name.eq_ignore_ascii_case("HMAC") {
            return Ok(SignAlgorithm::Hmac);
        }
        #[cfg(feature = "crypto-asymmetric")]
        if raw.name.eq_ignore_ascii_case("RSASSA-PKCS1-v1_5") {
            return Ok(SignAlgorithm::RsaSsaPkcs1);
        }
        #[cfg(feature = "crypto-asymmetric")]
        if raw.name.eq_ignore_ascii_case("RSA-PSS") {
            let params = require_params(ctx, &raw, "RsaPssParams")?;
            let salt_length: u32 = params.get("saltLength")?;
            return Ok(SignAlgorithm::RsaPss {
                salt_length: salt_length as usize,
            });
        }
        #[cfg(feature = "crypto-asymmetric")]
        if raw.name.eq_ignore_ascii_case("ECDSA") {
            let params = require_params(ctx, &raw, "EcdsaParams")?;
            let hash: HashAlgorithm = params.get("hash")?;
            return Ok(SignAlgorithm::Ecdsa { hash: hash.0 });
        }
        unrecognized_algorithm(ctx, &raw.name)
    }
}

/// `deriveBits()`/`deriveKey()`'s algorithm argument - `EcdhKeyDeriveParams`, `HkdfParams`, or
/// `Pbkdf2Params`.
#[cfg(feature = "crypto-asymmetric")]
pub enum DeriveBitsAlgorithm<'js> {
    Ecdh {
        public: rquickjs::Class<'js, super::key::CryptoKey>,
    },
    Hkdf {
        hash: Algo,
        salt: Vec<u8>,
        info: Vec<u8>,
    },
    Pbkdf2 {
        hash: Algo,
        salt: Vec<u8>,
        iterations: u32,
    },
}

#[cfg(feature = "crypto-asymmetric")]
impl<'js> FromJs<'js> for DeriveBitsAlgorithm<'js> {
    fn from_js(ctx: &Ctx<'js>, value: Value<'js>) -> rquickjs::Result<Self> {
        let obj = Object::from_js(ctx, value)?;
        let name: std::string::String = obj.get("name")?;
        if name.eq_ignore_ascii_case("ECDH") {
            let public: rquickjs::Class<'js, super::key::CryptoKey> = obj.get("public")?;
            return Ok(DeriveBitsAlgorithm::Ecdh { public });
        }
        if name.eq_ignore_ascii_case("HKDF") {
            let hash: HashAlgorithm = obj.get("hash")?;
            let salt: Buffer = obj.get("salt")?;
            let info: Buffer = obj.get("info")?;
            return Ok(DeriveBitsAlgorithm::Hkdf {
                hash: hash.0,
                salt: buffer_bytes(ctx, salt)?,
                info: buffer_bytes(ctx, info)?,
            });
        }
        if name.eq_ignore_ascii_case("PBKDF2") {
            let hash: HashAlgorithm = obj.get("hash")?;
            let salt: Buffer = obj.get("salt")?;
            let iterations: u32 = obj.get("iterations")?;
            return Ok(DeriveBitsAlgorithm::Pbkdf2 {
                hash: hash.0,
                salt: buffer_bytes(ctx, salt)?,
                iterations,
            });
        }
        unrecognized_algorithm(ctx, &name)
    }
}
