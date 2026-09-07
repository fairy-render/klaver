use std::cell::RefCell;

use http::HeaderMap;
use klaver_core::Exportable;
use klaver_core::value::{
    Pair, StringExt, TypedMultiMap,
    iterable::{IterableProtocol, JsIterator, JsNativeIterator, NativeIteratorExt, NativeIteratorInterface},
};
use rquickjs::{
    Class, Coerced, Ctx, FromJs, Function, JsLifetime, String, Value,
    class::{JsClass, Trace},
    function::Opt,
};

/// Builds an independent `Headers` from a `HeadersInit` (`Headers | sequence<sequence<ByteString>>
/// | record<ByteString, ByteString>`, per <https://fetch.spec.whatwg.org/#typedefdef-headersinit>).
/// Always constructs a *fresh* header list, even when the source is itself a `Headers` instance -
/// otherwise `new Request(url, { headers: existingHeaders })` would alias `existingHeaders`, and
/// mutating one after the fact would incorrectly mutate the other.
#[derive(Trace)]
pub struct HeadersInit<'js> {
    pub inner: Class<'js, Headers<'js>>,
}

impl<'js> FromJs<'js> for HeadersInit<'js> {
    fn from_js(ctx: &Ctx<'js>, value: Value<'js>) -> rquickjs::Result<Self> {
        let inner = TypedMultiMap::new(ctx.clone())?;

        if let Ok(headers) = Class::<'js, Headers<'js>>::from_js(ctx, value.clone()) {
            for pair in headers.borrow().inner.entries()?.into_iter(ctx) {
                let Pair(k, v) = pair?;
                inner.append(ctx, k, v)?;
            }
        } else if let Ok(iter) = JsIterator::from_js(ctx, value.clone()) {
            // A `sequence<sequence<ByteString>>` - e.g. `[["content-type", "text/plain"]]`.
            for pair in iter
                .from_javascript::<Pair<String<'js>, String<'js>>>()
                .into_iter(ctx)
            {
                let Pair(k, v) = pair?;
                inner.append(ctx, k.to_lowercase(ctx.clone())?, v)?;
            }
        } else if let Some(obj) = value.as_object() {
            // A plain `record<ByteString, ByteString>` init object.
            for k in obj.keys::<String<'js>>() {
                let k = k?;
                let v: String<'js> = obj.get(k.clone())?;
                inner.append(ctx, k.to_lowercase(ctx.clone())?, v)?;
            }
        } else {
            return Err(rquickjs::Error::new_from_js(
                value.type_name(),
                "Headers, iterable of pairs, or record",
            ));
        }

        Ok(HeadersInit {
            inner: Class::instance(ctx.clone(), Headers { inner })?,
        })
    }
}

#[derive(Trace)]
#[rquickjs::class]
pub struct Headers<'js> {
    pub inner: TypedMultiMap<'js, String<'js>, String<'js>>,
}

unsafe impl<'js> JsLifetime<'js> for Headers<'js> {
    type Changed<'to> = Headers<'to>;
}

impl<'js> Headers<'js> {
    pub fn new_native(ctx: Ctx<'js>) -> rquickjs::Result<Headers<'js>> {
        Ok(Headers {
            inner: TypedMultiMap::new(ctx)?,
        })
    }

    pub fn from_native(
        ctx: &Ctx<'js>,
        headers: HeaderMap,
    ) -> rquickjs::Result<Class<'js, Headers<'js>>> {
        let inner = TypedMultiMap::new(ctx.clone())?;

        for (k, v) in headers {
            let Some(k) = k else { continue };
            let Ok(v) = v.to_str() else { continue };

            let k = String::from_str(ctx.clone(), &k.as_str().to_lowercase())?;
            let v = String::from_str(ctx.clone(), v)?;

            inner.append(ctx, k, v)?;
        }

        Class::instance(ctx.clone(), Headers { inner })
    }

    /// Per <https://fetch.spec.whatwg.org/#concept-header-list-get>: combines every value stored
    /// under `key` (already-lowercased) into one comma-and-space-joined string.
    fn combined_value(&self, ctx: &Ctx<'js>, key: String<'js>) -> rquickjs::Result<Option<String<'js>>> {
        let Some(list) = self.inner.get_all(key)? else {
            return Ok(None);
        };

        let mut combined = std::string::String::new();
        for (idx, value) in list.values()?.into_iter(ctx).enumerate() {
            let value = value?;
            if idx > 0 {
                combined.push_str(", ");
            }
            combined.push_str(&value.to_string()?);
        }

        Ok(Some(String::from_str(ctx.clone(), &combined)?))
    }

