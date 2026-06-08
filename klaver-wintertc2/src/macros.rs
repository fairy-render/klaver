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
