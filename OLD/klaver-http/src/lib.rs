use std::borrow::Cow;

use rquickjs_modules::ModuleInfo;

mod module;
mod router;
mod serve;

pub struct Module;

impl ModuleInfo for Module {
    const NAME: &'static str = "@klaver/http";

    fn typings() -> Option<std::borrow::Cow<'static, str>> {
        Some(Cow::Borrowed(include_str!("../http.d.ts")))
    }

    fn register(modules: &mut rquickjs_modules::ModuleBuilder<'_, Self>) {
        modules.register::<module::HttpModule>();
    }
}
