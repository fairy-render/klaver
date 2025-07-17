mod bindings;
mod module;
mod registry;
mod traits;
mod value;

use rquickjs::{Class, Ctx, Object, Symbol, class::JsClass};
use rquickjs_util::{StringRef, throw};

pub use self::{bindings::structured_clone, module::*, registry::Registry, traits::*, value::*};

pub fn get_symbol<'js>(ctx: &Ctx<'js>) -> rquickjs::Result<Symbol<'js>> {
    ctx.eval("Symbol.for('$Tag')")
}

pub fn set_tag<'js, T: Clonable>(ctx: &Ctx<'js>) -> rquickjs::Result<()>
where
    T: JsClass<'js>,
    T::Cloner: StructuredClone<Item<'js> = Class<'js, T>>,
{
    let Some(proto) = Class::<T>::prototype(ctx)? else {
        throw!(ctx, "Could not get prototype")
    };

    proto.set(get_symbol(ctx)?, T::Cloner::TAG)?;

    Ok(())
}

pub fn get_tag<'js>(ctx: &Ctx<'js>, obj: &Object<'js>) -> rquickjs::Result<StringRef<'js>> {
    obj.get(get_symbol(ctx)?)
}
