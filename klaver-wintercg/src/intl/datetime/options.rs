use std::rc::Rc;

use icu::calendar::{AnyCalendar, AnyCalendarKind};
use icu::datetime::{
    options::{
        components::{Bag, Day, Month, Numeric, Text, TimeZoneName, Year},
        length,
    }, DateTimeFormatterOptions,
};
use icu_provider::DataLocale;
use rquickjs::{Ctx, FromJs, IntoJs, Object};
use rquickjs_util::{throw, throw_if};

use crate::WinterCG;

use super::timezone::TimeZone;

#[derive(Debug, Clone, Copy)]
pub struct JsTimeZoneName(TimeZoneName);

impl<'js> FromJs<'js> for JsTimeZoneName {
    fn from_js(ctx: &Ctx<'js>, value: rquickjs::Value<'js>) -> rquickjs::Result<Self> {
        let str = String::from_js(ctx, value)?;

        match str.as_str() {
            "long" => Ok(Self(TimeZoneName::LongSpecific)),
            "short" => Ok(Self(TimeZoneName::ShortSpecific)),
            "longGeneric" => Ok(Self(TimeZoneName::LongGeneric)),
            "shortGeneric" => Ok(Self(TimeZoneName::ShortGeneric)),
            "shortOffset" | "longOffset" => Ok(Self(TimeZoneName::GmtOffset)),
            _ => Err(rquickjs::Error::new_from_js("string", "TimeZoneName")),
        }
    }
}

impl From<JsTimeZoneName> for TimeZoneName {
    fn from(value: JsTimeZoneName) -> Self {
        value.0
    }
}

#[derive(Debug, Clone, Copy)]
enum TextLength {
    Long,
    Short,
    Narrow,
}

impl From<TextLength> for Text {
    fn from(value: TextLength) -> Self {
        match value {
            TextLength::Long => Text::Long,
            TextLength::Narrow => Text::Narrow,
            TextLength::Short => Text::Short,
        }
    }
}

impl<'js> FromJs<'js> for TextLength {
    fn from_js(ctx: &Ctx<'js>, value: rquickjs::Value<'js>) -> rquickjs::Result<Self> {
        let v = String::from_js(ctx, value)?;

        let v = match v.as_str() {
            "short" => TextLength::Short,
            "long" => TextLength::Long,
            "narrow" => TextLength::Narrow,
            _ => {
                return Err(rquickjs::Error::new_from_js(
                    "string",
                    "short, long or narrow",
                ))
            }
        };

        Ok(v)
    }
}

