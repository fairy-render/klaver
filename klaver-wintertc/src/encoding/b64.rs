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

#[cfg(test)]
mod tests {
    use super::*;
    use rquickjs::{CatchResultExt, Context, Function, Runtime, prelude::Func};

    /// Runs `body` as the contents of a plain function, with global `atob`/`btoa` available.
    /// `body` is expected to throw on failure (e.g. via a plain `if (...) throw ...`).
    fn run(body: &str) {
        let runtime = Runtime::new().unwrap();
        let context = Context::full(&runtime).unwrap();

        context
            .with(|ctx| {
                ctx.globals().set("atob", Func::new(atob))?;
                ctx.globals().set("btoa", Func::new(btoa))?;

                let test_fn: Function = ctx.eval(format!("(() => {{\n{body}\n}})"))?;

                if let Err(err) = test_fn.call::<_, ()>(()).catch(&ctx) {
                    panic!("{err}");
                }

                rquickjs::Result::Ok(())
            })
            .unwrap();
    }

    #[test]
    fn btoa_encodes_known_vector() {
        run(r#"
            if (btoa("Hello, World!") !== "SGVsbG8sIFdvcmxkIQ==") {
                throw new Error(`btoa was ${btoa("Hello, World!")}`);
            }
        "#);
    }

    #[test]
    fn atob_decodes_known_vector() {
        run(r#"
            if (atob("SGVsbG8sIFdvcmxkIQ==") !== "Hello, World!") {
                throw new Error(`atob was ${atob("SGVsbG8sIFdvcmxkIQ==")}`);
            }
        "#);
    }

    #[test]
    fn round_trips_through_btoa_and_atob() {
        run(r#"
            const original = "the quick brown fox";
            if (atob(btoa(original)) !== original) throw new Error("round trip failed");
        "#);
    }

    #[test]
    fn atob_strips_ascii_whitespace() {
        run(r#"
            if (atob(" SGVs bG8s\tIFdv\ncmxk IQ==\r\n") !== "Hello, World!") {
                throw new Error(`atob was ${atob(" SGVs bG8s\tIFdv\ncmxk IQ==\r\n")}`);
            }
        "#);
    }

    #[test]
    fn atob_rejects_invalid_base64() {
        run(r#"
            let threw = false;
            try {
                atob("not valid base64!!!");
            } catch {
                threw = true;
            }
            if (!threw) throw new Error("expected atob to throw");
        "#);
    }

    #[test]
    fn btoa_rejects_characters_outside_latin1() {
        run(r#"
            let threw = false;
            try {
                btoa("emoji \u{1F600}");
            } catch {
                threw = true;
            }
            if (!threw) throw new Error("expected btoa to throw");
        "#);
    }
}
