use klaver_core::value::Buffer;
use klaver_core::{Exportable, Registry};
use rquickjs::{
    Ctx, FromJs, Object,
    module::ModuleDef,
    prelude::{Async, Func, Opt},
};

use super::digest::{Algo, Digest};

#[cfg(feature = "crypto-cipher")]
use super::{
    aes,
    algorithm::{self, CipherAlgorithm, SignAlgorithm},
    hmac as hmac_ops,
    key::{AesVariant, CryptoKey, KeyAlgorithm, KeyUsage},
};
#[cfg(feature = "crypto-cipher")]
use crate::dom_exception::DOMException;
#[cfg(feature = "crypto-cipher")]
use rquickjs::{ArrayBuffer, Class};

#[cfg(feature = "crypto-asymmetric")]
use super::{
    algorithm::{DeriveBitsAlgorithm, ImportAlgorithm},
    ec, kdf, rsa as rsa_backend,
    key::{DeriveKind, EcCurve, EcVariant, KeyFormat, KeyType, RsaVariant},
};
#[cfg(feature = "crypto-asymmetric")]
use rquickjs::prelude::Flat;

pub struct CryptoModule;

#[cfg(feature = "crypto-cipher")]
fn require_usage<'js>(ctx: &Ctx<'js>, key: &CryptoKey, usage: KeyUsage) -> rquickjs::Result<()> {
    if key.usage_list().contains(&usage) {
        Ok(())
    } else {
        throw_dom!(
            ctx,
            "InvalidAccessError",
            format!("key does not support the {usage:?} usage")
        )
    }
}

#[cfg(feature = "crypto-cipher")]
fn require_aes_variant<'js>(
    ctx: &Ctx<'js>,
    key: &CryptoKey,
    expected: AesVariant,
) -> rquickjs::Result<()> {
    match key.algorithm_variant() {
        KeyAlgorithm::Aes { name, .. } if name == expected => Ok(()),
        _ => throw_dom!(ctx, "InvalidAccessError", "key's algorithm does not match"),
    }
}

#[cfg(feature = "crypto-cipher")]
fn require_hmac<'js>(ctx: &Ctx<'js>, key: &CryptoKey) -> rquickjs::Result<Algo> {
    match key.algorithm_variant() {
        KeyAlgorithm::Hmac { hash, .. } => Ok(hash),
        _ => throw_dom!(ctx, "InvalidAccessError", "key is not an HMAC key"),
    }
}

/// Returns the key's stored hash - always its `algorithm.hash`, since (unlike ECDSA) RSA's hash
/// is fixed to the key rather than supplied per `sign()`/`verify()`/`encrypt()`/`decrypt()` call.
#[cfg(feature = "crypto-asymmetric")]
fn require_rsa_variant<'js>(
    ctx: &Ctx<'js>,
    key: &CryptoKey,
    expected: RsaVariant,
) -> rquickjs::Result<Algo> {
    match key.algorithm_variant() {
        KeyAlgorithm::RsaHashed { variant, hash, .. } if variant == expected => Ok(hash),
        _ => throw_dom!(ctx, "InvalidAccessError", "key's algorithm does not match"),
    }
}

#[cfg(feature = "crypto-asymmetric")]
fn require_ec_variant<'js>(
    ctx: &Ctx<'js>,
    key: &CryptoKey,
    expected: EcVariant,
) -> rquickjs::Result<EcCurve> {
    match key.algorithm_variant() {
        KeyAlgorithm::Ec {
            variant,
            named_curve,
        } if variant == expected => Ok(named_curve),
        _ => throw_dom!(ctx, "InvalidAccessError", "key's algorithm does not match"),
    }
}

#[cfg(feature = "crypto-asymmetric")]
fn require_key_type<'js>(
    ctx: &Ctx<'js>,
    key: &CryptoKey,
    expected: KeyType,
) -> rquickjs::Result<()> {
    if key.key_type() == expected {
        Ok(())
    } else {
        throw_dom!(
            ctx,
            "InvalidAccessError",
            format!("expected a {expected:?} key")
        )
    }
}

#[cfg(feature = "crypto-cipher")]
fn cipher_error<'js>(ctx: &Ctx<'js>, err: aes::CipherError) -> rquickjs::Error {
    match err {
        aes::CipherError::WrongKeyLength
        | aes::CipherError::WrongIvLength
        | aes::CipherError::WrongCounterLength => {
            DOMException::throw_named(ctx, "OperationError", "invalid key, IV or counter length")
        }
        aes::CipherError::UnsupportedTagLength => {
            DOMException::throw_named(ctx, "NotSupportedError", "unsupported tagLength")
        }
        aes::CipherError::AuthenticationFailed | aes::CipherError::InvalidPadding => {
            DOMException::throw_named(ctx, "OperationError", "decryption failed")
        }
    }
}

/// The actual encrypt operation, with no `KeyUsage` check - shared by `encrypt()` (which requires
/// the `"encrypt"` usage) and `wrap_key()` (which requires `"wrapKey"` instead, per spec - a key
/// usable only for wrapping need not also carry a plain `"encrypt"` usage).
#[cfg(feature = "crypto-cipher")]
fn encrypt_bytes<'js>(
    ctx: &Ctx<'js>,
    algorithm: CipherAlgorithm,
    key_ref: &CryptoKey,
    plaintext: &[u8],
) -> rquickjs::Result<Vec<u8>> {
    #[cfg(feature = "crypto-asymmetric")]
    if let CipherAlgorithm::RsaOaep { label } = algorithm {
        require_key_type(ctx, key_ref, KeyType::Public)?;
        let hash = require_rsa_variant(ctx, key_ref, RsaVariant::Oaep)?;
        let rsa_backend::RsaKeyPair::Public(pub_key) = key_ref.rsa_key_pair().expect("checked above")
        else {
            throw_dom!(ctx, "InvalidAccessError", "key is not an RSA public key");
        };
        return rsa_backend::oaep_encrypt(pub_key, hash, label.as_deref(), plaintext)
            .map_err(|e| super::key::rsa_error(ctx, e));
    }

    require_aes_variant(ctx, key_ref, aes_variant_of(&algorithm))?;
    let key_bytes = key_ref.key_bytes();

    let result = match algorithm {
        CipherAlgorithm::AesGcm {
            iv,
            additional_data,
            tag_length_bits,
        } => aes::gcm_encrypt(
            key_bytes,
            &iv,
            additional_data.as_deref(),
            tag_length_bits,
            plaintext,
        ),
        CipherAlgorithm::AesCbc { iv } => aes::cbc_encrypt(key_bytes, &iv, plaintext),
        CipherAlgorithm::AesCtr {
            counter,
            length_bits,
        } => {
            let Ok(counter): Result<[u8; 16], _> = counter.try_into() else {
                return Err(cipher_error(ctx, aes::CipherError::WrongCounterLength));
            };
            aes::ctr_encrypt_decrypt(key_bytes, &counter, length_bits, plaintext)
        }
        #[cfg(feature = "crypto-asymmetric")]
        CipherAlgorithm::RsaOaep { .. } => unreachable!("handled above"),
    };

    result.map_err(|err| cipher_error(ctx, err))
}

