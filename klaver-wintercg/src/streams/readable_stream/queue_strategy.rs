use rquickjs::{class::Trace, ArrayBuffer, Class, Ctx, FromJs, Function, Object};

#[rquickjs::class]
#[derive(Debug, Trace)]
pub struct CountQueuingStrategy {
    #[qjs(get, rename = "highWaterHighway")]
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

    pub fn size(&self, _chunk: ArrayBuffer<'_>) -> usize {
        1
    }
}

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
            options.get::<_, Option<u64>>("highWaterMark")?.unwrap_or(1)
        } else {
            1
        };

        Ok(ByteLengthQueuingStrategy { high_water_mark })
    }

    pub fn size(&self, chunk: ArrayBuffer<'_>) -> usize {
        chunk.len()
    }
}

#[derive(Trace, Clone)]
pub enum QueuingStrategy<'js> {
    Count(Class<'js, CountQueuingStrategy>),
    BytesLength(Class<'js, ByteLengthQueuingStrategy>),
    Custom {
        high_water_mark: u64,
        size: Option<Function<'js>>,
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
}

impl<'js> FromJs<'js> for QueuingStrategy<'js> {
    fn from_js(ctx: &rquickjs::Ctx<'js>, value: rquickjs::Value<'js>) -> rquickjs::Result<Self> {
        if let Ok(count) = Class::<CountQueuingStrategy>::from_js(ctx, value.clone()) {
            Ok(Self::Count(count))
        } else if let Ok(count) = Class::<ByteLengthQueuingStrategy>::from_js(ctx, value.clone()) {
            Ok(Self::BytesLength(count))
        } else if let Ok(obj) = Object::from_value(value.clone()) {
            let high: u64 = obj.get("highWaterMark")?;
            let size: Option<Function> = obj.get("size")?;

            Ok(Self::Custom {
                high_water_mark: high,
                size: size,
            })
        } else {
            Err(rquickjs::Error::new_from_js(
                value.type_name(),
                "queuing streategy",
            ))
        }
    }
}
