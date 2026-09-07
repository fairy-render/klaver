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
/// (e.g. `"A128GCM"`, `"HS256"`).
pub fn oct_to_jwk<'js>(
    ctx: &Ctx<'js>,
    key_bytes: &[u8],
    alg_tag: &str,
    extractable: bool,
) -> rquickjs::Result<Object<'js>> {
    let obj = Object::new(ctx.clone())?;
    obj.set("kty", "oct")?;
    obj.set("k", b64url_encode(key_bytes))?;
    obj.set("alg", alg_tag)?;
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
