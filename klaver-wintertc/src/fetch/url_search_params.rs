use klaver_core::Exportable;

use klaver_core::value::{
    Pair, StringRef,
    iterable::{IterableProtocol, JsIterator, JsNativeIterator, NativeIteratorExt, NativeIteratorInterface},
};
use rquickjs::{
    Class, Ctx, FromJs, Function, JsLifetime,
    atom::PredefinedAtom,
    class::{JsClass, Trace},
    prelude::Opt,
};
use std::{cell::RefCell, fmt::Write};

use super::url::Url;

type Entry<'js> = (rquickjs::String<'js>, rquickjs::String<'js>);

/// Parses a query string (with or without a leading `?`) into an ordered list of pairs, per
/// `application/x-www-form-urlencoded` parsing rules. Kept as a flat, insertion-ordered list
/// (rather than grouped by key) because the URLSearchParams spec requires iteration order to
/// match the exact original order of *all* pairs, including interleaved duplicate keys - e.g.
/// `a=1&b=2&a=3` must iterate as `a,b,a`, not grouped as `a,a,b`.
fn parse_query<'js>(ctx: Ctx<'js>, qs: &str) -> rquickjs::Result<Vec<Entry<'js>>> {
    let qs = qs.strip_prefix('?').unwrap_or(qs);
    let mut out = Vec::new();

    for (key, value) in form_urlencoded::parse(qs.as_bytes()) {
        let key = rquickjs::String::from_str(ctx.clone(), &key)?;
        let value = rquickjs::String::from_str(ctx.clone(), &value)?;
        out.push((key, value));
    }

    Ok(out)
}

pub struct URLSearchParamsInit<'js> {
    entries: Vec<Entry<'js>>,
}

impl<'js> URLSearchParamsInit<'js> {
    pub fn from_str(ctx: Ctx<'js>, qs: &str) -> rquickjs::Result<URLSearchParamsInit<'js>> {
        Ok(URLSearchParamsInit {
            entries: parse_query(ctx, qs)?,
        })
    }
}

impl<'js> FromJs<'js> for URLSearchParamsInit<'js> {
    fn from_js(ctx: &rquickjs::Ctx<'js>, value: rquickjs::Value<'js>) -> rquickjs::Result<Self> {
        let entries = if let Ok(qs) = StringRef::from_js(ctx, value.clone()) {
            // We got a query string - parse it
            parse_query(ctx.clone(), qs.as_str())?
        } else if let Ok(iter) = JsIterator::from_js(ctx, value.clone()) {
            // We got an iterable of key/value pairs (e.g. an array of 2-tuples, or a Map).
            iter.from_javascript::<Pair<rquickjs::String<'_>, rquickjs::String<'_>>>()
                .into_iter(ctx)
                .map(|pair| pair.map(|p| (p.0, p.1)))
                .collect::<rquickjs::Result<Vec<_>>>()?
        } else if let Some(obj) = value.as_object() {
            // A plain `record<USVString, USVString>` init object.
            let mut entries = Vec::new();

            for k in obj.keys::<rquickjs::String<'js>>() {
                let k = k?;
                let v: rquickjs::String = obj.get(k.clone())?;
                entries.push((k, v));
            }

            entries
        } else {
            return Err(rquickjs::Error::new_from_js(
                value.type_name(),
                "iterator, record, or string",
            ));
        };

        Ok(URLSearchParamsInit { entries })
    }
}

/// Snapshotting iterator over a `URLSearchParams`' entries, used for `entries()`, `keys()`,
/// `values()` and the `for...of` protocol.
pub struct EntriesIter<'js> {
    items: Vec<Entry<'js>>,
    pos: RefCell<usize>,
}

impl<'js> EntriesIter<'js> {
    fn new(items: Vec<Entry<'js>>) -> Self {
        EntriesIter {
            items,
            pos: RefCell::new(0),
        }
    }
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
    type Item = Pair<rquickjs::String<'js>, rquickjs::String<'js>>;

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
    items: Vec<rquickjs::String<'js>>,
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
    type Item = rquickjs::String<'js>;

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

#[rquickjs::class]
pub struct URLSearchParams<'js> {
    entries: RefCell<Vec<Entry<'js>>>,
    // Set only when this instance was obtained via `Url.prototype.searchParams`, so mutations
    // can be reflected back into the owning URL's `search`/`href`, keeping the two "live"
    // per https://url.spec.whatwg.org/#concept-urlsearchparams-update.
    owner: Option<Class<'js, Url<'js>>>,
}

