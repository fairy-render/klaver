use std::time::{Instant, SystemTime};

use rquickjs::{class::Trace, qjs, Ctx, Exception, Value};

#[rquickjs::class]
pub struct Performance {
    origin: SystemTime,
}

impl<'js> Trace<'js> for Performance {
    fn trace<'a>(&self, _tracer: rquickjs::class::Tracer<'a, 'js>) {}
}

#[rquickjs::methods]
impl Performance {
    #[qjs(constructor)]
    pub fn new() -> Performance {
        Performance {
            origin: SystemTime::now(),
        }
    }
    pub fn now<'js>(&self, ctx: Ctx<'js>) -> rquickjs::Result<f64> {
        let now = match self.origin.elapsed() {
            Ok(ret) => ret,
            Err(err) => {
                return Err(ctx.throw(Value::from_exception(Exception::from_message(
                    ctx.clone(),
                    &*err.to_string(),
                )?)))
            }
        };
        Ok(now.as_secs_f64() * 1000.)
    }

    #[qjs(get, rename = "timeOrigin")]
    pub fn time_origin<'js>(&self, ctx: Ctx<'js>) -> rquickjs::Result<f64> {
        let ret = match self.origin.duration_since(std::time::UNIX_EPOCH) {
            Ok(ret) => ret,
            Err(err) => {
                return Err(ctx.throw(Value::from_exception(Exception::from_message(
                    ctx.clone(),
                    &*err.to_string(),
                )?)))
            }
        };

        Ok(ret.as_secs_f64() * 1000.)
    }
}

// #[rquickjs::function]
// pub fn now<'js>(ctx: Ctx<'js>) -> rquickjs::Result<f64> {
// let ret = match std::time::UNIX_EPOCH.elapsed() {
//     Ok(ret) => ret,
//     Err(err) => {
//         return Err(ctx.throw(Value::from_exception(Exception::from_message(
//             ctx.clone(),
//             &*err.to_string(),
//         )?)))
//     }
// };

//     Ok(ret.as_secs_f64() * 1000f64)
// }
