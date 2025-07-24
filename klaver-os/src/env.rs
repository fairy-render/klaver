use rquickjs::{Class, Coerced, Ctx, FromJs, JsLifetime, String, Value, class::Trace};
use rquickjs_util::{
    Entry, MapEntries, ProxyHandler, create_proxy, iterator::Iterable, typed_map::TypedMap,
};

#[derive(Trace)]
#[rquickjs::class]
pub struct Env<'js> {
    map: TypedMap<'js, String<'js>, String<'js>>,
}

unsafe impl<'js> JsLifetime<'js> for Env<'js> {
    type Changed<'to> = Env<'to>;
}

impl<'js> Env<'js> {
    pub fn create_proxy(ctx: &Ctx<'js>, env: Class<'js, Self>) -> rquickjs::Result<Value<'js>> {
        create_proxy(ctx.clone(), env, EnvProxyHandler)
    }
}

impl<'js> Env<'js> {
    pub fn get(&self, key: String<'js>) -> rquickjs::Result<Option<String<'js>>> {
        self.map.get(key)
    }

    pub fn set(
        &self,
        key: String<'js>,
        Coerced(value): Coerced<String<'js>>,
    ) -> rquickjs::Result<Option<String<'js>>> {
        self.map.set(key, value)
    }

    pub fn del(&self, key: String<'js>) -> rquickjs::Result<()> {
        self.map.del(key)
    }
}

#[derive(Trace)]
struct EnvProxyHandler;

impl<'js> ProxyHandler<'js, Class<'js, Env<'js>>> for EnvProxyHandler {
    fn get(
        &self,
        ctx: rquickjs::Ctx<'js>,
        target: Class<'js, Env<'js>>,
        prop: rquickjs_util::Prop<'js>,
        receiver: rquickjs::Value<'js>,
    ) -> rquickjs::Result<rquickjs::Value<'js>> {
        match prop {
            rquickjs_util::Prop::String(str) => Ok(target
                .borrow()
                .get(str)?
                .map(|m| m.into_value())
                .unwrap_or_else(|| Value::new_null(ctx))),
            rquickjs_util::Prop::Symbol(symbol) => todo!(),
        }
    }

    fn set(
        &self,
        ctx: rquickjs::Ctx<'js>,
        target: Class<'js, Env<'js>>,
        prop: rquickjs_util::Prop<'js>,
        value: rquickjs::Value<'js>,
        receiver: rquickjs::Value<'js>,
    ) -> rquickjs::Result<bool> {
        match prop {
            rquickjs_util::Prop::String(str) => {
                let string = Coerced::<String<'js>>::from_js(&ctx, value)?;
                target.borrow().set(str, string)?;
            }
            rquickjs_util::Prop::Symbol(symbol) => todo!(),
        }
        Ok(true)
    }

    fn apply(
        &self,
        ctx: rquickjs::Ctx<'js>,
        target: Class<'js, Env<'js>>,
        this: rquickjs::Value<'js>,
        args: rquickjs::Array<'js>,
    ) -> rquickjs::Result<()> {
        Ok(())
    }

    fn own_keys(
        &self,
        ctx: rquickjs::Ctx<'js>,
        target: Class<'js, Env<'js>>,
    ) -> rquickjs::Result<rquickjs::Array<'js>> {
        rquickjs::Array::new(ctx)
    }
}

// impl<'js> Iterable<'js> for Env<'js> {
//     type Item = Entry<String<'js>, String<'js>>;

//     type Iter = MapEntries<'js, String<'js>, String<'js>>;

//     fn entries(&mut self) -> rquickjs::Result<rquickjs_util::iterator::NativeIter<Self::Iter>> {
//         todo!()
//     }
// }
