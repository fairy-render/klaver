use core::fmt;
use std::{collections::HashMap, time::Instant};

use rquickjs::{class::Trace, function::Rest, Ctx, Function, Value};

use crate::{format_value, FormatOptions};

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

pub trait ConsoleWriter {
    fn write(&self, level: Level, message: String);
}

#[derive(Debug, Default)]
pub struct StdConsoleWriter;

impl ConsoleWriter for StdConsoleWriter {
    fn write(&self, level: Level, message: String) {
        if level == Level::Error {
            eprintln!("{} {}", level, message);
        } else if level == Level::Log {
            println!("{message}");
        } else {
            println!("{} {}", level, message);
        }
    }
}

#[rquickjs::class]
pub struct Console {
    writer: Box<dyn ConsoleWriter>,
    timers: HashMap<String, Instant>,
}

impl<'js> Trace<'js> for Console {
    fn trace<'a>(&self, _tracer: rquickjs::class::Tracer<'a, 'js>) {}
}

impl Console {
    pub fn new_with<W>(writer: W) -> Console
    where
        W: ConsoleWriter + 'static,
    {
        Console {
            writer: Box::new(writer),
            timers: HashMap::default(),
        }
    }

    pub fn set_writer<W>(&mut self, writer: W) -> rquickjs::Result<()>
    where
        W: ConsoleWriter + 'static,
    {
        self.writer = Box::new(writer);
        Ok(())
    }

    fn log_inner<'js>(
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

        self.writer.write(level, output);

        Ok(())
    }
}

#[rquickjs::methods]
impl Console {
    #[qjs(constructor)]
    pub fn new() -> Console {
        Console::new_with(StdConsoleWriter::default())
    }

    pub fn log<'js>(&self, ctx: Ctx<'js>, values: Rest<Value<'js>>) -> rquickjs::Result<()> {
        self.log_inner(ctx, Level::Log, values)
    }

    pub fn debug<'js>(&self, ctx: Ctx<'js>, values: Rest<Value<'js>>) -> rquickjs::Result<()> {
        self.log_inner(ctx, Level::Debug, values)
    }

    pub fn info<'js>(&self, ctx: Ctx<'js>, values: Rest<Value<'js>>) -> rquickjs::Result<()> {
        self.log_inner(ctx, Level::Debug, values)
    }

    pub fn error<'js>(&self, ctx: Ctx<'js>, values: Rest<Value<'js>>) -> rquickjs::Result<()> {
        self.log_inner(ctx, Level::Error, values)
    }

    pub fn warn<'js>(&self, ctx: Ctx<'js>, values: Rest<Value<'js>>) -> rquickjs::Result<()> {
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
                .write(Level::Log, format!("{name}: {:?}", timer.elapsed()))
        }
        Ok(())
    }

    pub fn assert<'js>(
        &self,
        ctx: Ctx<'js>,
        condition: Value<'js>,
        values: Rest<Value<'js>>,
    ) -> rquickjs::Result<()> {
        let ret = ctx.eval::<Function, _>("(t) => !!t")?;
        let ret: bool = ret.call((condition,))?;

        if ret {
            self.log(ctx, values)?;
        }

        Ok(())
    }
}
