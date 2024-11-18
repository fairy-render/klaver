use std::rc::Rc;

use icu_calendar::{AnyCalendar, DateTime};
use icu_datetime::{
    DateTimeFormatter as IcuDateTimeFormatter, DateTimeFormatterOptions, Error as DateTimeError,
};
use icu_locid::{locale, Locale};
use rquickjs::{class::Trace, prelude::Opt, Ctx, FromJs};
use rquickjs_util::{throw, throw_if, Date};

#[rquickjs::class]
pub struct DateTimeFormat {
    i: IcuDateTimeFormatter,
    calendar: Rc<AnyCalendar>,
}

impl<'js> Trace<'js> for DateTimeFormat {
    fn trace<'a>(&self, _tracer: rquickjs::class::Tracer<'a, 'js>) {}
}

pub enum LocalesInit {
    Single(String),
    Fallback(Vec<String>),
}

impl LocalesInit {
    pub fn into_locale(self, ctx: &Ctx<'_>) -> rquickjs::Result<Vec<Locale>> {
        match self {
            Self::Single(m) => {
                let locale: Locale = throw_if!(ctx, m.parse());
                Ok(vec![locale])
            }
            Self::Fallback(m) => {
                let mut locales = Vec::default();
                for l in m {
                    let locale: Locale = throw_if!(ctx, l.parse());
                    locales.push(locale);
                }
                Ok(locales)
            }
        }
    }
}

impl<'js> FromJs<'js> for LocalesInit {
    fn from_js(ctx: &rquickjs::Ctx<'js>, value: rquickjs::Value<'js>) -> rquickjs::Result<Self> {
        todo!()
    }
}

pub struct Options {
    // local options
    calendar: Option<String>,
    hour12: Option<bool>,
    hour_cycle: Option<String>,
    time_zone: Option<String>,
    // Datetime components
    weekday: Option<String>,
}

impl Options {
    fn into_formatter_options(self) -> rquickjs::Result<DateTimeFormatterOptions> {
        todo!()
    }
}

impl<'js> FromJs<'js> for Options {
    fn from_js(ctx: &rquickjs::Ctx<'js>, value: rquickjs::Value<'js>) -> rquickjs::Result<Self> {
        todo!()
    }
}

#[rquickjs::methods]
impl DateTimeFormat {
    #[qjs(constructor)]
    pub fn new(
        ctx: Ctx<'_>,
        locals: Opt<LocalesInit>,
        options: Opt<Options>,
    ) -> rquickjs::Result<DateTimeFormat> {
        let locales = if let Some(locale) = locals.0 {
            locale.into_locale(&ctx)?
        } else {
            vec![locale!("en_gb")]
        };

        let options = if let Some(options) = options.0 {
            options.into_formatter_options()?
        } else {
            DateTimeFormatterOptions::default()
        };

        let mut err: Option<DateTimeError> = None;

        for locale in locales {
            let date_locale = locale.into();
            let formatter = IcuDateTimeFormatter::try_new(&date_locale, options.clone());
            match formatter {
                Ok(ret) => {
                    let calendar = AnyCalendar::new_for_locale(&date_locale);

                    return Ok(DateTimeFormat {
                        i: ret,
                        calendar: calendar.into(),
                    });
                }
                Err(e) => {
                    err = Some(e);
                }
            }
        }

        if let Some(err) = err {
            throw!(ctx, err)
        }

        let locale = locale!("en_gb").into();
        let calendar = AnyCalendar::new_for_locale(&locale);

        let i = throw_if!(ctx, IcuDateTimeFormatter::try_new(&locale, options));
        Ok(DateTimeFormat {
            i,
            calendar: calendar.into(),
        })
    }

    fn format<'js>(&self, ctx: Ctx<'js>, date: Date<'js>) -> rquickjs::Result<String> {
        let date_time = throw_if!(
            ctx,
            DateTime::try_new_iso_datetime(
                date.year()?,
                date.month()?,
                date.date()?,
                date.hours()?,
                date.minutes()?,
                date.seconds()?,
            )
        )
        .to_calendar(self.calendar.clone());

        let output = throw_if!(ctx, self.i.format_to_string(&date_time));

        Ok(output)
    }

    pub fn calendar(&self) -> rquickjs::Result<String> {
        Ok(self.calendar.kind().as_bcp47_string().to_string())
    }
}
