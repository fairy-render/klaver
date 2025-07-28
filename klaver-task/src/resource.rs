use crate::{
    Shutdown,
    hook::{HookListeners, get_hooks},
    state::{AsyncId, AsyncState},
};
use klaver_util::rquickjs::{self, Class, Ctx, FromJs, Function, String, prelude::IntoArgs};

pub struct TaskCtx<'js> {
    pub ctx: Ctx<'js>,
    pub id: AsyncId,
    pub parent: AsyncId,
    pub ty: String<'js>,
    pub hook_list: Class<'js, HookListeners<'js>>,
}

impl<'js> TaskCtx<'js> {
    pub(crate) fn new(
        ctx: Ctx<'js>,
        ty: String<'js>,
        parent: AsyncId,
        id: AsyncId,
    ) -> rquickjs::Result<TaskCtx<'js>> {
        let hook_list = get_hooks(&ctx)?;
        Ok(TaskCtx {
            ctx,
            id,
            hook_list,
            parent,
            ty,
        })
    }

    pub(crate) fn init(&self) -> rquickjs::Result<()> {
        self.hook_list
            .borrow_mut()
            .init(&self.ctx, self.id, self.ty.clone(), self.parent)
    }
}

impl<'js> TaskCtx<'js> {
    pub fn invoke_callback<A, R>(&self, cb: Function<'js>, args: A) -> rquickjs::Result<R>
    where
        A: IntoArgs<'js>,
        R: FromJs<'js>,
    {
        self.hook_list.borrow().before(&self.ctx, self.id.clone())?;
        AsyncState::get(&self.ctx)?.enter(self.id);
        let ret = cb.call(args);
        AsyncState::get(&self.ctx)?.leave();
        self.hook_list.borrow().after(&self.ctx, self.id.clone())?;
        ret
    }
}

pub trait Resource<'js>: Sized {
    fn run(
        &self,
        ctx: TaskCtx<'js>,
        shutdown: Shutdown,
    ) -> impl Future<Output = rquickjs::Result<()>>;
}
