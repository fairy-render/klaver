use rand::prelude::*;
use rquickjs::{
    Class, Ctx, FromJs, IntoJs, Object, Value,
    class::{Trace, Tracer},
};

use super::algorithm::{ImportAlgorithm, KeyGenAlgorithm, buffer_bytes};
use super::digest::Algo;
use super::jwk;
use klaver_core::{throw, value::Buffer};

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
#[derive(Debug, Clone, Copy)]
pub enum KeyAlgorithm {
    Aes { name: AesVariant, length: u16 },
    Hmac { hash: Algo, length: u32 },
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
        }
        Ok(obj)
    }

    /// The `alg` tag WebCrypto's JWK export/import uses for this algorithm, per
    /// <https://w3c.github.io/webcrypto/#jwk-mapping-algorithm-2> (e.g. `"A128GCM"`, `"HS256"`).
    fn jwk_alg_tag(self) -> std::string::String {
        match self {
            KeyAlgorithm::Aes { name, length } => {
                let prefix = match length {
                    128 => "A128",
                    192 => "A192",
                    256 => "A256",
                    _ => "A???",
                };
                format!("{prefix}{}", name.jwk_suffix())
            }
            KeyAlgorithm::Hmac { hash, .. } => match hash {
                Algo::Sha1 => "HS1".to_string(),
                Algo::Sha256 => "HS256".to_string(),
                Algo::Sha384 => "HS384".to_string(),
                Algo::Sha512 => "HS512".to_string(),
            },
        }
    }
}

/// Raw key bytes, tagged by which kind of key they are. Milestone 2 (RSA/EC) will add `Rsa(..)`/
/// `Ec(..)` variants alongside these - `CryptoKey`'s own shape doesn't need to change for that.
#[derive(Clone)]
enum KeyMaterial {
    Aes(Vec<u8>),
    Hmac(Vec<u8>),
}

impl KeyMaterial {
    fn bytes(&self) -> &[u8] {
        match self {
            Self::Aes(b) | Self::Hmac(b) => b,
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
        self.algorithm
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
        self.algorithm.into_object(&ctx)
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
        format!("key format {format:?} is not yet supported for symmetric keys")
    )
}

pub async fn generate_key<'js>(
    ctx: Ctx<'js>,
    algorithm: KeyGenAlgorithm,
    extractable: bool,
    usages: Vec<KeyUsage>,
) -> rquickjs::Result<Class<'js, CryptoKey>> {
    let (algorithm, material) = match algorithm {
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
            (
                KeyAlgorithm::Aes {
                    name: variant,
                    length,
                },
                KeyMaterial::Aes(bytes),
            )
        }
        KeyGenAlgorithm::Hmac { hash, length } => {
            let length_bits = length.unwrap_or_else(|| super::hmac::default_key_length_bits(hash));
            let mut bytes = vec![0u8; length_bits.div_ceil(8) as usize];
            rand::rng().fill_bytes(&mut bytes);
            (
                KeyAlgorithm::Hmac {
                    hash,
                    length: length_bits,
                },
                KeyMaterial::Hmac(bytes),
            )
        }
    };

    Class::instance(
        ctx,
        CryptoKey::create(KeyType::Secret, extractable, usages, algorithm, material),
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
    let raw_bytes = match format {
        KeyFormat::Raw => {
            let buffer = Buffer::from_js(&ctx, key_data)?;
            buffer_bytes(&ctx, buffer)?
        }
        KeyFormat::Jwk => {
            let obj = Object::from_js(&ctx, key_data)?;
            jwk::oct_from_jwk(&ctx, &obj)?
        }
        KeyFormat::Pkcs8 | KeyFormat::Spki => return not_supported_format(&ctx, format),
    };

    let (algorithm, material) = match algorithm {
        ImportAlgorithm::Aes(variant) => {
            let length = (raw_bytes.len() * 8) as u16;
            if !matches!(length, 128 | 192 | 256) {
                throw_dom!(ctx, "DataError", format!("invalid AES key length {length}"));
            }
            (
                KeyAlgorithm::Aes {
                    name: variant,
                    length,
                },
                KeyMaterial::Aes(raw_bytes),
            )
        }
        ImportAlgorithm::Hmac { hash } => {
            let length = (raw_bytes.len() * 8) as u32;
            (
                KeyAlgorithm::Hmac { hash, length },
                KeyMaterial::Hmac(raw_bytes),
            )
        }
    };

    Class::instance(
        ctx,
        CryptoKey::create(KeyType::Secret, extractable, usages, algorithm, material),
    )
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

    match format {
        KeyFormat::Raw => {
            let buf = rquickjs::ArrayBuffer::new(ctx, key_ref.material.bytes().to_vec())?;
            Ok(buf.into_value())
        }
        KeyFormat::Jwk => {
            let alg_tag = key_ref.algorithm.jwk_alg_tag();
            let obj = jwk::oct_to_jwk(
                &ctx,
                key_ref.material.bytes(),
                &alg_tag,
                key_ref.extractable,
            )?;
            Ok(obj.into_value())
        }
        KeyFormat::Pkcs8 | KeyFormat::Spki => not_supported_format(&ctx, format),
    }
}
