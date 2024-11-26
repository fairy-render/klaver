use rquickjs::{promise::PromiseState, Array, Ctx, FromJs, Function, Object, Type, Value};
use std::fmt::Write;

use crate::{Buffer, Date, StringRef};

#[derive(Debug, Clone, Default)]
pub struct FormatOptions {
    #[allow(unused)]
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
    options: Option<FormatOptions>,
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
        Type::String => write!(f, "{}", StringRef::from_js(ctx, rest)?),
        Type::Symbol => {
            write!(f, r#"Symbol(""#).unwrap();
            format_value(ctx, rest.as_symbol().unwrap().description()?, f, options)?;
            write!(f, r#"")"#)
        }
        Type::Array => {
            let array = rest.into_array().unwrap();
            format_array(ctx, array, f, options)?;
            Ok(())
        }
        Type::Constructor => {
            let ctor = rest.into_function().unwrap();
            write!(f, "[class {}]", ctor.get::<_, StringRef>("name")?)
        }
        Type::Function => {
            let func = rest.into_object().unwrap();
            write!(f, "[function {}]", func.get::<_, StringRef>("name")?)
        }
        Type::Promise => {
            let rest = rest.into_promise().unwrap();
            let state = match rest.state() {
                PromiseState::Pending => "pending",
                PromiseState::Rejected => "rejected",
                PromiseState::Resolved => "resolved",
            };

            write!(f, "Promise[state = {state}]")
        }
        Type::Exception => {
            let excp = rest.into_exception().unwrap();
            write!(f, "{}", excp)
        }
        Type::Object => {
            format_object(ctx, rest.into_object().unwrap(), f, options)?;
            Ok(())
        }
        _ => {
            write!(f, "Unknown")
        }
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
    if Date::is(ctx, obj.as_value())? {
        write!(o, "{}", Date::from_js(ctx, obj.into_value())?.to_string()?).expect("write");
        return Ok(());
    } else if Buffer::is(ctx, obj.as_value())? {
        write!(o, "{}", Buffer::from_js(ctx, obj.into_value())?).expect("write");
        return Ok(());
    }

    let ctor = obj.get::<_, Option<Function<'js>>>("constructor")?;

    if let Some(ctor) = ctor {
        if let Ok(name) = ctor.get::<_, String>("name") {
            o.write_str(&name).ok();
            o.write_char(' ').ok();
        }
    }

    o.write_str("{ ").expect("write");
    for (idx, v) in obj.props::<Value, Value>().enumerate() {
        if idx > 0 {
            write!(o, ", ").expect("write");
        }
        let (k, v) = v?;
        format_value(ctx, k, o, options)?;
        o.write_str(" : ").expect("write");
        format_value(ctx, v, o, options)?;
    }

    o.write_str(" }").expect("write");

    Ok(())
}

fn format_array<'js, W: Write>(
    ctx: &Ctx<'js>,
    obj: Array<'js>,
    o: &mut W,
    options: &FormatOptions,
) -> rquickjs::Result<()> {
    o.write_str("[ ").expect("write");
    for (idx, v) in obj.iter::<rquickjs::Value>().enumerate() {
        if idx > 0 {
            write!(o, ", ").expect("write");
        }
        let v = v?;
        format_value(ctx, v, o, options)?;
    }

    o.write_str(" ]").expect("write");

    Ok(())
}
