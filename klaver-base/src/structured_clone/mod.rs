mod bindings;
mod context;
mod module;
mod registry;
mod tag;
mod traits;
mod value;

use klaver_util::{Date, Map, Set, StringRef, is_plain_object, throw};
use rquickjs::{Class, Ctx, Function, Symbol, Type, Value, class::JsClass, prelude::Opt};

pub use self::{
    bindings::{serialize, structured_clone},
    context::SerializationContext,
    module::*,
    registry::Registry,
    tag::Tag,
    traits::*,
    value::*,
};

pub fn register<'js, T>(ctx: &Ctx<'js>, registry: &crate::Registry) -> rquickjs::Result<()>
where
    T: Clonable + JsClass<'js>,
    T::Cloner: StructuredClone<Item<'js> = Class<'js, T>> + Send + Sync,
{
    set_tag::<T>(ctx)?;
    // It only throws an error, if the type is already registered
    // So it's okay here
    registry.register::<T>().ok();

    Ok(())
}

fn get_symbol<'js>(ctx: &Ctx<'js>) -> rquickjs::Result<Symbol<'js>> {
    ctx.eval("Symbol.for('$Tag')")
}

fn set_tag<'js, T: Clonable>(ctx: &Ctx<'js>) -> rquickjs::Result<()>
where
    T: JsClass<'js>,
    T::Cloner: StructuredClone<Item<'js> = Class<'js, T>>,
{
    let Some(proto) = Class::<T>::prototype(ctx)? else {
        throw!(ctx, "Could not get prototype")
    };

    proto.set(get_symbol(ctx)?, T::Cloner::tag().clone())?;

    Ok(())
}

pub fn get_tag_value<'js>(ctx: &Ctx<'js>, value: &Value<'js>) -> rquickjs::Result<Tag> {
    let tag = match value.type_of() {
        Type::Null | Type::Uninitialized | Type::Undefined => NullClone::<StringCloner>::tag(),
        Type::Bool => BoolCloner::tag(),
        Type::Int => IntCloner::tag(),
        Type::Float => FloatCloner::tag(),
        Type::String => StringCloner::tag(),
        Type::Array => ArrayCloner::tag(),
        Type::Object => {
            let Some(obj) = value.as_object() else {
                throw!(@type ctx, "Expected object")
            };

            if let Ok(tag) = obj.get(get_symbol(ctx)?) {
                return Ok(tag);
            }

            if Map::is(ctx, value)? {
                todo!()
            } else if Set::is(ctx, value)? {
                todo!()
            } else if Date::is(ctx, value)? {
                DateCloner::tag()
            } else if is_plain_object(ctx, value, Opt(Some(true)))? {
                ObjectCloner::tag()
            } else {
                let ctor = obj.get::<_, Option<Function<'js>>>("constructor")?;

                let mut error_string = std::string::String::from("Cannot serialize ");

                if let Some(ctor) = ctor {
                    if let Ok(name) = ctor.get::<_, StringRef<'js>>("name") {
                        error_string.push_str(name.as_str());
                    } else {
                        error_string.push_str("Anonymous");
                    }
                } else {
                    error_string.push_str("Object");
                };

                throw!(@type ctx, error_string)
            }
        }
        Type::BigInt => {
            todo!("Serialize big int")
        }
        Type::Exception => {
            todo!("Seialize exception")
        }
        ty => {
            throw!(@type ctx, format!("Cannot serialize {}", ty));
        } // Type::Constructor => todo!(),
          // Type::Function => todo!(),
          // Type::Promise => todo!(),
          // Type::Exception => todo!(),
          // Type::Module => todo!(),
          // Type::BigInt => todo!(),
          // Type::Unknown => todo!(),
          // Type::Symbol => todo!(),
    };

    Ok(tag.clone())
}