/// The actual decrypt operation, with no `KeyUsage` check - see `encrypt_bytes`.
#[cfg(feature = "crypto-cipher")]
fn decrypt_bytes<'js>(
    ctx: &Ctx<'js>,
    algorithm: CipherAlgorithm,
    key_ref: &CryptoKey,
    ciphertext: &[u8],
) -> rquickjs::Result<Vec<u8>> {
    #[cfg(feature = "crypto-asymmetric")]
    if let CipherAlgorithm::RsaOaep { label } = algorithm {
        require_key_type(ctx, key_ref, KeyType::Private)?;
        let hash = require_rsa_variant(ctx, key_ref, RsaVariant::Oaep)?;
        let rsa_backend::RsaKeyPair::Private(priv_key) = key_ref.rsa_key_pair().expect("checked above")
        else {
            throw_dom!(ctx, "InvalidAccessError", "key is not an RSA private key");
        };
        return rsa_backend::oaep_decrypt(priv_key, hash, label.as_deref(), ciphertext)
            .map_err(|e| super::key::rsa_error(ctx, e));
    }

    require_aes_variant(ctx, key_ref, aes_variant_of(&algorithm))?;
    let key_bytes = key_ref.key_bytes();

    let result = match algorithm {
        CipherAlgorithm::AesGcm {
            iv,
            additional_data,
            tag_length_bits,
        } => aes::gcm_decrypt(
            key_bytes,
            &iv,
            additional_data.as_deref(),
            tag_length_bits,
            ciphertext,
        ),
        CipherAlgorithm::AesCbc { iv } => aes::cbc_decrypt(key_bytes, &iv, ciphertext),
        CipherAlgorithm::AesCtr {
            counter,
            length_bits,
        } => {
            let Ok(counter): Result<[u8; 16], _> = counter.try_into() else {
                return Err(cipher_error(ctx, aes::CipherError::WrongCounterLength));
            };
            aes::ctr_encrypt_decrypt(key_bytes, &counter, length_bits, ciphertext)
        }
        #[cfg(feature = "crypto-asymmetric")]
        CipherAlgorithm::RsaOaep { .. } => unreachable!("handled above"),
    };

    result.map_err(|err| cipher_error(ctx, err))
}

#[cfg(feature = "crypto-cipher")]
async fn encrypt<'js>(
    ctx: Ctx<'js>,
    algorithm: CipherAlgorithm,
    key: Class<'js, CryptoKey>,
    data: Buffer<'js>,
) -> rquickjs::Result<ArrayBuffer<'js>> {
    let key_ref = key.borrow();
    require_usage(&ctx, &key_ref, KeyUsage::Encrypt)?;
    let plaintext = algorithm::buffer_bytes(&ctx, data)?;
    let ciphertext = encrypt_bytes(&ctx, algorithm, &key_ref, &plaintext)?;
    ArrayBuffer::new(ctx, ciphertext)
}

#[cfg(feature = "crypto-cipher")]
async fn decrypt<'js>(
    ctx: Ctx<'js>,
    algorithm: CipherAlgorithm,
    key: Class<'js, CryptoKey>,
    data: Buffer<'js>,
) -> rquickjs::Result<ArrayBuffer<'js>> {
    let key_ref = key.borrow();
    require_usage(&ctx, &key_ref, KeyUsage::Decrypt)?;
    let ciphertext = algorithm::buffer_bytes(&ctx, data)?;
    let plaintext = decrypt_bytes(&ctx, algorithm, &key_ref, &ciphertext)?;
    ArrayBuffer::new(ctx, plaintext)
}

/// The `AesVariant` a non-RSA `CipherAlgorithm` corresponds to - only called once RSA-OAEP has
/// already been handled separately, so every remaining variant is an AES one.
#[cfg(feature = "crypto-cipher")]
fn aes_variant_of(algorithm: &CipherAlgorithm) -> AesVariant {
    match algorithm {
        CipherAlgorithm::AesGcm { .. } => AesVariant::Gcm,
        CipherAlgorithm::AesCbc { .. } => AesVariant::Cbc,
        CipherAlgorithm::AesCtr { .. } => AesVariant::Ctr,
        #[cfg(feature = "crypto-asymmetric")]
        CipherAlgorithm::RsaOaep { .. } => unreachable!("RSA-OAEP is handled before this is called"),
    }
}

#[cfg(feature = "crypto-cipher")]
async fn sign<'js>(
    ctx: Ctx<'js>,
    algorithm: SignAlgorithm,
    key: Class<'js, CryptoKey>,
    data: Buffer<'js>,
) -> rquickjs::Result<ArrayBuffer<'js>> {
    let key_ref = key.borrow();
    require_usage(&ctx, &key_ref, KeyUsage::Sign)?;
    let data = algorithm::buffer_bytes(&ctx, data)?;

    let signature = match algorithm {
        SignAlgorithm::Hmac => {
            let hash = require_hmac(&ctx, &key_ref)?;
            hmac_ops::sign(hash, key_ref.key_bytes(), &data)
        }
        #[cfg(feature = "crypto-asymmetric")]
        SignAlgorithm::RsaSsaPkcs1 => {
            require_key_type(&ctx, &key_ref, KeyType::Private)?;
            let hash = require_rsa_variant(&ctx, &key_ref, RsaVariant::Pkcs1v15)?;
            let rsa_backend::RsaKeyPair::Private(priv_key) = key_ref.rsa_key_pair().expect("checked above") else {
                throw_dom!(ctx, "InvalidAccessError", "key is not an RSA private key");
            };
            let hashed = super::digest::hash_bytes(hash, &data);
            rsa_backend::pkcs1v15_sign(priv_key, hash, &hashed)
                .map_err(|e| super::key::rsa_error(&ctx, e))?
        }
        #[cfg(feature = "crypto-asymmetric")]
        SignAlgorithm::RsaPss { salt_length } => {
            require_key_type(&ctx, &key_ref, KeyType::Private)?;
            let hash = require_rsa_variant(&ctx, &key_ref, RsaVariant::Pss)?;
            let rsa_backend::RsaKeyPair::Private(priv_key) = key_ref.rsa_key_pair().expect("checked above") else {
                throw_dom!(ctx, "InvalidAccessError", "key is not an RSA private key");
            };
            let hashed = super::digest::hash_bytes(hash, &data);
            rsa_backend::pss_sign(priv_key, hash, salt_length, &hashed)
                .map_err(|e| super::key::rsa_error(&ctx, e))?
        }
        #[cfg(feature = "crypto-asymmetric")]
        SignAlgorithm::Ecdsa { hash } => {
            require_key_type(&ctx, &key_ref, KeyType::Private)?;
            require_ec_variant(&ctx, &key_ref, EcVariant::Ecdsa)?;
            let pair = key_ref.ec_key_pair().expect("checked above");
            let hashed = super::digest::hash_bytes(hash, &data);
            ec::ecdsa_sign(pair, &hashed).map_err(|e| super::key::ec_error(&ctx, e))?
        }
    };
    ArrayBuffer::new(ctx, signature)
}

#[cfg(feature = "crypto-cipher")]
async fn verify<'js>(
    ctx: Ctx<'js>,
    algorithm: SignAlgorithm,
    key: Class<'js, CryptoKey>,
    signature: Buffer<'js>,
    data: Buffer<'js>,
) -> rquickjs::Result<bool> {
    let key_ref = key.borrow();
    require_usage(&ctx, &key_ref, KeyUsage::Verify)?;
    let signature = algorithm::buffer_bytes(&ctx, signature)?;
    let data = algorithm::buffer_bytes(&ctx, data)?;

    Ok(match algorithm {
        SignAlgorithm::Hmac => {
            let hash = require_hmac(&ctx, &key_ref)?;
            hmac_ops::verify(hash, key_ref.key_bytes(), &data, &signature)
        }
        #[cfg(feature = "crypto-asymmetric")]
        SignAlgorithm::RsaSsaPkcs1 => {
            require_key_type(&ctx, &key_ref, KeyType::Public)?;
            let hash = require_rsa_variant(&ctx, &key_ref, RsaVariant::Pkcs1v15)?;
            let rsa_backend::RsaKeyPair::Public(pub_key) = key_ref.rsa_key_pair().expect("checked above") else {
                throw_dom!(ctx, "InvalidAccessError", "key is not an RSA public key");
            };
            let hashed = super::digest::hash_bytes(hash, &data);
            rsa_backend::pkcs1v15_verify(pub_key, hash, &hashed, &signature)
        }
        #[cfg(feature = "crypto-asymmetric")]
        SignAlgorithm::RsaPss { salt_length } => {
            require_key_type(&ctx, &key_ref, KeyType::Public)?;
            let hash = require_rsa_variant(&ctx, &key_ref, RsaVariant::Pss)?;
            let rsa_backend::RsaKeyPair::Public(pub_key) = key_ref.rsa_key_pair().expect("checked above") else {
                throw_dom!(ctx, "InvalidAccessError", "key is not an RSA public key");
            };
            let hashed = super::digest::hash_bytes(hash, &data);
            rsa_backend::pss_verify(pub_key, hash, salt_length, &hashed, &signature)
        }
        #[cfg(feature = "crypto-asymmetric")]
        SignAlgorithm::Ecdsa { hash } => {
            require_key_type(&ctx, &key_ref, KeyType::Public)?;
            require_ec_variant(&ctx, &key_ref, EcVariant::Ecdsa)?;
            let pair = key_ref.ec_key_pair().expect("checked above");
            let hashed = super::digest::hash_bytes(hash, &data);
            ec::ecdsa_verify(pair, &hashed, &signature)
        }
    })
}

