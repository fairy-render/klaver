use std::borrow::Cow;

pub struct Typings {
    pub name: Cow<'static, str>,
    pub typings: Cow<'static, str>,
}
