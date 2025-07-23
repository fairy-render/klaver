use klaver_base::create_export;
use rquickjs::{
    Array, Ctx, FromJs, Function, IntoJs, JsLifetime, Value, atom::PredefinedAtom, class::Trace,
    prelude::Opt,
};
use rquickjs_util::{
    Entry, StringRef,
    iterator::{Iterable, JsIterator, NativeIter},
    typed_list::TypedList,
    typed_multi_map::{TypedMultiMap, TypedMultiMapIter},
};
use std::fmt::Write;

// TODO: Deprecate - Use entry of utils
pub struct Pair<T, V> {
    first: T,
    second: V,
}

impl<'js, T, V> FromJs<'js> for Pair<T, V>
where
    T: FromJs<'js>,
    V: FromJs<'js>,
{
    fn from_js(ctx: &rquickjs::Ctx<'js>, value: rquickjs::Value<'js>) -> rquickjs::Result<Self> {
        let array = Array::from_js(ctx, value)?;
        if array.len() != 2 {
            return Err(rquickjs::Error::new_from_js("array", "pair"));
        }

        Ok(Pair {
            first: array.get(0)?,
            second: array.get(1)?,
        })
    }
}

pub struct URLSearchParamsInit<'js> {
    map: TypedMultiMap<'js, rquickjs::String<'js>, rquickjs::String<'js>>,
}

impl<'js> URLSearchParamsInit<'js> {
    pub fn from_str(ctx: Ctx<'js>, qs: &str) -> rquickjs::Result<URLSearchParamsInit<'js>> {
        let iter = form_urlencoded::parse(qs.as_bytes());
        let map = TypedMultiMap::new(ctx.clone())?;

        for (first, next) in iter {
            let first = rquickjs::String::from_str(ctx.clone(), &*first)?;
            let second = rquickjs::String::from_str(ctx.clone(), &*next)?;

            map.append(&ctx, first, second)?;
        }
        Ok(URLSearchParamsInit { map })
    }
}

impl<'js> FromJs<'js> for URLSearchParamsInit<'js> {
    fn from_js(ctx: &rquickjs::Ctx<'js>, value: rquickjs::Value<'js>) -> rquickjs::Result<Self> {
        let map = if let Ok(qs) = String::from_js(ctx, value.clone()) {
            // We got a query string - parse it
            let iter = form_urlencoded::parse(qs.as_bytes());
            let map = TypedMultiMap::new(ctx.clone())?;

            for (first, next) in iter {
                let first = rquickjs::String::from_str(ctx.clone(), &*first)?;
                let second = rquickjs::String::from_str(ctx.clone(), &*next)?;

                map.append(ctx, first, second)?;
            }

            map
        } else if let Ok(iter) =
            JsIterator::<Pair<rquickjs::String, rquickjs::String>>::from_js(ctx, value.clone())
        {
            // We got a iterator of key/value pairs
            let map = TypedMultiMap::new(ctx.clone())?;

            for pair in iter {
                let pair = pair?;
                map.append(ctx, pair.first, pair.second)?;
            }

            map
        } else {
            return Err(rquickjs::Error::new_from_js(
                value.type_name(),
                "iterator or string",
            ));
        };

        Ok(URLSearchParamsInit { map })
    }
}

#[derive(Trace)]
#[rquickjs::class]
pub struct URLSearchParams<'js> {
    map: TypedMultiMap<'js, rquickjs::String<'js>, rquickjs::String<'js>>,
}

unsafe impl<'js> JsLifetime<'js> for URLSearchParams<'js> {
    type Changed<'to> = URLSearchParams<'to>;
}

#[rquickjs::methods]
impl<'js> URLSearchParams<'js> {
    #[qjs(constructor)]
    pub fn new(
        ctx: Ctx<'js>,
        init: Opt<URLSearchParamsInit<'js>>,
    ) -> rquickjs::Result<URLSearchParams<'js>> {
        let map = if let Some(init) = init.0 {
            init.map
        } else {
            TypedMultiMap::new(ctx)?
        };
        Ok(URLSearchParams { map })
    }

    pub fn get(
        &self,
        key: rquickjs::String<'js>,
    ) -> rquickjs::Result<Option<rquickjs::String<'js>>> {
        self.map.get(key)
    }

    pub fn has(&self, key: rquickjs::String<'js>) -> rquickjs::Result<bool> {
        self.map.has(key)
    }

    #[qjs(rename = "getAll")]
    pub fn get_all(
        &self,
        key: rquickjs::String<'js>,
    ) -> rquickjs::Result<Option<TypedList<'js, rquickjs::String<'js>>>> {
        self.map.get_all(key)
    }

    pub fn set(
        &self,
        ctx: Ctx<'js>,
        key: rquickjs::String<'js>,
        value: rquickjs::String<'js>,
    ) -> rquickjs::Result<()> {
        self.map.set(&ctx, key, value)
    }

    pub fn append(
        &self,
        ctx: Ctx<'js>,
        key: rquickjs::String<'js>,
        value: rquickjs::String<'js>,
    ) -> rquickjs::Result<()> {
        self.map.append(&ctx, key, value)
    }

    pub fn delete(&self, key: rquickjs::String<'js>) -> rquickjs::Result<()> {
        self.map.delete(key)
    }

    pub fn entries(&mut self, ctx: Ctx<'js>) -> rquickjs::Result<Value<'js>> {
        <URLSearchParams as Iterable>::entries(self)?.into_js(&ctx)
    }

    #[qjs(rename = "forEach")]
    pub fn for_each(&self, func: Function<'js>) -> rquickjs::Result<()> {
        let entries = self.map.entries()?;

        for entry in entries {
            let entry = entry?;
            func.call::<_, ()>((entry,))?
        }

        Ok(())
    }

    #[qjs(rename = PredefinedAtom::ToString)]
    pub fn to_string(&mut self, _ctx: Ctx<'js>) -> rquickjs::Result<String> {
        let entries = self.map.entries()?;
        let mut output = String::new();
        for (idx, entry) in entries.enumerate() {
            if idx > 0 {
                output.push('&');
            }
            let entry = entry?;
            let key = StringRef::from_string(entry.key)?;
            let value = StringRef::from_string(entry.value)?;
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

impl<'js> Iterable<'js> for URLSearchParams<'js> {
    type Item = Entry<rquickjs::String<'js>, rquickjs::String<'js>>;

    type Iter = TypedMultiMapIter<'js, rquickjs::String<'js>, rquickjs::String<'js>>;

    fn entries(&mut self) -> rquickjs::Result<NativeIter<Self::Iter>> {
        Ok(NativeIter::new(self.map.entries()?))
    }
}

create_export!(URLSearchParams<'js>);