impl<'js> IntoJs<'js> for TextLength {
    fn into_js(self, ctx: &Ctx<'js>) -> rquickjs::Result<rquickjs::Value<'js>> {
        let ret = match self {
            Self::Long => "long",
            Self::Short => "short",
            Self::Narrow => "narrow",
        };

        ret.into_js(ctx)
    }
}

#[derive(Debug, Clone, Copy)]
enum NumericLength {
    Numeric,
    Digit2,
}

impl From<NumericLength> for Numeric {
    fn from(value: NumericLength) -> Self {
        match value {
            NumericLength::Numeric => Numeric::Numeric,
            NumericLength::Digit2 => Numeric::TwoDigit,
        }
    }
}

impl From<NumericLength> for Year {
    fn from(value: NumericLength) -> Self {
        match value {
            NumericLength::Digit2 => Year::TwoDigit,
            NumericLength::Numeric => Year::Numeric,
        }
    }
}

impl From<NumericLength> for Day {
    fn from(value: NumericLength) -> Self {
        match value {
            NumericLength::Digit2 => Day::TwoDigitDayOfMonth,
            NumericLength::Numeric => Day::NumericDayOfMonth,
        }
    }
}

impl<'js> FromJs<'js> for NumericLength {
    fn from_js(ctx: &Ctx<'js>, value: rquickjs::Value<'js>) -> rquickjs::Result<Self> {
        let v = String::from_js(ctx, value)?;

        let v = match v.as_str() {
            "numeric" => NumericLength::Numeric,
            "2-digit" => NumericLength::Digit2,
            _ => return Err(rquickjs::Error::new_from_js("string", "numeric or 2-digit")),
        };

        Ok(v)
    }
}

impl<'js> IntoJs<'js> for NumericLength {
    fn into_js(self, ctx: &Ctx<'js>) -> rquickjs::Result<rquickjs::Value<'js>> {
        match self {
            Self::Digit2 => "2-digit",
            Self::Numeric => "numeric",
        }
        .into_js(ctx)
    }
}

#[derive(Debug, Clone, Copy)]
enum MonthLength {
    Long,
    Short,
    Narrow,
    Numeric,
    Digit2,
}

impl From<MonthLength> for Month {
    fn from(value: MonthLength) -> Self {
        match value {
            MonthLength::Narrow => Month::Narrow,
            MonthLength::Long => Month::Long,
            MonthLength::Numeric => Month::Numeric,
            MonthLength::Digit2 => Month::TwoDigit,
            MonthLength::Short => Month::Short,
        }
    }
}

impl<'js> FromJs<'js> for MonthLength {
    fn from_js(ctx: &Ctx<'js>, value: rquickjs::Value<'js>) -> rquickjs::Result<Self> {
        let v = String::from_js(ctx, value)?;

        let v = match v.as_str() {
            "numeric" => MonthLength::Numeric,
            "2-digit" => MonthLength::Digit2,
            "short" => MonthLength::Short,
            "long" => MonthLength::Long,
            "narrow" => MonthLength::Narrow,
            _ => {
                return Err(rquickjs::Error::new_from_js(
                    "string",
                    "short, long, narrow, numeric or 2-digit",
                ))
            }
        };

        Ok(v)
    }
}

impl<'js> IntoJs<'js> for MonthLength {
    fn into_js(self, ctx: &Ctx<'js>) -> rquickjs::Result<rquickjs::Value<'js>> {
        match self {
            Self::Digit2 => "2-digit",
            Self::Long => "long",
            Self::Narrow => "narrow",
            Self::Short => "short",
            Self::Numeric => "numeric",
        }
        .into_js(ctx)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Style {
    Short,
    Medium,
    Long,
    Full,
}

impl<'js> FromJs<'js> for Style {
    fn from_js(ctx: &Ctx<'js>, value: rquickjs::Value<'js>) -> rquickjs::Result<Self> {
        let v = String::from_js(ctx, value)?;
        let v = match v.as_str() {
            "full" => Self::Full,
            "long" => Self::Long,
            "short" => Self::Short,
            "medium" => Self::Medium,

            _ => {
                return Err(rquickjs::Error::new_from_js(
                    "string",
                    "full, long, medium or short",
                ))
            }
        };

        Ok(v)
    }
}

impl<'js> IntoJs<'js> for Style {
    fn into_js(self, ctx: &Ctx<'js>) -> rquickjs::Result<rquickjs::Value<'js>> {
        match self {
            Self::Full => "full",
            Self::Long => "long",
            Self::Medium => "medium",
            Self::Short => "short",
        }
        .into_js(ctx)
    }
}

impl From<Style> for length::Date {
    fn from(value: Style) -> Self {
        match value {
            Style::Full => length::Date::Full,
            Style::Long => length::Date::Long,
            Style::Medium => length::Date::Medium,
            Style::Short => length::Date::Short,
        }
    }
}

impl From<Style> for length::Time {
    fn from(value: Style) -> Self {
        match value {
            Style::Full => length::Time::Full,
            Style::Long => length::Time::Long,
            Style::Medium => length::Time::Medium,
            Style::Short => length::Time::Short,
        }
    }
}

pub struct Options {
    // local options
    calendar: Option<String>,
    hour12: Option<bool>,
    hour_cycle: Option<String>,
    time_zone: TimeZone,
    // Datetime components
    weekday: Option<TextLength>,

    era: Option<TextLength>,
    year: Option<NumericLength>,
    month: Option<MonthLength>,
    day: Option<NumericLength>,
    hour: Option<NumericLength>,
    minute: Option<NumericLength>,
    second: Option<NumericLength>,
    day_period: Option<TextLength>,
    timezone_name: Option<JsTimeZoneName>,
    date_style: Option<Style>,
    time_style: Option<Style>,
}

impl<'js> FromJs<'js> for Options {
    fn from_js(ctx: &rquickjs::Ctx<'js>, value: rquickjs::Value<'js>) -> rquickjs::Result<Self> {
        let obj = Object::from_js(ctx, value)?;

        let timezone: Option<TimeZone> = obj.get("timeZone")?;
        let timezone = match timezone {
            Some(t) => t,
            None => TimeZone::current(ctx)?,
        };
        Ok(Options {
            calendar: obj.get("calendar")?,
            hour12: obj.get("hour12")?,
            hour_cycle: obj.get("hourCycle")?,
            time_zone: timezone,
            timezone_name: obj.get("timeZoneName")?,
            weekday: obj.get("weekday")?,
            era: obj.get("era")?,
            year: obj.get("year")?,
            day: obj.get("day")?,
            hour: obj.get("hour")?,
            minute: obj.get("minute")?,
            second: obj.get("second")?,
            month: obj.get("month")?,
            day_period: obj.get("dayPeriod")?,
            date_style: obj.get("dateStyle")?,
            time_style: obj.get("timeStyle")?,
        })
    }
}

impl Options {
    pub fn get_default(ctx: &Ctx<'_>) -> rquickjs::Result<Options> {
        let timezone = TimeZone::current(ctx)?;

        Ok(Options {
            calendar: Default::default(),
            hour12: Default::default(),
            hour_cycle: Default::default(),
            timezone_name: Default::default(),
            time_zone: timezone,
            weekday: Default::default(),
            era: Default::default(),
            year: Default::default(),
            month: Default::default(),
            day: Default::default(),
            hour: Default::default(),
            minute: Default::default(),
            second: Default::default(),
            day_period: Default::default(),
            date_style: Default::default(),
            time_style: Default::default(),
        })
    }

    pub fn into_formatter_options(&self) -> rquickjs::Result<DateTimeFormatterOptions> {
        if self.date_style.is_none() && self.time_style.is_none() {
            let mut opts = Bag::default();

            opts.day = self.day.map(Into::into);
            opts.era = self.era.map(Into::into);
            opts.month = self.month.map(Into::into);
            opts.year = self.year.map(Into::into);
            opts.hour = self.hour.map(Into::into);
            opts.minute = self.minute.map(Into::into);
            opts.weekday = self.weekday.map(Into::into);
            opts.second = self.second.map(Into::into);
            opts.time_zone_name = self.timezone_name.map(Into::into);

            if opts == Bag::empty() {
                let opts = length::Bag::from_date_style(length::Date::Short);

                Ok(DateTimeFormatterOptions::Length(opts))
            } else {
                Ok(DateTimeFormatterOptions::Components(opts))
            }
        } else {
            let mut opts = length::Bag::default();

            opts.date = self.date_style.map(Into::into);
            opts.time = self.time_style.map(Into::into);

            Ok(DateTimeFormatterOptions::Length(opts))
        }
    }

    pub fn resolve_options(
        self,
        ctx: &Ctx<'_>,
        local: &DataLocale,
    ) -> rquickjs::Result<ResolvedOptions> {
        let provider = WinterCG::get(&ctx)?.borrow().icu_provider(&ctx).cloned()?;

        let calendar = if let Some(calendar) = &self.calendar {
            let Some(kind) = AnyCalendarKind::get_for_bcp47_string(&calendar) else {
                throw!(ctx, "Unknown calendar")
            };
            throw_if!(ctx, AnyCalendar::try_new_unstable(&provider, kind))
        } else {
            throw_if!(
                ctx,
                AnyCalendar::try_new_for_locale_unstable(&provider, local)
            )
        };

        Ok(ResolvedOptions {
            calendar: calendar.into(),
            hour12: self.hour12,
            hour_cycle: self.hour_cycle,
            time_zone: self.time_zone,
            weekday: self.weekday,
            era: self.era,
            year: self.year,
            month: self.month,
            day: self.day,
            hour: self.hour,
            minute: self.minute,
            second: self.second,
            day_period: self.day_period,
            time_style: self.time_style,
            date_style: self.date_style,
        })
    }
}

pub struct ResolvedOptions {
    // local options
    pub calendar: Rc<AnyCalendar>,
    pub hour12: Option<bool>,
    pub hour_cycle: Option<String>,
    pub time_zone: TimeZone,
    // Datetime components
    weekday: Option<TextLength>,

    era: Option<TextLength>,
    year: Option<NumericLength>,
    month: Option<MonthLength>,
    day: Option<NumericLength>,
    hour: Option<NumericLength>,
    minute: Option<NumericLength>,
    second: Option<NumericLength>,
    day_period: Option<TextLength>,

    date_style: Option<Style>,
    time_style: Option<Style>,
}
