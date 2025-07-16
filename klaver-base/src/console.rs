use core::fmt;
use std::{collections::HashMap, time::Instant};

use rquickjs::{Ctx, Function, JsLifetime, Value, class::Trace, function::Rest};

use rquickjs_util::format::{FormatOptions, format_value};

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
    fn write(&self, level: Level, message: String) -> rquickjs::Result<()>;
}

#[derive(Debug, Default)]
pub struct StdConsoleWriter;

impl<'js> Trace<'js> for StdConsoleWriter {
    fn trace<'a>(&self, _tracer: rquickjs::class::Tracer<'a, 'js>) {}
}

impl<'js> ConsoleWriter<'js> for StdConsoleWriter {
    fn write(&self, level: Level, message: String) -> rquickjs::Result<()> {
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

impl<'js> ConsoleWriter<'js> for Function<'js> {
    fn write(&self, level: Level, message: String) -> rquickjs::Result<()> {
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
        let mut output = String::new();

        let opts = FormatOptions::default();
        for (idx, v) in values.0.into_iter().enumerate() {
            if idx != 0 {
                output.push(' ');
            }
            format_value(&ctx, v, &mut output, &opts)?;
        }

        self.writer.write(level, output)?;

        Ok(())
    }
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
        self.log_inner(ctx, Level::Debug, values)
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
    pub fn time_end(&mut self, name: String) -> rquickjs::Result<()> {
        if let Some(timer) = self.timers.remove(&name) {
            self.writer
                .write(Level::Log, format!("{name}: {:?}", timer.elapsed()))?;
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
            self.log(ctx, values)?;
        }

        Ok(())
    }
}