    /// Per <https://fetch.spec.whatwg.org/#concept-header-list-sort-and-combine>: the header
    /// names (each appearing once, already-lowercased) sorted lexicographically, each paired with
    /// its [`Headers::combined_value`]. Backs `entries()`/`keys()`/`values()`/`forEach()`/
    /// `for...of`.
    fn sorted_combined_entries(&self, ctx: &Ctx<'js>) -> rquickjs::Result<Vec<(String<'js>, String<'js>)>> {
        let mut names = self
            .inner
            .keys()?
            .into_iter(ctx)
            .map(|k| {
                let k = k?;
                let s = k.to_string()?;
                rquickjs::Result::Ok((s, k))
            })
            .collect::<rquickjs::Result<Vec<_>>>()?;

        names.sort_by(|a, b| a.0.cmp(&b.0));

        names
            .into_iter()
            .map(|(_, key)| {
                let value = self
                    .combined_value(ctx, key.clone())?
                    .expect("key came from this map's own keys()");
                Ok((key, value))
            })
            .collect()
    }
}

/// Snapshotting iterator over a `Headers`' sorted-and-combined entries, used for `entries()`,
/// `keys()`, `values()`, and the `for...of` protocol.
pub struct EntriesIter<'js> {
    items: Vec<(String<'js>, String<'js>)>,
    pos: RefCell<usize>,
}

impl<'js> Trace<'js> for EntriesIter<'js> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        for (k, v) in &self.items {
            k.trace(tracer);
            v.trace(tracer);
        }
    }
}

impl<'js> NativeIteratorInterface<'js> for EntriesIter<'js> {
    type Item = Pair<String<'js>, String<'js>>;

    fn next(&self, _ctx: &Ctx<'js>) -> rquickjs::Result<Option<Self::Item>> {
        let mut pos = self.pos.borrow_mut();
        let Some((k, v)) = self.items.get(*pos) else {
            return Ok(None);
        };
        let pair = Pair(k.clone(), v.clone());
        *pos += 1;
        Ok(Some(pair))
    }

    fn returns(&self, _ctx: &Ctx<'js>) -> rquickjs::Result<()> {
        Ok(())
    }
}

/// Like [`EntriesIter`] but yields just one side of each pair, for `keys()`/`values()`.
struct ProjectedIter<'js> {
    items: Vec<String<'js>>,
    pos: RefCell<usize>,
}

impl<'js> Trace<'js> for ProjectedIter<'js> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        for item in &self.items {
            item.trace(tracer);
        }
    }
}

impl<'js> NativeIteratorInterface<'js> for ProjectedIter<'js> {
    type Item = String<'js>;

    fn next(&self, _ctx: &Ctx<'js>) -> rquickjs::Result<Option<Self::Item>> {
        let mut pos = self.pos.borrow_mut();
        let Some(item) = self.items.get(*pos) else {
            return Ok(None);
        };
        let item = item.clone();
        *pos += 1;
        Ok(Some(item))
    }

    fn returns(&self, _ctx: &Ctx<'js>) -> rquickjs::Result<()> {
        Ok(())
    }
}

#[rquickjs::methods]
impl<'js> Headers<'js> {
    #[qjs(constructor)]
    pub fn new(ctx: Ctx<'js>, init: Opt<HeadersInit<'js>>) -> rquickjs::Result<Self> {
        let inner = match init.0 {
            Some(init) => init.inner.borrow().inner.clone(),
            None => TypedMultiMap::new(ctx)?,
        };
        Ok(Headers { inner })
    }

    pub fn append(
        &mut self,
        ctx: Ctx<'js>,
        key: String<'js>,
        Coerced(value): Coerced<String<'js>>,
    ) -> rquickjs::Result<()> {
        self.inner
            .append(&ctx, key.to_lowercase(ctx.clone())?, value)
    }

