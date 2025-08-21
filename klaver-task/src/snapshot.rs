use klaver_util::{
    rquickjs::{
        Function, JsLifetime,
        class::{JsClass, Readable, Trace},
        function::{Args, ParamRequirement},
    },
    throw,
};

use crate::{AsyncId, TaskCtx, exec_state::ExecState};

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
            exec.destroy().ok();
        }
    }
}

impl<'js> JsClass<'js> for Snapshot<'js> {
    const NAME: &'static str = "Snapshot";

    type Mutable = Readable;

    const CALLABLE: bool = true;

    fn constructor(
        _ctx: &klaver_util::rquickjs::Ctx<'js>,
    ) -> klaver_util::rquickjs::Result<Option<klaver_util::rquickjs::function::Constructor<'js>>>
    {
        Ok(None)
    }

    fn call<'a>(
        this: &klaver_util::rquickjs::class::JsCell<'js, Self>,
        params: klaver_util::rquickjs::function::Params<'a, 'js>,
    ) -> klaver_util::rquickjs::Result<klaver_util::rquickjs::Value<'js>> {
        if params.is_empty() {
            throw!(params.ctx(), "Expected a function");
        }

        let len = params.len();
        let mut args = params.access();

        let func: Function<'js> = args.arg().get()?;

        let mut out_args = Args::new(args.ctx().clone(), len - 1);
        for _ in 1..len {
            let v = args.arg();
            out_args.push_arg(v)?;
        }

        let this = this.borrow();
        let Some(task) = &this.exec else {
            throw!(args.ctx(), "Task does not exts");
        };

        task.invoke_callback_arg(func, out_args)
    }
}
