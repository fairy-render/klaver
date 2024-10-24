use std::marker::PhantomData;

use klaver_shared::{
    iterator::JsIterator,
    typed_list::TypedList,
    typed_map::TypedMap,
    util::{is_iterator, ObjectExt},
};
use rquickjs::{
    atom::PredefinedAtom,
    class::Trace,
    prelude::{Func, This},
    Array, Ctx, FromJs, Function, IntoJs, Object, Value,
};

use crate::multimap::JsMultiMap;

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
    map: JsMultiMap<'js, rquickjs::String<'js>, rquickjs::String<'js>>,
}

impl<'js> FromJs<'js> for URLSearchParamsInit<'js> {
    fn from_js(ctx: &rquickjs::Ctx<'js>, value: rquickjs::Value<'js>) -> rquickjs::Result<Self> {
        let map = if let Ok(qs) = String::from_js(ctx, value.clone()) {
            let iter = form_urlencoded::parse(qs.as_bytes());
            let map = JsMultiMap::new(ctx.clone())?;

            for (first, next) in iter {
                let first = rquickjs::String::from_str(ctx.clone(), &*first)?;
                let second = rquickjs::String::from_str(ctx.clone(), &*next)?;

                map.append(ctx.clone(), first, second)?;
            }

            map
        } else if let Ok(iter) =
            JsIterator::<Pair<rquickjs::String, rquickjs::String>>::from_js(ctx, value.clone())
        {
            let map = JsMultiMap::new(ctx.clone())?;

            for pair in iter {
                let pair = pair?;
                map.append(ctx.clone(), pair.first, pair.second)?;
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
    map: JsMultiMap<'js, rquickjs::String<'js>, rquickjs::String<'js>>,
}

#[rquickjs::methods]
impl<'js> URLSearchParams<'js> {
    #[qjs(constructor)]
    pub fn new(init: URLSearchParamsInit<'js>) -> rquickjs::Result<URLSearchParams<'js>> {
        Ok(URLSearchParams { map: init.map })
    }

    pub fn get(
        &self,
        key: rquickjs::String<'js>,
    ) -> rquickjs::Result<Option<rquickjs::String<'js>>> {
        self.map.get(key)
    }

    pub fn set(
        &self,
        ctx: Ctx<'js>,
        key: rquickjs::String<'js>,
        value: rquickjs::String<'js>,
    ) -> rquickjs::Result<()> {
        self.map.set(ctx, key, value)
    }

    pub fn append(
        &self,
        ctx: Ctx<'js>,
        key: rquickjs::String<'js>,
        value: rquickjs::String<'js>,
    ) -> rquickjs::Result<()> {
        self.map.append(ctx, key, value)
    }
}
