macro_rules! export {
  ($export: expr, $ctx: expr, $($instance: ty),*) => {
      $(
          let i = Class::<$instance>::create_constructor($ctx)?.expect(stringify!($instance));
          $export.export(stringify!($instance), i)?;
      )*
  };
}
