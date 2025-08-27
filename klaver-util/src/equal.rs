use rquickjs::{Ctx, Object, Type, Value};

use crate::StringExt;

pub fn equal<'js>(ctx: &Ctx<'js>, left: Value<'js>, right: Value<'js>) -> rquickjs::Result<bool> {
    if left == right {
        return Ok(true);
    }

    // Both Arrays
    if let Some(la) = left.as_array()
        && let Some(ra) = right.as_array()
    {
        if la.len() != ra.len() {
            Ok(false)
        } else {
            let iter = la.iter::<Value<'js>>().zip(ra.iter::<Value<'js>>());

            for (l, r) in iter {
                let (l, r) = (l?, r?);
                if !equal(ctx, l, r)? {
                    return Ok(false);
                }
            }
            Ok(true)
        }
    // Both big ints
    } else if let Some(l) = left.as_big_int()
        && let Some(r) = right.as_big_int()
    {
        Ok(l.clone().to_i64()? == r.clone().to_i64()?)
    // Both string
    } else if let Some(l) = left.as_string()
        && let Some(r) = right.as_string()
    {
        Ok(l.str_ref()? == r.str_ref()?)
    } else {
        if left.type_of() != right.type_of() {
            return Ok(false);
        }

        let left: Object = left.get()?;
        let right: Object = right.get()?;

        let left_keys: Vec<Value<'js>> = left
            .keys::<Value<'js>>()
            .collect::<rquickjs::Result<Vec<_>>>()?;

        let right_keys: Vec<Value<'js>> = right
            .keys::<Value<'js>>()
            .collect::<rquickjs::Result<Vec<_>>>()?;

        if left_keys.len() != right_keys.len() {
            return Ok(false);
        }

        Ok(false)
    }
}
