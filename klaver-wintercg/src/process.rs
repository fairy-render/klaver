use rquickjs::{class::Trace, Array, Class, Ctx, FromIteratorJs, Object, Value};
use rquickjs_util::{create_proxy, throw, typed_map::TypedMap, Prop, ProxyHandler};

use crate::WinterCG;

#[derive(Trace)]
struct Env;

impl<'js> ProxyHandler<'js, TypedMap<'js, rquickjs::String<'js>, rquickjs::String<'js>>> for Env {
    fn get(
        &self,
        ctx: Ctx<'js>,
        target: TypedMap<'js, rquickjs::String<'js>, rquickjs::String<'js>>,
        prop: rquickjs_util::Prop<'js>,
        _receiver: rquickjs::Value<'js>,
    ) -> rquickjs::Result<rquickjs::Value<'js>> {
        let prop = match prop {
            Prop::String(prop) => prop,
            Prop::Symbol(_sym) => {
                // if sym == Symbol::iterator(ctx.clone()) {
                //     target.entries()?.this()
                // } else {
                //     return Ok(rquickjs::Value::new_undefined(ctx));
                // }
                return Ok(rquickjs::Value::new_undefined(ctx));
            }
        };

        // let Prop::String(prop) = prop else {
        //     return Ok(rquickjs::Value::new_undefined(ctx));
        // };

        match target.get(prop)? {
            Some(v) => Ok(v.into_value()),
            None => Ok(Value::new_undefined(ctx)),
        }
    }

    fn set(
        &self,
        ctx: Ctx<'js>,
        target: TypedMap<'js, rquickjs::String<'js>, rquickjs::String<'js>>,
        prop: Prop<'js>,
        value: Value<'js>,
        _receiver: Value<'js>,
    ) -> rquickjs::Result<bool> {
        let Prop::String(prop) = prop else {
            return Ok(false);
        };

        let Ok(value) = value.try_into_string() else {
            throw!(ctx, "value should be a string")
        };

        target.set(prop, value)?;

        Ok(true)
    }

    fn own_keys(
        &self,
        ctx: Ctx<'js>,
        target: TypedMap<'js, rquickjs::String<'js>, rquickjs::String<'js>>,
    ) -> rquickjs::Result<rquickjs::Array<'js>> {
        let entries = target
            .entries()?
            .map(|m| m.map(|(k, _)| k))
            .collect::<rquickjs::Result<Vec<_>>>()?;

        Array::from_iter_js(&ctx, entries)
    }
}

pub fn process<'js>(
    ctx: Ctx<'js>,
    winter: &Class<'js, WinterCG<'js>>,
) -> rquickjs::Result<Object<'js>> {
    let obj = Object::new(ctx.clone())?;

    obj.set(
        "env",
        create_proxy(ctx.clone(), winter.borrow().env().clone(), Env),
    )?;

    obj.set("args", Array::new(ctx)?)?;

    Ok(obj)
}
