use rquickjs::{
    function::{Opt, Rest},
    Ctx, FromJs, Type, Value,
};
use std::fmt::Write;

#[derive(Debug, Clone)]
pub struct FormatOptions {
    colors: bool,
}

impl<'js> FromJs<'js> for FormatOptions {
    fn from_js(_ctx: &Ctx<'js>, value: Value<'js>) -> rquickjs::Result<Self> {
        let Some(obj) = value.as_object() else {
            return Err(rquickjs::Error::new_from_js(value.type_name(), "object"));
        };

        let colors = if let Ok(colors) = obj.get("colors") {
            colors
        } else {
            false
        };

        Ok(FormatOptions { colors })
    }
}

pub fn format<'js>(
    ctx: Ctx<'js>,
    rest: Value<'js>,
    Opt(options): Opt<FormatOptions>,
) -> rquickjs::Result<String> {
    let mut f = String::default();
    match rest.type_of() {
        Type::Uninitialized => write!(f, "undefined"),
        Type::Undefined => write!(f, "undefined"),
        Type::Null => write!(f, "null"),
        Type::Bool => write!(f, "{}", rest.as_bool().unwrap()),
        Type::Int => write!(f, "{}", rest.as_int().unwrap()),
        Type::Float => write!(f, "{}", rest.as_float().unwrap()),
        Type::String => write!(f, "{}", rest.as_string().unwrap().to_string().unwrap()),
        Type::Symbol => write!(
            f,
            "Symbol({})",
            format(
                ctx,
                rest.as_symbol().unwrap().description()?,
                Opt(options.clone())
            )?
        ),
        Type::Array => todo!(),
        Type::Constructor => todo!(),
        Type::Function => todo!(),
        Type::Promise => todo!(),
        Type::Exception => todo!(),
        Type::Object => todo!(),
        Type::Module => todo!(),
        Type::BigInt => todo!(),
        Type::Unknown => todo!(),
    }
    .unwrap();

    Ok(f)
}
