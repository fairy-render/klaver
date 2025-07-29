use klaver_util::{
    FunctionExt, ObjectExt, TypedMap,
    rquickjs::{
        self, Class, Ctx, Function, IntoJs, JsLifetime, Object, Value, class::Trace, prelude::Func,
    },
};

use crate::{AsyncState, HandleMap, HookListeners, ResourceHandle, exec_state::AsyncId};

pub struct FinalizationRegistry<'js> {
    obj: Object<'js>,
}

impl<'js> Trace<'js> for FinalizationRegistry<'js> {
    fn trace<'a>(&self, tracer: klaver_util::rquickjs::class::Tracer<'a, 'js>) {
        self.obj.trace(tracer);
    }
}

impl<'js> FinalizationRegistry<'js> {
    pub fn new(
        ctx: Ctx<'js>,
        hooks: Class<'js, HookListeners<'js>>,
    ) -> rquickjs::Result<FinalizationRegistry<'js>> {
        let ctor: Function<'js> = ctx.eval("(handler) => new FinalizationRegistry(handler)")?;

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
        .bind(&ctx, (ctx.globals(), hooks));

        let obj = ctor.call((handler,))?;

        Ok(FinalizationRegistry { obj })
    }

    pub fn register(&self, target: Value<'js>, value: AsyncId) -> rquickjs::Result<()> {
        self.obj
            .call_property::<_, _, ()>("register", (target, value))?;

        Ok(())
    }
}

#[rquickjs::class(crate = "rquickjs")]
pub struct HookState<'js> {
    pub registry: FinalizationRegistry<'js>,
    pub hooks: Class<'js, HookListeners<'js>>,
    pub resources: HandleMap<'js>,
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
        let hooks = Class::instance(
            ctx.clone(),
            HookListeners::new(ctx.clone(), resources.clone())?,
        )?;
        let registry = FinalizationRegistry::new(ctx, hooks.clone())?;

        Ok(HookState {
            registry,
            hooks,
            resources,
        })
    }
}

impl<'js> Trace<'js> for HookState<'js> {
    fn trace<'a>(&self, tracer: klaver_util::rquickjs::class::Tracer<'a, 'js>) {
        self.registry.trace(tracer);
        self.hooks.trace(tracer);
        self.resources.trace(tracer);
    }
}

unsafe impl<'js> JsLifetime<'js> for HookState<'js> {
    type Changed<'to> = HookState<'to>;
}
