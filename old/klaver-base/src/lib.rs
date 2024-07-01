use module::BASE_KEY;
use rquickjs::{Class, Ctx, Function};

mod base;
mod config;
pub mod module;
pub mod streams;

pub use self::{base::*, config::*};

pub type Module = module::js_base_mod;

pub fn register(ctx: &Ctx<'_>) -> rquickjs::Result<()> {
    rquickjs::Module::declare_def::<module::js_base_mod, _>(ctx.clone(), "@klaver/base")?;

    Ok(())
}

pub fn get_base<'js>(ctx: &Ctx<'js>) -> rquickjs::Result<Class<'js, Base<'js>>> {
    if !ctx.globals().contains_key(BASE_KEY)? {
        let base = Class::instance(ctx.clone(), Base::default())?;
        ctx.globals().prop(BASE_KEY, base)?;
    }
    ctx.globals().get(BASE_KEY)
}

pub fn get_config<F, U>(ctx: &Ctx<'_>, func: F) -> rquickjs::Result<U>
where
    F: FnOnce(&mut Config) -> rquickjs::Result<U>,
{
    let base_class = get_base(ctx)?;
    let mut base = base_class.try_borrow_mut()?;
    func(&mut base.config)
}

pub fn register_global(ctx: &Ctx<'_>) -> rquickjs::Result<()> {
    Class::<module::TextDecoder>::register(ctx)?;
    Class::<module::TextDecoder>::define(&ctx.globals())?;

    Class::<module::TextEncoder>::register(ctx)?;
    Class::<module::TextEncoder>::define(&ctx.globals())?;

    ctx.globals().set(
        "setTimeout",
        Function::new(ctx.clone(), module::base_mod::set_timeout),
    )?;

    ctx.globals().set(
        "setInterval",
        Function::new(ctx.clone(), module::base_mod::set_interval),
    )?;

    ctx.globals().set(
        "clearTimeout",
        Function::new(ctx.clone(), module::base_mod::clear_timeout),
    )?;

    ctx.globals().set(
        "clearInterval",
        Function::new(ctx.clone(), module::base_mod::clear_timeout),
    )?;

    Ok(())
}
