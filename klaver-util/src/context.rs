use rquickjs::Ctx;

pub trait AsContext<'js> {
    fn as_ctx(&self) -> &Ctx<'js>;
}

impl<'js> AsContext<'js> for Ctx<'js> {
    fn as_ctx(&self) -> &Ctx<'js> {
        self
    }
}

impl<'js, 'a, T> AsContext<'js> for &'a T
where
    T: AsContext<'js>,
{
    fn as_ctx(&self) -> &Ctx<'js> {
        (**self).as_ctx()
    }
}
