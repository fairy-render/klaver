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
      <$name as $crate::Exportable<'js>>::export($ctx, $registry, $target)?;
    )+
  };
}

#[macro_export]
macro_rules! create_export {
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
