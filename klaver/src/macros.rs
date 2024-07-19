#[macro_export]
macro_rules! throw {
    ($ctx: ident, $err: ident) => {
        return Err($ctx.throw(rquickjs::Value::from_exception(
            rquickjs::Exception::from_message($ctx.clone(), &*$err.to_string())?,
        )))
    };
}
