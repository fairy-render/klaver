macro_rules! declare {
    ($module: ident, $($name: ident),+) => {
      $(
        $module.declare($name::NAME)?;
      )+
    };
}

macro_rules! define {
  ($module: ident, $ctx: ident, $($name: ident),+) => {
    $(
      $module.export($name::NAME, rquickjs::class::Class::<$name>::create_constructor($ctx)?.unwrap())?;
    )+
  };
}

#[macro_export]
macro_rules! export {
    ($item: ty) => {
        impl<'js> $crate::Exportable<'js> for $item {
            fn export<T>(
                ctx: &rquickjs::Ctx<'js>,
                _registry: &$crate::Registry,
                target: &T,
            ) -> rquickjs::Result<()>
            where
                T: $crate::ExportTarget<'js>,
            {
                target.set(
                    ctx,
                    <$item as rquickjs::class::JsClass<'js>>::NAME,
                    rquickjs::class::Class::<$item>::create_constructor(ctx)?,
                )
            }
        }
    };
}
