use std::{borrow::Borrow, ffi::c_char, fmt::Display, hash::Hash, mem, ptr::NonNull};

use rquickjs::{
    class::Trace, function::Args, qjs, Ctx, Error, FromJs, Function, IntoJs, Result, String, Value,
};

pub fn concat<'js>(
    ctx: Ctx<'js>,
    first: rquickjs::String<'js>,
    second: rquickjs::String<'js>,
) -> rquickjs::Result<rquickjs::String<'js>> {
    ctx.eval::<Function, _>("(a,b) => a + b")?
        .call((first, second))
}

pub fn concat_many<'js>(
    ctx: Ctx<'js>,
    args: &[rquickjs::String<'js>],
) -> rquickjs::Result<rquickjs::String<'js>> {
    let mut a = Args::new(ctx.clone(), args.len());

    a.push_args(args.iter().map(|m| m.clone()))?;

    ctx.eval::<Function, _>("(...a) => a.join('')")?.call_arg(a)
}

#[derive(Debug)]
pub struct StringRef<'js> {
    ptr: NonNull<c_char>,
    len: usize,
    value: String<'js>,
}

impl<'js> From<StringRef<'js>> for String<'js> {
    fn from(value: StringRef<'js>) -> Self {
        value.value.clone()
    }
}

impl<'js> StringRef<'js> {
    pub fn from_string(string: String<'js>) -> Result<Self> {
        let mut len = mem::MaybeUninit::uninit();
        // SAFETY: The pointer points to a JSString content which is ref counted
        let ptr = unsafe {
            qjs::JS_ToCStringLen(
                string.ctx().as_raw().as_ptr(),
                len.as_mut_ptr(),
                string.as_raw(),
            )
        };
        if ptr.is_null() {
            // Might not ever happen but I am not 100% sure
            // so just incase check it.
            return Err(Error::Unknown);
        }
        let len = unsafe { len.assume_init() };
        Ok(Self {
            // SAFETY: We already checked for null ptr above
            ptr: unsafe { NonNull::new_unchecked(ptr as *mut _) },
            len,
            value: string,
        })
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn as_string(&self) -> &String<'js> {
        &self.value
    }

    pub fn as_bytes(&self) -> &[u8] {
        // SAFETY: The pointer points to a JSString content which is ref counted
        let bytes =
            unsafe { core::slice::from_raw_parts(self.ptr.as_ptr() as *const u8, self.len) };
        bytes
    }

    pub fn as_str(&self) -> &str {
        // SAFETY: The pointer points to a JSString content which is ref counted
        let bytes =
            unsafe { core::slice::from_raw_parts(self.ptr.as_ptr() as *const u8, self.len) };
        // SAFETY: The bytes are garanteed to be valid utf8 by QuickJS
        unsafe { core::str::from_utf8_unchecked(bytes) }
    }

    pub fn try_clone(&self) -> Result<StringRef<'js>> {
        Self::from_string(self.value.clone())
    }
}

impl<'js> PartialEq for StringRef<'js> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl<'js> Eq for StringRef<'js> {}

impl<'js, 'a> PartialEq<&'a str> for StringRef<'js> {
    fn eq(&self, other: &&'a str) -> bool {
        self.as_str() == *other
    }
}

impl<'js> PartialEq<str> for StringRef<'js> {
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl<'js> PartialOrd for StringRef<'js> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.as_str().partial_cmp(other.as_str())
    }
}

impl<'js> Hash for StringRef<'js> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.as_str().hash(state);
    }
}

impl<'js> Trace<'js> for StringRef<'js> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        self.value.trace(tracer);
    }
}

impl<'js> Display for StringRef<'js> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_str().fmt(f)
    }
}

impl<'js> Drop for StringRef<'js> {
    fn drop(&mut self) {
        unsafe { qjs::JS_FreeCString(self.value.ctx().as_raw().as_ptr(), self.ptr.as_ptr()) };
    }
}

impl<'js> AsRef<str> for StringRef<'js> {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl<'js> Borrow<str> for StringRef<'js> {
    fn borrow(&self) -> &str {
        self.as_ref()
    }
}

impl<'js> FromJs<'js> for StringRef<'js> {
    fn from_js(_ctx: &Ctx<'js>, value: Value<'js>) -> Result<Self> {
        Self::from_string(String::from_value(value)?)
    }
}

impl<'js> IntoJs<'js> for StringRef<'js> {
    fn into_js(self, _ctx: &Ctx<'js>) -> Result<Value<'js>> {
        Ok(self.value.clone().into_value())
    }
}

impl<'js> IntoJs<'js> for &StringRef<'js> {
    fn into_js(self, _ctx: &Ctx<'js>) -> Result<Value<'js>> {
        Ok(self.value.clone().into_value())
    }
}

#[cfg(test)]
mod test {
    use rquickjs::{Context, Runtime};

    use super::*;
    pub(crate) fn test_with<F, R>(func: F) -> R
    where
        F: FnOnce(Ctx) -> R,
    {
        let rt = Runtime::new().unwrap();
        let ctx = Context::full(&rt).unwrap();
        ctx.with(func)
    }

    #[test]
    fn from_js_value() {
        test_with(|ctx| {
            let ret = ctx.eval::<StringRef, _>("'Hello, World!'").unwrap();

            assert_eq!(ret.as_str(), "Hello, World!")
        });
    }

    #[test]
    fn into_js_value() {
        test_with(|ctx| {
            let ret = rquickjs::String::from_str(ctx.clone(), "Hello, World").unwrap();

            let ret = StringRef::from_string(ret).unwrap();

            let func = ctx.eval::<Function, _>("(i) => i + '!'").unwrap();

            let ret = func.call::<_, StringRef>((ret,)).unwrap();

            assert_eq!(ret.as_str(), "Hello, World!")
        });
    }
}
