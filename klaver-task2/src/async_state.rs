use futures::FutureExt;
use klaver_util::{
    CaugthException,
    rquickjs::{self, CatchResultExt, Ctx, JsLifetime, String},
    throw, throw_if,
};
use std::rc::Rc;

use crate::{
    cell::ObservableRefCell,
    exec_state::ExecState,
    resource::{Resource, TaskCtx},
};

#[derive(Clone)]
pub struct AsyncState {
    pub(crate) exec: ExecState,
    pub(crate) exception: Rc<ObservableRefCell<Option<CaugthException>>>,
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
                        exception: Rc::new(ObservableRefCell::new(None))
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
        // let parent_id = self.exec.trigger_async_id();
        let id = self.exec.create_task(None, false);

        let ctx = ctx.clone();

        let ty = String::from_str(ctx.clone(), resource.ty())?;

        let exec = self.exec.clone();

        let task_ctx = TaskCtx::new(ctx.clone(), exec.clone(), ty, id)?;
        let exception = self.exception.clone();

        if exception.borrow().is_some() {
            return Ok(());
        }

        if let Err(err) = task_ctx.init().catch(&task_ctx.ctx) {
            exception.update(move |mut m| *m = Some(err.into()));
            exec.destroy_task(id, false);
            return Ok(());
        }

        ctx.spawn(async move {
            if exception.borrow().is_some() {
                exec.destroy_task(id, false);
                return;
            }

            let ctx = task_ctx.ctx.clone();

            let future = resource.run(task_ctx);

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

            exec.destroy_task(id, false);
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
        let id = self.exec.create_task(None, false);

        let ty = String::from_str(ctx.clone(), "entry")?;
        let task_ctx = TaskCtx::new(ctx.clone(), self.exec.clone(), ty, id)?;

        // let current_id = self.exec.trigger_async_id();

        println!("ROOT {:?}", id);

        self.exec.set_current(id);

        let ret = func(task_ctx);

        futures::select! {
            ret = ret.fuse() => {
                match ret.catch(&ctx) {
                    Ok(ret) => {

                         self.dump();

                        self.exec.shutdown(id).await?;

                        self.exec.destroy_task(id, false);

                         self.dump();


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
