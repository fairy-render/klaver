use std::borrow::Cow;

use rquickjs::Class;
use rquickjs_modules::GlobalInfo;

use crate::{config::WINTERCG_KEY, console::Console, performance::Performance, WinterCG};

const TYPES: &'static str = include_str!(concat!(env!("OUT_DIR"), "/global.d.ts"));

pub struct Globals;

impl GlobalInfo for Globals {
    fn register(builder: &mut rquickjs_modules::GlobalBuilder<'_, Self>) {
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

            let config = Class::instance(ctx.clone(), WinterCG::new(ctx.clone())?)?;

            crate::base::register(&ctx)?;
            crate::streams::register(&ctx)?;
            crate::timers::register(&ctx, &config)?;

            #[cfg(feature = "http")]
            crate::http::register(&ctx, &config)?;

            #[cfg(feature = "encoding")]
            crate::encoding::register(&ctx)?;

            #[cfg(feature = "crypto")]
            crate::encoding::register(&ctx)?;

            #[cfg(feature = "icu")]
            crate::intl::register(&ctx)?;

            let console = Class::instance(ctx.clone(), Console::new())?;
            let performance = Class::instance(ctx.clone(), Performance::new())?;

            globals.set("performance", performance)?;
            globals.set("console", console)?;

            let process = crate::process::process(ctx.clone(), &config)?;
            globals.set("process", process)?;

            globals.set(WINTERCG_KEY, config)?;

            Ok(())
        }
    }
}
