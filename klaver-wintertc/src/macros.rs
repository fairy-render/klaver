macro_rules! declare {
    ($module: ident, $($name: ident),+) => {
      $(
        $module.declare($name::NAME)?;
      )+
    };
}

macro_rules! export {
  ($ctx: ident, $registry: ident, $target: ident, $($name: ident),+) => {
    $(
      <$name as klaver_core::Exportable<'js>>::export($ctx, $registry, $target)?;
    )+
  };
}

/// Throws a `DOMException` with the given spec `name` (e.g. `"OperationError"`,
/// `"NotSupportedError"`, `"InvalidAccessError"`) and message, exactly like `klaver_core::throw!`
/// does for plain `TypeError`/`RangeError`/etc. Reserve plain `klaver_core::throw!(@type ctx, ...)`
/// for argument-shape errors (wrong JS value type, missing required dict field) - WebCrypto's own
/// operational failures are always specifically-named `DOMException`s.
macro_rules! throw_dom {
    ($ctx: expr, $name: expr, $msg: expr) => {
        return Err($crate::dom_exception::DOMException::throw_named(
            &$ctx,
            $name,
            &$msg.to_string(),
        ))
    };
}
