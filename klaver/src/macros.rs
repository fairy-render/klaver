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
            Err(err) => $crate::throw!($ctx, err),
        }
    };
}

#[macro_export]
macro_rules! async_with{
  ($context:expr => |$ctx:ident| { $($t:tt)* }) => {
      $crate::vm::Vm::async_with(&$context, move|$ctx| {
          let fut = Box::pin(async move {
              $($t)*
          });
          /// SAFETY: While rquickjs objects have a 'js lifetime attached to them,
          /// they actually life much longer an the lifetime is just for checking
          /// if they belong to the correct context.
          /// By requiring that everything is moved into the closure outside
          /// environments still can't life shorter than the closure.
          /// This allows use to recast the future to a higher lifetime without problems.
          /// Second, the future will always acquire a lock before running. The closure
          /// enforces that everything moved into the future is send, but non of the
          /// rquickjs objects are send so the future will never be send.
          /// Since we acquire a lock before running the future and nothing can escape the closure
          /// and future it is safe to recast the future as send.
          unsafe fn uplift<'a,'b,R>(f: std::pin::Pin<Box<dyn std::future::Future<Output = Result<R, $crate::Error>> + 'a>>) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<R, $crate::Error>> + 'b + Send>>{
              std::mem::transmute(f)
          }
          unsafe{ uplift(fut) }
      })
  };
}