    pub fn set(
        &mut self,
        ctx: Ctx<'js>,
        key: String<'js>,
        Coerced(value): Coerced<String<'js>>,
    ) -> rquickjs::Result<()> {
        self.inner.set(&ctx, key.to_lowercase(ctx.clone())?, value)
    }

    pub fn get(
        &self,
        ctx: Ctx<'js>,
        key: String<'js>,
    ) -> rquickjs::Result<Option<rquickjs::String<'js>>> {
        self.combined_value(&ctx, key.to_lowercase(ctx.clone())?)
    }

    pub fn has(&self, ctx: Ctx<'js>, key: String<'js>) -> rquickjs::Result<bool> {
        self.inner.has(key.to_lowercase(ctx)?)
    }

    pub fn delete(&mut self, ctx: Ctx<'js>, key: String<'js>) -> rquickjs::Result<()> {
        self.inner.delete(key.to_lowercase(ctx)?)
    }

    pub fn entries(&self, ctx: Ctx<'js>) -> rquickjs::Result<JsNativeIterator<'js>> {
        Ok(JsNativeIterator::new(EntriesIter {
            items: self.sorted_combined_entries(&ctx)?,
            pos: RefCell::new(0),
        }))
    }

    pub fn keys(&self, ctx: Ctx<'js>) -> rquickjs::Result<JsNativeIterator<'js>> {
        let items = self
            .sorted_combined_entries(&ctx)?
            .into_iter()
            .map(|(k, _)| k)
            .collect();
        Ok(JsNativeIterator::new(ProjectedIter {
            items,
            pos: RefCell::new(0),
        }))
    }

    pub fn values(&self, ctx: Ctx<'js>) -> rquickjs::Result<JsNativeIterator<'js>> {
        let items = self
            .sorted_combined_entries(&ctx)?
            .into_iter()
            .map(|(_, v)| v)
            .collect();
        Ok(JsNativeIterator::new(ProjectedIter {
            items,
            pos: RefCell::new(0),
        }))
    }

    #[qjs(rename = "forEach")]
    pub fn for_each(&self, ctx: Ctx<'js>, func: Function<'js>) -> rquickjs::Result<()> {
        for (k, v) in self.sorted_combined_entries(&ctx)? {
            // Per spec, the callback is invoked as `callback(value, key)`.
            func.call::<_, ()>((v, k))?;
        }
        Ok(())
    }
}

impl<'js> IterableProtocol<'js> for Headers<'js> {
    type Iterator = EntriesIter<'js>;

    fn create_iterator(&self, ctx: &Ctx<'js>) -> rquickjs::Result<Self::Iterator> {
        Ok(EntriesIter {
            items: self.sorted_combined_entries(ctx)?,
            pos: RefCell::new(0),
        })
    }
}

// create_export!(Headers<'js>);

impl<'js> Exportable<'js> for Headers<'js> {
    fn export<T>(
        ctx: &Ctx<'js>,
        _registry: &klaver_core::Registry,
        target: &T,
    ) -> rquickjs::Result<()>
    where
        T: klaver_core::ExportTarget<'js>,
    {
        target.set(ctx, Self::NAME, Class::<Self>::create_constructor(ctx)?)?;
        Self::add_iterable_prototype(ctx)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rquickjs::{CatchResultExt, Context, Function, Runtime};

    /// Runs `body` as the contents of a plain function, with a global `Headers` constructor
    /// available. `body` is expected to throw on failure (e.g. via a plain `if (...) throw ...`).
    fn run(body: &str) {
        let runtime = Runtime::new().unwrap();
        let context = Context::full(&runtime).unwrap();

        context
            .with(|ctx| {
                // `Headers` (via `TypedMultiMap`) goes through `BasePrimordials`, which needs
                // the `$runtime` Core global that `klaver_core::register` sets up - normally
                // done by the `Environ`/`Vm` builder, but this test drives a bare `Context`.
                klaver_core::register(&ctx)?;

                ctx.globals()
                    .set(Headers::NAME, Class::<Headers>::create_constructor(&ctx)?)?;
                Headers::add_iterable_prototype(&ctx)?;

                let test_fn: Function = ctx.eval(format!("(() => {{\n{body}\n}})"))?;

                if let Err(err) = test_fn.call::<_, ()>(()).catch(&ctx) {
                    panic!("{err}");
                }

                rquickjs::Result::Ok(())
            })
            .unwrap();
    }

    #[test]
    fn constructed_from_array_of_pairs() {
        run(r#"
            const headers = new Headers([["Content-Type", "text/plain"], ["X-Foo", "bar"]]);
            if (headers.get("content-type") !== "text/plain") {
                throw new Error(`content-type was ${headers.get("content-type")}`);
            }
            if (headers.get("x-foo") !== "bar") throw new Error(`x-foo was ${headers.get("x-foo")}`);
        "#);
    }

    #[test]
    fn constructed_from_record_object() {
        run(r#"
            const headers = new Headers({ "Content-Type": "text/plain" });
            if (headers.get("content-type") !== "text/plain") {
                throw new Error(`content-type was ${headers.get("content-type")}`);
            }
        "#);
    }

    #[test]
    fn constructed_from_another_headers_copies_independently() {
        run(r#"
            const a = new Headers({ "X-Foo": "1" });
            const b = new Headers(a);
            b.set("X-Foo", "2");
            if (a.get("x-foo") !== "1") throw new Error(`a was mutated: ${a.get("x-foo")}`);
            if (b.get("x-foo") !== "2") throw new Error(`b was ${b.get("x-foo")}`);
        "#);
    }

    #[test]
    fn header_names_are_case_insensitive() {
        run(r#"
            const headers = new Headers();
            headers.set("Content-Type", "text/plain");
            if (!headers.has("content-type")) throw new Error("expected has(content-type)");
            if (headers.get("CONTENT-TYPE") !== "text/plain") {
                throw new Error(`get was ${headers.get("CONTENT-TYPE")}`);
            }
        "#);
    }

    #[test]
    fn append_combines_values_with_comma_space_on_get() {
        run(r#"
            const headers = new Headers();
            headers.append("X-Foo", "a");
            headers.append("X-Foo", "b");
            if (headers.get("X-Foo") !== "a, b") throw new Error(`get was ${headers.get("X-Foo")}`);
        "#);
    }

    #[test]
    fn set_replaces_all_previously_appended_values() {
        run(r#"
            const headers = new Headers();
            headers.append("X-Foo", "a");
            headers.append("X-Foo", "b");
            headers.set("X-Foo", "c");
            if (headers.get("X-Foo") !== "c") throw new Error(`get was ${headers.get("X-Foo")}`);
        "#);
    }

    #[test]
    fn delete_removes_the_header() {
        run(r#"
            const headers = new Headers({ "X-Foo": "1" });
            if (!headers.has("x-foo")) throw new Error("expected has(x-foo)");
            headers.delete("X-Foo");
            if (headers.has("x-foo")) throw new Error("did not expect has(x-foo) after delete");
            if (headers.get("x-foo") !== null && headers.get("x-foo") !== undefined) {
                throw new Error(`get after delete was ${headers.get("x-foo")}`);
            }
        "#);
    }

    #[test]
    fn iteration_is_sorted_by_name_with_combined_values() {
        run(r#"
            const headers = new Headers();
            headers.append("b", "2");
            headers.append("a", "1");
            headers.append("a", "1b");

            const entries = [...headers].map(([k, v]) => `${k}=${v}`);
            if (entries.join(",") !== "a=1, 1b,b=2") throw new Error(`entries were ${entries}`);

            const keys = [...headers.keys()];
            if (keys.join(",") !== "a,b") throw new Error(`keys were ${keys}`);

            const values = [...headers.values()];
            if (values.join(",") !== "1, 1b,2") throw new Error(`values were ${values}`);
        "#);
    }

    #[test]
    fn for_each_calls_back_with_value_then_key_in_sorted_order() {
        run(r#"
            const headers = new Headers({ "b": "2", "a": "1" });
            const seen = [];
            headers.forEach((value, key) => seen.push(`${key}=${value}`));
            if (seen.join(",") !== "a=1,b=2") throw new Error(`seen was ${seen}`);
        "#);
    }
}
