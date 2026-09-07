use base64::prelude::*;
use klaver_core::{throw, value::StringRef};
use rquickjs::Ctx;

/// Decodes a base64-encoded ASCII string into a binary string (ASCII -> binary).
pub fn atob<'js>(ctx: Ctx<'js>, input: StringRef<'js>) -> rquickjs::Result<String> {
    // Per the forgiving-base64 algorithm, ASCII whitespace is stripped before decoding.
    let filtered: String = input
        .as_str()
        .chars()
        .filter(|c| !matches!(c, ' ' | '\t' | '\n' | '\r' | '\x0c'))
        .collect();

    match BASE64_STANDARD.decode(filtered) {
        // Every decoded byte becomes a single UTF-16 code unit (Latin1), it must not be
        // re-interpreted as UTF-8 since the decoded bytes are arbitrary binary data.
        Ok(bytes) => Ok(bytes.into_iter().map(|b| b as char).collect()),
        Err(err) => throw!(@type ctx, err),
    }
}

/// Encodes a binary string into a base64-encoded ASCII string (binary -> ASCII).
pub fn btoa<'js>(ctx: Ctx<'js>, input: StringRef<'js>) -> rquickjs::Result<String> {
    let mut bytes = Vec::with_capacity(input.len());

    for c in input.as_str().chars() {
        if c as u32 > 0xFF {
            throw!(@type ctx, "String contains characters outside of the Latin1 range")
        }
        bytes.push(c as u8);
    }

    Ok(BASE64_STANDARD.encode(bytes))
}
