use klaver_structuredclone::{
    Clonable, Registry, StringCloner, StructuredClone, Tag, set_tag, structured_clone,
};
use rquickjs::{
    CatchResultExt, Class, Context, Ctx, JsLifetime, Runtime, String, Value, class::Trace,
    prelude::Func,
};
use rquickjs_util::{RuntimeError, format::format, util::is_plain_object};

fn is_object<'js>(ctx: Ctx<'js>, value: Value<'js>) -> rquickjs::Result<bool> {
    is_plain_object(&ctx, &value)
}

#[derive(Trace, JsLifetime)]
#[rquickjs::class]
pub struct TestClass<'js> {
    #[qjs(get, set, enumerable)]
    name: String<'js>,
}

#[rquickjs::methods]
impl<'js> TestClass<'js> {
    #[qjs(constructor)]
    fn new(name: String<'js>) -> rquickjs::Result<TestClass<'js>> {
        Ok(TestClass { name })
    }
}

pub struct TestClassCloner;

impl StructuredClone for TestClassCloner {
    fn tag() -> &'static Tag {
        static TAG: Tag = Tag::new();
        &TAG
    }

    type Item<'js> = Class<'js, TestClass<'js>>;

    fn from_transfer_object<'js>(
        ctx: &Ctx<'js>,
        registry: &Registry,
        obj: klaver_structuredclone::TransferData,
    ) -> rquickjs::Result<Self::Item<'js>> {
        let name = StringCloner::from_transfer_object(ctx, registry, obj)?;
        Class::instance(ctx.clone(), TestClass { name })
    }

    fn to_transfer_object<'js>(
        ctx: &Ctx<'js>,
        registry: &Registry,
        value: &Self::Item<'js>,
    ) -> rquickjs::Result<klaver_structuredclone::TransferData> {
        StringCloner::to_transfer_object(ctx, registry, &value.borrow().name)
    }
}

impl<'js> Clonable for TestClass<'js> {
    type Cloner = TestClassCloner;
}

fn main() -> Result<(), RuntimeError> {
    let runtime = Runtime::new()?;
    let context = Context::full(&runtime)?;

    context.with(|ctx| {
        //
        let mut registry = Registry::new()?;
        registry.register::<TestClass>()?;

        ctx.store_userdata(registry).unwrap();

        ctx.globals()
            .set("structuredClone", Func::new(structured_clone))?;

        ctx.globals().set(
            "print",
            Func::new(|m: Value<'_>| {
                println!("{}", format(m.ctx().clone(), m, None)?);
                rquickjs::Result::Ok(())
            }),
        )?;

        ctx.globals().set("isPlainObject", Func::from(is_object))?;

        Class::<TestClass>::define(&ctx.globals())?;
        set_tag::<TestClass>(&ctx)?;

        ctx.eval::<(), _>(include_str!("./example.js"))
            .catch(&ctx)?;

        let date = ctx.eval::<Value, _>("new Date()")?;

        // println!("Is date {}", Date::is(&ctx, &date)?);

        Result::<_, RuntimeError>::Ok(())
    })?;

    Ok(())
}
