use core::fmt;
use std::{
    fmt::Debug,
    hash::Hash,
    sync::{
        LazyLock,
        atomic::{AtomicU32, Ordering},
    },
};

use rquickjs::{FromJs, IntoJs, JsLifetime, Value, class::JsClass, class::Trace};

fn next_id() -> u32 {
    static COUNTER: AtomicU32 = AtomicU32::new(1);
    COUNTER.fetch_add(1, Ordering::Relaxed)
}

enum Inner {
    Value(u32),
    Lazy(LazyLock<u32>),
}

pub struct Tag(Inner);

impl Tag {
    pub const fn new() -> Tag {
        Tag(Inner::Lazy(LazyLock::new(|| next_id())))
    }

    pub fn inner(&self) -> u32 {
        match &self.0 {
            Inner::Lazy(m) => **m,
            Inner::Value(m) => *m,
        }
    }
}

impl fmt::Display for Tag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Tag({})", self.inner())
    }
}

impl Debug for Tag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Tag").field(&self.inner()).finish()
    }
}

impl PartialEq for Tag {
    fn eq(&self, other: &Self) -> bool {
        self.inner() == other.inner()
    }
}

impl Eq for Tag {}

impl PartialOrd for Tag {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.inner().partial_cmp(&other.inner())
    }
}

impl Ord for Tag {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.inner().cmp(&other.inner())
    }
}

impl Hash for Tag {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.inner().hash(state);
    }
}

impl Clone for Tag {
    fn clone(&self) -> Self {
        match &self.0 {
            Inner::Lazy(m) => Self(Inner::Value(**m)),
            Inner::Value(m) => Self(Inner::Value(*m)),
        }
    }
}

unsafe impl<'js> JsLifetime<'js> for Tag {
    type Changed<'to> = Tag;
}

impl<'js> Trace<'js> for Tag {
    fn trace<'a>(&self, _tracer: rquickjs::class::Tracer<'a, 'js>) {}
}

impl<'js> FromJs<'js> for Tag {
    fn from_js(ctx: &rquickjs::Ctx<'js>, value: rquickjs::Value<'js>) -> rquickjs::Result<Self> {
        let id = u32::from_js(ctx, value)?;
        Ok(Tag(Inner::Value(id)))
    }
}

impl<'js> IntoJs<'js> for Tag {
    fn into_js(self, ctx: &rquickjs::Ctx<'js>) -> rquickjs::Result<rquickjs::Value<'js>> {
        Ok(Value::new_int(ctx.clone(), self.inner() as i32))
    }
}
