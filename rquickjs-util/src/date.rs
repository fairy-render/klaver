use rquickjs::{class::Trace, function::This, Ctx, FromJs, Function, IntoJs, Object, Value};

use crate::StringRef;

#[derive(Debug, Trace)]
pub struct Date<'js> {
    object: Object<'js>,
}

impl<'js> Date<'js> {
    pub fn year(&self) -> rquickjs::Result<i32> {
        let func = self.object.get::<_, Function>("getFullYear")?;
        func.call((This(self.object.clone()),))
    }

    pub fn month(&self) -> rquickjs::Result<u8> {
        let func = self.object.get::<_, Function>("getMonth")?;
        func.call((This(self.object.clone()),))
    }

    pub fn date(&self) -> rquickjs::Result<u8> {
        let func = self.object.get::<_, Function>("getDate")?;
        func.call((This(self.object.clone()),))
    }

    pub fn hours(&self) -> rquickjs::Result<u8> {
        let func = self.object.get::<_, Function>("getHours")?;
        func.call((This(self.object.clone()),))
    }

    pub fn minutes(&self) -> rquickjs::Result<u8> {
        let func = self.object.get::<_, Function>("getMinutes")?;
        func.call((This(self.object.clone()),))
    }

    pub fn seconds(&self) -> rquickjs::Result<u8> {
        let func = self.object.get::<_, Function>("getSeconds")?;
        func.call((This(self.object.clone()),))
    }

    pub fn timezone_offset(&self) -> rquickjs::Result<i32> {
        let func = self.object.get::<_, Function>("getTimezoneOffset")?;
        func.call((This(self.object.clone()),))
    }

    pub fn utc_year(&self) -> rquickjs::Result<i32> {
        let func = self.object.get::<_, Function>("getUTCFullYear")?;
        func.call((This(self.object.clone()),))
    }

    pub fn utc_month(&self) -> rquickjs::Result<u8> {
        let func = self.object.get::<_, Function>("getUTCMonth")?;
        func.call((This(self.object.clone()),))
    }

    pub fn utc_date(&self) -> rquickjs::Result<u8> {
        let func = self.object.get::<_, Function>("getUTCDate")?;
        func.call((This(self.object.clone()),))
    }

    pub fn utc_hours(&self) -> rquickjs::Result<u8> {
        let func = self.object.get::<_, Function>("getUTCHours")?;
        func.call((This(self.object.clone()),))
    }

    pub fn utc_minutes(&self) -> rquickjs::Result<u8> {
        let func = self.object.get::<_, Function>("getUTCMinutes")?;
        func.call((This(self.object.clone()),))
    }

    pub fn utc_seconds(&self) -> rquickjs::Result<u8> {
        let func = self.object.get::<_, Function>("getUTCSeconds")?;
        func.call((This(self.object.clone()),))
    }

    pub fn is(ctx: &Ctx<'js>, value: &rquickjs::Value<'js>) -> rquickjs::Result<bool> {
        let date_ctor: Value<'_> = ctx.globals().get::<_, Value>("Date")?;

        let Some(obj) = value.as_object() else {
            return Ok(false);
        };

        Ok(obj.is_instance_of(&date_ctor))
    }

    pub fn timestamp(&self) -> rquickjs::Result<i64> {
        let func = self.object.get::<_, Function>("getTime")?;
        func.call((This(self.object.clone()),))
    }

    #[cfg(feature = "chrono")]
    pub fn to_datetime(self) -> rquickjs::Result<chrono::DateTime<chrono::Utc>> {
        let Some(date) = chrono::DateTime::from_timestamp_millis(self.timestamp()?) else {
            panic!()
        };

        Ok(date)
    }

    #[cfg(feature = "chrono")]
    pub fn from_chrono<T: chrono::TimeZone>(
        ctx: &Ctx<'js>,
        date: chrono::DateTime<T>,
    ) -> rquickjs::Result<Date<'js>> {
        let date_string = date.to_rfc3339();
        let ctor = ctx.eval::<Function, _>("(dateString) => new Date(dateString)")?;
        let date_obj = ctor.call::<_, Value>((date_string,))?;
        Date::from_js(ctx, date_obj)
    }

    pub fn to_string(&self) -> rquickjs::Result<StringRef<'js>> {
        let func = self.object.get::<_, Function>("toString")?;
        func.call((This(self.object.clone()),))
    }
}

impl<'js> FromJs<'js> for Date<'js> {
    fn from_js(
        ctx: &rquickjs::prelude::Ctx<'js>,
        value: rquickjs::Value<'js>,
    ) -> rquickjs::Result<Self> {
        let date_ctor: Value<'_> = ctx.globals().get::<_, Value>("Date")?;

        let obj = value
            .try_into_object()
            .map_err(|_| rquickjs::Error::new_from_js("value", "date"))?;

        if !obj.is_instance_of(&date_ctor) {
            return Err(rquickjs::Error::new_from_js("object", "date"));
        }

        Ok(Date { object: obj })
    }
}

impl<'js> IntoJs<'js> for Date<'js> {
    fn into_js(self, _ctx: &rquickjs::prelude::Ctx<'js>) -> rquickjs::Result<Value<'js>> {
        Ok(self.object.into())
    }
}

#[cfg(test)]
mod test {
    use rquickjs::{Context, Runtime};

    use crate::Date;

    #[test]
    fn test_date() {
        let runtime = Runtime::new().unwrap();
        let context = Context::full(&runtime).unwrap();

        context
            .with(|ctx| {
                //
                let _date = ctx.eval::<Date, _>("new Date")?;

                rquickjs::Result::Ok(())
            })
            .unwrap();
    }
}
