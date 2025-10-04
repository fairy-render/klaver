use futures::future::LocalBoxFuture;
use klaver_util::{
    FunctionExt, NativeIteratorExt, StringRef, TypedList, TypedMap, TypedMultiMap,
    rquickjs::{
        self, Class, Ctx, FromJs, Function, IntoJs, JsLifetime, Object, Result, String, Value,
        class::Trace, prelude::This,
    },
};

use crate::reporter::Reporter;

pub struct Test<'js> {
    desc: String<'js>,
    func: Function<'js>,
}

impl<'js> Test<'js> {
    pub fn run<'a>(
        &'a self,
        runner: &'a TestRunner<'js>,
        ctx: &'a Ctx<'js>,
    ) -> LocalBoxFuture<'a, rquickjs::Result<()>> {
        Box::pin(async move {
            let runner = Class::instance(
                ctx.clone(),
                TestRunner {
                    tests: TypedList::new(ctx.clone())?,
                    reports: runner.reports.clone(),
                },
            )?;

            self.func.call_async::<_, ()>((runner.clone(),)).await?;

            runner.borrow().run(ctx.clone()).await?;

            Ok(())
        })
    }
}

impl<'js> Trace<'js> for Test<'js> {
    fn trace<'a>(&self, tracer: klaver_util::rquickjs::class::Tracer<'a, 'js>) {
        self.desc.trace(tracer);
        self.func.trace(tracer);
    }
}

impl<'js> IntoJs<'js> for Test<'js> {
    fn into_js(self, ctx: &Ctx<'js>) -> Result<rquickjs::Value<'js>> {
        let obj = Object::new(ctx.clone())?;
        obj.set("description", self.desc)?;
        obj.set("test", self.func)?;
        Ok(obj.into_value())
    }
}

impl<'js> FromJs<'js> for Test<'js> {
    fn from_js(_ctx: &Ctx<'js>, value: rquickjs::Value<'js>) -> Result<Self> {
        let obj: Object<'js> = value.get()?;

        Ok(Self {
            desc: obj.get("description")?,
            func: obj.get("test")?,
        })
    }
}

#[rquickjs::class(crate = "rquickjs")]
pub struct TestRunner<'js> {
    tests: TypedList<'js, Test<'js>>,
    reports: Class<'js, Reporter<'js>>,
}

impl<'js> Trace<'js> for TestRunner<'js> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        self.tests.trace(tracer);
    }
}

unsafe impl<'js> JsLifetime<'js> for TestRunner<'js> {
    type Changed<'to> = TestRunner<'to>;
}

#[rquickjs::methods(crate = "rquickjs")]
impl<'js> TestRunner<'js> {
    pub fn describe(
        This(this): This<Class<'js, Self>>,
        desc: String<'js>,
        func: Function<'js>,
    ) -> rquickjs::Result<Class<'js, Self>> {
        let test = Test { desc, func };

        this.borrow_mut().tests.push(test)?;

        Ok(this)
    }

    pub async fn run(&self, ctx: Ctx<'js>) -> rquickjs::Result<()> {
        for test in self.tests.values()?.into_iter(&ctx) {
            let test = test?;
            test.run(self, &ctx).await?;
        }
        Ok(())
    }
}
