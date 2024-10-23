use klaver::Vm;
use rquickjs::{Class, Ctx, Module, Object};

use crate::{console::Console, performance::Performance};

pub async fn init_globals<'js>(ctx: Ctx<'js>) -> rquickjs::Result<()> {
    let globals = ctx.globals();

    if globals.contains_key("__engine")? {
        return Ok(());
    }

    let module = Module::import(&ctx, "@klaver/wintercg")?
        .into_future::<Object>()
        .await?;

    for k in module.keys::<String>() {
        let k = k?;
        if k.as_str() == "Performance" || k.as_str() == "Console" || k.as_str() == "Client" {
            continue;
        }
        let value = module.get::<_, rquickjs::Value>(&k)?;
        globals.set(k, value)?;
    }

    let console = Class::instance(ctx.clone(), Console::new())?;
    let performance = Class::instance(ctx.clone(), Performance::new())?;

    globals.set("performance", performance)?;
    globals.set("console", console)?;

    globals.set("__engine", "klaver")?;

    Ok(())
}

pub async fn install_globals(vm: &Vm) -> Result<(), klaver::Error> {
    klaver::async_with!(vm => |ctx| {
      init_globals(ctx).await?;
      Ok(())
    })
    .await
}
