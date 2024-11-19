use chrono::{Local, Offset as _, TimeZone as _};
use icu_calendar::{DateTime, Iso};
use icu_timezone::{CustomTimeZone, GmtOffset, MetazoneCalculator, TimeZoneIdMapper};
use rquickjs::{Ctx, FromJs};
use rquickjs_util::{throw, throw_if};

pub fn current_timezone() -> Option<TimeZone> {
    let tz = localzone::get_local_zone()?;
    Some(TimeZone(tz.parse().ok()?))
}

pub struct TimeZone(chrono_tz::Tz);

impl<'js> FromJs<'js> for TimeZone {
    fn from_js(ctx: &Ctx<'js>, value: rquickjs::Value<'js>) -> rquickjs::Result<Self> {
        let timezone = String::from_js(ctx, value)?;

        let tz = if let Result::<CustomTimeZone, _>::Ok(tz) = timezone.parse() {
            let tz: chrono_tz::Tz = throw_if!(ctx, tz.time_zone_id.unwrap().as_str().parse());
            tz
        } else if let Result::<chrono_tz::Tz, _>::Ok(tz) = timezone.parse() {
            tz
        } else {
            throw!(ctx, "Invalid timezone")
        };

        Ok(TimeZone(tz))
    }
}

impl TimeZone {
    pub fn current(ctx: &Ctx<'_>) -> rquickjs::Result<Self> {
        let Some(tz) = current_timezone() else {
            throw!(ctx, "Could not uptain timezone")
        };

        Ok(tz)
    }

    pub fn adjust<T: chrono::TimeZone>(
        &self,
        date: chrono::DateTime<T>,
    ) -> chrono::DateTime<chrono_tz::Tz> {
        date.with_timezone(&self.0)
    }

    pub fn custom_timezone(
        &self,
        ctx: &Ctx<'_>,
        datetime: &DateTime<Iso>,
    ) -> rquickjs::Result<CustomTimeZone> {
        let mapper = TimeZoneIdMapper::new();
        let mzc = MetazoneCalculator::new();

        let mut timezone = CustomTimeZone::new_empty();
        timezone.time_zone_id = mapper.as_borrowed().iana_to_bcp47(self.0.name());

        let offset = self
            .0
            .offset_from_utc_datetime(&Local::now().naive_utc())
            .fix()
            .local_minus_utc();

        timezone.gmt_offset = throw_if!(ctx, GmtOffset::try_from_offset_seconds(offset)).into();

        timezone.maybe_calculate_metazone(&mzc, &datetime);

        Ok(timezone)
    }
}
