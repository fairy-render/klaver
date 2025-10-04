use rquickjs::{Array, FromJs, Function, IntoJs, prelude::This};

pub trait ArrayExt<'js> {
    fn push<V: IntoJs<'js>>(&self, value: V) -> rquickjs::Result<()>;
    fn pop<V: FromJs<'js>>(&self) -> rquickjs::Result<Option<V>>;
    fn join<T: FromJs<'js>, N: IntoJs<'js>>(&self, value: N) -> rquickjs::Result<T>;
    fn sort(&self) -> rquickjs::Result<()>;
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

    fn sort(&self) -> rquickjs::Result<()> {
        self.as_object()
            .get::<_, Function>("sort")?
            .call((This(self.clone()),))
    }
}
