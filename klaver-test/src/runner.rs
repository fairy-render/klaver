use std::pin::Pin;

use klaver_util::{
    FunctionExt, StringExt,
    rquickjs::{
        self, Class, Ctx, Function, IntoJs, JsLifetime, Object, String, Value,
        class::{JsClass, Trace, Tracer, Writable},
        function::Constructor,
        prelude::{Async, Func, Opt, This},
    },
};

use crate::reporter::Reporter;

pub struct TestRunner<'js> {
    suites: Vec<Class<'js, Suite<'js>>>,
    result: Vec<Class<'js, Suite<'js>>>,
    report: Reporter<'js>,
}

impl<'js> Trace<'js> for TestRunner<'js> {
    fn trace<'a>(&self, tracer: Tracer<'a, 'js>) {
        self.suites.trace(tracer);
        self.result.trace(tracer);
        self.report.trace(tracer);
    }
}

unsafe impl<'js> JsLifetime<'js> for TestRunner<'js> {
    type Changed<'to> = TestRunner<'to>;
}

impl<'js> TestRunner<'js> {
    pub fn push(&mut self, ctx: &Ctx<'js>, name: String<'js>) -> rquickjs::Result<()> {
        let suite = Class::instance(
            ctx.clone(),
            Suite {
                parent: !self.suites.is_empty(),
                desc: name,
                tests: Default::default(),
                children: Default::default(),
            },
        )?;
        if let Some(parent) = self.suites.last() {
            parent.borrow_mut().children.push(suite.clone());
        }
        self.suites.push(suite);

        Ok(())
    }

    pub fn push_test(&self, test: TestDesc<'js>) {
        let Some(parent) = self.suites.last() else {
            panic!("No active suite")
        };
        parent.borrow_mut().tests.push(test)
    }

    pub fn pop(&mut self) {
        if let Some(last) = self.suites.pop() {
            if !last.borrow().parent {
                self.result.push(last);
            }
        }
    }

    pub async fn run(&self, ctx: &Ctx<'js>) -> rquickjs::Result<()> {
        self.report.prepare(ctx, &self.result)?;

        for suite in &self.result {
            suite.borrow().run(ctx, &self.report).await?;
        }

        Ok(())
    }
}

impl<'js> JsClass<'js> for TestRunner<'js> {
    const NAME: &'static str = "TestRunner";

    type Mutable = Writable;

    fn constructor(
        ctx: &rquickjs::Ctx<'js>,
    ) -> rquickjs::Result<Option<rquickjs::function::Constructor<'js>>> {
        let ctor = Constructor::new_class::<TestRunner, _, _>(
            ctx.clone(),
            |reporter: Opt<Reporter<'js>>| {
                //

                let report = reporter.0.unwrap_or_else(|| Reporter { ts: None });

                TestRunner {
                    suites: Default::default(),
                    result: Default::default(),
                    report,
                }
            },
        )?;

        Ok(Some(ctor))
    }

    fn prototype(ctx: &rquickjs::Ctx<'js>) -> rquickjs::Result<Option<rquickjs::Object<'js>>> {
        let obj = Object::new(ctx.clone())?;

        let desc = Func::new(
            |ctx: Ctx<'js>,
             This(this): This<Class<'js, Self>>,
             desc: String<'js>,
             func: Function<'js>| {
                this.borrow_mut().push(&ctx, desc)?;
                let ret = func.call::<_, ()>(());
                this.borrow_mut().pop();

                let _ = ret?;

                rquickjs::Result::Ok(this)
            },
        );

        let it = Func::new(
            |ctx: Ctx<'js>,
             This(this): This<Class<'js, Self>>,
             desc: String<'js>,
             func: Function<'js>| {
                this.borrow_mut().push_test(TestDesc { desc, func });
                rquickjs::Result::Ok(this)
            },
        );

        let run = Func::new(Async(
            |ctx: Ctx<'js>, This(this): This<Class<'js, Self>>| async move {
                this.borrow().run(&ctx).await?;
                rquickjs::Result::Ok(())
            },
        ));

        obj.set("describe", desc)?;
        obj.set("it", it)?;
        obj.set("run", run)?;

        Ok(Some(obj))
    }
}

impl<'js> IntoJs<'js> for TestRunner<'js> {
    fn into_js(self, ctx: &Ctx<'js>) -> rquickjs::Result<Value<'js>> {
        Class::instance(ctx.clone(), self).into_js(ctx)
    }
}

#[rquickjs::class(crate = "rquickjs")]
pub struct Suite<'js> {
    parent: bool,
    desc: String<'js>,
    tests: Vec<TestDesc<'js>>,
    children: Vec<Class<'js, Suite<'js>>>,
}

impl<'js> Suite<'js> {
    fn run<'a>(
        &'a self,
        ctx: &'a Ctx<'js>,
        reporter: &'a Reporter<'js>,
    ) -> Pin<Box<dyn Future<Output = rquickjs::Result<()>> + 'a>> {
        Box::pin(async move {
            println!("Suite {}", self.desc.str_ref()?);
            for test in &self.tests {
                println!("  {}", test.desc.str_ref()?);
                test.func.call_async::<_, ()>(()).await?;
            }
            for child in &self.children {
                child.borrow().run(ctx, reporter).await?;
            }
            Ok(())
        })
    }
}

impl<'js> Trace<'js> for Suite<'js> {
    fn trace<'a>(&self, tracer: Tracer<'a, 'js>) {
        self.tests.trace(tracer);
        self.desc.trace(tracer);
        self.children.trace(tracer);
    }
}

unsafe impl<'js> JsLifetime<'js> for Suite<'js> {
    type Changed<'to> = Suite<'to>;
}

pub struct TestDesc<'js> {
    desc: String<'js>,
    func: Function<'js>,
}

impl<'js> Trace<'js> for TestDesc<'js> {
    fn trace<'a>(&self, tracer: Tracer<'a, 'js>) {
        self.desc.trace(tracer);
        self.func.trace(tracer);
    }
}
