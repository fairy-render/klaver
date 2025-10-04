macro_rules! fail {
    ($ctx: ident, $msg: expr) => {
        return Err($ctx.throw(rquickjs::Value::from_exception(
            rquickjs::Exception::from_message($ctx.clone(), $msg)?,
        )))
    };
}
