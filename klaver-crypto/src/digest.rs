use klaver_base::create_export;
use klaver_util::{Buffer, throw};
use rquickjs::{ArrayBuffer, Ctx, FromJs, IntoJs, class::Trace};
use sha1::{Sha1, digest::Digest as _};
use sha2::{Sha256, Sha384, Sha512};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Trace)]
pub enum Algo {
    Sha1,
    Sha256,
    Sha384,
    Sha512,
}

impl Algo {
    fn to_impl(self) -> DigestImpl {
        match self {
            Algo::Sha1 => DigestImpl::Sha1(Sha1::new()),
            Algo::Sha256 => DigestImpl::Sha2(Sha256::new()),
            Algo::Sha384 => DigestImpl::Sha384(Sha384::new()),
            Algo::Sha512 => DigestImpl::Sha512(Sha512::new()),
        }
    }
}

impl<'js> FromJs<'js> for Algo {
    fn from_js(
        ctx: &rquickjs::prelude::Ctx<'js>,
        value: rquickjs::Value<'js>,
    ) -> rquickjs::Result<Self> {
        let str = String::from_js(ctx, value)?;

        let algo = match &*str {
            "sha1" | "SHA-1" => Algo::Sha1,
            "sha256" | "sha2" | "SHA-256" => Algo::Sha256,
            "sha384" | "SHA-384" => Algo::Sha384,
            "sha512" | "SHA-512" => Algo::Sha512,
            _ => return Err(rquickjs::Error::new_from_js("string", "algo")),
        };

        Ok(algo)
    }
}

impl<'js> IntoJs<'js> for Algo {
    fn into_js(self, ctx: &rquickjs::prelude::Ctx<'js>) -> rquickjs::Result<rquickjs::Value<'js>> {
        match self {
            Self::Sha1 => "sha1".into_js(ctx),
            Self::Sha256 => "sha256".into_js(ctx),
            Self::Sha384 => "sha384".into_js(ctx),
            Self::Sha512 => "sha512".into_js(ctx),
        }
    }
}

enum DigestImpl {
    Sha1(Sha1),
    Sha2(Sha256),
    Sha384(Sha384),
    Sha512(Sha512),
}

impl DigestImpl {
    fn update(&mut self, data: &[u8]) {
        match self {
            Self::Sha1(b) => b.update(data),
            Self::Sha2(b) => b.update(data),
            Self::Sha384(b) => b.update(data),
            Self::Sha512(b) => b.update(data),
        }
    }

    fn digest(&self) -> Vec<u8> {
        match self {
            Self::Sha1(b) => b.clone().finalize().to_vec(),
            Self::Sha2(b) => b.clone().finalize().to_vec(),
            Self::Sha384(b) => b.clone().finalize().to_vec(),
            Self::Sha512(b) => b.clone().finalize().to_vec(),
        }
    }
}

#[derive(rquickjs::JsLifetime)]
#[rquickjs::class]
pub struct Digest {
    inner: DigestImpl,
}

#[rquickjs::methods]
impl Digest {
    #[qjs(constructor)]
    pub fn new(algo: Algo) -> rquickjs::Result<Digest> {
        Ok(Digest {
            inner: algo.to_impl(),
        })
    }

    pub fn update<'js>(&mut self, ctx: Ctx<'js>, buffer: Buffer<'js>) -> rquickjs::Result<()> {
        let Some(buffer) = buffer.as_raw() else {
            throw!(ctx, "buffer is detached")
        };

        self.inner.update(buffer.slice());

        Ok(())
    }

    pub fn digest<'js>(&self, ctx: Ctx<'js>) -> rquickjs::Result<ArrayBuffer<'js>> {
        ArrayBuffer::new(ctx, self.inner.digest())
    }
}

impl<'js> Trace<'js> for Digest {
    fn trace<'a>(&self, _tracer: rquickjs::class::Tracer<'a, 'js>) {}
}

create_export!(Digest);
