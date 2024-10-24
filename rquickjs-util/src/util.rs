use rquickjs::{
    atom::PredefinedAtom, function::Args, object, prelude::IntoArgs, Ctx, FromJs, Function,
    IntoAtom, Object, Symbol, Value,
};

pub fn is_iterator(value: &Value<'_>) -> bool {
    let Some(obj) = value.as_object() else {
        return false;
    };

    obj.get::<_, Function>(PredefinedAtom::SymbolIterator)
        .is_ok()
}

pub fn is_async_iterator<'js>(ctx: &Ctx<'js>, value: &Value<'js>) -> bool {
    let Some(obj) = value.as_object() else {
        return false;
    };

    let symbol = Symbol::async_iterator(ctx.clone());

    obj.get::<_, Function>(symbol).is_ok()
}

pub trait ObjectExt<'js> {
    fn call_property<K: IntoAtom<'js>, A: IntoArgs<'js>, R: FromJs<'js>>(
        &self,
        ctx: Ctx<'js>,
        props: K,
        args: A,
    ) -> rquickjs::Result<R>;
}

impl<'js> ObjectExt<'js> for Object<'js> {
    fn call_property<K: IntoAtom<'js>, A: IntoArgs<'js>, R: FromJs<'js>>(
        &self,
        ctx: Ctx<'js>,
        prop: K,
        args: A,
    ) -> rquickjs::Result<R> {
        let mut a = Args::new(ctx, args.num_args());
        args.into_args(&mut a)?;
        a.this(self.clone())?;
        self.get::<_, Function>(prop)?.call_arg(a)
    }
}

impl<'js> ObjectExt<'js> for Value<'js> {
    fn call_property<K: IntoAtom<'js>, A: IntoArgs<'js>, R: FromJs<'js>>(
        &self,
        ctx: Ctx<'js>,
        prop: K,
        args: A,
    ) -> rquickjs::Result<R> {
        let mut a = Args::new(ctx.clone(), args.num_args());
        args.into_args(&mut a)?;
        a.this(self.clone())?;
        Object::from_js(&ctx, self.clone())?
            .get::<_, Function>(prop)?
            .call_arg(a)
    }
}

pub trait FunctionExt<'js> {
    fn bind<A: IntoArgs<'js>>(&self, ctx: Ctx<'js>, args: A) -> rquickjs::Result<Function<'js>>;
}

impl<'js> FunctionExt<'js> for Function<'js> {
    fn bind<A: IntoArgs<'js>>(&self, ctx: Ctx<'js>, args: A) -> rquickjs::Result<Function<'js>> {
        let mut a = Args::new(ctx.clone(), args.num_args());
        args.into_args(&mut a)?;
        a.this(self.clone())?;
        self.get::<_, Function>("bind")?.call_arg::<Function>(a)
    }
}