/// Raw ECDH shared secret, or HKDF/PBKDF2 output, truncated/validated against `length` (bits).
#[cfg(feature = "crypto-asymmetric")]
async fn derive_bits<'js>(
    ctx: Ctx<'js>,
    algorithm: DeriveBitsAlgorithm<'js>,
    base_key: Class<'js, CryptoKey>,
    length: Opt<Option<u32>>,
) -> rquickjs::Result<ArrayBuffer<'js>> {
    let key_ref = base_key.borrow();
    require_usage(&ctx, &key_ref, KeyUsage::DeriveBits)?;
    let bytes = derive_bits_bytes(&ctx, &algorithm, &key_ref, length.0)?;
    ArrayBuffer::new(ctx, bytes)
}

/// A [`CryptoKey`] whose `algorithm` isn't `DeriveOnly(expected)` - checked before reading key
/// bytes for HKDF/PBKDF2, mirroring `require_hmac`/`require_aes_variant`'s role for the cipher
/// algorithms.
#[cfg(feature = "crypto-asymmetric")]
fn require_derive_key<'js, 'k>(
    ctx: &Ctx<'js>,
    key: &'k CryptoKey,
    expected: DeriveKind,
) -> rquickjs::Result<&'k [u8]> {
    match key.algorithm_variant() {
        KeyAlgorithm::DeriveOnly(kind) if kind == expected => Ok(key.key_bytes()),
        _ => throw_dom!(
            ctx,
            "InvalidAccessError",
            format!("key is not a {} key", expected.name())
        ),
    }
}

/// HKDF/PBKDF2 both require an explicit, non-zero, byte-aligned `length` (bits) - unlike ECDH,
/// which defaults to "the full shared secret" when `length` is omitted.
#[cfg(feature = "crypto-asymmetric")]
fn require_derive_length<'js>(
    ctx: &Ctx<'js>,
    length: Option<Option<u32>>,
    algorithm_name: &str,
) -> rquickjs::Result<usize> {
    match length {
        Some(Some(bits)) if bits != 0 && bits % 8 == 0 => Ok((bits / 8) as usize),
        _ => throw_dom!(
            ctx,
            "OperationError",
            format!("{algorithm_name} requires a non-zero length that is a multiple of 8")
        ),
    }
}

/// The actual derivation, shared by `deriveBits()` and `deriveKey()` - per spec, each checks a
/// different usage on `base_key` (`"deriveBits"` vs. `"deriveKey"`) before reaching this, so the
/// usage check lives in the two callers rather than here.
#[cfg(feature = "crypto-asymmetric")]
fn derive_bits_bytes<'js>(
    ctx: &Ctx<'js>,
    algorithm: &DeriveBitsAlgorithm<'js>,
    key_ref: &CryptoKey,
    length: Option<Option<u32>>,
) -> rquickjs::Result<Vec<u8>> {
    match algorithm {
        DeriveBitsAlgorithm::Ecdh { public } => {
            require_key_type(ctx, key_ref, KeyType::Private)?;
            require_ec_variant(ctx, key_ref, EcVariant::Ecdh)?;
            let private = key_ref.ec_key_pair().expect("checked above");

            let public_ref = public.borrow();
            require_ec_variant(ctx, &public_ref, EcVariant::Ecdh)?;
            let public = public_ref.ec_key_pair().expect("checked above");

            let mut bytes =
                ec::ecdh_derive_bits(private, public).map_err(|e| super::key::ec_error(ctx, e))?;

            if let Some(Some(length_bits)) = length {
                if length_bits as usize > bytes.len() * 8 {
                    throw_dom!(
                        ctx,
                        "OperationError",
                        "requested length is longer than the derived shared secret"
                    );
                }
                let length_bytes = length_bits.div_ceil(8) as usize;
                bytes.truncate(length_bytes);
                if length_bits % 8 != 0 {
                    // Zero out the padding bits in the last, partially-used byte.
                    if let Some(last) = bytes.last_mut() {
                        let used_bits = length_bits % 8;
                        *last &= 0xFFu8 << (8 - used_bits);
                    }
                }
            }

            Ok(bytes)
        }
        DeriveBitsAlgorithm::Hkdf { hash, salt, info } => {
            let length_bytes = require_derive_length(ctx, length, "HKDF")?;
            let ikm = require_derive_key(ctx, key_ref, DeriveKind::Hkdf)?;
            kdf::hkdf_derive_bits(*hash, salt, ikm, info, length_bytes).map_err(|_| {
                DOMException::throw_named(ctx, "OperationError", "requested length is too long")
            })
        }
        DeriveBitsAlgorithm::Pbkdf2 {
            hash,
            salt,
            iterations,
        } => {
            let length_bytes = require_derive_length(ctx, length, "PBKDF2")?;
            if *iterations == 0 {
                throw_dom!(
                    ctx,
                    "OperationError",
                    "PBKDF2 iterations must be greater than zero"
                );
            }
            let password = require_derive_key(ctx, key_ref, DeriveKind::Pbkdf2)?;
            Ok(kdf::pbkdf2_derive_bits(
                *hash,
                password,
                salt,
                *iterations,
                length_bytes,
            ))
        }
    }
}

/// `deriveBits` followed by wrapping the resulting bytes as a symmetric `CryptoKey` - the KDF
/// itself (if any) is whichever `algorithm` names (plain ECDH has none; HKDF/PBKDF2 *are* the
/// KDF), never a second one layered on top of the derived bits.
#[cfg(feature = "crypto-asymmetric")]
async fn derive_key<'js>(
    ctx: Ctx<'js>,
    algorithm: DeriveBitsAlgorithm<'js>,
    base_key: Class<'js, CryptoKey>,
    derived_key_algorithm: super::algorithm::KeyGenAlgorithm,
    extractable: bool,
    usages: Vec<KeyUsage>,
) -> rquickjs::Result<Class<'js, CryptoKey>> {
    let length_bits = match &derived_key_algorithm {
        super::algorithm::KeyGenAlgorithm::Aes { length, .. } => *length as u32,
        super::algorithm::KeyGenAlgorithm::Hmac { hash, length } => {
            length.unwrap_or_else(|| super::hmac::default_key_length_bits(*hash))
        }
        _ => throw_dom!(
            ctx,
            "NotSupportedError",
            "deriveKey only supports deriving an AES or HMAC key"
        ),
    };

    let bytes = {
        let key_ref = base_key.borrow();
        require_usage(&ctx, &key_ref, KeyUsage::DeriveKey)?;
        derive_bits_bytes(&ctx, &algorithm, &key_ref, Some(Some(length_bits)))?
    };
    let bits = ArrayBuffer::new(ctx.clone(), bytes)?;
    let import_algorithm = match derived_key_algorithm {
        super::algorithm::KeyGenAlgorithm::Aes { variant, .. } => ImportAlgorithm::Aes(variant),
        super::algorithm::KeyGenAlgorithm::Hmac { hash, .. } => ImportAlgorithm::Hmac { hash },
        _ => unreachable!("checked above"),
    };
    super::key::import_key(
        ctx,
        KeyFormat::Raw,
        bits.into_value(),
        import_algorithm,
        extractable,
        usages,
    )
    .await
}

