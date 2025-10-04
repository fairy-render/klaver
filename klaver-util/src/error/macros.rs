#[macro_export]
macro_rules! throw {
    ($ctx: expr, $err: expr) => {
        return {
            use $crate::AsContext;
            Err($crate::rquickjs::Exception::throw_message(
                $ctx.as_ctx(),
                &*$err.to_string(),
            ))
        }
    };
    (@type $ctx: expr, $err: expr) => {
        return {
            use $crate::AsContext;
            Err($crate::rquickjs::Exception::throw_type(
                $ctx.as_ctx(),
                &*$err.to_string(),
            ))
        }
    };
    (@range $ctx: expr, $err: expr) => {
        return {
            use $crate::AsContext;
            Err($crate::rquickjs::Exception::throw_range(
                $ctx.as_ctx(),
                &*$err.to_string(),
            ))
        }
    };
    (@internal $ctx: expr, $err: expr) => {
        return {
            use $crate::AsContext;
            Err($crate::rquickjs::Exception::throw_internal(
                $ctx.as_ctx(),
                &*$err.to_string(),
            ))
        }
    };
    (@reference $ctx: expr, $err: expr) => {
        return {
            use $crate::AsContext;
            Err($crate::rquickjs::Exception::throw_reference(
                $ctx.as_ctx(),
                &*$err.to_string(),
            ))
        }
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
    (@type $ctx: expr, $ret: expr) => {
        match $ret {
            Ok(ret) => ret,
            Err(err) => $crate::throw!(@type $ctx, err),
        }
    };
    (@range $ctx: expr, $ret: expr) => {
      match $ret {
          Ok(ret) => ret,
          Err(err) => $crate::throw!(@range $ctx, err),
      }
  };
    (@internal $ctx: expr, $ret: expr) => {
      match $ret {
          Ok(ret) => ret,
          Err(err) => $crate::throw!(@internal $ctx, err),
      }
    };
    (@reference $ctx: expr, $ret: expr) => {
      match $ret {
          Ok(ret) => ret,
          Err(err) => $crate::throw!(@reference $ctx, err),
      }
    };
}
