use core::fmt;
use std::{collections::HashMap, fmt::Write as _, time::Instant};

use rquickjs::{
    Class, Coerced, Ctx, FromJs, Function, JsLifetime, Type, Value,
    class::{JsClass, Trace},
    function::Rest,
};

use klaver_core::value::{FormatOptions, StringRef, format_to};

use klaver_core::Exportable;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Level {
    Debug,
    Info,
    Warn,
    Error,
    Log,
}

impl fmt::Display for Level {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Level::Debug => write!(f, "DEBUG"),
            Level::Info => write!(f, "INFO"),
            Level::Warn => write!(f, "WARN"),
            Level::Error => write!(f, "ERROR"),
            Level::Log => write!(f, "LOG"),
        }
    }
}

pub trait ConsoleWriter<'js>: Trace<'js> {
    fn write(&self, ctx: &Ctx<'js>, level: Level, message: String) -> rquickjs::Result<()>;
}

#[derive(Debug, Default)]
pub struct StdConsoleWriter;

impl<'js> Trace<'js> for StdConsoleWriter {
    fn trace<'a>(&self, _tracer: rquickjs::class::Tracer<'a, 'js>) {}
}

impl<'js> ConsoleWriter<'js> for StdConsoleWriter {
    fn write(&self, _ctx: &Ctx<'js>, level: Level, message: String) -> rquickjs::Result<()> {
        if level == Level::Error || level == Level::Warn {
            eprintln!("{} {}", level, message);
        } else if level == Level::Log {
            println!("{message}");
        } else {
            println!("{} {}", level, message);
        }

        Ok(())
    }
}

#[derive(Trace, Default)]
pub struct NullWriter;

impl<'js> ConsoleWriter<'js> for NullWriter {
    fn write(&self, _ctx: &Ctx<'js>, _level: Level, _message: String) -> rquickjs::Result<()> {
        Ok(())
    }
}

impl<'js> ConsoleWriter<'js> for Function<'js> {
    fn write(&self, _ctx: &Ctx<'js>, level: Level, message: String) -> rquickjs::Result<()> {
        self.call::<_, ()>((level.to_string(), message))
    }
}

#[rquickjs::class]
pub struct Console<'js> {
    writer: Box<dyn ConsoleWriter<'js> + 'js>,
    timers: HashMap<String, Instant>,
}

unsafe impl<'js> JsLifetime<'js> for Console<'js> {
    type Changed<'to> = Console<'to>;
}

impl<'js> Trace<'js> for Console<'js> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        self.writer.trace(tracer);
    }
}

impl<'js> Console<'js> {
    pub fn new_with<W>(writer: W) -> Console<'js>
    where
        W: ConsoleWriter<'js> + 'js,
    {
        Console {
            writer: Box::new(writer),
            timers: HashMap::default(),
        }
    }

    pub fn set_writer<W>(&mut self, writer: W) -> rquickjs::Result<()>
    where
        W: ConsoleWriter<'js> + 'js,
    {
        self.writer = Box::new(writer);
        Ok(())
    }

    fn log_inner(
        &self,
        ctx: Ctx<'js>,
        level: Level,
        values: Rest<Value<'js>>,
    ) -> rquickjs::Result<()> {
        let values = values.0;
        let mut output = String::new();

        // https://console.spec.whatwg.org/#formatter
        // The Formatter only applies when there is a format string *and* at least one
        // more argument to substitute into it.
        let consumed = if values.len() > 1 && values[0].type_of() == Type::String {
            format_specifiers(&ctx, &values, &mut output)?
        } else {
            0
        };

        for (idx, v) in values[consumed..].iter().enumerate() {
            if consumed > 0 || idx != 0 {
                output.push(' ');
            }
            format_to(&ctx, v, &mut output, Some(FormatOptions::default()))?;
        }

        self.writer.write(&ctx, level, output)?;

        Ok(())
    }
}

/// Coerces `value` to a number the way JavaScript's `ToNumber` would, falling back to
/// `NaN` on any conversion failure (e.g. Symbols, BigInts) rather than throwing, since a
/// bad format argument must never make `console.log` itself fail.
fn coerce_number<'js>(ctx: &Ctx<'js>, value: &Value<'js>) -> f64 {
    Coerced::<f64>::from_js(ctx, value.clone())
        .map(|c| c.0)
        .unwrap_or(f64::NAN)
}

