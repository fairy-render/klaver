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