/// Wraps `key`'s exported bytes with `wrapping_key`/`wrap_algorithm` (any of `encrypt()`'s
/// algorithms). Only the byte-shaped formats (`"raw"`/`"pkcs8"`/`"spki"`) are supported - `"jwk"`
/// would need a JSON-serialize-then-encrypt round trip this milestone doesn't implement.
#[cfg(feature = "crypto-asymmetric")]
async fn wrap_key<'js>(
    ctx: Ctx<'js>,
    format: KeyFormat,
    key: Class<'js, CryptoKey>,
    wrapping_key: Class<'js, CryptoKey>,
    wrap_algorithm: CipherAlgorithm,
) -> rquickjs::Result<ArrayBuffer<'js>> {
    if matches!(format, KeyFormat::Jwk) {
        throw_dom!(
            ctx,
            "NotSupportedError",
            "wrapKey/unwrapKey with format \"jwk\" is not supported"
        );
    }
    let wrapping_key_ref = wrapping_key.borrow();
    require_usage(&ctx, &wrapping_key_ref, KeyUsage::WrapKey)?;
    let exported = super::key::export_key(ctx.clone(), format, key).await?;
    let data = algorithm::buffer_bytes(&ctx, Buffer::from_js(&ctx, exported)?)?;
    let ciphertext = encrypt_bytes(&ctx, wrap_algorithm, &wrapping_key_ref, &data)?;
    ArrayBuffer::new(ctx, ciphertext)
}

#[cfg(feature = "crypto-asymmetric")]
#[allow(clippy::too_many_arguments)]
async fn unwrap_key<'js>(
    ctx: Ctx<'js>,
    format: KeyFormat,
    wrapped_key: Buffer<'js>,
    unwrapping_key: Class<'js, CryptoKey>,
    unwrap_algorithm: CipherAlgorithm,
    unwrapped_key_algorithm: ImportAlgorithm,
    Flat((extractable, usages)): Flat<(bool, Vec<KeyUsage>)>,
) -> rquickjs::Result<Class<'js, CryptoKey>> {
    if matches!(format, KeyFormat::Jwk) {
        throw_dom!(
            ctx,
            "NotSupportedError",
            "wrapKey/unwrapKey with format \"jwk\" is not supported"
        );
    }
    let wrapped_bytes = algorithm::buffer_bytes(&ctx, wrapped_key)?;
    let plaintext = {
        let unwrapping_key_ref = unwrapping_key.borrow();
        require_usage(&ctx, &unwrapping_key_ref, KeyUsage::UnwrapKey)?;
        decrypt_bytes(&ctx, unwrap_algorithm, &unwrapping_key_ref, &wrapped_bytes)?
    };
    super::key::import_key(
        ctx.clone(),
        format,
        ArrayBuffer::new(ctx, plaintext)?.into_value(),
        unwrapped_key_algorithm,
        extractable,
        usages,
    )
    .await
}

impl ModuleDef for CryptoModule {
    fn declare<'js>(decl: &rquickjs::module::Declarations<'js>) -> rquickjs::Result<()> {
        decl.declare("randomUUID")?;
        decl.declare("getRandomValues")?;
        decl.declare("subtle")?;
        Ok(())
    }

    fn evaluate<'js>(
        ctx: &Ctx<'js>,
        exports: &rquickjs::module::Exports<'js>,
    ) -> rquickjs::Result<()> {
        Self::export(ctx, &Registry::instance(ctx)?, exports)?;
        Ok(())
    }
}

impl<'js> Exportable<'js> for CryptoModule {
    fn export<T>(
        ctx: &rquickjs::Ctx<'js>,
        registry: &klaver_core::Registry,
        target: &T,
    ) -> rquickjs::Result<()>
    where
        T: klaver_core::ExportTarget<'js>,
    {
        let subtle = Object::new(ctx.clone())?;

        Digest::export(ctx, registry, &subtle)?;

        subtle.set(
            "digest",
            Func::new(Async(
                |ctx: Ctx<'js>, algo: Algo, buffer: Buffer<'js>| async move {
                    let mut digest = Digest::new(algo)?;
                    digest.update(ctx.clone(), buffer)?;
                    digest.digest(ctx)
                },
            )),
        )?;

        // `CryptoKey` is a global interface object per spec (`globalThis.CryptoKey`), not a
        // property of `crypto`/`crypto.subtle` - `target` here is the `crypto` object itself
        // (see `Global::define` below), so this reaches past it to real globals.
        #[cfg(feature = "crypto-cipher")]
        CryptoKey::export(ctx, registry, &ctx.globals())?;

        #[cfg(feature = "crypto-cipher")]
        subtle.set("generateKey", Func::new(Async(super::key::generate_key)))?;
        #[cfg(feature = "crypto-cipher")]
        subtle.set("importKey", Func::new(Async(super::key::import_key)))?;
        #[cfg(feature = "crypto-cipher")]
        subtle.set("exportKey", Func::new(Async(super::key::export_key)))?;
        #[cfg(feature = "crypto-cipher")]
        subtle.set("encrypt", Func::new(Async(encrypt)))?;
        #[cfg(feature = "crypto-cipher")]
        subtle.set("decrypt", Func::new(Async(decrypt)))?;
        #[cfg(feature = "crypto-cipher")]
        subtle.set("sign", Func::new(Async(sign)))?;
        #[cfg(feature = "crypto-cipher")]
        subtle.set("verify", Func::new(Async(verify)))?;

        #[cfg(feature = "crypto-asymmetric")]
        subtle.set("deriveBits", Func::new(Async(derive_bits)))?;
        #[cfg(feature = "crypto-asymmetric")]
        subtle.set("deriveKey", Func::new(Async(derive_key)))?;
        #[cfg(feature = "crypto-asymmetric")]
        subtle.set("wrapKey", Func::new(Async(wrap_key)))?;
        #[cfg(feature = "crypto-asymmetric")]
        subtle.set("unwrapKey", Func::new(Async(unwrap_key)))?;

        target.set(ctx, "randomUUID", Func::new(super::random::random_uuid))?;
        target.set(
            ctx,
            "getRandomValues",
            Func::new(super::random::random_values),
        )?;

        target.set(ctx, "subtle", subtle)?;

        Ok(())
    }
}

#[cfg(feature = "module")]
impl klaver_modules::Global for CryptoModule {
    fn define<'a, 'js: 'a>(
        &'a self,
        ctx: rquickjs::Ctx<'js>,
    ) -> impl Future<Output = rquickjs::Result<()>> + 'a {
        async move {
            let obj = Object::new(ctx.clone())?;

            Self::export(&ctx, &Registry::instance(&ctx)?, &obj)?;

            ctx.globals().set("crypto", obj)?;

            Ok(())
        }
    }
}

#[cfg(feature = "module")]
impl klaver_modules::GlobalInfo for CryptoModule {
    fn register(builder: &mut klaver_modules::GlobalBuilder<'_, Self>) {
        builder.register(CryptoModule {});
    }

