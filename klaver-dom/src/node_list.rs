use domjohnson::{NodeId, Selection};
use locket::LockApi;
use rquickjs::{
    atom::PredefinedAtom, class::Trace, function::MutFn, Array, Class, Ctx, Function, IntoJs,
    Object, Value,
};

use crate::{element::JsElement, lock::Locket};

#[rquickjs::class]
pub struct Children {
    pub dom: Locket<domjohnson::Document>,
    pub node: NodeId,
}

impl<'js> Trace<'js> for Children {
    fn trace<'a>(&self, _tracer: rquickjs::class::Tracer<'a, 'js>) {}
}

#[rquickjs::methods]
impl Children {
    pub fn iter<'js>(&self, ctx: Ctx<'js>) -> rquickjs::Result<Object<'js>> {
        let res = Object::new(ctx.clone())?;

        let mut nodes = {
            let dom = self.dom.read().unwrap();
            let children = dom.children(self.node);
            children.collect::<Vec<_>>().into_iter()
        };

        let dom = self.dom.clone();

        res.set(
            PredefinedAtom::Next,
            Function::new(
                ctx,
                MutFn::new(move |ctx: Ctx<'js>| -> rquickjs::Result<Object<'js>> {
                    let next = nodes.next();
                    let res = Object::new(ctx.clone())?;
                    res.set(PredefinedAtom::Done, next.is_none())?;

                    if let Some(next) = next {
                        res.set(
                            PredefinedAtom::Value,
                            Class::instance(
                                ctx.clone(),
                                JsElement {
                                    id: next,
                                    dom: dom.clone(),
                                },
                            )?,
                        )?;
                    }

                    Ok(res)
                }),
            ),
        )?;
        Ok(res)
    }
}

#[rquickjs::class]
pub struct NodeList {
    nodes: Vec<NodeId>,
    dom: Locket<domjohnson::Document>,
}

impl<'js> Trace<'js> for NodeList {
    fn trace<'a>(&self, _tracer: rquickjs::class::Tracer<'a, 'js>) {}
}

impl NodeList {
    pub fn new(dom: Locket<domjohnson::Document>, nodes: Vec<NodeId>) -> NodeList {
        NodeList { nodes, dom }
    }
}

#[rquickjs::methods]
impl NodeList {
    pub fn item(&self, idx: usize) -> Option<JsElement> {
        self.nodes.get(idx).and_then(|id| {
            Some(JsElement {
                id: *id,
                dom: self.dom.clone(),
            })
        })
    }

    pub fn entries<'js>(&self, ctx: Ctx<'js>) -> rquickjs::Result<Value<'js>> {
        // FIXME: Done clone here
        let dom = self.dom.clone();
        klaver_shared::iter::Iter::new(self.nodes.clone().into_iter().enumerate().map(
            move |(index, id)| Entry {
                index,
                element: JsElement {
                    id,
                    dom: dom.clone(),
                },
            },
        ))
        .into_js(&ctx)
    }

    pub fn values<'js>(&self, ctx: Ctx<'js>) -> rquickjs::Result<Value<'js>> {
        // FIXME: Done clone here
        let dom = self.dom.clone();
        klaver_shared::iter::Iter::new(self.nodes.clone().into_iter().map(move |id| JsElement {
            id,
            dom: dom.clone(),
        }))
        .into_js(&ctx)
    }
}

struct Entry {
    index: usize,
    element: JsElement,
}

impl<'js> IntoJs<'js> for Entry {
    fn into_js(self, ctx: &Ctx<'js>) -> rquickjs::Result<Value<'js>> {
        let array = Array::new(ctx.clone())?;

        array.set(0, self.index)?;
        array.set(1, self.element)?;

        Ok(array.into_value())
    }
}
