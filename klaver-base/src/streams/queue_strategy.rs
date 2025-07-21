use rquickjs::{Class, Ctx, FromJs, Function, Object, Value, class::Trace};
use rquickjs_util::{Buffer, StringRef};

#[derive(rquickjs::JsLifetime)]
#[rquickjs::class]
#[derive(Debug, Trace)]
pub struct CountQueuingStrategy {
    #[qjs(get, rename = "highWaterMark")]
    pub high_water_mark: u64,
}

#[rquickjs::methods]
impl CountQueuingStrategy {
    #[qjs(constructor)]
    pub fn new(options: Option<Object<'_>>) -> rquickjs::Result<CountQueuingStrategy> {
        let high_water_mark = if let Some(options) = options {
            options.get::<_, Option<u64>>("highWaterMark")?.unwrap_or(1)
        } else {
            1
        };

        Ok(CountQueuingStrategy { high_water_mark })
    }

    pub fn size(&self, _chunk: Value<'_>) -> u64 {
        1
    }
}

create_export!(CountQueuingStrategy);

#[derive(rquickjs::JsLifetime)]
#[rquickjs::class]
#[derive(Debug, Trace)]
pub struct ByteLengthQueuingStrategy {
    #[qjs(get, rename = "highWaterHighway")]
    pub high_water_mark: u64,
}

#[rquickjs::methods]
impl ByteLengthQueuingStrategy {
    #[qjs(constructor)]
    pub fn new(options: Option<Object<'_>>) -> rquickjs::Result<ByteLengthQueuingStrategy> {
        let high_water_mark = if let Some(options) = options {
            options
                .get::<_, Option<u64>>("highWaterMark")?
                .unwrap_or(1)
                .max(1)
        } else {
            1
        };

        Ok(ByteLengthQueuingStrategy { high_water_mark })
    }

    pub fn size<'js>(&self, ctx: Ctx<'js>, chunk: Value<'js>) -> u64 {
        if let Ok(buffer) = Buffer::from_js(&ctx, chunk.clone()) {
            buffer.len() as u64
        } else if let Ok(str) = StringRef::from_js(&ctx, chunk) {
            str.as_str().len() as u64
        } else {
            1
        }
    }
}

create_export!(ByteLengthQueuingStrategy);

#[derive(Trace, Clone)]
pub enum QueuingStrategy<'js> {
    Count(Class<'js, CountQueuingStrategy>),
    BytesLength(Class<'js, ByteLengthQueuingStrategy>),
    Custom {
        high_water_mark: u64,
        size: Function<'js>,
    },
}

impl<'js> QueuingStrategy<'js> {
    pub fn create_default(ctx: Ctx<'js>) -> rquickjs::Result<QueuingStrategy<'js>> {
        Ok(Self::Count(Class::instance(
            ctx,
            CountQueuingStrategy { high_water_mark: 1 },
        )?))
    }

    pub fn high_water_mark(&self) -> u64 {
        match self {
            Self::BytesLength(b) => b.borrow().high_water_mark,
            Self::Count(c) => c.borrow().high_water_mark,
            Self::Custom {
                high_water_mark, ..
            } => *high_water_mark,
        }
    }

    pub fn size(&self, ctx: Ctx<'js>, chunk: &Value<'js>) -> rquickjs::Result<u64> {
        match self {
            QueuingStrategy::Count(class) => Ok(class.borrow().size(chunk.clone())),
            QueuingStrategy::BytesLength(class) => Ok(class.borrow().size(ctx, chunk.clone())),
            QueuingStrategy::Custom { size, .. } => size.call((chunk,)),
        }
    }
}

impl<'js> FromJs<'js> for QueuingStrategy<'js> {
    fn from_js(ctx: &rquickjs::Ctx<'js>, value: rquickjs::Value<'js>) -> rquickjs::Result<Self> {
        if let Ok(count) = Class::<CountQueuingStrategy>::from_js(ctx, value.clone()) {
            Ok(Self::Count(count))
        } else if let Ok(count) = Class::<ByteLengthQueuingStrategy>::from_js(ctx, value.clone()) {
            Ok(Self::BytesLength(count))
        } else if let Ok(obj) = Object::from_value(value.clone()) {
            let high: u64 = obj.get("highWaterMark")?;
            let size: Function<'js> = obj.get("size")?;

            Ok(Self::Custom {
                high_water_mark: high,
                size,
            })
        } else {
            Err(rquickjs::Error::new_from_js(
                value.type_name(),
                "queuing streategy",
            ))
        }
    }
}