    fn typings() -> Option<std::borrow::Cow<'static, str>> {
        Some(std::borrow::Cow::Borrowed(include_str!(
            "../../types/crypto.d.ts"
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rquickjs::{CatchResultExt, Context, Runtime};

    /// Runs `body` as the contents of a plain function, with a global `crypto` object available.
    /// `body` is expected to throw on failure (e.g. via a plain `if (...) throw ...`).
    fn run(body: &str) {
        let runtime = Runtime::new().unwrap();
        let context = Context::full(&runtime).unwrap();

        context
            .with(|ctx| {
                let crypto = Object::new(ctx.clone())?;
                CryptoModule::export(&ctx, &Registry::instance(&ctx)?, &crypto)?;
                ctx.globals().set("crypto", crypto)?;

                let test_fn: rquickjs::Function = ctx.eval(format!("(() => {{\n{body}\n}})"))?;

                if let Err(err) = test_fn.call::<_, ()>(()).catch(&ctx) {
                    panic!("{err}");
                }

                rquickjs::Result::Ok(())
            })
            .unwrap();
    }

    #[test]
    fn get_random_values_is_registered_under_the_spec_name() {
        run(r#"
            if (typeof crypto.getRandomValues !== "function") {
                throw new Error("crypto.getRandomValues is not a function");
            }
            if (typeof crypto.randomValues !== "undefined") {
                throw new Error("crypto.randomValues should not exist");
            }

            const buf = new Uint8Array(16);
            crypto.getRandomValues(buf);
            if (buf.every((b) => b === 0)) throw new Error("buffer was not filled");
        "#);
    }
}

#[cfg(all(test, feature = "crypto-cipher"))]
mod cipher_tests {
    use super::*;
    use klaver_core::value::FunctionExt;
    use rquickjs::{AsyncContext, AsyncRuntime, CatchResultExt, Function};

    /// Runs `body` as the contents of an async IIFE, with a global `crypto` object (including
    /// `subtle`'s cipher surface) available. `body` is expected to throw on failure.
    fn run(body: &str) {
        futures::executor::block_on(async move {
            let rt = AsyncRuntime::new().unwrap();
            let ctx = AsyncContext::full(&rt).await.unwrap();

            ctx.async_with(async |ctx| {
                let crypto = Object::new(ctx.clone())?;
                CryptoModule::export(&ctx, &Registry::instance(&ctx)?, &crypto)?;
                ctx.globals().set("crypto", crypto)?;

                let test_fn: Function = ctx.eval(format!("(async () => {{\n{body}\n}})"))?;

                if let Err(err) = test_fn.call_async::<_, ()>(()).await.catch(&ctx) {
                    panic!("{err}");
                }

                rquickjs::Result::Ok(())
            })
            .await
            .unwrap();
        });
    }

    #[test]
    fn aes_gcm_round_trips_and_rejects_tampering() {
        run(r#"
            const key = await crypto.subtle.generateKey(
                { name: "AES-GCM", length: 256 }, true, ["encrypt", "decrypt"]);
            const iv = new Uint8Array(12);
            crypto.getRandomValues(iv);
            const data = new Uint8Array([1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);

            const ciphertext = await crypto.subtle.encrypt({ name: "AES-GCM", iv }, key, data);
            const plaintext = new Uint8Array(
                await crypto.subtle.decrypt({ name: "AES-GCM", iv }, key, ciphertext));
            if (plaintext.length !== data.length || !plaintext.every((b, i) => b === data[i])) {
                throw new Error("round trip did not recover the original plaintext");
            }

            const tampered = new Uint8Array(ciphertext);
            tampered[0] ^= 1;
            let threw = false;
            try {
                await crypto.subtle.decrypt({ name: "AES-GCM", iv }, key, tampered);
            } catch (err) {
                threw = true;
                if (err.name !== "OperationError") throw new Error(`wrong error name: ${err.name}`);
            }
            if (!threw) throw new Error("decrypt did not reject tampered ciphertext");
        "#);
    }

    #[test]
    fn aes_cbc_round_trips_non_block_aligned_data() {
        run(r#"
            const key = await crypto.subtle.generateKey(
                { name: "AES-CBC", length: 128 }, true, ["encrypt", "decrypt"]);
            const iv = new Uint8Array(16);
            crypto.getRandomValues(iv);
            const data = new Uint8Array([1, 2, 3, 4, 5]);

            const ciphertext = await crypto.subtle.encrypt({ name: "AES-CBC", iv }, key, data);
            const plaintext = new Uint8Array(
                await crypto.subtle.decrypt({ name: "AES-CBC", iv }, key, ciphertext));
            if (plaintext.length !== data.length || !plaintext.every((b, i) => b === data[i])) {
                throw new Error("round trip did not recover the original plaintext");
            }
        "#);
    }

    #[test]
    fn aes_ctr_round_trips() {
        run(r#"
            const key = await crypto.subtle.generateKey(
                { name: "AES-CTR", length: 128 }, true, ["encrypt", "decrypt"]);
            const counter = new Uint8Array(16);
            crypto.getRandomValues(counter);
            const data = new Uint8Array(64).map((_, i) => i);

            const ciphertext = await crypto.subtle.encrypt(
                { name: "AES-CTR", counter, length: 64 }, key, data);
            const plaintext = new Uint8Array(
                await crypto.subtle.decrypt({ name: "AES-CTR", counter, length: 64 }, key, ciphertext));
            if (plaintext.length !== data.length || !plaintext.every((b, i) => b === data[i])) {
                throw new Error("round trip did not recover the original plaintext");
            }
        "#);
    }

    #[test]
    fn hmac_sign_and_verify() {
        run(r#"
            const key = await crypto.subtle.generateKey(
                { name: "HMAC", hash: "SHA-256" }, true, ["sign", "verify"]);
            const data = new Uint8Array([1, 2, 3, 4]);

            const signature = await crypto.subtle.sign("HMAC", key, data);
            const ok = await crypto.subtle.verify("HMAC", key, signature, data);
            if (!ok) throw new Error("verify() rejected a genuine signature");

            const tampered = new Uint8Array(signature);
            tampered[0] ^= 1;
            const shouldFail = await crypto.subtle.verify("HMAC", key, tampered, data);
            if (shouldFail) throw new Error("verify() accepted a tampered signature");
        "#);
    }

    #[test]
    fn generate_import_export_round_trips_through_raw_and_jwk() {
        run(r#"
            const key = await crypto.subtle.generateKey(
                { name: "AES-GCM", length: 128 }, true, ["encrypt", "decrypt"]);

            const raw = await crypto.subtle.exportKey("raw", key);
            const imported = await crypto.subtle.importKey(
                "raw", raw, "AES-GCM", true, ["encrypt", "decrypt"]);
            const rawAgain = new Uint8Array(await crypto.subtle.exportKey("raw", imported));
            if (!rawAgain.every((b, i) => b === new Uint8Array(raw)[i])) {
                throw new Error("raw round trip changed the key bytes");
            }

            const jwk = await crypto.subtle.exportKey("jwk", key);
            if (jwk.kty !== "oct") throw new Error(`unexpected kty: ${jwk.kty}`);
            const importedFromJwk = await crypto.subtle.importKey(
                "jwk", jwk, "AES-GCM", true, ["encrypt", "decrypt"]);
            const rawFromJwk = new Uint8Array(await crypto.subtle.exportKey("raw", importedFromJwk));
            if (!rawFromJwk.every((b, i) => b === new Uint8Array(raw)[i])) {
                throw new Error("jwk round trip changed the key bytes");
            }
        "#);
    }

    #[test]
    fn crypto_key_getters_and_illegal_constructor() {
        run(r#"
            const key = await crypto.subtle.generateKey(
                { name: "AES-GCM", length: 128 }, true, ["encrypt", "decrypt"]);

            if (key.type !== "secret") throw new Error(`type was ${key.type}`);
            if (key.extractable !== true) throw new Error("extractable was false");
            if (key.algorithm.name !== "AES-GCM") throw new Error("algorithm.name mismatch");
            if (key.algorithm.length !== 128) throw new Error("algorithm.length mismatch");
            if (key.algorithm === key.algorithm) {
                // Getter must build a fresh object each access, not cache a live JS value.
                throw new Error("algorithm getter returned the same object twice");
            }
            if (!key.usages.includes("encrypt") || !key.usages.includes("decrypt")) {
                throw new Error("usages did not round-trip");
            }

            let threw = false;
            try {
                new CryptoKey();
            } catch (err) {
                threw = err instanceof TypeError;
            }
            if (!threw) throw new Error("new CryptoKey() did not throw a TypeError");
        "#);
    }

    #[test]
    fn encrypt_with_wrong_key_kind_throws_invalid_access_error() {
        run(r#"
            const hmacKey = await crypto.subtle.generateKey(
                { name: "HMAC", hash: "SHA-256" }, true, ["sign", "verify"]);
            const iv = new Uint8Array(12);

            let threw = false;
            try {
                await crypto.subtle.encrypt({ name: "AES-GCM", iv }, hmacKey, new Uint8Array(4));
            } catch (err) {
                threw = true;
                if (err.name !== "InvalidAccessError") throw new Error(`wrong error name: ${err.name}`);
            }
            if (!threw) throw new Error("encrypt() did not reject a non-AES key");
        "#);
    }

    #[test]
    fn non_extractable_key_cannot_be_exported() {
        run(r#"
            const key = await crypto.subtle.generateKey(
                { name: "AES-GCM", length: 128 }, false, ["encrypt", "decrypt"]);

            let threw = false;
            try {
                await crypto.subtle.exportKey("raw", key);
            } catch (err) {
                threw = true;
                if (err.name !== "InvalidAccessError") throw new Error(`wrong error name: ${err.name}`);
            }
            if (!threw) throw new Error("exportKey() did not reject a non-extractable key");
        "#);
    }
}

#[cfg(all(test, feature = "crypto-asymmetric"))]
mod asymmetric_tests {
    use super::*;
    use klaver_core::value::FunctionExt;
    use rquickjs::{AsyncContext, AsyncRuntime, CatchResultExt, Function};

    /// Same pattern as `cipher_tests::run` in the parent module.
    fn run(body: &str) {
        futures::executor::block_on(async move {
            let rt = AsyncRuntime::new().unwrap();
            let ctx = AsyncContext::full(&rt).await.unwrap();

            ctx.async_with(async |ctx| {
                let crypto = Object::new(ctx.clone())?;
                CryptoModule::export(&ctx, &Registry::instance(&ctx)?, &crypto)?;
                ctx.globals().set("crypto", crypto)?;

                let test_fn: Function = ctx.eval(format!("(async () => {{\n{body}\n}})"))?;

                if let Err(err) = test_fn.call_async::<_, ()>(()).await.catch(&ctx) {
                    panic!("{err}");
                }

                rquickjs::Result::Ok(())
            })
            .await
            .unwrap();
        });
    }

    #[test]
    fn generate_key_returns_a_crypto_key_pair() {
        run(r#"
            const pair = await crypto.subtle.generateKey(
                { name: "RSASSA-PKCS1-v1_5", modulusLength: 1024, publicExponent: new Uint8Array([1, 0, 1]), hash: "SHA-256" },
                true, ["sign", "verify"]);
            if (pair.privateKey.type !== "private") throw new Error(`privateKey.type was ${pair.privateKey.type}`);
            if (pair.publicKey.type !== "public") throw new Error(`publicKey.type was ${pair.publicKey.type}`);
            if (!pair.privateKey.usages.includes("sign")) throw new Error("privateKey missing sign usage");
            if (!pair.publicKey.usages.includes("verify")) throw new Error("publicKey missing verify usage");
            if (pair.privateKey.usages.includes("verify")) throw new Error("privateKey should not get verify usage");
        "#);
    }

    #[test]
    fn rsassa_pkcs1_v1_5_sign_and_verify_round_trips_and_rejects_tampering() {
        run(r#"
            const { publicKey, privateKey } = await crypto.subtle.generateKey(
                { name: "RSASSA-PKCS1-v1_5", modulusLength: 1024, publicExponent: new Uint8Array([1, 0, 1]), hash: "SHA-256" },
                true, ["sign", "verify"]);
            const data = new Uint8Array([1, 2, 3, 4, 5]);

            const signature = await crypto.subtle.sign("RSASSA-PKCS1-v1_5", privateKey, data);
            const ok = await crypto.subtle.verify("RSASSA-PKCS1-v1_5", publicKey, signature, data);
            if (!ok) throw new Error("verify() rejected a genuine signature");

            const tampered = new Uint8Array(signature);
            tampered[0] ^= 1;
            const shouldFail = await crypto.subtle.verify("RSASSA-PKCS1-v1_5", publicKey, tampered, data);
            if (shouldFail) throw new Error("verify() accepted a tampered signature");
        "#);
    }

    #[test]
    fn rsa_oaep_encrypt_and_decrypt_round_trips() {
        run(r#"
            const { publicKey, privateKey } = await crypto.subtle.generateKey(
                { name: "RSA-OAEP", modulusLength: 1024, publicExponent: new Uint8Array([1, 0, 1]), hash: "SHA-256" },
                true, ["encrypt", "decrypt"]);
            const data = new Uint8Array([1, 2, 3, 4, 5, 6, 7, 8]);

            const ciphertext = await crypto.subtle.encrypt("RSA-OAEP", publicKey, data);
            const plaintext = new Uint8Array(await crypto.subtle.decrypt("RSA-OAEP", privateKey, ciphertext));
            if (plaintext.length !== data.length || !plaintext.every((b, i) => b === data[i])) {
                throw new Error("round trip did not recover the original plaintext");
            }
        "#);
    }

    #[test]
    fn ecdsa_p256_and_p384_sign_and_verify_round_trip() {
        run(r#"
            for (const namedCurve of ["P-256", "P-384", "P-521"]) {
                const { publicKey, privateKey } = await crypto.subtle.generateKey(
                    { name: "ECDSA", namedCurve }, true, ["sign", "verify"]);
                const data = new Uint8Array([9, 8, 7, 6, 5]);

                const signature = await crypto.subtle.sign({ name: "ECDSA", hash: "SHA-256" }, privateKey, data);
                const ok = await crypto.subtle.verify({ name: "ECDSA", hash: "SHA-256" }, publicKey, signature, data);
                if (!ok) throw new Error(`${namedCurve}: verify() rejected a genuine signature`);

                const tampered = new Uint8Array(signature);
                tampered[0] ^= 1;
                const shouldFail = await crypto.subtle.verify({ name: "ECDSA", hash: "SHA-256" }, publicKey, tampered, data);
                if (shouldFail) throw new Error(`${namedCurve}: verify() accepted a tampered signature`);
            }
        "#);
    }

    #[test]
    fn ecdh_derive_bits_produces_a_matching_shared_secret_on_both_sides() {
        run(r#"
            const alice = await crypto.subtle.generateKey({ name: "ECDH", namedCurve: "P-256" }, true, ["deriveBits"]);
            const bob = await crypto.subtle.generateKey({ name: "ECDH", namedCurve: "P-256" }, true, ["deriveBits"]);

            const aliceSecret = new Uint8Array(await crypto.subtle.deriveBits(
                { name: "ECDH", public: bob.publicKey }, alice.privateKey, 256));
            const bobSecret = new Uint8Array(await crypto.subtle.deriveBits(
                { name: "ECDH", public: alice.publicKey }, bob.privateKey, 256));

            if (aliceSecret.length !== 32) throw new Error(`expected 32 bytes, got ${aliceSecret.length}`);
            if (!aliceSecret.every((b, i) => b === bobSecret[i])) {
                throw new Error("shared secrets did not match");
            }
        "#);
    }

    #[test]
    fn ecdh_derive_key_produces_a_usable_aes_gcm_key() {
        run(r#"
            const alice = await crypto.subtle.generateKey({ name: "ECDH", namedCurve: "P-256" }, true, ["deriveKey"]);
            const bob = await crypto.subtle.generateKey({ name: "ECDH", namedCurve: "P-256" }, true, ["deriveKey"]);

            const aliceKey = await crypto.subtle.deriveKey(
                { name: "ECDH", public: bob.publicKey }, alice.privateKey,
                { name: "AES-GCM", length: 128 }, false, ["encrypt"]);
            const bobKey = await crypto.subtle.deriveKey(
                { name: "ECDH", public: alice.publicKey }, bob.privateKey,
                { name: "AES-GCM", length: 128 }, false, ["decrypt"]);

            const iv = new Uint8Array(12);
            crypto.getRandomValues(iv);
            const data = new Uint8Array([1, 2, 3, 4]);
            const ciphertext = await crypto.subtle.encrypt({ name: "AES-GCM", iv }, aliceKey, data);
            const plaintext = new Uint8Array(await crypto.subtle.decrypt({ name: "AES-GCM", iv }, bobKey, ciphertext));
            if (!plaintext.every((b, i) => b === data[i])) {
                throw new Error("derived keys did not agree");
            }
        "#);
    }

    #[test]
    fn rsa_pkcs8_and_spki_export_import_round_trip() {
        run(r#"
            const { publicKey, privateKey } = await crypto.subtle.generateKey(
                { name: "RSASSA-PKCS1-v1_5", modulusLength: 1024, publicExponent: new Uint8Array([1, 0, 1]), hash: "SHA-256" },
                true, ["sign", "verify"]);

            const pkcs8 = await crypto.subtle.exportKey("pkcs8", privateKey);
            const importedPrivate = await crypto.subtle.importKey(
                "pkcs8", pkcs8, { name: "RSASSA-PKCS1-v1_5", hash: "SHA-256" }, true, ["sign"]);

            const spki = await crypto.subtle.exportKey("spki", publicKey);
            const importedPublic = await crypto.subtle.importKey(
                "spki", spki, { name: "RSASSA-PKCS1-v1_5", hash: "SHA-256" }, true, ["verify"]);

            const data = new Uint8Array([1, 2, 3]);
            const signature = await crypto.subtle.sign("RSASSA-PKCS1-v1_5", importedPrivate, data);
            const ok = await crypto.subtle.verify("RSASSA-PKCS1-v1_5", importedPublic, signature, data);
            if (!ok) throw new Error("round-tripped RSA keys did not agree");
        "#);
    }

    #[test]
    fn ec_pkcs8_and_spki_export_import_round_trip() {
        run(r#"
            const { publicKey, privateKey } = await crypto.subtle.generateKey(
                { name: "ECDSA", namedCurve: "P-256" }, true, ["sign", "verify"]);

            const pkcs8 = await crypto.subtle.exportKey("pkcs8", privateKey);
            const importedPrivate = await crypto.subtle.importKey(
                "pkcs8", pkcs8, { name: "ECDSA", namedCurve: "P-256" }, true, ["sign"]);

            const spki = await crypto.subtle.exportKey("spki", publicKey);
            const importedPublic = await crypto.subtle.importKey(
                "spki", spki, { name: "ECDSA", namedCurve: "P-256" }, true, ["verify"]);

            const raw = await crypto.subtle.exportKey("raw", importedPublic);
            const importedFromRaw = await crypto.subtle.importKey(
                "raw", raw, { name: "ECDSA", namedCurve: "P-256" }, true, ["verify"]);

            const data = new Uint8Array([4, 5, 6]);
            const signature = await crypto.subtle.sign({ name: "ECDSA", hash: "SHA-256" }, importedPrivate, data);
            if (!(await crypto.subtle.verify({ name: "ECDSA", hash: "SHA-256" }, importedPublic, signature, data))) {
                throw new Error("round-tripped EC keys (spki) did not agree");
            }
            if (!(await crypto.subtle.verify({ name: "ECDSA", hash: "SHA-256" }, importedFromRaw, signature, data))) {
                throw new Error("round-tripped EC keys (raw) did not agree");
            }
        "#);
    }

    #[test]
    fn rsa_jwk_export_import_round_trip() {
        run(r#"
            const { publicKey, privateKey } = await crypto.subtle.generateKey(
                { name: "RSASSA-PKCS1-v1_5", modulusLength: 1024, publicExponent: new Uint8Array([1, 0, 1]), hash: "SHA-256" },
                true, ["sign", "verify"]);

            const jwk = await crypto.subtle.exportKey("jwk", privateKey);
            if (jwk.kty !== "RSA") throw new Error(`unexpected kty: ${jwk.kty}`);
            if (!jwk.d || !jwk.p || !jwk.q) throw new Error("private JWK missing d/p/q");

            const importedPrivate = await crypto.subtle.importKey(
                "jwk", jwk, { name: "RSASSA-PKCS1-v1_5", hash: "SHA-256" }, true, ["sign"]);

            const pubJwk = await crypto.subtle.exportKey("jwk", publicKey);
            if (pubJwk.d) throw new Error("public JWK should not have d");
            const importedPublic = await crypto.subtle.importKey(
                "jwk", pubJwk, { name: "RSASSA-PKCS1-v1_5", hash: "SHA-256" }, true, ["verify"]);

            const data = new Uint8Array([7, 8, 9]);
            const signature = await crypto.subtle.sign("RSASSA-PKCS1-v1_5", importedPrivate, data);
            if (!(await crypto.subtle.verify("RSASSA-PKCS1-v1_5", importedPublic, signature, data))) {
                throw new Error("round-tripped RSA JWK keys did not agree");
            }
        "#);
    }

    #[test]
    fn ec_jwk_export_import_round_trip() {
        run(r#"
            const { publicKey, privateKey } = await crypto.subtle.generateKey(
                { name: "ECDSA", namedCurve: "P-256" }, true, ["sign", "verify"]);

            const jwk = await crypto.subtle.exportKey("jwk", privateKey);
            if (jwk.kty !== "EC") throw new Error(`unexpected kty: ${jwk.kty}`);
            if (jwk.crv !== "P-256") throw new Error(`unexpected crv: ${jwk.crv}`);
            if (!jwk.d) throw new Error("private JWK missing d");

            const importedPrivate = await crypto.subtle.importKey(
                "jwk", jwk, { name: "ECDSA", namedCurve: "P-256" }, true, ["sign"]);

            const pubJwk = await crypto.subtle.exportKey("jwk", publicKey);
            const importedPublic = await crypto.subtle.importKey(
                "jwk", pubJwk, { name: "ECDSA", namedCurve: "P-256" }, true, ["verify"]);

            const data = new Uint8Array([1, 1, 2, 3, 5]);
            const signature = await crypto.subtle.sign({ name: "ECDSA", hash: "SHA-256" }, importedPrivate, data);
            if (!(await crypto.subtle.verify({ name: "ECDSA", hash: "SHA-256" }, importedPublic, signature, data))) {
                throw new Error("round-tripped EC JWK keys did not agree");
            }
        "#);
    }

    #[test]
    fn wrap_key_and_unwrap_key_round_trip_with_an_aes_wrapping_key() {
        run(r#"
            const toWrap = await crypto.subtle.generateKey(
                { name: "AES-GCM", length: 128 }, true, ["encrypt", "decrypt"]);
            const wrappingKey = await crypto.subtle.generateKey(
                { name: "AES-GCM", length: 256 }, true, ["wrapKey", "unwrapKey"]);
            const iv = new Uint8Array(12);
            crypto.getRandomValues(iv);

            const wrapped = await crypto.subtle.wrapKey("raw", toWrap, wrappingKey, { name: "AES-GCM", iv });
            const unwrapped = await crypto.subtle.unwrapKey(
                "raw", wrapped, wrappingKey, { name: "AES-GCM", iv }, "AES-GCM", true, ["encrypt", "decrypt"]);

            const rawOriginal = new Uint8Array(await crypto.subtle.exportKey("raw", toWrap));
            const rawUnwrapped = new Uint8Array(await crypto.subtle.exportKey("raw", unwrapped));
            if (!rawOriginal.every((b, i) => b === rawUnwrapped[i])) {
                throw new Error("unwrapped key bytes did not match the original");
            }
        "#);
    }

    #[test]
    fn wrap_key_and_unwrap_key_round_trip_with_an_rsa_oaep_wrapping_key() {
        run(r#"
            const toWrap = await crypto.subtle.generateKey(
                { name: "AES-GCM", length: 128 }, true, ["encrypt", "decrypt"]);
            const { publicKey, privateKey } = await crypto.subtle.generateKey(
                { name: "RSA-OAEP", modulusLength: 1024, publicExponent: new Uint8Array([1, 0, 1]), hash: "SHA-256" },
                true, ["wrapKey", "unwrapKey"]);

            const wrapped = await crypto.subtle.wrapKey("raw", toWrap, publicKey, "RSA-OAEP");
            const unwrapped = await crypto.subtle.unwrapKey(
                "raw", wrapped, privateKey, "RSA-OAEP", "AES-GCM", true, ["encrypt", "decrypt"]);

            const rawOriginal = new Uint8Array(await crypto.subtle.exportKey("raw", toWrap));
            const rawUnwrapped = new Uint8Array(await crypto.subtle.exportKey("raw", unwrapped));
            if (!rawOriginal.every((b, i) => b === rawUnwrapped[i])) {
                throw new Error("unwrapped key bytes did not match the original");
            }
        "#);
    }

    #[test]
    fn rsassa_pss_sign_and_verify_round_trips_and_rejects_tampering() {
        run(r#"
            const { publicKey, privateKey } = await crypto.subtle.generateKey(
                { name: "RSA-PSS", modulusLength: 1024, publicExponent: new Uint8Array([1, 0, 1]), hash: "SHA-256" },
                true, ["sign", "verify"]);
            const data = new Uint8Array([1, 2, 3, 4, 5]);

            const signature = await crypto.subtle.sign({ name: "RSA-PSS", saltLength: 32 }, privateKey, data);
            const ok = await crypto.subtle.verify({ name: "RSA-PSS", saltLength: 32 }, publicKey, signature, data);
            if (!ok) throw new Error("verify() rejected a genuine signature");

            const tampered = new Uint8Array(signature);
            tampered[0] ^= 1;
            const shouldFail = await crypto.subtle.verify(
                { name: "RSA-PSS", saltLength: 32 }, publicKey, tampered, data);
            if (shouldFail) throw new Error("verify() accepted a tampered signature");
        "#);
    }

    #[test]
    fn rsassa_pss_signatures_are_randomized() {
        run(r#"
            const { publicKey, privateKey } = await crypto.subtle.generateKey(
                { name: "RSA-PSS", modulusLength: 1024, publicExponent: new Uint8Array([1, 0, 1]), hash: "SHA-256" },
                true, ["sign", "verify"]);
            const data = new Uint8Array([1, 2, 3]);

            const sig1 = new Uint8Array(
                await crypto.subtle.sign({ name: "RSA-PSS", saltLength: 32 }, privateKey, data));
            const sig2 = new Uint8Array(
                await crypto.subtle.sign({ name: "RSA-PSS", saltLength: 32 }, privateKey, data));
            if (sig1.every((b, i) => b === sig2[i])) {
                throw new Error("two PSS signatures over the same message were identical");
            }
            if (!(await crypto.subtle.verify({ name: "RSA-PSS", saltLength: 32 }, publicKey, sig2, data))) {
                throw new Error("second signature did not verify");
            }
        "#);
    }

    #[test]
    fn ecdh_p521_derive_bits_produces_a_matching_shared_secret_on_both_sides() {
        run(r#"
            const alice = await crypto.subtle.generateKey({ name: "ECDH", namedCurve: "P-521" }, true, ["deriveBits"]);
            const bob = await crypto.subtle.generateKey({ name: "ECDH", namedCurve: "P-521" }, true, ["deriveBits"]);

            const aliceSecret = new Uint8Array(await crypto.subtle.deriveBits(
                { name: "ECDH", public: bob.publicKey }, alice.privateKey, 528));
            const bobSecret = new Uint8Array(await crypto.subtle.deriveBits(
                { name: "ECDH", public: alice.publicKey }, bob.privateKey, 528));

            if (aliceSecret.length !== 66) throw new Error(`expected 66 bytes, got ${aliceSecret.length}`);
            if (!aliceSecret.every((b, i) => b === bobSecret[i])) {
                throw new Error("shared secrets did not match");
            }
        "#);
    }

    #[test]
    fn hkdf_derive_bits_matches_rfc5869_test_case_1() {
        run(r#"
            const ikm = new Uint8Array(22).fill(0x0b);
            const salt = new Uint8Array([0,1,2,3,4,5,6,7,8,9,10,11,12]);
            const info = new Uint8Array([0xf0,0xf1,0xf2,0xf3,0xf4,0xf5,0xf6,0xf7,0xf8,0xf9]);
            const expected = new Uint8Array([
                0x3c,0xb2,0x5f,0x25,0xfa,0xac,0xd5,0x7a,0x90,0x43,0x4f,0x64,0xd0,0x36,
                0x2f,0x2a,0x2d,0x2d,0x0a,0x90,0xcf,0x1a,0x5a,0x4c,0x5d,0xb0,0x2d,0x56,
                0xec,0xc4,0xc5,0xbf,0x34,0x00,0x72,0x08,0xd5,0xb8,0x87,0x18,0x58,0x65,
            ]);

            const baseKey = await crypto.subtle.importKey("raw", ikm, "HKDF", false, ["deriveBits"]);
            const okm = new Uint8Array(await crypto.subtle.deriveBits(
                { name: "HKDF", hash: "SHA-256", salt, info }, baseKey, 42 * 8));
            if (!okm.every((b, i) => b === expected[i])) {
                throw new Error("HKDF output did not match the RFC 5869 test vector");
            }
        "#);
    }

    #[test]
    fn hkdf_base_key_cannot_be_extractable() {
        run(r#"
            let threw = false;
            try {
                await crypto.subtle.importKey("raw", new Uint8Array(16), "HKDF", true, ["deriveBits"]);
            } catch (err) {
                threw = true;
                if (err.name !== "SyntaxError") throw new Error(`wrong error name: ${err.name}`);
            }
            if (!threw) throw new Error("importKey() did not reject an extractable HKDF key");
        "#);
    }

    #[test]
    fn hkdf_derive_key_produces_a_usable_aes_gcm_key() {
        run(r#"
            const baseKey = await crypto.subtle.importKey(
                "raw", new Uint8Array(32).fill(7), "HKDF", false, ["deriveKey"]);
            const salt = new Uint8Array(16);
            crypto.getRandomValues(salt);

            const aesKey = await crypto.subtle.deriveKey(
                { name: "HKDF", hash: "SHA-256", salt, info: new Uint8Array(0) },
                baseKey, { name: "AES-GCM", length: 128 }, false, ["encrypt", "decrypt"]);

            const iv = new Uint8Array(12);
            crypto.getRandomValues(iv);
            const data = new Uint8Array([1, 2, 3, 4]);
            const ciphertext = await crypto.subtle.encrypt({ name: "AES-GCM", iv }, aesKey, data);
            const plaintext = new Uint8Array(await crypto.subtle.decrypt({ name: "AES-GCM", iv }, aesKey, ciphertext));
            if (!plaintext.every((b, i) => b === data[i])) {
                throw new Error("HKDF-derived key did not round-trip AES-GCM");
            }
        "#);
    }

    #[test]
    fn pbkdf2_derive_bits_matches_rfc6070_test_case_1() {
        run(r#"
            // "password" / "salt" as raw ASCII bytes (no TextEncoder in this bare test harness).
            const password = new Uint8Array([0x70, 0x61, 0x73, 0x73, 0x77, 0x6f, 0x72, 0x64]);
            const salt = new Uint8Array([0x73, 0x61, 0x6c, 0x74]);
            const expected = new Uint8Array([
                0x0c, 0x60, 0xc8, 0x0f, 0x96, 0x1f, 0x0e, 0x71, 0xf3, 0xa9,
                0xb5, 0x24, 0xaf, 0x60, 0x12, 0x06, 0x2f, 0xe0, 0x37, 0xa6,
            ]);

            const baseKey = await crypto.subtle.importKey("raw", password, "PBKDF2", false, ["deriveBits"]);
            const bits = new Uint8Array(await crypto.subtle.deriveBits(
                { name: "PBKDF2", hash: "SHA-1", salt, iterations: 1 }, baseKey, 20 * 8));
            if (!bits.every((b, i) => b === expected[i])) {
                throw new Error("PBKDF2 output did not match the RFC 6070 test vector");
            }
        "#);
    }

    #[test]
    fn pbkdf2_derive_bits_rejects_a_length_that_is_not_a_multiple_of_eight() {
        run(r#"
            const baseKey = await crypto.subtle.importKey(
                "raw", new Uint8Array(8), "PBKDF2", false, ["deriveBits"]);

            let threw = false;
            try {
                await crypto.subtle.deriveBits(
                    { name: "PBKDF2", hash: "SHA-256", salt: new Uint8Array(8), iterations: 1000 }, baseKey, 4);
            } catch (err) {
                threw = true;
                if (err.name !== "OperationError") throw new Error(`wrong error name: ${err.name}`);
            }
            if (!threw) throw new Error("deriveBits() did not reject a non-byte-aligned length");
        "#);
    }
}
