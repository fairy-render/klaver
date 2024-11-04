use domjohnson::{CaseSensitivity, NodeId};
use locket::LockApi;
use rquickjs::{class::Trace, prelude::Rest, Ctx, IntoJs, Value};
use rquickjs_util::throw_if;

use crate::lock::Locket;

#[rquickjs::class]
pub struct ClassList {
    pub node: NodeId,
    pub dom: Locket<domjohnson::Document>,
}

impl<'js> Trace<'js> for ClassList {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {}
}

#[rquickjs::methods]
impl ClassList {
    #[qjs(get)]
    pub fn length(&self, ctx: Ctx<'_>) -> rquickjs::Result<usize> {
        let dom = throw_if!(ctx, self.dom.read());
        let count = dom
            .get(self.node)
            .unwrap()
            .as_element()
            .unwrap()
            .classes()
            .count();
        Ok(count)
    }

    #[qjs(get)]
    pub fn value(&self, ctx: Ctx<'_>) -> rquickjs::Result<String> {
        let dom = throw_if!(ctx, self.dom.read());
        let ret = dom
            .get(self.node)
            .unwrap()
            .as_element()
            .unwrap()
            .classes()
            .collect::<String>();

        Ok(ret)
    }

    pub fn item(&self, ctx: Ctx<'_>, idx: usize) -> rquickjs::Result<Option<String>> {
        let dom = throw_if!(ctx, self.dom.read());
        let ret = dom
            .get(self.node)
            .unwrap()
            .as_element()
            .unwrap()
            .classes()
            .nth(idx);

        Ok(ret.map(ToString::to_string))
    }

    pub fn add(&self, ctx: Ctx<'_>, Rest(classes): Rest<String>) -> rquickjs::Result<()> {
        let mut dom = throw_if!(ctx, self.dom.write());
        for class in classes {
            dom.get_mut(self.node)
                .unwrap()
                .as_element_mut()
                .unwrap()
                .append_class(&class);
        }

        Ok(())
    }

    pub fn remove(&self, ctx: Ctx<'_>, Rest(classes): Rest<String>) -> rquickjs::Result<()> {
        let mut dom = throw_if!(ctx, self.dom.write());
        for class in classes {
            dom.get_mut(self.node)
                .unwrap()
                .as_element_mut()
                .unwrap()
                .remove_class(&class);
        }

        Ok(())
    }

    pub fn toggle(&self, ctx: Ctx<'_>, class: String) -> rquickjs::Result<()> {
        let mut dom = throw_if!(ctx, self.dom.write());

        let element = dom.get_mut(self.node).unwrap().as_element_mut().unwrap();

        if element.has_class(&class, CaseSensitivity::CaseSensitive) {
            element.remove_class(&class);
        } else {
            element.append_class(&class);
        }

        Ok(())
    }

    pub fn values<'js>(&self, ctx: Ctx<'js>) -> rquickjs::Result<Value<'js>> {
        let dom = throw_if!(ctx, self.dom.write());

        let element = dom.get(self.node).unwrap().as_element().unwrap();

        let items = element
            .classes()
            .map(ToString::to_string)
            .collect::<Vec<_>>();

        rquickjs_util::iterator::NativeIter::new(items.into_iter()).into_js(&ctx)
    }
}
