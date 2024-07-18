use rquickjs::{
    function::{Opt},
    Ctx, FromJs, Object, Type, Value,
};
use std::{fmt::Write};

#[derive(Debug, Clone, Default)]
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
    let options = options.unwrap_or_default();

    format_value(&ctx, rest, &mut f, &options)?;

    Ok(f)
}

pub fn format_value<'js, W: Write>(
    ctx: &Ctx<'js>,
    rest: Value<'js>,
    f: &mut W,
    options: &FormatOptions,
) -> rquickjs::Result<()> {
    match rest.type_of() {
        Type::Uninitialized => write!(f, "undefined"),
        Type::Undefined => write!(f, "undefined"),
        Type::Null => write!(f, "null"),
        Type::Bool => write!(f, "{}", rest.as_bool().unwrap()),
        Type::Int => write!(f, "{}", rest.as_int().unwrap()),
        Type::Float => write!(f, "{}", rest.as_float().unwrap()),
        Type::String => write!(f, "{}", rest.as_string().unwrap().to_string().unwrap()),
        Type::Symbol => {
            write!(f, r#"Symbol(""#).unwrap();
            format_value(ctx, rest.as_symbol().unwrap().description()?, f, options)?;
            write!(f, r#"")"#)
        }
        Type::Array => todo!(),
        Type::Constructor => todo!(),
        Type::Function => todo!(),
        Type::Promise => todo!(),
        Type::Exception => todo!(),
        Type::Object => {
            format_object(ctx, rest.into_object().unwrap(), f, options)?;
            Ok(())
        }
        Type::Module => todo!(),
        Type::BigInt => todo!(),
        Type::Unknown => todo!(),
    }
    .unwrap();

    Ok(())
}

fn format_object<'js, W: Write>(
    ctx: &Ctx<'js>,
    obj: Object<'js>,
    o: &mut W,
    options: &FormatOptions,
) -> rquickjs::Result<()> {
    o.write_str("{ ");
    for (idx, v) in obj.props::<Value, Value>().enumerate() {
        if idx > 0 {
            write!(o, ", ");
        }
        let (k, v) = v?;
        format_value(ctx, k, o, options)?;
        o.write_str(" : ");
        format_value(ctx, v, o, options)?;
    }

    o.write_str(" }");

    Ok(())
}
