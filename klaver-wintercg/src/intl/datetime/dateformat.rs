use chrono::{Datelike, Timelike};
use icu::calendar::DateTime;
use icu::datetime::{
    time_zone::TimeZoneFormatterOptions, Error as DateTimeError, ZonedDateTimeFormatter,
};
use icu::locid::Locale;

use rquickjs::{class::Trace, prelude::Opt, Ctx, FromJs};
use rquickjs_util::{throw, throw_if, Date};

use crate::intl::locale::current_local;
use crate::WinterCG;

use super::options::{Options, ResolvedOptions};

// // Locales Init
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
        if value.is_undefined() {
            return Ok(LocalesInit::Fallback(vec![]));
        }

        if value.is_string() {
            Ok(LocalesInit::Single(String::from_js(ctx, value)?))
        } else if value.is_array() {
            Ok(LocalesInit::Fallback(Vec::<String>::from_js(ctx, value)?))
        } else {
            Err(rquickjs::Error::new_from_js(
                value.type_name(),
                "string or array of string",
            ))
        }
    }
}

#[derive(rquickjs::JsLifetime)]
#[rquickjs::class]
pub struct DateTimeFormat {
    formatter: ZonedDateTimeFormatter,
    options: ResolvedOptions,
}

impl<'js> Trace<'js> for DateTimeFormat {
    fn trace<'a>(&self, _tracer: rquickjs::class::Tracer<'a, 'js>) {}
}

#[rquickjs::methods]
impl DateTimeFormat {
    #[qjs(constructor)]
    pub fn new(
        ctx: Ctx<'_>,
        locals: Opt<LocalesInit>,
        options: Opt<Options>,
    ) -> rquickjs::Result<DateTimeFormat> {
        let provider = WinterCG::get(&ctx)?.borrow().icu_provider(&ctx).cloned()?;

        let locales = if let Some(locale) = locals.0 {
            locale.into_locale(&ctx)?
        } else {
            vec![current_local(&ctx)?]
        };

        let options = match options.0 {
            Some(o) => o,
            None => Options::get_default(&ctx)?,
        };

        let format_options = options.into_formatter_options()?;

        let mut err: Option<DateTimeError> = None;

        let time_zone_options = TimeZoneFormatterOptions::default();

        for locale in locales {
            let date_locale = locale.into();
            let formatter = ZonedDateTimeFormatter::try_new_experimental_unstable(
                &provider,
                &date_locale,
                format_options.clone(),
                time_zone_options.clone(),
            );
            match formatter {
                Ok(ret) => {
                    let options = options.resolve_options(&ctx, &date_locale)?;

                    return Ok(DateTimeFormat {
                        formatter: ret,
                        options,
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

        let locale = current_local(&ctx)?.into();
        let options = options.resolve_options(&ctx, &locale)?;

        let formatter = throw_if!(
            ctx,
            ZonedDateTimeFormatter::try_new_experimental_unstable(
                &provider,
                &locale,
                format_options,
                time_zone_options
            )
        );
        Ok(DateTimeFormat { formatter, options })
    }

    fn format<'js>(&self, ctx: Ctx<'js>, date: Date<'js>) -> rquickjs::Result<String> {
        let date = date.to_datetime()?;

        let date = self.options.time_zone.adjust(date);

        let datetime = throw_if!(
            ctx,
            DateTime::try_new_iso_datetime(
                date.year(),
                date.month() as u8,
                date.day() as u8,
                date.hour() as u8,
                date.minute() as u8,
                date.second() as u8,
            )
        );

        let timezone = self.options.time_zone.custom_timezone(&ctx, &datetime)?;

        let datetime = datetime.to_calendar(self.options.calendar.clone());

        let output = throw_if!(ctx, self.formatter.format_to_string(&datetime, &timezone));

        Ok(output)
    }

    // pub fn calendar(&self) -> rquickjs::Result<String> {
    //     Ok(self.calendar.kind().as_bcp47_string().to_string())
    // }
}
