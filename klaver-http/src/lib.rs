use rquickjs_modules::ModuleInfo;

mod module;
mod serve;

pub struct Module;

impl ModuleInfo for Module {
    const NAME: &'static str = "@klaver/http";
    fn register(modules: &mut rquickjs_modules::ModuleBuilder<'_, Self>) {
        modules.register::<module::HttpModule>();
    }
}
