use rquickjs::{Array, Ctx, Filter, FromJs, Function, Object, Type, Value, promise::PromiseState};
use std::{collections::HashSet, fmt::Write};

use crate::{Buffer, Date, StringRef};

struct FormatCtx<'js, 'a> {
    parent: Option<&'a FormatCtx<'js, 'a>>,
    cache: HashSet<Value<'js>>,
}

impl<'js, 'a> FormatCtx<'js, 'a> {
    fn new(parent: Option<&'a FormatCtx<'js, 'a>>) -> Self {
        Self {
            parent,
            cache: HashSet::new(),
        }
    }

    fn is_cached(&self, value: &Value<'js>) -> bool {
        self.cache.contains(value) || self.parent.map_or(false, |p| p.is_cached(value))
    }

    fn cache(&mut self, value: Value<'js>) {
        self.cache.insert(value);
    }

    fn child(&'a self) -> Self {
        Self::new(Some(self))
    }
}

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
    ctx: &Ctx<'js>,
    rest: &Value<'js>,
    options: Option<FormatOptions>,
) -> rquickjs::Result<String> {
    let mut f = String::default();
    format_to(ctx, rest, &mut f, options)?;

    Ok(f)
}

pub fn format_to<'js, W: Write>(
    ctx: &Ctx<'js>,
    rest: &Value<'js>,
    output: &mut W,
    options: Option<FormatOptions>,
) -> rquickjs::Result<()> {
    let options = options.unwrap_or_default();

    let mut format_ctx = FormatCtx::new(None);

    format_value(&ctx, rest, output, &options, &mut format_ctx)?;

    Ok(())
}

fn format_value<'js, W: Write>(
    ctx: &Ctx<'js>,
    rest: &Value<'js>,
    f: &mut W,
    options: &FormatOptions,
    format_ctx: &mut FormatCtx<'js, '_>,
) -> rquickjs::Result<()> {
    match rest.type_of() {
        Type::Uninitialized => write!(f, "undefined"),
        Type::Undefined => write!(f, "undefined"),
        Type::Null => write!(f, "null"),
        Type::Bool => write!(f, "{}", rest.as_bool().unwrap()),
        Type::Int => write!(f, "{}", rest.as_int().unwrap()),
        Type::Float => write!(f, "{}", rest.as_float().unwrap()),
        Type::String => write!(f, "\"{}\"", StringRef::from_js(ctx, rest.clone())?),
        Type::Symbol => {
            write!(f, r#"Symbol(""#).unwrap();
            format_value(
                ctx,
                &rest.as_symbol().unwrap().description()?,
                f,
                options,
                format_ctx,
            )?;
            write!(f, r#"")"#)
        }
        Type::Array => {
            let array = rest.clone().into_array().unwrap();
            format_array(ctx, array, f, options, format_ctx)?;
            Ok(())
        }
        Type::Constructor => {
            let ctor = rest.clone().into_function().unwrap();
            write!(f, "[class {}]", ctor.get::<_, StringRef>("name")?)
        }
        Type::Function => {
            let func = rest.clone().into_object().unwrap();
            write!(f, "[function {}]", func.get::<_, StringRef>("name")?)
        }
        Type::Promise => {
            let rest = rest.clone().into_promise().unwrap();
            let state = match rest.state() {
                PromiseState::Pending => "pending",
                PromiseState::Rejected => "rejected",
                PromiseState::Resolved => "resolved",
            };

            write!(f, "Promise[state = {state}]")
        }
        Type::Exception => {
            let excp = rest.clone().into_exception().unwrap();
            write!(f, "{}", excp)
        }
        Type::Object => {
            format_object(
                ctx,
                rest.clone().into_object().unwrap(),
                f,
                options,
                format_ctx,
            )?;
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
    format_ctx: &mut FormatCtx<'js, '_>,
) -> rquickjs::Result<()> {
    if Date::is(ctx, obj.as_value())? {
        write!(o, "{}", Date::from_js(ctx, obj.into_value())?.to_string()?).expect("write");
        return Ok(());
    } else if Buffer::is(ctx, obj.as_value())? {
        write!(o, "{}", Buffer::from_js(ctx, obj.into_value())?).expect("write");
        return Ok(());
    }

    if format_ctx.is_cached(obj.as_value()) {
        o.write_str("[Circular]").ok();
        return Ok(());
    }

    format_ctx.cache(obj.clone().into_value());

    let ctor = obj.get::<_, Option<Function<'js>>>("constructor")?;

    if let Some(ctor) = ctor {
        if let Ok(name) = ctor.get::<_, StringRef<'js>>("name") {
            o.write_str(name.as_str()).ok();
            o.write_char(' ').ok();
        }
    }

    o.write_str("{ ").expect("write");
    for (idx, v) in obj
        .own_props::<Value, Value>(Filter::new().private().string().symbol())
        .enumerate()
    {
        if idx > 0 {
            write!(o, ", ").expect("write");
        }
        let (k, v) = v?;
        format_value(ctx, &k, o, options, &mut format_ctx.child())?;
        o.write_str(" : ").expect("write");
        format_value(ctx, &v, o, options, &mut format_ctx.child())?;
    }

    o.write_str(" }").expect("write");

    Ok(())
}

fn format_array<'js, W: Write>(
    ctx: &Ctx<'js>,
    obj: Array<'js>,
    o: &mut W,
    options: &FormatOptions,
    format_ctx: &mut FormatCtx<'js, '_>,
) -> rquickjs::Result<()> {
    if format_ctx.is_cached(obj.as_value()) {
        o.write_str("[Circular]").ok();
        return Ok(());
    }

    format_ctx.cache(obj.clone().into_value());

    o.write_str("[ ").expect("write");
    for (idx, v) in obj.iter::<rquickjs::Value>().enumerate() {
        if idx > 0 {
            write!(o, ", ").expect("write");
        }
        let v = v?;
        format_value(ctx, &v, o, options, &mut format_ctx.child())?;
    }

    o.write_str(" ]").expect("write");

    Ok(())
}
