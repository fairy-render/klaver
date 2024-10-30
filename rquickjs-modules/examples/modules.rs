use oxc::transformer::JsxOptions;
use rquickjs::{prelude::Func, CatchResultExt, Context, Module, Runtime};
use rquickjs_modules::{
    transformer::{Compiler, FileLoader},
    Builder, ModuleResolver,
};

fn main() -> rquickjs::Result<()> {
    let runtime = Runtime::new()?;

    let compiler = Compiler::default();
    // compiler.transform_options.

    let builder = Builder::default();

    let env = builder.build();

    runtime.set_loader(ModuleResolver::new(), FileLoader::default());

    let context = Context::full(&runtime)?;

    context.with(|ctx| {
        ctx.globals().set(
            "print",
            Func::new(|msg: String| {
                println!("{msg}");
                rquickjs::Result::Ok(())
            }),
        )?;

        Module::import(&ctx, "./rquickjs-modules/examples/test.js")?
            .finish::<()>()
            .catch(&ctx)
            .unwrap();

        rquickjs::Result::Ok(())
    })?;

    runtime.execute_pending_job().unwrap();

    Ok(())
}
