use klaver_base::create_export;

use klaver_util::{
    Iter, Iterable, IterableProtocol, NativeIterator, NativeIteratorExt, Pair, StringExt,
    StringRef, TypedList, TypedMultiMap, TypedMultiMapEntries,
};
use rquickjs::{
    Array, Class, Ctx, FromJs, Function, IntoJs, JsLifetime, String, Value, atom::PredefinedAtom,
    class::Trace, prelude::Opt,
};
use std::fmt::Write;

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
        let map = if let Ok(qs) = StringRef::from_js(ctx, value.clone()) {
            // We got a query string - parse it
            let iter = form_urlencoded::parse(qs.as_bytes());
            let map = TypedMultiMap::new(ctx.clone())?;

            for (first, next) in iter {
                let first = rquickjs::String::from_str(ctx.clone(), &*first)?;
                let second = rquickjs::String::from_str(ctx.clone(), &*next)?;

                map.append(ctx, first, second)?;
            }

            map
        } else if let Ok(iter) = Iter::from_js(ctx, value.clone()) {
            // We got a iterator of key/value pairs
            let map = TypedMultiMap::new(ctx.clone())?;

            for pair in iter
                .from_javascript::<Pair<String<'_>, String<'_>>>()
                .into_iter(ctx)
            {
                let pair = pair?;
                map.append(ctx, pair.0, pair.1)?;
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
        let iterator = NativeIterator::new(self.map.entries()?);
        Ok(Class::instance(ctx, iterator)?.into_value())
    }

    #[qjs(rename = "forEach")]
    pub fn for_each(&self, ctx: Ctx<'js>, func: Function<'js>) -> rquickjs::Result<()> {
        let entries = self.map.entries()?;

        for entry in entries.into_iter(&ctx) {
            let entry = entry?;
            func.call::<_, ()>((entry,))?
        }

        Ok(())
    }

    #[qjs(rename = PredefinedAtom::ToString)]
    pub fn to_string(&self, ctx: Ctx<'js>) -> rquickjs::Result<std::string::String> {
        let entries = self.map.entries()?;
        let mut output = std::string::String::new();
        for (idx, entry) in entries.into_iter(&ctx).enumerate() {
            if idx > 0 {
                output.push('&');
            }
            let entry = entry?;
            let key = StringRef::from_string(entry.0)?;
            let value = StringRef::from_string(entry.1)?;
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

impl<'js> IterableProtocol<'js> for URLSearchParams<'js> {
    type Iterator = TypedMultiMapEntries<'js, String<'js>, String<'js>>;

    fn create_iterator(&self, _ctx: &Ctx<'js>) -> rquickjs::Result<Self::Iterator> {
        self.map.entries()
    }
}

// impl<'js> Iterable<'js> for URLSearchParams<'js> {
//     type Item = Entry<rquickjs::String<'js>, rquickjs::String<'js>>;

//     type Iter = TypedMultiMapIter<'js, rquickjs::String<'js>, rquickjs::String<'js>>;

//     fn entries(&mut self) -> rquickjs::Result<NativeIter<Self::Iter>> {
//         Ok(NativeIter::new(self.map.entries()?))
//     }
// }

create_export!(URLSearchParams<'js>);
