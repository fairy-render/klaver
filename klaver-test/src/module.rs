use rquickjs::{Class, Function, Object, class::JsClass, module::ModuleDef};

use crate::{assert, runner::TestRunner};
use klaver_modules::module_info;

pub struct TestModule;

impl ModuleDef for TestModule {
    fn declare<'js>(decl: &rquickjs::module::Declarations<'js>) -> rquickjs::Result<()> {
        decl.declare(TestRunner::NAME)?;
        decl.declare("assert")?;
        Ok(())
    }

    fn evaluate<'js>(
        ctx: &rquickjs::Ctx<'js>,
        exports: &rquickjs::module::Exports<'js>,
    ) -> rquickjs::Result<()> {
        exports.export(
            TestRunner::NAME,
            Class::<TestRunner>::create_constructor(ctx)?,
        )?;

        let assert_obj = Object::new(ctx.clone())?;
        assert_obj.set("ok", Function::new(ctx.clone(), assert::ok)?)?;
        assert_obj.set("equal", Function::new(ctx.clone(), assert::equal)?)?;
        assert_obj.set("deepEqual", Function::new(ctx.clone(), assert::deep_equal)?)?;
        exports.export("assert", assert_obj)?;

        Ok(())
    }
}

module_info!("klaver:test" => TestModule);
