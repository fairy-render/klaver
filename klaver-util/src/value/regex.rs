use rquickjs::{Object, class::Trace};

#[derive(Trace)]
pub struct Regexp<'js> {
    object: Object<'js>,
}

impl<'js> Regexp<'js> {}
