use rquickjs::{
    atom::PredefinedAtom,
    function::Args,
    prelude::{IntoArgs, This},
    Array, Ctx, FromJs, Function, IntoAtom, IntoJs, Object, Symbol, Value,
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

pub trait ArrayExt<'js> {
    fn push<V: IntoJs<'js>>(&self, value: V) -> rquickjs::Result<()>;
    fn pop<V: FromJs<'js>>(&self) -> rquickjs::Result<Option<V>>;
    fn join<T: FromJs<'js>, N: IntoJs<'js>>(&self, value: N) -> rquickjs::Result<T>;
}

impl<'js> ArrayExt<'js> for Array<'js> {
    fn pop<V: FromJs<'js>>(&self) -> rquickjs::Result<Option<V>> {
        self.as_object()
            .get::<_, Function>("pop")?
            .call((This(self.clone()),))
    }

    fn push<V: IntoJs<'js>>(&self, value: V) -> rquickjs::Result<()> {
        self.as_object()
            .get::<_, Function>("push")?
            .call((This(self.clone()), value))
    }

    fn join<T: FromJs<'js>, N: IntoJs<'js>>(&self, value: N) -> rquickjs::Result<T> {
        self.as_object()
            .get::<_, Function>("join")?
            .call((This(self.clone()), value))
    }
}

pub trait StringExt<'js> {
    fn starts_with<K: IntoJs<'js>>(&self, ctx: Ctx<'js>, prefix: K) -> rquickjs::Result<bool>;
    fn length(&self, ctx: Ctx<'js>) -> rquickjs::Result<usize>;
}

impl<'js> StringExt<'js> for rquickjs::String<'js> {
    fn starts_with<K: IntoJs<'js>>(&self, ctx: Ctx<'js>, prefix: K) -> rquickjs::Result<bool> {
        self.call_property(ctx, "startsWith", (prefix,))
    }

    fn length(&self, ctx: Ctx<'js>) -> rquickjs::Result<usize> {
        ctx.eval::<Function, _>("(a) => a.length")?
            .call((self.clone(),))
    }
}
