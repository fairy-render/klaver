use klaver::{modules::ModuleInfo, quick::CatchResultExt, vm::Vm};

pub struct Compat;

const COMPAT: &[u8] = include_bytes!("compat.js");

impl ModuleInfo for Compat {
    fn register(modules: &mut klaver::modules::Builder<'_>) {
        modules.register_src("@klaver/compat", COMPAT.to_vec());
    }
}

pub async fn init(vm: &Vm) -> Result<(), klaver::Error> {
    klaver::async_with!(vm => |ctx| {
        ctx.eval_promise(r#"await (await import("@klaver/compat")).default(globalThis)"#).catch(&ctx)?.into_future().await.catch(&ctx)?;
        Ok(())
    })
    .await?;

    Ok(())
}