impl<'js> Trace<'js> for URLSearchParams<'js> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        for (k, v) in self.entries.borrow().iter() {
            k.trace(tracer);
            v.trace(tracer);
        }
        self.owner.trace(tracer);
    }
}

unsafe impl<'js> JsLifetime<'js> for URLSearchParams<'js> {
    type Changed<'to> = URLSearchParams<'to>;
}

impl<'js> URLSearchParams<'js> {
    pub fn from_query(ctx: Ctx<'js>, query: Option<&str>) -> rquickjs::Result<URLSearchParams<'js>> {
        let entries = match query {
            Some(q) => parse_query(ctx, q)?,
            None => Vec::new(),
        };
        Ok(URLSearchParams {
            entries: RefCell::new(entries),
            owner: None,
        })
    }

    /// Replaces the whole contents from a fresh query string. Used by the owning `Url` when
    /// its `search`/`href` changes; does not notify the owner back (it *is* the owner calling
    /// this).
    pub fn reset_from_query(&mut self, ctx: Ctx<'js>, query: Option<&str>) -> rquickjs::Result<()> {
        self.entries = RefCell::new(match query {
            Some(q) => parse_query(ctx, q)?,
            None => Vec::new(),
        });
        Ok(())
    }

    pub fn set_owner(&mut self, owner: Class<'js, Url<'js>>) {
        self.owner = Some(owner);
    }

    fn notify_owner(&self) -> rquickjs::Result<()> {
        let Some(owner) = &self.owner else {
            return Ok(());
        };

        let query = self.serialize()?;
        owner.borrow_mut().set_query_from_search_params(&query);
        Ok(())
    }

    fn serialize(&self) -> rquickjs::Result<std::string::String> {
        let mut output = std::string::String::new();
        for (idx, (k, v)) in self.entries.borrow().iter().enumerate() {
            if idx > 0 {
                output.push('&');
            }
            let key = StringRef::from_string(k.clone())?;
            let value = StringRef::from_string(v.clone())?;
            write!(
                output,
                "{}={}",
                urlencoding::encode(key.as_str()),
                urlencoding::encode(value.as_str())
            )
            .expect("write to string");
        }

        Ok(output)
    }
}

#[rquickjs::methods]
impl<'js> URLSearchParams<'js> {
    #[qjs(constructor)]
    pub fn new(init: Opt<URLSearchParamsInit<'js>>) -> rquickjs::Result<URLSearchParams<'js>> {
        let entries = init.0.map(|init| init.entries).unwrap_or_default();
        Ok(URLSearchParams {
            entries: RefCell::new(entries),
            owner: None,
        })
    }

    pub fn get(
        &self,
        key: rquickjs::String<'js>,
    ) -> rquickjs::Result<Option<rquickjs::String<'js>>> {
        let key = key.to_string()?;
        for (k, v) in self.entries.borrow().iter() {
            if k.to_string()? == key {
                return Ok(Some(v.clone()));
            }
        }
        Ok(None)
    }

    pub fn has(&self, key: rquickjs::String<'js>) -> rquickjs::Result<bool> {
        let key = key.to_string()?;
        for (k, _) in self.entries.borrow().iter() {
            if k.to_string()? == key {
                return Ok(true);
            }
        }
        Ok(false)
    }

    #[qjs(rename = "getAll")]
    pub fn get_all(&self, key: rquickjs::String<'js>) -> rquickjs::Result<Vec<rquickjs::String<'js>>> {
        let key = key.to_string()?;
        let mut out = Vec::new();
        for (k, v) in self.entries.borrow().iter() {
            if k.to_string()? == key {
                out.push(v.clone());
            }
        }
        Ok(out)
    }

    pub fn set(
        &self,
        key: rquickjs::String<'js>,
        value: rquickjs::String<'js>,
    ) -> rquickjs::Result<()> {
        let key_str = key.to_string()?;
        let mut found = false;

        {
            let mut entries = self.entries.borrow_mut();
            let mut i = 0;
            while i < entries.len() {
                if entries[i].0.to_string()? == key_str {
                    if !found {
                        entries[i].1 = value.clone();
                        found = true;
                        i += 1;
                    } else {
                        entries.remove(i);
                    }
                } else {
                    i += 1;
                }
            }

            if !found {
                entries.push((key, value));
            }
        }

        self.notify_owner()
    }

    pub fn append(
        &self,
        key: rquickjs::String<'js>,
        value: rquickjs::String<'js>,
    ) -> rquickjs::Result<()> {
        self.entries.borrow_mut().push((key, value));
        self.notify_owner()
    }

    pub fn delete(&self, key: rquickjs::String<'js>) -> rquickjs::Result<()> {
        let key_str = key.to_string()?;

        {
            let mut entries = self.entries.borrow_mut();
            let mut i = 0;
            while i < entries.len() {
                if entries[i].0.to_string()? == key_str {
                    entries.remove(i);
                } else {
                    i += 1;
                }
            }
        }

        self.notify_owner()
    }

    #[qjs(get, enumerable, rename = "size")]
    pub fn size(&self) -> usize {
        self.entries.borrow().len()
    }

    pub fn entries(&self) -> JsNativeIterator<'js> {
        JsNativeIterator::new(EntriesIter::new(self.entries.borrow().clone()))
    }

