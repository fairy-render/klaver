use klaver_util::{
    rquickjs::{
        self, Class, Ctx, Function, IntoJs, JsLifetime, Value,
        class::{JsClass, Readable, Trace},
        function::{Args, ParamRequirement},
        prelude::Rest,
    },
    throw,
};

use crate::{AsyncId, TaskCtx, exec_state::ExecState};

#[rquickjs::class(crate = "rquickjs")]
pub struct Snapshot<'js> {
    pub id: AsyncId,
    pub exec: Option<TaskCtx<'js>>,
}

unsafe impl<'js> JsLifetime<'js> for Snapshot<'js> {
    type Changed<'to> = Snapshot<'to>;
}

impl<'js> Trace<'js> for Snapshot<'js> {
    fn trace<'a>(&self, tracer: klaver_util::rquickjs::class::Tracer<'a, 'js>) {
        self.exec.trace(tracer);
    }
}

impl<'js> Drop for Snapshot<'js> {
    fn drop(&mut self) {
        if let Some(exec) = self.exec.take() {
            // exec.destroy().ok();
        }
    }
}

#[rquickjs::function(crate = "rquickjs")]
pub fn snapshot<'js>(
    ctx: Ctx<'js>,
    snapshot: Class<'js, Snapshot<'js>>,
    func: Function<'js>,
    args: Rest<Value<'js>>,
) -> rquickjs::Result<Value<'js>> {
    let Some(task_ctx) = snapshot.borrow().exec.clone() else {
        throw!(ctx, "Task not active")
    };

    let mut builder = Args::new(ctx.clone(), args.len());
    builder.push_args(args.iter())?;

    task_ctx.invoke_callback_arg(func, builder)
}
