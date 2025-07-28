use std::collections::HashMap;

use klaver_util::rquickjs::{
    self, Class, Ctx, FromJs, Function, JsLifetime, Object, String, class::Trace,
};

use crate::exec_state::AsyncId;

pub fn get_listeners<'js>(ctx: &Ctx<'js>) -> rquickjs::Result<Class<'js, HookListeners<'js>>> {
    if let Ok(hooks) = ctx
        .globals()
        .get::<_, Class<'js, HookListeners<'js>>>("$__hooks")
    {
        return Ok(hooks);
    } else {
        let hooks = Class::instance(
            ctx.clone(),
            HookListeners {
                listeners: Default::default(),
                handles: Default::default(),
            },
        )?;

        ctx.globals().set("$__hooks", hooks.clone())?;

        Ok(hooks)
    }
}

pub type ResourceHandle<'js> = rquickjs::Object<'js>;

pub trait NativeListener<'js>: Trace<'js> {
    fn init(
        &self,
        ctx: &Ctx<'js>,
        id: AsyncId,
        ty: String<'js>,
        trigger: Option<AsyncId>,
        resource: &ResourceHandle<'js>,
    ) -> rquickjs::Result<()>;

    fn before(&self, ctx: &Ctx<'js>, id: AsyncId) -> rquickjs::Result<()>;

    fn after(&self, ctx: &Ctx<'js>, id: AsyncId) -> rquickjs::Result<()>;

    fn destroy(&self, ctx: &Ctx<'js>, id: AsyncId) -> rquickjs::Result<()>;
}

pub struct ScriptHook<'js> {
    init: Option<Function<'js>>,
    before: Option<Function<'js>>,
    after: Option<Function<'js>>,
    destroy: Option<Function<'js>>,
}

impl<'js> Trace<'js> for ScriptHook<'js> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        self.init.trace(tracer);
        self.before.trace(tracer);
        self.after.trace(tracer);
        self.destroy.trace(tracer);
    }
}

impl<'js> FromJs<'js> for ScriptHook<'js> {
    fn from_js(_ctx: &Ctx<'js>, value: rquickjs::Value<'js>) -> rquickjs::Result<Self> {
        let obj: Object = value.get()?;

        Ok(ScriptHook {
            init: obj.get("init")?,
            before: obj.get("before")?,
            after: obj.get("after")?,
            destroy: obj.get("destroy")?,
        })
    }
}

pub enum Hook<'js> {
    Native(Box<dyn NativeListener<'js> + 'js>),
    Script(ScriptHook<'js>),
}

impl<'js> Hook<'js> {
    fn init(
        &self,
        ctx: &Ctx<'js>,
        id: AsyncId,
        ty: String<'js>,
        trigger: Option<AsyncId>,
        resource: &ResourceHandle<'js>,
    ) -> rquickjs::Result<()> {
        match self {
            Self::Native(native) => native.init(ctx, id, ty, trigger, resource),
            Self::Script(script) => {
                let Some(init) = &script.init else {
                    return Ok(());
                };
                init.call((id, ty, trigger, resource.clone()))
            }
        }
    }

    fn before(&self, ctx: &Ctx<'js>, id: AsyncId) -> rquickjs::Result<()> {
        match self {
            Self::Native(native) => native.before(ctx, id),
            Self::Script(script) => {
                let Some(init) = &script.before else {
                    return Ok(());
                };
                init.call((id,))
            }
        }
    }

    fn after(&self, ctx: &Ctx<'js>, id: AsyncId) -> rquickjs::Result<()> {
        match self {
            Self::Native(native) => native.after(ctx, id),
            Self::Script(script) => {
                let Some(after) = &script.after else {
                    return Ok(());
                };
                after.call((id,))
            }
        }
    }

    fn destroy(&self, ctx: &Ctx<'js>, id: AsyncId) -> rquickjs::Result<()> {
        match self {
            Self::Native(native) => native.destroy(ctx, id),
            Self::Script(script) => {
                let Some(destroy) = &script.destroy else {
                    return Ok(());
                };
                destroy.call((id,))
            }
        }
    }
}

impl<'js> Trace<'js> for Hook<'js> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        match self {
            Self::Native(native) => native.trace(tracer),
            Self::Script(script) => script.trace(tracer),
        }
    }
}

#[rquickjs::class(crate = "rquickjs")]
pub struct HookListeners<'js> {
    listeners: Vec<Hook<'js>>,
    handles: HashMap<AsyncId, ResourceHandle<'js>>,
}

impl<'js> Trace<'js> for HookListeners<'js> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        self.listeners.trace(tracer);
        self.handles.trace(tracer);
    }
}

unsafe impl<'js> JsLifetime<'js> for HookListeners<'js> {
    type Changed<'to> = HookListeners<'to>;
}

impl<'js> HookListeners<'js> {
    pub fn add_listener(&mut self, listener: Hook<'js>) {
        self.listeners.push(listener);
    }

    pub fn init(
        &mut self,
        ctx: &Ctx<'js>,
        id: AsyncId,
        ty: String<'js>,
        trigger: Option<AsyncId>,
    ) -> rquickjs::Result<()> {
        let handle = Object::new(ctx.clone())?;

        self.handles.insert(id, handle.clone());
        for hook in &self.listeners {
            hook.init(ctx, id, ty.clone(), trigger, &handle)?;
        }

        Ok(())
    }

    pub fn before(&self, ctx: &Ctx<'js>, id: AsyncId) -> rquickjs::Result<()> {
        for hook in &self.listeners {
            hook.before(ctx, id)?;
        }
        Ok(())
    }

    pub fn after(&self, ctx: &Ctx<'js>, id: AsyncId) -> rquickjs::Result<()> {
        for hook in &self.listeners {
            hook.after(ctx, id)?;
        }
        Ok(())
    }

    pub fn destroy(&mut self, ctx: &Ctx<'js>, id: AsyncId) -> rquickjs::Result<()> {
        let _ = self.handles.remove(&id);
        for hook in &self.listeners {
            hook.destroy(ctx, id)?;
        }
        Ok(())
    }
}
