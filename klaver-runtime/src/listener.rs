use std::rc::Rc;

use klaver_util::{
    TypedMap,
    rquickjs::{self, Ctx, FromJs, Function, JsLifetime, Object, class::Trace},
};

use crate::{id::AsyncId, resource::ResourceKind};

pub type ResourceHandle<'js> = rquickjs::Object<'js>;

pub trait NativeListener<'js>: Trace<'js> {
    fn init(
        &self,
        ctx: &Ctx<'js>,
        id: AsyncId,
        ty: ResourceKind,
        trigger: Option<AsyncId>,
        resource: &ResourceHandle<'js>,
    ) -> rquickjs::Result<()>;

    fn before(&self, ctx: &Ctx<'js>, id: AsyncId) -> rquickjs::Result<()>;

    fn after(&self, ctx: &Ctx<'js>, id: AsyncId) -> rquickjs::Result<()>;

    fn destroy(&self, ctx: &Ctx<'js>, id: AsyncId) -> rquickjs::Result<()>;

    fn promise_resolve(&self, ctx: &Ctx<'js>, id: AsyncId) -> rquickjs::Result<()>;
}

#[derive(Clone)]
pub struct ScriptListener<'js> {
    init: Option<Function<'js>>,
    before: Option<Function<'js>>,
    after: Option<Function<'js>>,
    destroy: Option<Function<'js>>,
    promise_resolve: Option<Function<'js>>,
}

impl<'js> Trace<'js> for ScriptListener<'js> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        self.init.trace(tracer);
        self.before.trace(tracer);
        self.after.trace(tracer);
        self.destroy.trace(tracer);
        self.promise_resolve.trace(tracer);
    }
}

impl<'js> FromJs<'js> for ScriptListener<'js> {
    fn from_js(_ctx: &Ctx<'js>, value: rquickjs::Value<'js>) -> rquickjs::Result<Self> {
        let obj: Object = value.get()?;

        Ok(ScriptListener {
            init: obj.get("init")?,
            before: obj.get("before")?,
            after: obj.get("after")?,
            destroy: obj.get("destroy")?,
            promise_resolve: obj.get("promiseResolve")?,
        })
    }
}

#[derive(Clone)]
pub enum Listener<'js> {
    Native(Rc<dyn NativeListener<'js> + 'js>),
    Script(ScriptListener<'js>),
}

impl<'js> Listener<'js> {
    fn init(
        &self,
        ctx: &Ctx<'js>,
        id: AsyncId,
        ty: ResourceKind,
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

    fn promise_resolve(&self, ctx: &Ctx<'js>, id: AsyncId) -> rquickjs::Result<()> {
        match self {
            Self::Native(native) => native.promise_resolve(ctx, id),
            Self::Script(script) => {
                let Some(promise_resolve) = &script.promise_resolve else {
                    return Ok(());
                };
                promise_resolve.call((id,))
            }
        }
    }
}

impl<'js> Trace<'js> for Listener<'js> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        match self {
            Self::Native(native) => native.trace(tracer),
            Self::Script(script) => script.trace(tracer),
        }
    }
}

#[derive(Clone)]
pub struct HandleMap<'js> {
    pub handles: TypedMap<'js, AsyncId, ResourceHandle<'js>>,
}

impl<'js> Trace<'js> for HandleMap<'js> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        self.handles.trace(tracer);
    }
}

impl<'js> HandleMap<'js> {
    pub fn get_handle(&self, ctx: &Ctx<'js>, id: AsyncId) -> rquickjs::Result<ResourceHandle<'js>> {
        if let Some(handle) = self.handles.get(id)? {
            Ok(handle)
        } else {
            let handle = Object::new(ctx.clone())?;
            self.handles.set(id, handle.clone())?;
            Ok(handle)
        }
    }
}

#[rquickjs::class(crate = "rquickjs")]
pub struct HookListeners<'js> {
    listeners: slotmap::SlotMap<slotmap::DefaultKey, Listener<'js>>,
    handles: HandleMap<'js>,
}

impl<'js> HookListeners<'js> {
    pub fn new(handles: HandleMap<'js>) -> rquickjs::Result<HookListeners<'js>> {
        Ok(HookListeners {
            listeners: Default::default(),
            handles,
        })
    }
}

impl<'js> Trace<'js> for HookListeners<'js> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        for value in self.listeners.values() {
            value.trace(tracer);
        }
        self.handles.trace(tracer);
    }
}

unsafe impl<'js> JsLifetime<'js> for HookListeners<'js> {
    type Changed<'to> = HookListeners<'to>;
}

impl<'js> HookListeners<'js> {
    pub fn add_listener(&mut self, listener: Listener<'js>) -> slotmap::DefaultKey {
        let key = self.listeners.insert(listener);
        key
    }

    pub fn remove_listener(&mut self, key: slotmap::DefaultKey) {
        self.listeners.remove(key);
    }

    pub fn get_resource_handle(
        &self,
        ctx: &Ctx<'js>,
        id: AsyncId,
    ) -> rquickjs::Result<ResourceHandle<'js>> {
        self.handles.get_handle(ctx, id)
    }

    pub fn init(
        &self,
        ctx: &Ctx<'js>,
        id: AsyncId,
        ty: ResourceKind,
        trigger: Option<AsyncId>,
    ) -> rquickjs::Result<()> {
        let handle = self.handles.get_handle(ctx, id)?;
        for hook in self.listeners.values() {
            hook.init(ctx, id, ty.clone(), trigger, &handle)?;
        }

        Ok(())
    }

    pub fn before(&self, ctx: &Ctx<'js>, id: AsyncId) -> rquickjs::Result<()> {
        for hook in self.listeners.values() {
            hook.before(ctx, id)?;
        }
        Ok(())
    }

    pub fn after(&self, ctx: &Ctx<'js>, id: AsyncId) -> rquickjs::Result<()> {
        for hook in self.listeners.values() {
            hook.after(ctx, id)?;
        }
        Ok(())
    }

    pub fn destroy(&self, ctx: &Ctx<'js>, id: AsyncId) -> rquickjs::Result<()> {
        let _ = self.handles.handles.del(id)?;
        for hook in self.listeners.values() {
            hook.destroy(ctx, id)?;
        }
        Ok(())
    }

    pub fn promise_resolve(&self, ctx: &Ctx<'js>, id: AsyncId) -> rquickjs::Result<()> {
        for hook in self.listeners.values() {
            hook.promise_resolve(ctx, id)?;
        }

        Ok(())
    }
}
