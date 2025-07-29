use futures::FutureExt;
use klaver_util::{
    CaugthException,
    rquickjs::{self, CatchResultExt, Ctx, JsLifetime, String},
    throw, throw_if,
};
use std::{
    any::TypeId,
    cell::{Cell, RefCell},
    collections::HashMap,
    rc::Rc,
};

use crate::{
    NEXT_ID, ResourceKind,
    cell::ObservableRefCell,
    exec_state::ExecState,
    resource::{Resource, TaskCtx},
};

#[derive(Clone)]
pub struct AsyncState {
    pub(crate) exec: ExecState,
    pub(crate) exception: Rc<ObservableRefCell<Option<CaugthException>>>,
    pub(crate) id_map: Rc<RefCell<HashMap<TypeId, ResourceKind>>>,
    pub(crate) next_id: Rc<Cell<u32>>,
}

unsafe impl<'js> JsLifetime<'js> for AsyncState {
    type Changed<'to> = AsyncState;
}

impl AsyncState {}
impl AsyncState {
    pub fn get(ctx: &Ctx<'_>) -> rquickjs::Result<AsyncState> {
        match ctx.userdata::<Self>() {
            Some(ret) => Ok(ret.clone()),
            None => {
                let _ = throw_if!(
                    ctx,
                    ctx.store_userdata(AsyncState {
                        exec: Default::default(),
                        exception: Rc::new(ObservableRefCell::new(None)),
                        id_map: Default::default(),
                        next_id: Rc::new(Cell::new(NEXT_ID))
                    })
                );

                Ok(ctx.userdata::<Self>().unwrap().clone())
            }
        }
    }

    pub fn dump(&self) {
        self.exec.dump();
    }

    pub fn push<'js, T: Resource<'js> + 'js>(
        &self,
        ctx: &Ctx<'js>,
        resource: T,
    ) -> rquickjs::Result<()> {
        let type_id = TypeId::of::<T::Id>();

        let kind = if let Some(id) = self.id_map.borrow().get(&type_id) {
            *id
        } else {
            let kind = self.next_id.get();
            self.next_id.update(|id| id + 1);
            self.id_map.borrow_mut().insert(type_id, ResourceKind(kind));
            ResourceKind(kind)
        };

        let id = self.exec.create_task(None, kind);

        let ctx = ctx.clone();

        let exec = self.exec.clone();

        let task_ctx = TaskCtx::new(ctx.clone(), exec.clone(), kind, id)?;
        let exception = self.exception.clone();

        if exception.borrow().is_some() {
            return Ok(());
        }

        if let Err(err) = task_ctx.init().catch(&task_ctx.ctx) {
            exception.update(move |mut m| *m = Some(err.into()));
            return task_ctx.destroy();
        }

        ctx.spawn(async move {
            if exception.borrow().is_some() {
                task_ctx.destroy().ok();
                return;
            }

            let ctx = task_ctx.ctx.clone();

            let future = resource.run(task_ctx.clone());

            futures::select! {
                ret = future.fuse() => {
                    if let Err(err) = ret.catch(&ctx) {
                        exception.update(|mut m| *m = Some(err.into()));
                    } else {

                    }
                }
                _ = exception.subscribe().fuse() => {
                }
            }

            exec.wait_children(id).await;

            task_ctx.destroy().ok();
        });

        Ok(())
    }
}

impl AsyncState {
    pub(crate) async fn run<'js, T, U, R>(&self, ctx: Ctx<'js>, func: T) -> rquickjs::Result<R>
    where
        T: FnOnce(TaskCtx<'js>) -> U,
        U: Future<Output = rquickjs::Result<R>>,
    {
        let id = self.exec.create_task(None, ResourceKind::Root);

        let task_ctx = TaskCtx::new(ctx.clone(), self.exec.clone(), ResourceKind::Root, id)?;

        self.exec.set_current(id);

        let ret = func(task_ctx);

        futures::select! {
            ret = ret.fuse() => {
                match ret.catch(&ctx) {
                    Ok(ret) => {


                        self.exec.shutdown(id).await?;

                        self.exec.destroy_task(id);

                        if let Some(err) = &*self.exception.borrow() {
                            throw!(ctx, err)
                        }

                        Ok(ret)
                    }
                    Err(err) => {
                        let err: CaugthException = err.into();
                        self.exception.update(|mut m| *m = Some(err.clone()));
                        throw!(ctx, err)
                    }
                }
            }
            _ = self.exception.subscribe().fuse() => {
                if let Some(err) = &*self.exception.borrow() {
                    throw!(ctx, err);
                } else {
                    throw!(ctx, "Work did not finalize")
                }
            }
        }
    }
}
