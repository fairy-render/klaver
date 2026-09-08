use base64::prelude::*;
use rquickjs::{Ctx, Object};

/// Encodes `bytes` as base64url with no padding, per RFC 7515 §2 ("base64url encoding").
pub fn b64url_encode(bytes: &[u8]) -> std::string::String {
    BASE64_URL_SAFE_NO_PAD.encode(bytes)
}

/// Decodes a base64url (no padding required, but tolerated) string per RFC 7515 §2.
pub fn b64url_decode<'js>(ctx: &Ctx<'js>, s: &str) -> rquickjs::Result<Vec<u8>> {
    // Some producers still pad JWK base64url fields; both are accepted.
    let s = s.trim_end_matches('=');
    match BASE64_URL_SAFE_NO_PAD.decode(s) {
        Ok(bytes) => Ok(bytes),
        Err(_) => throw_dom!(ctx, "DataError", "invalid base64url in JSON Web Key"),
    }
}

/// Builds a symmetric (`kty: "oct"`) JSON Web Key per RFC 7518 §6.4, tagging it with `alg`
/// (e.g. `"A128GCM"`, `"HS256"`) when one applies.
pub fn oct_to_jwk<'js>(
    ctx: &Ctx<'js>,
    key_bytes: &[u8],
    alg_tag: Option<&str>,
    extractable: bool,
) -> rquickjs::Result<Object<'js>> {
    let obj = Object::new(ctx.clone())?;
    obj.set("kty", "oct")?;
    obj.set("k", b64url_encode(key_bytes))?;
    if let Some(alg_tag) = alg_tag {
        obj.set("alg", alg_tag)?;
    }
    obj.set("ext", extractable)?;
    Ok(obj)
}

/// Reads the raw key bytes back out of a symmetric (`kty: "oct"`) JSON Web Key.
pub fn oct_from_jwk<'js>(ctx: &Ctx<'js>, obj: &Object<'js>) -> rquickjs::Result<Vec<u8>> {
    let kty: std::string::String = obj
        .get("kty")
        .unwrap_or_else(|_| std::string::String::new());
    if kty != "oct" {
        throw_dom!(
            ctx,
            "DataError",
            format!("expected a JWK with kty \"oct\", got {kty:?}")
        );
    }

    let k: std::string::String = match obj.get("k") {
        Ok(k) => k,
        Err(_) => throw_dom!(ctx, "DataError", "JSON Web Key is missing its \"k\" field"),
    };

    b64url_decode(ctx, &k)
}

#[cfg(feature = "crypto-asymmetric")]
fn require_str_field<'js>(
    ctx: &Ctx<'js>,
    obj: &Object<'js>,
    field: &str,
) -> rquickjs::Result<std::string::String> {
    match obj.get(field) {
        Ok(v) => Ok(v),
        Err(_) => throw_dom!(
            ctx,
            "DataError",
            format!("JSON Web Key is missing its \"{field}\" field")
        ),
    }
}

/// `n`/`e`, plus `d`/`p`/`q` for a private key, off an RSA (`kty: "RSA"`) JSON Web Key, as raw
/// big-endian bytes.
#[cfg(feature = "crypto-asymmetric")]
pub struct RsaJwkFields {
    pub n: Vec<u8>,
    pub e: Vec<u8>,
    pub private: Option<RsaJwkPrivateFields>,
}

#[cfg(feature = "crypto-asymmetric")]
pub struct RsaJwkPrivateFields {
    pub d: Vec<u8>,
    pub p: Vec<u8>,
    pub q: Vec<u8>,
}

#[cfg(feature = "crypto-asymmetric")]
pub fn rsa_from_jwk<'js>(ctx: &Ctx<'js>, obj: &Object<'js>) -> rquickjs::Result<RsaJwkFields> {
    let kty: std::string::String = obj
        .get("kty")
        .unwrap_or_else(|_| std::string::String::new());
    if kty != "RSA" {
        throw_dom!(
            ctx,
            "DataError",
            format!("expected a JWK with kty \"RSA\", got {kty:?}")
        );
    }

    let n = b64url_decode(ctx, &require_str_field(ctx, obj, "n")?)?;
    let e = b64url_decode(ctx, &require_str_field(ctx, obj, "e")?)?;
    let d: Option<std::string::String> = obj.get("d").ok();
    let Some(d) = d else {
        return Ok(RsaJwkFields {
            n,
            e,
            private: None,
        });
    };
    let d = b64url_decode(ctx, &d)?;
    let p = b64url_decode(ctx, &require_str_field(ctx, obj, "p")?)?;
    let q = b64url_decode(ctx, &require_str_field(ctx, obj, "q")?)?;
    Ok(RsaJwkFields {
        n,
        e,
        private: Some(RsaJwkPrivateFields { d, p, q }),
    })
}

