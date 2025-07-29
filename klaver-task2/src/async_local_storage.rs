use klaver_util::{
    TypedMap,
    rquickjs::{self, Class, Ctx, Function, Value, class::Trace},
};

use crate::{
    AsyncState, NativeListener, ResourceHandle, ResourceKind, TaskCtx, exec_state::AsyncId,
    state::HookState,
};

pub struct AsyncLocalStorage<'js> {
    state: AsyncState,
    key: Option<slotmap::DefaultKey>,
    test: &'js (),
    hook: Class<'js, HookState<'js>>,
}

impl<'js> AsyncLocalStorage<'js> {
    pub fn run(
        &self,
        ctx: Ctx<'js>,
        store: Value<'js>,
        func: Function<'js>,
    ) -> rquickjs::Result<Value<'js>> {
        let id = self.state.exec.create_task(None, ResourceKind::Script);

        let task_ctx = TaskCtx::new(
            ctx.clone(),
            self.state.exec.clone(),
            ResourceKind::Script,
            id,
        )?;
        task_ctx.init()?;

        let handle = self.hook.borrow().resources.get_handle(&ctx, id)?;
        handle.set("store", store)?;

        let ret = task_ctx.invoke_callback::<_, Value<'js>>(func, ())?;

        task_ctx.destroy()?;

        Ok(ret)
    }

    pub fn get_store(&self, ctx: Ctx<'js>) -> rquickjs::Result<Value<'js>> {
        let id = self.state.exec.exectution_trigger_id();
        let resource = self.hook.borrow().resources.get_handle(&ctx, id)?;
        resource.get("store")
    }
}

struct AsyncStorageHook<'js> {
    state: AsyncState,
    hook: Class<'js, HookState<'js>>,
}

impl<'js> Trace<'js> for AsyncStorageHook<'js> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {}
}

impl<'js> NativeListener<'js> for AsyncStorageHook<'js> {
    fn init(
        &self,
        ctx: &Ctx<'js>,
        id: crate::exec_state::AsyncId,
        ty: crate::ResourceKind,
        trigger: Option<crate::exec_state::AsyncId>,
        resource: &crate::ResourceHandle<'js>,
    ) -> rquickjs::Result<()> {
        let curent_id = self.state.exec.exectution_trigger_id();
        let current_resource = self.hook.borrow().resources.get_handle(ctx, curent_id)?;
        resource.set("store", current_resource.get::<_, Value<'js>>("store")?)?;
        todo!()
    }

    fn before(&self, ctx: &Ctx<'js>, id: crate::exec_state::AsyncId) -> rquickjs::Result<()> {
        todo!()
    }

    fn after(&self, ctx: &Ctx<'js>, id: crate::exec_state::AsyncId) -> rquickjs::Result<()> {
        todo!()
    }

    fn destroy(&self, ctx: &Ctx<'js>, id: crate::exec_state::AsyncId) -> rquickjs::Result<()> {
        todo!()
    }

    fn promise_resolve(
        &self,
        ctx: &Ctx<'js>,
        id: crate::exec_state::AsyncId,
    ) -> rquickjs::Result<()> {
        todo!()
    }
}
