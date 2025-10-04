use crate::runner::TestRunner;
use klaver_modules::module_info;
use klaver_util::rquickjs::{Class, class::JsClass, module::ModuleDef};

pub struct TestModule;

impl ModuleDef for TestModule {
    fn declare<'js>(
        decl: &klaver_util::rquickjs::module::Declarations<'js>,
    ) -> klaver_util::rquickjs::Result<()> {
        decl.declare(TestRunner::NAME)?;
        Ok(())
    }

    fn evaluate<'js>(
        ctx: &klaver_util::rquickjs::Ctx<'js>,
        exports: &klaver_util::rquickjs::module::Exports<'js>,
    ) -> klaver_util::rquickjs::Result<()> {
        exports.export(
            TestRunner::NAME,
            Class::<TestRunner>::create_constructor(ctx)?,
        )?;
        Ok(())
    }
}

module_info!("klaver:test" => TestModule);
