use rquickjs::{Ctx, Value, function::Opt};

pub fn is_plain_object<'js>(
    ctx: &Ctx<'js>,
    obj: &Value<'js>,
    strict: Opt<bool>,
) -> rquickjs::Result<bool> {
    if obj.is_null() || obj.is_undefined() {
        return Ok(false);
    }

    let Some(obj) = obj.as_object() else {
        return Ok(false);
    };

    let strict = strict.unwrap_or(true);

    let object_ctor = ctx.globals().get::<_, Value<'js>>("Object")?;
    let ctor = obj.get::<_, Value<'js>>("constructor")?;

    let is_instance = obj.is_instance_of(&object_ctor);
    let is_typeof = obj.type_of() == rquickjs::Type::Object;
    let is_ctor_undefined = ctor.is_undefined() || ctor.is_null();
    let is_ctor_object = ctor == object_ctor;
    let is_ctor_fn =
        ctor.type_of() == rquickjs::Type::Function || ctor.type_of() == rquickjs::Type::Constructor;

    Ok(if strict {
        (is_instance || is_typeof) && (is_ctor_undefined || is_ctor_object)
    } else {
        is_ctor_undefined || is_ctor_fn
    })
}
