use klaver_base::{Exportable, Registry};
use klaver_modules::WeakEnviron;
use rquickjs::{class::{JsClass, Trace}, context::EvalOptions, qjs, CatchResultExt, Class, Ctx, JsLifetime, Promise, Value};
use klaver_util::{StringRef, throw, throw_if};

use crate::{Vm, async_with};

#[derive(JsLifetime)]
#[rquickjs::class(rename = "Vm")]
pub struct JsVm {
    vm: Vm,
}

impl<'js> Trace<'js> for JsVm {
    fn trace<'a>(&self, _tracer: rquickjs::class::Tracer<'a, 'js>) {}
}



#[rquickjs::methods]
impl JsVm {
    #[qjs(constructor)]
    fn new(ctx: Ctx<'_>) -> rquickjs::Result<JsVm> {
        throw!(ctx, "Vm should instantiated with the open static function")
    }

    #[qjs(static)]
    pub async fn open<'js>(ctx: Ctx<'js>) -> rquickjs::Result<JsVm> {
        let Some(env) = ctx.userdata::<WeakEnviron>() else {
            throw!(ctx, "Environment not registered")
        };

        let env = env.clone().upgrade(&ctx)?;

        let vm = throw_if!(ctx, Vm::new(&env, Default::default()).await);

        Ok(JsVm { vm })
    }

    #[qjs(rename = "evalPath")]
    pub async fn eval_path<'js>(
        &self,
        ctx: Ctx<'js>,
        script_path: StringRef<'js>,
    ) -> rquickjs::Result<Value<'js>> {
        let ret = async_with!(self.vm => |ctx| {

          let mut opts = EvalOptions::default();
          opts.promise = true;

          let value = ctx.eval_file_with_options::<Promise, _>(script_path.as_str(), opts).catch(&ctx)?.into_future::<Value>().await.catch(&ctx)?;
          let registry = Registry::get(&ctx)?;

          let data = registry.serialize(&ctx, &value, &Default::default()).catch(&ctx)?;
    
          Ok(data)
        })
        .await;

        let ret = throw_if!(ctx, ret);

        let registry = Registry::get(&ctx)?;

        let value = registry.deserialize(&ctx, ret)?;

        Ok(value)
    }

    pub async fn eval<'js>(
        &self,
        ctx: Ctx<'js>,
        script: StringRef<'js>,
    ) -> rquickjs::Result<Value<'js>> {
        let ret = async_with!(self.vm => |ctx| {

          let value = ctx.eval_promise(script.as_str()).catch(&ctx)?.into_future::<Value>().await.catch(&ctx)?;
          let registry = Registry::get(&ctx)?;
          
          let data = registry.serialize(&ctx, &value, &Default::default()).catch(&ctx)?;
    
          Ok(data)
        })
        .await;

        let ret = throw_if!(ctx, ret);

        let registry = Registry::get(&ctx)?;
        let value = registry.deserialize(&ctx, ret)?;

        Ok(value)
    }
}


impl<'js> Exportable<'js> for JsVm {
    fn export<T>(ctx: &Ctx<'js>, _registry: &Registry, target: &T) -> rquickjs::Result<()>
    where
        T: klaver_base::ExportTarget<'js> {
        target.set(ctx, JsVm::NAME, Class::<JsVm>::create_constructor(ctx)?)?;
        Ok(())
    }
}