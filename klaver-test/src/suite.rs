use futures::future::LocalBoxFuture;
use klaver_util::{
    FunctionExt, NativeIteratorExt, StringRef, TypedList, TypedMap, TypedMultiMap,
    rquickjs::{
        self, Class, Ctx, FromJs, Function, IntoJs, JsLifetime, Object, Result, String, Value,
        class::Trace, prelude::This,
    },
};

use crate::reporter::Reporter;

#[rquickjs::class(crate = "rquickjs")]
pub struct Suites<'js> {
    stack: Vec<Suite<'js>>,
    result: Vec<Suite<'js>>,
}

unsafe impl<'js> JsLifetime<'js> for Suites<'js> {
    type Changed<'to> = Suites<'to>;
}

impl<'js> Trace<'js> for Suites<'js> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        self.stack.trace(tracer)
    }
}

impl<'js> Suites<'js> {
    pub fn push(&mut self, suite: Suite<'js>) {
        let idx = self.stack.len();
        if let Some(parent) = self.stack.last_mut() {
            parent.children.push(idx);
        }
        self.stack.push(suite);
    }

    pub fn push_test(&mut self, test: TestDesc<'js>) {
        let idx = self.stack.len();
        let Some(parent) = self.stack.last_mut() else {
            panic!("No active suite")
        };
        parent.tests.push(test)
    }

    pub fn pop(&mut self) {
        if let Some(last) = self.stack.pop() {
            self.result.push(last);
        }
    }
}

pub struct Suite<'js> {
    description: String<'js>,
    tests: Vec<TestDesc<'js>>,
    children: Vec<usize>,
}

impl<'js> Trace<'js> for Suite<'js> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        self.tests.trace(tracer)
    }
}

pub struct TestDesc<'js> {
    desc: String<'js>,
    func: Function<'js>,
}

impl<'js> Trace<'js> for TestDesc<'js> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        self.desc.trace(tracer);
        self.func.trace(tracer);
    }
}

pub struct SuiteDesc<'js> {
    desc: String<'js>,
    func: Function<'js>,
}

pub fn describe<'js>(
    ctx: Ctx<'js>,
    suites: Class<'js, Suites<'js>>,
    desc: String<'js>,
    func: Function<'js>,
) -> rquickjs::Result<()> {
    suites.borrow_mut().push(Suite {
        tests: Vec::default(),
        children: Vec::default(),
        description: desc,
    });

    let ret = func.call::<_, ()>(());

    suites.borrow_mut().pop();

    ret
}

pub fn it<'js>(
    ctx: Ctx<'js>,
    suites: Class<'js, Suites<'js>>,
    desc: String<'js>,
    func: Function<'js>,
) -> rquickjs::Result<()> {
    suites.borrow_mut().push_test(TestDesc { desc, func });
    Ok(())
}

// impl<'js> Test<'js> {
//     pub fn run<'a>(
//         &'a self,
//         runner: &'a TestRunner<'js>,
//         ctx: &'a Ctx<'js>,
//     ) -> LocalBoxFuture<'a, rquickjs::Result<()>> {
//         Box::pin(async move {
//             let runner = Class::instance(
//                 ctx.clone(),
//                 TestRunner {
//                     tests: TypedList::new(ctx.clone())?,
//                     reports: runner.reports.clone(),
//                 },
//             )?;

//             self.func.call_async::<_, ()>((runner.clone(),)).await?;

//             runner.borrow().run(ctx.clone()).await?;

//             Ok(())
//         })
//     }
// }

impl<'js> Trace<'js> for SuiteDesc<'js> {
    fn trace<'a>(&self, tracer: klaver_util::rquickjs::class::Tracer<'a, 'js>) {
        self.desc.trace(tracer);
        self.func.trace(tracer);
    }
}

impl<'js> IntoJs<'js> for SuiteDesc<'js> {
    fn into_js(self, ctx: &Ctx<'js>) -> Result<rquickjs::Value<'js>> {
        let obj = Object::new(ctx.clone())?;
        obj.set("description", self.desc)?;
        obj.set("test", self.func)?;
        Ok(obj.into_value())
    }
}

impl<'js> FromJs<'js> for SuiteDesc<'js> {
    fn from_js(ctx: &Ctx<'js>, value: rquickjs::Value<'js>) -> Result<Self> {
        // let obj: Object = value.get()?;

        // Ok(SuiteDesc { desc: (), func: () })
        todo!()
    }
}
