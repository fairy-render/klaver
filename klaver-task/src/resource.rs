use std::{marker::PhantomData, process::Output};

use klaver_util::rquickjs::{self, Ctx, Function};

use crate::{async_id::ResourceId, chan::Hook};

pub struct TaskCtx<'js> {
    pub ctx: Ctx<'js>,
    pub id: ResourceId,
    pub chan: flume::Sender<Hook>,
}

impl<'js> TaskCtx<'js> {
    pub async fn invoke_callback<A, R>(&self, cb: Function<'js>, args: A) -> rquickjs::Result<R> {
        todo!()
    }
}

pub trait Resource<'js>: Sized {
    fn run(&self, ctx: TaskCtx<'js>) -> impl Future<Output = rquickjs::Result<()>> + 'js;
}
