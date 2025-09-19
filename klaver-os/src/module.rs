use rquickjs::{module::ModuleDef, prelude::Func};
use sysinfo::System;

pub struct OsModule {}

impl ModuleDef for OsModule {
    fn declare<'js>(decl: &rquickjs::module::Declarations<'js>) -> rquickjs::Result<()> {
        decl.declare("arch")?;
        decl.declare("env")?;
        decl.declare("cwd")?;
        Ok(())
    }

    fn evaluate<'js>(
        ctx: &rquickjs::Ctx<'js>,
        exports: &rquickjs::module::Exports<'js>,
    ) -> rquickjs::Result<()> {
        exports.export("arch", System::cpu_arch())?;
        // exports.export("cwd", value)

        Ok(())
    }
}
