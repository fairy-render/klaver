use klaver_base::{Exportable, create_export};
use klaver_util::{
    IterableProtocol, NativeIterator, Prop, ProxyHandler, TypedMap, TypedMapEntries, create_proxy,
    throw,
};
use rquickjs::{
    Class, Coerced, Ctx, FromJs, JsLifetime, String, Symbol, Value,
    class::{JsClass, Trace},
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

#[rquickjs::methods]
impl<'js> Env<'js> {
    #[qjs(constructor)]
    fn construct(ctx: Ctx<'js>) -> rquickjs::Result<Env<'js>> {
        throw!(@type ctx, "Env cannot be constructed directly");
    }

    pub fn get(&self, key: String<'js>) -> rquickjs::Result<Option<String<'js>>> {
        self.map.get(key)
    }

    pub fn entries(&self) -> rquickjs::Result<NativeIterator<'js>> {
        Ok(NativeIterator::new(self.map.entries()?))
    }

    pub fn values(&self) -> rquickjs::Result<NativeIterator<'js>> {
        Ok(NativeIterator::new(self.map.values()?))
    }

    pub fn keys(&self) -> rquickjs::Result<NativeIterator<'js>> {
        Ok(NativeIterator::new(self.map.keys()?))
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

impl<'js> IterableProtocol<'js> for Env<'js> {
    type Iterator = TypedMapEntries<'js, String<'js>, String<'js>>;

    fn create_iterator(&self, _ctx: &Ctx<'js>) -> rquickjs::Result<Self::Iterator> {
        self.map.entries()
    }
}

impl<'js> Exportable<'js> for Env<'js> {
    fn export<T>(
        ctx: &Ctx<'js>,
        _registry: &klaver_base::Registry,
        target: &T,
    ) -> rquickjs::Result<()>
    where
        T: klaver_base::ExportTarget<'js>,
    {
        target.set(ctx, Self::NAME, Class::<Self>::create_constructor(ctx)?)?;
        Self::add_iterable_prototype(ctx)?;

        Ok(())
    }
}

#[derive(Trace)]
struct EnvProxyHandler;

impl<'js> ProxyHandler<'js, Class<'js, Env<'js>>> for EnvProxyHandler {
    fn get(
        &self,
        ctx: rquickjs::Ctx<'js>,
        target: Class<'js, Env<'js>>,
        prop: Prop<'js>,
        _receiver: rquickjs::Value<'js>,
    ) -> rquickjs::Result<rquickjs::Value<'js>> {
        match prop {
            Prop::String(str) => Ok(target
                .borrow()
                .get(str)?
                .map(|m| m.into_value())
                .unwrap_or_else(|| Value::new_null(ctx))),
            Prop::Symbol(symbol) => {
                let iterator = Symbol::iterator(ctx.clone());
                if symbol == iterator {
                    let entries = target.borrow().map.entries()?;
                    return Ok(Class::instance(ctx, NativeIterator::new(entries))?.into_value());
                }

                throw!(@type ctx, format!("Symbol not supported: {:?}", symbol))
            }
        }
    }

    fn set(
        &self,
        ctx: rquickjs::Ctx<'js>,
        target: Class<'js, Env<'js>>,
        prop: Prop<'js>,
        value: rquickjs::Value<'js>,
        _receiver: rquickjs::Value<'js>,
    ) -> rquickjs::Result<bool> {
        match prop {
            Prop::String(str) => {
                let string = Coerced::<String<'js>>::from_js(&ctx, value)?;
                target.borrow().set(str, string)?;
            }
            Prop::Symbol(symbol) => {
                throw!(@type ctx, format!("Symbol not supported: {:?}", symbol))
            }
        }
        Ok(true)
    }
}

// impl<'js> Iterable<'js> for Env<'js> {
//     type Item = Entry<String<'js>, String<'js>>;

//     type Iter = MapEntries<'js, String<'js>, String<'js>>;

//     fn entries(&mut self) -> rquickjs::Result<rquickjs_util::iterator::NativeIter<Self::Iter>> {
//         todo!()
//     }
// }
