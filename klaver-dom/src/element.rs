use crate::{class_list::ClassList, lock::Locket, node_list::NodeList};
use domjohnson::NodeId;
use locket::LockApi;
use rquickjs::{class::Trace, Class, Ctx};

#[rquickjs::class(rename = "Element")]
pub struct JsElement {
    pub dom: Locket<domjohnson::Document>,
    pub id: NodeId,
}

impl<'js> Trace<'js> for JsElement {
    fn trace<'a>(&self, _tracer: rquickjs::class::Tracer<'a, 'js>) {}
}

#[rquickjs::methods]
impl JsElement {
    pub fn kind(&self, ctx: Ctx<'_>) -> Result<(), rquickjs::Error> {
        let Some(el) = self.dom.read().unwrap().get(self.id) else {
            fail!(ctx, "Element does no longer exists")
        };

        Ok(())
    }

    #[qjs(get, rename = "appendChild")]
    pub fn append_child<'js>(&self, child: Class<'js, JsElement>) -> rquickjs::Result<()> {
        let mut dom = self.dom.write().unwrap();
        dom.append(self.id, child.try_borrow()?.id);
        Ok(())
    }

    pub fn remove(&self) -> rquickjs::Result<()> {
        let mut dom = self.dom.write().unwrap();
        dom.remove(self.id);
        Ok(())
    }

    pub fn remove_child<'js>(
        &self,
        ctx: Ctx<'js>,
        child: Class<'js, JsElement>,
    ) -> rquickjs::Result<Class<'js, JsElement>> {
        let mut dom = self.dom.write().unwrap();

        if let Some(_) = dom.children(self.id).find(|id| *id == child.borrow().id) {
            dom.remove(child.borrow().id);
        };

        Ok(child)
    }

    #[qjs(rename = "getAttribute")]
    pub fn get_attribute(&self, name: String) -> rquickjs::Result<Option<String>> {
        let dom = self.dom.write().unwrap();
        Ok(dom
            .get(self.id)
            .unwrap()
            .as_element()
            .unwrap()
            .attr(&name)
            .cloned())
    }

    #[qjs(get, rename = "classList")]
    pub fn class_list(&self) -> rquickjs::Result<ClassList> {
        Ok(ClassList {
            node: self.id,
            dom: self.dom.clone(),
        })
    }

    #[qjs(get, rename = "innerHTML")]
    pub fn inner_html(&self) -> rquickjs::Result<String> {
        let dom = self.dom.read().unwrap();
        Ok(dom.inner_html(self.id))
    }

    #[qjs(get, rename = "innerText")]
    pub fn inner_text(&self) -> rquickjs::Result<String> {
        let dom = self.dom.read().unwrap();
        Ok(dom.text(self.id).map(|m| &**m).collect::<String>())
    }

    #[qjs(rename = "querySelector")]
    pub fn query_selector(&self, query: String) -> Option<JsElement> {
        let dom = self.dom.read().expect("dom");
        dom.select_from(self.id, &query).get(0).map(|id| JsElement {
            dom: self.dom.clone(),
            id,
        })
    }

    #[qjs(rename = "querySelectorAll")]
    pub fn query_selector_all(&self, query: String) -> NodeList {
        let dom = self.dom.read().expect("dom");
        NodeList::new(self.dom.clone(), dom.select_from(self.id, &query).into())
    }
}
