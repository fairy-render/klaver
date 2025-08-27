use rquickjs::{
    Array, Filter, FromJs, Function, IntoAtom, Object, Value, function::Args, prelude::IntoArgs,
};

use crate::ArrayExt;

pub trait ObjectExt<'js> {
    fn call_property<K: IntoAtom<'js>, A: IntoArgs<'js>, R: FromJs<'js>>(
        &self,
        props: K,
        args: A,
    ) -> rquickjs::Result<R>;

    fn keys_array(&self, filter: Filter) -> rquickjs::Result<Array<'js>>;
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

    fn keys_array(&self, filter: Filter) -> rquickjs::Result<Array<'js>> {
        let array = Array::new(self.ctx().clone())?;

        let iter = self.own_keys::<Value<'js>>(filter);

        for (idx, next) in iter.enumerate() {
            array.set(idx, next?)?;
        }

        Ok(array)
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

    fn keys_array(&self, filter: Filter) -> rquickjs::Result<Array<'js>> {
        self.get::<Object>()?.keys_array(filter)
    }
}
