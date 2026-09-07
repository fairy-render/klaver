use std::pin::Pin;

use klaver_core::{throw, value::StringExt};
use rquickjs::{
    CatchResultExt, Class, Ctx, Function, IntoJs, JsLifetime, Object, Result, String, Value,
    class::{JsClass, Trace, Tracer, Writable},
    function::Constructor,
    prelude::{Async, Func, This},
};

use crate::reporter::Reporter;

pub struct TestRunner<'js> {
    suites: Vec<Class<'js, Suite<'js>>>,
    result: Vec<Class<'js, Suite<'js>>>,
    reporter: Reporter,
}

impl<'js> Trace<'js> for TestRunner<'js> {
    fn trace<'a>(&self, tracer: Tracer<'a, 'js>) {
        self.suites.trace(tracer);
        self.result.trace(tracer);
    }
}

unsafe impl<'js> JsLifetime<'js> for TestRunner<'js> {
    type Changed<'to> = TestRunner<'to>;
}

impl<'js> TestRunner<'js> {
    pub fn push(&mut self, ctx: &Ctx<'js>, name: String<'js>) -> Result<()> {
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
            panic!("`it` called outside of a `describe` block")
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

    pub async fn run(&self, ctx: &Ctx<'js>) -> Result<()> {
        for suite in &self.result {
            suite
                .borrow()
                .run(ctx, &self.reporter, 0, std::string::String::new())
                .await?;
        }

        println!("\n{}", self.reporter.summary());

        if self.reporter.failed() > 0 {
            throw!(ctx, self.reporter.failure_report())
        }

        Ok(())
    }
}

impl<'js> JsClass<'js> for TestRunner<'js> {
    const NAME: &'static str = "TestRunner";

    type Mutable = Writable;

    fn constructor(ctx: &Ctx<'js>) -> Result<Option<Constructor<'js>>> {
        let ctor = Constructor::new_class::<TestRunner, _, _>(ctx.clone(), || TestRunner {
            suites: Default::default(),
            result: Default::default(),
            reporter: Reporter::new(),
        })?;

        Ok(Some(ctor))
    }

    fn prototype(ctx: &Ctx<'js>) -> Result<Option<Object<'js>>> {
        let obj = Object::new(ctx.clone())?;

        let desc = Func::new(
            |ctx: Ctx<'js>,
             This(this): This<Class<'js, Self>>,
             desc: String<'js>,
             func: Function<'js>| {
                this.borrow_mut().push(&ctx, desc)?;
                let ret = func.call::<_, ()>(());
                this.borrow_mut().pop();

                ret?;

                Result::Ok(this)
            },
        );

        let it = Func::new(
            |This(this): This<Class<'js, Self>>, desc: String<'js>, func: Function<'js>| {
                this.borrow().push_test(TestDesc { desc, func });
                Result::Ok(this)
            },
        );

        let run = Func::new(Async(
            |ctx: Ctx<'js>, This(this): This<Class<'js, Self>>| async move {
                this.borrow().run(&ctx).await?;
                Result::Ok(())
            },
        ));

        obj.set("describe", desc)?;
        obj.set("it", it)?;
        obj.set("run", run)?;

        Ok(Some(obj))
    }
}

impl<'js> IntoJs<'js> for TestRunner<'js> {
    fn into_js(self, ctx: &Ctx<'js>) -> Result<Value<'js>> {
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
        reporter: &'a Reporter,
        depth: usize,
        path: std::string::String,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + 'a>> {
        Box::pin(async move {
            let desc = self.desc.str_ref()?;
            let path = if path.is_empty() {
                desc.as_str().to_string()
            } else {
                format!("{path} > {}", desc.as_str())
            };

            reporter.enter_suite(depth, desc.as_str());

            for test in &self.tests {
                let test_desc = test.desc.str_ref()?;
                let test_path = format!("{path} > {}", test_desc.as_str());

                let outcome: Result<()> = match test.func.call::<_, Value<'js>>(()) {
                    Ok(ret) => match ret.as_promise() {
                        Some(promise) => promise.clone().into_future::<()>().await,
                        None => Ok(()),
                    },
                    Err(err) => Err(err),
                };

                match outcome.catch(ctx) {
                    Ok(()) => reporter.pass(depth + 1, test_desc.as_str()),
                    Err(err) => reporter.fail(depth + 1, test_path, err.into()),
                }
            }

            for child in &self.children {
                child
                    .borrow()
                    .run(ctx, reporter, depth + 1, path.clone())
                    .await?;
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
