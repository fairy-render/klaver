use klaver_util::rquickjs::{
    self, Class, Ctx, Function, JsLifetime, String, Value,
    class::Trace,
    function::Args,
    prelude::{Rest, This},
};

use crate::{AsyncId, Context, ResourceKind, runtime::Runtime};

#[rquickjs::class(crate = "rquickjs")]
pub struct AsyncResource<'js> {
    context: Context<'js>,
    destroyed: bool,
}

impl<'js> Trace<'js> for AsyncResource<'js> {
    fn trace<'a>(&self, tracer: klaver_util::rquickjs::class::Tracer<'a, 'js>) {
        self.context.trace(tracer);
    }
}

unsafe impl<'js> JsLifetime<'js> for AsyncResource<'js> {
    type Changed<'to> = AsyncResource<'to>;
}

#[rquickjs::methods(crate = "rquickjs")]
impl<'js> AsyncResource<'js> {
    #[qjs(constructor)]
    pub fn new(ctx: Ctx<'js>, _ty: String<'js>) -> rquickjs::Result<AsyncResource<'js>> {
        let runtime = Runtime::from_ctx(&ctx)?;
        let runtime = runtime.borrow();
        let id = runtime
            .manager
            .create_task(None, ResourceKind::ASYNC_RESOURCE, false, false);

        let context = Context {
            id,
            internal: false,
            tasks: runtime.manager.clone(),
            exception: runtime.exception.clone(),
            hooks: runtime.hooks.clone(),
            ctx: ctx.clone(),
        };

        Ok(AsyncResource {
            context,
            destroyed: false,
        })
    }

    #[qjs(rename = "runInAsyncScope")]
    pub fn run_in_scope(
        this: This<Class<'js, Self>>,
        ctx: Ctx<'js>,
        func: Function<'js>,
        args: Rest<Value<'js>>,
    ) -> rquickjs::Result<Value<'js>> {
        let mut fn_args = Args::new(ctx.clone(), args.len());
        fn_args.push_args(args.0)?;
        let context = this.borrow().context.clone();
        context.invoke_callback_arg(func, fn_args)
    }

    #[qjs(rename = "asyncId")]
    pub fn async_id(&self) -> rquickjs::Result<AsyncId> {
        Ok(self.context.id())
    }

    #[qjs(rename = "emitDestroy")]
    pub fn emit_destroy(&mut self) -> rquickjs::Result<()> {
        if self.destroyed {
            return Ok(());
        }

        self.context.tasks.destroy_task(
            self.context.id,
            &self.context.ctx,
            &self.context.hooks,
            true,
        )?;

        self.destroyed = true;

        Ok(())
    }
}