    pub fn keys(&self) -> JsNativeIterator<'js> {
        let items = self.entries.borrow().iter().map(|(k, _)| k.clone()).collect();
        JsNativeIterator::new(ProjectedIter {
            items,
            pos: RefCell::new(0),
        })
    }

    pub fn values(&self) -> JsNativeIterator<'js> {
        let items = self.entries.borrow().iter().map(|(_, v)| v.clone()).collect();
        JsNativeIterator::new(ProjectedIter {
            items,
            pos: RefCell::new(0),
        })
    }

    #[qjs(rename = "forEach")]
    pub fn for_each(&self, func: Function<'js>) -> rquickjs::Result<()> {
        let items = self.entries.borrow().clone();

        for (k, v) in items {
            // Per spec, the callback is invoked as `callback(value, key)`.
            func.call::<_, ()>((v, k))?
        }

        Ok(())
    }

    #[qjs(rename = PredefinedAtom::ToString)]
    pub fn to_string(&self) -> rquickjs::Result<std::string::String> {
        self.serialize()
    }
}

impl<'js> IterableProtocol<'js> for URLSearchParams<'js> {
    type Iterator = EntriesIter<'js>;

    fn create_iterator(&self, _ctx: &Ctx<'js>) -> rquickjs::Result<Self::Iterator> {
        Ok(EntriesIter::new(self.entries.borrow().clone()))
    }
}

