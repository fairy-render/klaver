use klaver::{
    modules::ModuleInfo,
    quick::{CatchResultExt, Class},
    vm::Vm,
};
use klaver_shared::console::Console;

pub struct Compat;

const COMPAT: &[u8] = include_bytes!("compat.js");

impl ModuleInfo for Compat {
    fn register(modules: &mut klaver::modules::Builder<'_>) {
        // Include deps
        klaver_encoding::Encoding::register(modules);
        klaver_http::Module::register(modules);
        klaver_crypto::Crypto::register(modules);
        modules.register_src("@klaver/compat", COMPAT.to_vec());
    }
}

pub async fn init(vm: &Vm) -> Result<(), klaver::Error> {
    klaver::async_with!(vm => |ctx| {
        let console = Class::instance(ctx.clone(), Console::new());
        ctx.globals().set("console", console)?;
        ctx.eval_promise(r#"await (await import("@klaver/compat")).default(globalThis)"#).catch(&ctx)?.into_future().await.catch(&ctx)?;
        Ok(())
    })
    .await?;

    Ok(())
}
