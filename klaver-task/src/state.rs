use crate::{AsyncState, HandleMap, HookListeners, exec_state::AsyncId};
use klaver_util::{
    FinalizationRegistry, FunctionExt, TypedMap,
    rquickjs::{
        self, Class, Ctx, Function, IntoJs, JsLifetime, Symbol, class::Trace, prelude::Func,
    },
};

#[rquickjs::class(crate = "rquickjs")]
pub struct HookState<'js> {
    pub registry: FinalizationRegistry<'js>,
    pub hooks: Class<'js, HookListeners<'js>>,
    pub resources: HandleMap<'js>,
    pub promise_symbol: Symbol<'js>,
}

impl<'js> HookState<'js> {
    pub fn get(ctx: &Ctx<'js>) -> rquickjs::Result<Class<'js, HookState<'js>>> {
        if let Ok(state) = ctx.globals().get("$__hooks") {
            Ok(state)
        } else {
            let state = Class::instance(ctx.clone(), HookState::new(ctx.clone())?)?;
            ctx.globals().set("$__hooks", state.clone())?;

            Ok(state)
        }
    }

    fn new(ctx: Ctx<'js>) -> rquickjs::Result<HookState<'js>> {
        let resources = HandleMap {
            handles: TypedMap::new(ctx.clone())?,
        };
        let hooks = Class::instance(ctx.clone(), HookListeners::new(resources.clone())?)?;

        let handler = Func::new(
            |ctx: Ctx<'js>, hooks: Class<'js, HookListeners<'js>>, id: AsyncId| {
                let state = AsyncState::get(&ctx)?;

                hooks.borrow_mut().destroy(&ctx, id)?;

                state.exec.destroy_task(id);

                rquickjs::Result::Ok(())
            },
        )
        .into_js(&ctx)?
        .get::<Function<'js>>()?
        .bind(&ctx, (ctx.globals(), hooks.clone()))?;

        let registry = FinalizationRegistry::new(ctx.clone(), handler)?;

        let promise_symbol = ctx.eval("Symbol()")?;

        Ok(HookState {
            registry,
            hooks,
            resources,
            promise_symbol,
        })
    }
}

impl<'js> Trace<'js> for HookState<'js> {
    fn trace<'a>(&self, tracer: klaver_util::rquickjs::class::Tracer<'a, 'js>) {
        self.registry.trace(tracer);
        self.hooks.trace(tracer);
        self.resources.trace(tracer);
        self.promise_symbol.trace(tracer);
    }
}

unsafe impl<'js> JsLifetime<'js> for HookState<'js> {
    type Changed<'to> = HookState<'to>;
}
