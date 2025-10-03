use crate::{BasePrimordials, ObjectExt};
use rquickjs::{Class, Ctx, FromJs, IntoJs, JsLifetime, Object, class::Trace};

#[derive(Trace)]
#[rquickjs::class]
pub struct Core<'js> {
    primordials: BasePrimordials<'js>,
    store: Object<'js>,
}

unsafe impl<'js> JsLifetime<'js> for Core<'js> {
    type Changed<'to> = Core<'to>;
}

impl<'js> Core<'js> {
    const GLOBAL_KEY: &'static str = "Core";

    pub fn instance(ctx: &Ctx<'js>) -> rquickjs::Result<Class<'js, Core<'js>>> {
        if let Ok(core) = ctx.globals().get(Self::GLOBAL_KEY) {
            Ok(core)
        } else {
            let primordials = BasePrimordials::new(ctx)?;
            let store: Object<'js> = primordials.construct_map(())?;

            let core = Class::instance(ctx.clone(), Core { primordials, store })?;

            ctx.globals().set(Self::GLOBAL_KEY, core.clone())?;

            Ok(core)
        }
    }

    pub fn primordials(&self) -> &BasePrimordials<'js> {
        &self.primordials
    }

    pub fn set<K: IntoJs<'js>, V: IntoJs<'js>>(&self, key: K, value: V) -> rquickjs::Result<()> {
        self.store.call_property::<_, _, ()>("set", (key, value))?;
        Ok(())
    }

    pub fn get<K: IntoJs<'js>, V: FromJs<'js>>(&self, key: K) -> rquickjs::Result<V> {
        self.store.call_property("get", (key,))
    }
}