impl<'js> Exportable<'js> for URLSearchParams<'js> {
    fn export<T>(
        ctx: &Ctx<'js>,
        _registry: &klaver_core::Registry,
        target: &T,
    ) -> rquickjs::Result<()>
    where
        T: klaver_core::ExportTarget<'js>,
    {
        target.set(
            ctx,
            URLSearchParams::NAME,
            Class::<Self>::create_constructor(ctx)?,
        )?;

        Self::add_iterable_prototype(ctx)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rquickjs::{CatchResultExt, Context, Runtime};

    /// Runs `body` as the contents of a plain function, with global `URLSearchParams` and `URL`
    /// constructors available. `body` is expected to throw on failure (e.g. via a plain
    /// `if (...) throw ...`).
    fn run(body: &str) {
        let runtime = Runtime::new().unwrap();
        let context = Context::full(&runtime).unwrap();

        context
            .with(|ctx| {
                ctx.globals().set(
                    URLSearchParams::NAME,
                    Class::<URLSearchParams>::create_constructor(&ctx)?,
                )?;
                URLSearchParams::add_iterable_prototype(&ctx)?;
                ctx.globals()
                    .set("URL", Class::<Url>::create_constructor(&ctx)?)?;

                let test_fn: Function = ctx.eval(format!("(() => {{\n{body}\n}})"))?;

                if let Err(err) = test_fn.call::<_, ()>(()).catch(&ctx) {
                    panic!("{err}");
                }

                rquickjs::Result::Ok(())
            })
            .unwrap();
    }

    #[test]
    fn from_query_string() {
        run(r#"
            const params = new URLSearchParams("?a=1&b=2");
            if (params.size !== 2) throw new Error(`size was ${params.size}`);
            if (params.get("a") !== "1") throw new Error(`a was ${params.get("a")}`);
            if (params.get("b") !== "2") throw new Error(`b was ${params.get("b")}`);
            if (!params.has("a")) throw new Error("expected has(a)");
            if (params.has("c")) throw new Error("did not expect has(c)");
            if (params.get("missing") !== undefined) throw new Error("expected undefined for missing key");
        "#);
    }

    #[test]
    fn from_record_object() {
        run(r#"
            const params = new URLSearchParams({ a: "1", b: "2" });
            if (params.get("a") !== "1") throw new Error(`a was ${params.get("a")}`);
            if (params.get("b") !== "2") throw new Error(`b was ${params.get("b")}`);
        "#);
    }

    #[test]
    fn from_iterable_of_pairs() {
        run(r#"
            const params = new URLSearchParams([["a", "1"], ["b", "2"]]);
            if (params.get("a") !== "1") throw new Error(`a was ${params.get("a")}`);
            if (params.get("b") !== "2") throw new Error(`b was ${params.get("b")}`);
        "#);
    }

    #[test]
    fn get_all_returns_every_matching_value() {
        run(r#"
            const params = new URLSearchParams("a=1&a=2&b=3");
            const all = params.getAll("a");
            if (all.length !== 2 || all[0] !== "1" || all[1] !== "2") {
                throw new Error(`getAll(a) was ${all}`);
            }
            if (params.getAll("missing").length !== 0) throw new Error("expected empty array");
        "#);
    }

    #[test]
    fn set_collapses_duplicates_keeping_first_position() {
        run(r#"
            const params = new URLSearchParams("a=1&b=2&a=3");
            params.set("a", "new");
            if (params.toString() !== "a=new&b=2") throw new Error(`toString was ${params.toString()}`);
        "#);
    }

    #[test]
    fn set_appends_when_key_missing() {
        run(r#"
            const params = new URLSearchParams();
            params.set("a", "1");
            if (params.toString() !== "a=1") throw new Error(`toString was ${params.toString()}`);
        "#);
    }

    #[test]
    fn append_and_delete() {
        run(r#"
            const params = new URLSearchParams("a=1");
            params.append("a", "2");
            if (params.getAll("a").join(",") !== "1,2") throw new Error("append failed");

            params.delete("a");
            if (params.size !== 0) throw new Error(`size after delete was ${params.size}`);
        "#);
    }

    #[test]
    fn iteration_preserves_original_interleaved_order() {
        run(r#"
            const params = new URLSearchParams("a=1&b=2&a=3");

            const keys = [...params.keys()];
            if (keys.join(",") !== "a,b,a") throw new Error(`keys were ${keys}`);

            const values = [...params.values()];
            if (values.join(",") !== "1,2,3") throw new Error(`values were ${values}`);

            const entries = [...params.entries()].map(([k, v]) => `${k}=${v}`);
            if (entries.join(",") !== "a=1,b=2,a=3") throw new Error(`entries were ${entries}`);

            // The default for...of iterates entries() too.
            const spread = [...params].map(([k, v]) => `${k}=${v}`);
            if (spread.join(",") !== "a=1,b=2,a=3") throw new Error(`spread was ${spread}`);
        "#);
    }

    #[test]
    fn for_each_calls_back_with_value_then_key() {
        run(r#"
            const params = new URLSearchParams("a=1&b=2");
            const seen = [];
            params.forEach((value, key) => seen.push(`${key}=${value}`));
            if (seen.join(",") !== "a=1,b=2") throw new Error(`seen was ${seen}`);
        "#);
    }

    #[test]
    fn to_string_percent_encodes_and_joins_with_ampersand() {
        run(r#"
            const params = new URLSearchParams();
            params.append("a b", "c&d");
            if (params.toString() !== "a%20b=c%26d") throw new Error(`toString was ${params.toString()}`);
        "#);
    }

    #[test]
    fn mutating_url_search_params_updates_url_href() {
        run(r#"
            const url = new URL("https://example.com/?a=1");

            url.searchParams.append("b", "2");
            if (url.search !== "?a=1&b=2") throw new Error(`search was ${url.search}`);
            if (url.href !== "https://example.com/?a=1&b=2") throw new Error(`href was ${url.href}`);

            url.searchParams.delete("a");
            if (url.href !== "https://example.com/?b=2") throw new Error(`href after delete was ${url.href}`);
        "#);
    }

    #[test]
    fn setting_url_search_resyncs_search_params() {
        run(r#"
            const url = new URL("https://example.com/?a=1");
            url.search = "?b=2";
            if (url.searchParams.get("a") !== undefined) throw new Error("stale param a survived");
            if (url.searchParams.get("b") !== "2") throw new Error(`b was ${url.searchParams.get("b")}`);
        "#);
    }
}
