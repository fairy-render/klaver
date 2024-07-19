#[macro_export]
macro_rules! throw {
    ($ctx: ident, $err: expr) => {
        return Err($ctx.throw(rquickjs::Value::from_exception(
            rquickjs::Exception::from_message($ctx.clone(), &*$err.to_string())?,
        )))
    };
}

#[macro_export]
macro_rules! throw_if {
    ($ctx: ident, $ret: expr) => {
        match $ret {
            Ok(ret) => ret,
            Err(err) => throw!($ctx, err),
        }
    };
}
