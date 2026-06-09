use rquickjs::{Ctx, Filter, Object, Value};

use crate::{ArrayExt, ObjectExt, StringExt};

// TODO: Map, Set, Regex, Buffer
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

        let left_keys = left.keys_array(Filter::default())?;

        let right_keys = right.keys_array(Filter::default())?;

        if left_keys.len() != right_keys.len() {
            return Ok(false);
        }

        left_keys.sort()?;
        right_keys.sort()?;

        if !equal(ctx, left_keys.clone().into_value(), right_keys.into_value())? {
            return Ok(false);
        }

        for key in left_keys.iter::<Value<'js>>() {
            let key = key?;
            let l = left.get::<_, Value>(key.clone())?;
            let r = right.get::<_, Value>(key)?;
            if !equal(ctx, l, r)? {
                return Ok(false);
            }
        }

        Ok(true)
    }
}
