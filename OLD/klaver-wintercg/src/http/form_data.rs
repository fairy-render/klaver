use rquickjs::{
    class::Trace, prelude::Opt, Ctx, FromJs, IntoJs, JsLifetime, Object, String, Value,
};
use rquickjs_util::{
    iterator::{Iterable, NativeIter},
    Entry, StringRef,
};

use crate::{
    blob::Blob,
    file::File,
    multimap::{JsMultiMap, JsMultiMapIter},
};

#[derive(Trace)]
struct FormEntry<'js> {
    value: FormValue<'js>,
    file_name: Option<String<'js>>,
}

impl<'js> FromJs<'js> for FormEntry<'js> {
    fn from_js(ctx: &rquickjs::Ctx<'js>, value: rquickjs::Value<'js>) -> rquickjs::Result<Self> {
        let obj = Object::from_js(ctx, value)?;

        Ok(FormEntry {
            value: obj.get("value")?,
            file_name: obj.get("fileName")?,
        })
    }
}

impl<'js> IntoJs<'js> for FormEntry<'js> {
    fn into_js(self, ctx: &rquickjs::Ctx<'js>) -> rquickjs::Result<rquickjs::Value<'js>> {
        let obj = Object::new(ctx.clone())?;

        obj.set("value", self.value)?;
        obj.set("fileName", self.file_name)?;

        obj.into_js(ctx)
    }
}

#[derive(Trace)]
enum FormValue<'js> {
    Blob(Blob<'js>),
    File(File<'js>),
    String(String<'js>),
}

impl<'js> FromJs<'js> for FormValue<'js> {
    fn from_js(ctx: &rquickjs::Ctx<'js>, value: rquickjs::Value<'js>) -> rquickjs::Result<Self> {
        todo!()
    }
}

impl<'js> IntoJs<'js> for FormValue<'js> {
    fn into_js(self, ctx: &rquickjs::Ctx<'js>) -> rquickjs::Result<rquickjs::Value<'js>> {
        todo!()
    }
}

#[derive(Trace)]
#[rquickjs::class]
pub struct FormData<'js> {
    map: JsMultiMap<'js, String<'js>, FormEntry<'js>>,
}

unsafe impl<'js> JsLifetime<'js> for FormData<'js> {
    type Changed<'to> = FormData<'to>;
}

#[rquickjs::methods]
impl<'js> FormData<'js> {
    #[qjs(constructor)]
    fn new(ctx: Ctx<'js>) -> rquickjs::Result<FormData<'js>> {
        Ok(FormData {
            map: JsMultiMap::new(ctx)?,
        })
    }

    fn append(
        &self,
        ctx: Ctx<'js>,
        key: String<'js>,
        value: FormValue<'js>,
        file_name: Opt<String<'js>>,
    ) -> rquickjs::Result<()> {
        self.map.append(
            ctx,
            key,
            FormEntry {
                value,
                file_name: file_name.0,
            },
        )?;
        Ok(())
    }

    fn set(
        &self,
        ctx: Ctx<'js>,
        key: String<'js>,
        value: FormValue<'js>,
        file_name: Opt<String<'js>>,
    ) -> rquickjs::Result<()> {
        self.map.set(
            ctx,
            key,
            FormEntry {
                value,
                file_name: file_name.0,
            },
        )?;
        Ok(())
    }

    fn get(&self, ctx: Ctx<'js>, name: String<'js>) -> rquickjs::Result<Value<'js>> {
        let Some(ret) = self.map.get(name)? else {
            return Ok(Value::new_null(ctx));
        };

        ret.value.into_js(&ctx)
    }

    fn has(&self, name: String<'js>) -> rquickjs::Result<bool> {
        self.map.has(name)
    }
}

// impl<'js> Iterable<'js> for FormData<'js> {
//     type Item = Entry<rquickjs::String<'js>, FormValue<'js>>;

//     type Iter = JsMultiMapIter<'js, rquickjs::String<'js>, FormValue<'js>>;

//     fn entries(&mut self) -> rquickjs::Result<NativeIter<Self::Iter>> {
//         Ok(NativeIter::new(self.map.entries()?))
//     }
// }