/// Builds an RSA (`kty: "RSA"`) JSON Web Key per RFC 7518 §6.3, tagging it with `alg` when one
/// applies (see `KeyAlgorithm::jwk_alg_tag`).
#[cfg(feature = "crypto-asymmetric")]
pub fn rsa_to_jwk<'js>(
    ctx: &Ctx<'js>,
    components: &super::rsa::RsaComponents,
    alg_tag: Option<&str>,
    extractable: bool,
) -> rquickjs::Result<Object<'js>> {
    let obj = Object::new(ctx.clone())?;
    obj.set("kty", "RSA")?;
    obj.set("n", b64url_encode(&components.n))?;
    obj.set("e", b64url_encode(&components.e))?;
    if let Some(d) = &components.d {
        obj.set("d", b64url_encode(d))?;
    }
    if let Some(p) = &components.p {
        obj.set("p", b64url_encode(p))?;
    }
    if let Some(q) = &components.q {
        obj.set("q", b64url_encode(q))?;
    }
    if let Some(dp) = &components.dp {
        obj.set("dp", b64url_encode(dp))?;
    }
    if let Some(dq) = &components.dq {
        obj.set("dq", b64url_encode(dq))?;
    }
    if let Some(qi) = &components.qi {
        obj.set("qi", b64url_encode(qi))?;
    }
    if let Some(alg) = alg_tag {
        obj.set("alg", alg)?;
    }
    obj.set("ext", extractable)?;
    Ok(obj)
}

/// `crv`/`x`/`y`, plus `d` for a private key, off an EC (`kty: "EC"`) JSON Web Key, as raw bytes
/// (`crv` stays a plain string - `super::key::EcCurve::from_name` validates/parses it).
#[cfg(feature = "crypto-asymmetric")]
pub struct EcJwkFields {
    pub crv: std::string::String,
    pub x: Vec<u8>,
    pub y: Vec<u8>,
    pub d: Option<Vec<u8>>,
}

#[cfg(feature = "crypto-asymmetric")]
pub fn ec_from_jwk<'js>(ctx: &Ctx<'js>, obj: &Object<'js>) -> rquickjs::Result<EcJwkFields> {
    let kty: std::string::String = obj
        .get("kty")
        .unwrap_or_else(|_| std::string::String::new());
    if kty != "EC" {
        throw_dom!(
            ctx,
            "DataError",
            format!("expected a JWK with kty \"EC\", got {kty:?}")
        );
    }

    let crv = require_str_field(ctx, obj, "crv")?;
    let x = b64url_decode(ctx, &require_str_field(ctx, obj, "x")?)?;
    let y = b64url_decode(ctx, &require_str_field(ctx, obj, "y")?)?;
    let d: Option<std::string::String> = obj.get("d").ok();
    let d = d.map(|d| b64url_decode(ctx, &d)).transpose()?;
    Ok(EcJwkFields { crv, x, y, d })
}

/// Builds an EC (`kty: "EC"`) JSON Web Key per RFC 7518 §6.2. Unlike RSA/oct, EC JWKs never carry
/// an `alg` field here - see `KeyAlgorithm::jwk_alg_tag`'s doc comment for why.
#[cfg(feature = "crypto-asymmetric")]
pub fn ec_to_jwk<'js>(
    ctx: &Ctx<'js>,
    components: &super::ec::EcComponents,
    extractable: bool,
) -> rquickjs::Result<Object<'js>> {
    let obj = Object::new(ctx.clone())?;
    obj.set("kty", "EC")?;
    obj.set("crv", components.curve.jwk_crv())?;
    obj.set("x", b64url_encode(&components.x))?;
    obj.set("y", b64url_encode(&components.y))?;
    if let Some(d) = &components.d {
        obj.set("d", b64url_encode(d))?;
    }
    obj.set("ext", extractable)?;
    Ok(obj)
}
