use klaver_core::value::Buffer;
use klaver_core::{Exportable, Registry};
use rquickjs::{
    Ctx, Object,
    module::ModuleDef,
    prelude::{Async, Func},
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

#[cfg(feature = "crypto-cipher")]
async fn encrypt<'js>(
    ctx: Ctx<'js>,
    algorithm: CipherAlgorithm,
    key: Class<'js, CryptoKey>,
    data: Buffer<'js>,
) -> rquickjs::Result<ArrayBuffer<'js>> {
    let key_ref = key.borrow();
    require_usage(&ctx, &key_ref, KeyUsage::Encrypt)?;
    require_aes_variant(&ctx, &key_ref, algorithm.variant())?;
    let plaintext = algorithm::buffer_bytes(&ctx, data)?;
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
            &plaintext,
        ),
        CipherAlgorithm::AesCbc { iv } => aes::cbc_encrypt(key_bytes, &iv, &plaintext),
        CipherAlgorithm::AesCtr {
            counter,
            length_bits,
        } => {
            let Ok(counter): Result<[u8; 16], _> = counter.try_into() else {
                return Err(cipher_error(&ctx, aes::CipherError::WrongCounterLength));
            };
            aes::ctr_encrypt_decrypt(key_bytes, &counter, length_bits, &plaintext)
        }
    };

    let ciphertext = result.map_err(|err| cipher_error(&ctx, err))?;
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
    require_aes_variant(&ctx, &key_ref, algorithm.variant())?;
    let ciphertext = algorithm::buffer_bytes(&ctx, data)?;
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
            &ciphertext,
        ),
        CipherAlgorithm::AesCbc { iv } => aes::cbc_decrypt(key_bytes, &iv, &ciphertext),
        CipherAlgorithm::AesCtr {
            counter,
            length_bits,
        } => {
            let Ok(counter): Result<[u8; 16], _> = counter.try_into() else {
                return Err(cipher_error(&ctx, aes::CipherError::WrongCounterLength));
            };
            aes::ctr_encrypt_decrypt(key_bytes, &counter, length_bits, &ciphertext)
        }
    };

    let plaintext = result.map_err(|err| cipher_error(&ctx, err))?;
    ArrayBuffer::new(ctx, plaintext)
}

#[cfg(feature = "crypto-cipher")]
async fn sign<'js>(
    ctx: Ctx<'js>,
    _algorithm: SignAlgorithm,
    key: Class<'js, CryptoKey>,
    data: Buffer<'js>,
) -> rquickjs::Result<ArrayBuffer<'js>> {
    let key_ref = key.borrow();
    require_usage(&ctx, &key_ref, KeyUsage::Sign)?;
    let hash = require_hmac(&ctx, &key_ref)?;
    let data = algorithm::buffer_bytes(&ctx, data)?;
    let signature = hmac_ops::sign(hash, key_ref.key_bytes(), &data);
    ArrayBuffer::new(ctx, signature)
}

#[cfg(feature = "crypto-cipher")]
async fn verify<'js>(
    ctx: Ctx<'js>,
    _algorithm: SignAlgorithm,
    key: Class<'js, CryptoKey>,
    signature: Buffer<'js>,
    data: Buffer<'js>,
) -> rquickjs::Result<bool> {
    let key_ref = key.borrow();
    require_usage(&ctx, &key_ref, KeyUsage::Verify)?;
    let hash = require_hmac(&ctx, &key_ref)?;
    let signature = algorithm::buffer_bytes(&ctx, signature)?;
    let data = algorithm::buffer_bytes(&ctx, data)?;
    Ok(hmac_ops::verify(
        hash,
        key_ref.key_bytes(),
        &data,
        &signature,
    ))
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
