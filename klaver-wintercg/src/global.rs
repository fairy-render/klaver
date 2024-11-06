use std::borrow::Cow;

use rquickjs::{Class, Object, Promise, Value};
use rquickjs_modules::GlobalInfo;

use crate::{config::WINTERCG_KEY, console::Console, performance::Performance};

const TYPES: &'static str = include_str!(concat!(env!("OUT_DIR"), "/global.d.ts"));

pub struct Globals;

impl GlobalInfo for Globals {
    fn register(builder: &mut rquickjs_modules::GlobalBuilder<'_, Self>) {
        builder.dependency::<crate::Module>();
        builder.register(Global)
    }

    fn typings() -> Option<std::borrow::Cow<'static, str>> {
        Some(Cow::Borrowed(TYPES))
    }
}

struct Global;

impl rquickjs_modules::Global for Global {
    fn define<'a, 'js: 'a>(
        &'a self,
        ctx: rquickjs::Ctx<'js>,
    ) -> impl std::future::Future<Output = rquickjs::Result<()>> + 'a {
        async move {
            let globals = ctx.globals();

            if globals.contains_key(WINTERCG_KEY)? {
                return Ok(());
            }

            let module = ctx
                .eval::<Promise, _>("import('@klaver/wintercg')")?
                .into_future::<Object>()
                .await?;

            for k in module.keys::<String>() {
                let k = k?;
                if k.as_str() == "Performance"
                    || k.as_str() == "Console"
                    || k.as_str() == "Client"
                    || k.as_str() == "config"
                {
                    continue;
                }
                let value = module.get::<_, rquickjs::Value>(&k)?;
                globals.set(k, value)?;
            }

            let console = Class::instance(ctx.clone(), Console::new())?;
            let performance = Class::instance(ctx.clone(), Performance::new())?;
            let config: Value = module.get("config")?;

            globals.set("performance", performance)?;
            globals.set("console", console)?;

            globals.set(WINTERCG_KEY, config)?;

            Ok(())
        }
    }
}
