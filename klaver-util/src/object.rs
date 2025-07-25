use rquickjs::{FromJs, Function, IntoAtom, Object, Value, function::Args, prelude::IntoArgs};

pub trait ObjectExt<'js> {
    fn call_property<K: IntoAtom<'js>, A: IntoArgs<'js>, R: FromJs<'js>>(
        &self,
        props: K,
        args: A,
    ) -> rquickjs::Result<R>;
}

impl<'js> ObjectExt<'js> for Object<'js> {
    fn call_property<K: IntoAtom<'js>, A: IntoArgs<'js>, R: FromJs<'js>>(
        &self,
        prop: K,
        args: A,
    ) -> rquickjs::Result<R> {
        let mut a = Args::new(self.ctx().clone(), args.num_args());
        args.into_args(&mut a)?;
        a.this(self.clone())?;
        self.get::<_, Function>(prop)?.call_arg(a)
    }
}

impl<'js> ObjectExt<'js> for Value<'js> {
    fn call_property<K: IntoAtom<'js>, A: IntoArgs<'js>, R: FromJs<'js>>(
        &self,
        prop: K,
        args: A,
    ) -> rquickjs::Result<R> {
        let mut a = Args::new(self.ctx().clone(), args.num_args());
        args.into_args(&mut a)?;
        a.this(self.clone())?;
        Object::from_js(self.ctx(), self.clone())?
            .get::<_, Function>(prop)?
            .call_arg(a)
    }
}
