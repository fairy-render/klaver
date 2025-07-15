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
      $module.export($name::NAME, rquickjs::class::Class::<$name>::create_constructor($ctx))?;
    )+
  };
}
