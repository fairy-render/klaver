use base64::prelude::*;
use klaver::throw;
use rquickjs::Ctx;

#[rquickjs::function]
pub fn atob(input: String) -> rquickjs::Result<String> {
    Ok(BASE64_STANDARD.encode(input))
}

#[rquickjs::function]
pub fn btoa<'js>(ctx: Ctx<'js>, input: String) -> rquickjs::Result<String> {
    match BASE64_STANDARD.decode(input) {
        Ok(ret) => match String::from_utf8(ret) {
            Ok(ret) => Ok(ret),
            Err(err) => throw!(ctx, err),
        },
        Err(err) => {
            throw!(ctx, err)
        }
    }
}
