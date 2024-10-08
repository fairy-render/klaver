#[macro_export]
macro_rules! throw {
    ($ctx: expr, $err: expr) => {
        return Err($ctx.throw($crate::quick::Value::from_exception(
            $crate::quick::Exception::from_message($ctx.clone(), &*$err.to_string())?,
        )))
    };
}

#[macro_export]
macro_rules! throw_if {
    ($ctx: expr, $ret: expr) => {
        match $ret {
            Ok(ret) => ret,
            Err(err) => $crate::throw!($ctx, err),
        }
    };
}
