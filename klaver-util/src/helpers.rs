use rquickjs::{Array, Ctx, FromJs, IntoJs, Value};

use crate::array::ArrayExt;

pub struct Pair<K, V> {
    pub key: K,
    pub value: V,
}

impl<'js, K, V> FromJs<'js> for Pair<K, V>
where
    K: FromJs<'js>,
    V: FromJs<'js>,
{
    fn from_js(ctx: &Ctx<'js>, value: Value<'js>) -> rquickjs::Result<Self> {
        let array = Array::from_js(ctx, value)?;

        let key = array.get(0)?;
        let value = array.get(1)?;

        Ok(Pair { key, value })
    }
}

impl<'js, K, V> IntoJs<'js> for Pair<K, V>
where
    K: IntoJs<'js>,
    V: IntoJs<'js>,
{
    fn into_js(self, ctx: &Ctx<'js>) -> rquickjs::Result<Value<'js>> {
        let array = Array::new(ctx.clone())?;

        array.push(self.key)?;
        array.push(self.value)?;

        Ok(array.into_value())
    }
}
