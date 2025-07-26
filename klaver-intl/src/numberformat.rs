use fixed_decimal::{FixedDecimal, FloatPrecision};
use icu::decimal::DecimalError;
use klaver_base::create_export;
use klaver_util::{throw, throw_if};
use rquickjs::{Ctx, FromJs, JsLifetime, Value, class::Trace, prelude::Opt};

use crate::provider::DynProvider;

use super::{datetime::LocalesInit, locale::current_local};

pub enum Number {
    Float(f64),
    Int(i32),
    BigInt(i64),
}

impl Number {
    pub fn to_fixed_decimal(
        self,
        ctx: &Ctx<'_>,
        precision: FloatPrecision,
    ) -> rquickjs::Result<FixedDecimal> {
        match self {
            Self::BigInt(i) => Ok(FixedDecimal::from(i)),
            Self::Int(i) => Ok(FixedDecimal::from(i)),
            Self::Float(i) => {
                let ret = throw_if!(ctx, FixedDecimal::try_from_f64(i, precision));
                Ok(ret)
            }
        }
    }
}

impl<'js> FromJs<'js> for Number {
    fn from_js(_ctx: &Ctx<'js>, value: Value<'js>) -> rquickjs::Result<Self> {
        if let Some(i) = value.as_float() {
            Ok(Number::Float(i))
        } else if let Some(i) = value.as_int() {
            Ok(Number::Int(i))
        } else if let Some(i) = value.as_big_int() {
            Ok(Number::BigInt(i.clone().to_i64()?))
        } else {
            Err(rquickjs::Error::new_from_js(value.type_name(), "number"))
        }
    }
}

#[derive(JsLifetime)]
#[rquickjs::class]
pub struct NumberFormat {
    decimal: icu::decimal::FixedDecimalFormatter,
    precision: FloatPrecision,
}

impl<'js> Trace<'js> for NumberFormat {
    fn trace<'a>(&self, _tracer: rquickjs::class::Tracer<'a, 'js>) {}
}

#[rquickjs::methods]
impl NumberFormat {
    #[qjs(constructor)]
    pub fn new(ctx: Ctx<'_>, init: Opt<LocalesInit<'_>>) -> rquickjs::Result<NumberFormat> {
        let Some(provider) = ctx.userdata::<DynProvider>() else {
            throw!(ctx, "No ICU provider found")
        };

        let mut err: Option<DecimalError> = None;

        if let Some(init) = init.0 {
            for locale in init.into_locale(&ctx)? {
                let date_locale = locale.into();

                let formatter = icu::decimal::FixedDecimalFormatter::try_new_unstable(
                    &*provider,
                    &date_locale,
                    Default::default(),
                );

                match formatter {
                    Ok(ret) => {
                        return Ok(NumberFormat {
                            decimal: ret,
                            precision: FloatPrecision::Floating,
                        });
                    }
                    Err(e) => {
                        err = Some(e);
                    }
                }
            }
        }

        if let Some(err) = err {
            throw!(ctx, err)
        } else {
            let locale = current_local(&ctx)?.into();
            let formatter = throw_if!(
                ctx,
                icu::decimal::FixedDecimalFormatter::try_new_unstable(
                    &*provider,
                    &locale,
                    Default::default(),
                )
            );

            Ok(NumberFormat {
                decimal: formatter,
                precision: FloatPrecision::Floating,
            })
        }
    }

    pub fn format(&self, ctx: Ctx<'_>, number: Number) -> rquickjs::Result<String> {
        let decimal = number.to_fixed_decimal(&ctx, self.precision)?;
        Ok(self.decimal.format_to_string(&decimal))
    }
}

create_export!(NumberFormat);
