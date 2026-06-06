use rquickjs::{Class, FromJs, IntoJs, JsLifetime, Object, class::Trace};

#[rquickjs::class]
#[derive(Debug, Trace, JsLifetime)]
pub struct Core<'js> {
    items: Object<'js>,
}

impl<'js> Core<'js> {
    pub fn new(ctx: &rquickjs::Ctx<'js>) -> rquickjs::Result<Core<'js>> {
        Ok(Core {
            items: Object::new(ctx.clone())?,
        })
    }
}

impl<'js> Core<'js> {
    pub fn from_ctx(ctx: &rquickjs::Ctx<'js>) -> rquickjs::Result<Class<'js, Core<'js>>> {
        ctx.globals().get::<_, Class<'js, Core<'js>>>("$_runtime")
    }

    pub fn register<T: IntoJs<'js>>(
        &mut self,
        name: impl ToString,
        item: T,
    ) -> rquickjs::Result<()> {
        self.items
            .set(name.to_string(), item.into_js(self.items.ctx()))?;
        Ok(())
    }

    pub fn has(&self, name: &str) -> rquickjs::Result<bool> {
        self.items.contains_key(name)
    }

    pub fn get<T: FromJs<'js>>(&self, name: &str) -> rquickjs::Result<T> {
        self.items.get(name)
    }
}