/// Implements the Console Standard's [Formatter](https://console.spec.whatwg.org/#formatter):
/// scans `values[0]` (already known to be a string) for `%s`, `%d`/`%i`, `%f`, `%o`/`%O`,
/// `%c` and `%%` specifiers, substituting each recognized one (other than `%%`) with the
/// next unconsumed argument. A specifier with no argument left to substitute, or an
/// unrecognized one, is emitted literally. Returns the number of leading `values` consumed
/// (always at least 1, for the format string itself), so the caller can format and append
/// any remaining arguments as usual.
fn format_specifiers<'js>(
    ctx: &Ctx<'js>,
    values: &[Value<'js>],
    output: &mut String,
) -> rquickjs::Result<usize> {
    let target = StringRef::from_js(ctx, values[0].clone())?;
    let mut chars = target.as_str().chars();
    let mut arg_idx = 1usize;

    while let Some(c) = chars.next() {
        if c != '%' {
            output.push(c);
            continue;
        }

        match chars.next() {
            None => output.push('%'),
            Some('%') => output.push('%'),
            Some(spec @ ('s' | 'd' | 'i' | 'f' | 'o' | 'O' | 'c')) if arg_idx < values.len() => {
                let arg = &values[arg_idx];
                arg_idx += 1;

                match spec {
                    's' | 'o' | 'O' => {
                        format_to(ctx, arg, output, Some(FormatOptions::default()))?
                    }
                    'd' | 'i' => {
                        let n = coerce_number(ctx, arg);
                        if n.is_nan() {
                            output.push_str("NaN");
                        } else {
                            write!(output, "{}", n.trunc() as i64).ok();
                        }
                    }
                    'f' => {
                        let n = coerce_number(ctx, arg);
                        if n.is_nan() {
                            output.push_str("NaN");
                        } else {
                            write!(output, "{}", n).ok();
                        }
                    }
                    // CSS styling directive: consumes the argument, produces no output.
                    'c' => {}
                    _ => unreachable!(),
                }
            }
            // Unrecognized specifier, or a recognized one with no argument left to
            // substitute: emit it literally without consuming an argument.
            Some(spec) => {
                output.push('%');
                output.push(spec);
            }
        }
    }

    Ok(arg_idx)
}

#[rquickjs::methods]
impl<'js> Console<'js> {
    #[qjs(constructor)]
    pub fn new(func: Function<'js>) -> Console<'js> {
        Console::new_with(func)
    }

    pub fn log(&self, ctx: Ctx<'js>, values: Rest<Value<'js>>) -> rquickjs::Result<()> {
        self.log_inner(ctx, Level::Log, values)
    }

    pub fn debug(&self, ctx: Ctx<'js>, values: Rest<Value<'js>>) -> rquickjs::Result<()> {
        self.log_inner(ctx, Level::Debug, values)
    }

    pub fn info(&self, ctx: Ctx<'js>, values: Rest<Value<'js>>) -> rquickjs::Result<()> {
        self.log_inner(ctx, Level::Info, values)
    }

    pub fn error(&self, ctx: Ctx<'js>, values: Rest<Value<'js>>) -> rquickjs::Result<()> {
        self.log_inner(ctx, Level::Error, values)
    }

    pub fn warn(&self, ctx: Ctx<'js>, values: Rest<Value<'js>>) -> rquickjs::Result<()> {
        self.log_inner(ctx, Level::Warn, values)
    }

    pub fn time(&mut self, name: String) -> rquickjs::Result<()> {
        self.timers.insert(name, Instant::now());
        Ok(())
    }

    #[qjs(rename = "timeEnd")]
    pub fn time_end(&mut self, ctx: Ctx<'js>, name: String) -> rquickjs::Result<()> {
        if let Some(timer) = self.timers.remove(&name) {
            self.writer
                .write(&ctx, Level::Log, format!("{name}: {:?}", timer.elapsed()))?;
        }
        Ok(())
    }

    pub fn assert(
        &self,
        ctx: Ctx<'js>,
        condition: Value<'js>,
        values: Rest<Value<'js>>,
    ) -> rquickjs::Result<()> {
        let ret = ctx.eval::<Function, _>("(t) => !!t")?;
        let ret: rquickjs::Coerced<bool> = ret.call((condition,))?;

        if !ret.0 {
            let mut output = String::from("Assertion failed");

            if !values.0.is_empty() {
                output.push(':');
                for v in values.0.iter() {
                    output.push(' ');
                    format_to(&ctx, v, &mut output, Some(FormatOptions::default()))?;
                }
            }

            self.writer.write(&ctx, Level::Error, output)?;
        }

        Ok(())
    }
}

impl<'js> Exportable<'js> for Console<'js> {
    fn export<T>(
        ctx: &Ctx<'js>,
        _registry: &klaver_core::Registry,
        target: &T,
    ) -> rquickjs::Result<()>
    where
        T: klaver_core::ExportTarget<'js>,
    {
        target.set(
            ctx,
            Console::NAME,
            Class::<Console<'js>>::create_constructor(ctx)?,
        )
    }
}
