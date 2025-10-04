use base64::prelude::*;
use rquickjs::Ctx;
use rquickjs_util::{throw, StringRef};

#[rquickjs::function]
pub fn atob(input: StringRef<'_>) -> rquickjs::Result<String> {
    Ok(BASE64_STANDARD.encode(input.as_str()))
}

#[rquickjs::function]
pub fn btoa<'js>(ctx: Ctx<'js>, input: StringRef<'js>) -> rquickjs::Result<String> {
    match BASE64_STANDARD.decode(input.as_str()) {
        Ok(ret) => match String::from_utf8(ret) {
            Ok(ret) => Ok(ret),
            Err(err) => throw!(ctx, err),
        },
        Err(err) => {
            throw!(ctx, err)
        }
    }
}
