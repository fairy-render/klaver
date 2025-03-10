use rquickjs::{
    class::Trace, runtime::AsyncWeakRuntime, AsyncContext, CatchResultExt, Ctx, JsLifetime,
};
use rquickjs_modules::Environ;
use rquickjs_util::{throw, StringRef, Val};

use crate::{async_with, Vm};

#[derive(JsLifetime)]
pub struct JsRealm {
    runtime: AsyncWeakRuntime,
    env: Environ,
}

impl JsRealm {
    pub fn new(runtime: AsyncWeakRuntime, env: Environ) -> JsRealm {
        JsRealm { runtime, env }
    }

    pub async fn create(&self, ctx: Ctx<'_>) -> rquickjs::Result<JsRealmVm> {
        let Some(runtime) = self.runtime.try_ref() else {
            throw!(ctx, "Could not acquire runtime")
        };

        let context = AsyncContext::full(&runtime).await?;

        if let Err(err) = self.env.init(&context).await {
            throw!(ctx, err)
        }

        Ok(JsRealmVm { vm: context })
    }
}

#[derive(JsLifetime)]
#[rquickjs::class]
pub struct JsRealmVm {
    vm: AsyncContext,
}

impl<'js> Trace<'js> for JsRealmVm {
    fn trace<'a>(&self, _tracer: rquickjs::class::Tracer<'a, 'js>) {}
}

#[rquickjs::methods]
impl JsRealmVm {
    #[qjs(rename = "evalScript")]
    pub async fn eval_script<'js>(
        &self,
        ctx: Ctx<'js>,
        script: StringRef<'js>,
    ) -> rquickjs::Result<Val> {
        let ret = async_with!(self.vm => |ctx| {
            Ok(ctx.eval_promise(script.as_str()).catch(&ctx)?.into_future::<Val>().await.catch(&ctx)?)
        })
        .await;

        match ret {
            Err(err) => throw!(ctx, err),
            Ok(ret) => Ok(ret),
        }
    }
}

#[rquickjs::function]
pub async fn create_realm<'js>(ctx: Ctx<'js>) -> rquickjs::Result<JsRealmVm> {
    let Some(realm) = ctx.userdata::<JsRealm>() else {
        throw!(ctx, "Realm not found")
    };

    realm.create(ctx.clone()).await
}
