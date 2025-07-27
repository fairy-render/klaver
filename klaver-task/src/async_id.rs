use core::fmt;
use klaver_util::rquickjs::{
    self, Ctx, FromJs, IntoJs, JsLifetime, Value,
    class::{Trace, Tracer},
};
use std::{
    fmt::Debug,
    hash::Hash,
    sync::{
        LazyLock,
        atomic::{AtomicU32, Ordering},
    },
};

fn next_id() -> u32 {
    static COUNTER: AtomicU32 = AtomicU32::new(1);
    COUNTER.fetch_add(1, Ordering::Relaxed)
}

enum Inner {
    Value(u32),
    Lazy(LazyLock<u32>),
}

pub struct ResourceId(Inner);

impl ResourceId {
    pub const fn new() -> ResourceId {
        ResourceId(Inner::Lazy(LazyLock::new(|| next_id())))
    }

    pub fn inner(&self) -> u32 {
        match &self.0 {
            Inner::Lazy(m) => **m,
            Inner::Value(m) => *m,
        }
    }
}

impl fmt::Display for ResourceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Resource({})", self.inner())
    }
}

impl Debug for ResourceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Resource").field(&self.inner()).finish()
    }
}

impl PartialEq for ResourceId {
    fn eq(&self, other: &Self) -> bool {
        self.inner() == other.inner()
    }
}

impl Eq for ResourceId {}

impl PartialOrd for ResourceId {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.inner().partial_cmp(&other.inner())
    }
}

impl Ord for ResourceId {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.inner().cmp(&other.inner())
    }
}

impl Hash for ResourceId {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.inner().hash(state);
    }
}

impl Clone for ResourceId {
    fn clone(&self) -> Self {
        match &self.0 {
            Inner::Lazy(m) => Self(Inner::Value(**m)),
            Inner::Value(m) => Self(Inner::Value(*m)),
        }
    }
}

unsafe impl<'js> JsLifetime<'js> for ResourceId {
    type Changed<'to> = ResourceId;
}

impl<'js> Trace<'js> for ResourceId {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

impl<'js> FromJs<'js> for ResourceId {
    fn from_js(ctx: &Ctx<'js>, value: Value<'js>) -> rquickjs::Result<Self> {
        let id = u32::from_js(ctx, value)?;
        Ok(ResourceId(Inner::Value(id)))
    }
}

impl<'js> IntoJs<'js> for ResourceId {
    fn into_js(self, ctx: &Ctx<'js>) -> rquickjs::Result<Value<'js>> {
        Ok(Value::new_int(ctx.clone(), self.inner() as i32))
    }
}
