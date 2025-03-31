use hyper::{Request, Response};
use router::{BoxHandler, Handler as _};
use routing::{
    Params,
    router::{MethodFilter, Router as RouteTree},
};
use rquickjs::{Class, Ctx, FromJs, Function, JsLifetime, Value, class::Trace};
use rquickjs_util::{StringRef, throw_if};

pub struct JsRouteContext {}

#[derive(Clone)]
pub enum Handler<'js> {
    Script(Function<'js>),
    Handler(BoxHandler<reggie::Body, JsRouteContext>),
}

impl<'js> Trace<'js> for Handler<'js> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        match self {
            Self::Script(script) => script.trace(tracer),
            _ => {}
        }
    }
}

impl<'js> Handler<'js> {
    pub async fn call(
        &self,
        ctx: Ctx<'js>,
        req: Request<reggie::Body>,
        context: JsRouteContext,
    ) -> rquickjs::Result<Response<reggie::Body>> {
        match self {
            Self::Script(script) => {
                let req = klaver_wintercg::http::Request::from_request(&ctx, req)?;
                let mut ret = script.call::<_, Value>((req,))?;
                if let Some(promise) = ret.as_promise() {
                    ret = promise.clone().into_future::<Value>().await?;
                }

                let ret = Class::<klaver_wintercg::http::Response>::from_js(&ctx, ret)?;

                ret.borrow_mut().to_reggie(ctx).await
            }
            Self::Handler(handler) => {
                let ret = throw_if!(ctx, handler.call(&context, req).await);
                Ok(ret)
            }
        }
    }
}

struct RouteEntry<'js> {
    method: MethodFilter,
    handler: Handler<'js>,
}

impl<'js> Trace<'js> for RouteEntry<'js> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        self.handler.trace(tracer);
    }
}

struct RouteHandler<'js> {
    entries: Vec<RouteEntry<'js>>,
}

impl<'js> Trace<'js> for RouteHandler<'js> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        self.entries.trace(tracer);
    }
}

#[rquickjs::class]
pub struct Router<'js> {
    tree: RouteTree<Handler<'js>>,
}

unsafe impl<'js> JsLifetime<'js> for Router<'js> {
    type Changed<'to> = Router<'to>;
}

impl<'js> Trace<'js> for Router<'js> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        for (_, i) in self.tree.routes() {
            for entry in &i.entries {
                entry.handler.trace(tracer);
            }
        }
    }
}

impl<'js> Router<'js> {
    fn route(
        &mut self,
        ctx: Ctx<'js>,
        method: MethodFilter,
        path: StringRef<'js>,
        handler: Function<'js>,
    ) -> rquickjs::Result<()> {
        throw_if!(
            ctx,
            self.tree
                .route(method, path.as_str(), Handler::Script(handler))
        );

        Ok(())
    }

    pub fn match_route<P: Params>(
        &self,
        path: &str,
        method: MethodFilter,
        params: &mut P,
    ) -> Option<&Handler<'js>> {
        self.tree.match_route(path, method, params)
    }
}

#[rquickjs::methods]
impl<'js> Router<'js> {
    #[qjs(constructor)]
    pub fn new() -> Router<'js> {
        Router {
            tree: RouteTree::new(),
        }
    }

    pub fn get(
        &mut self,
        ctx: Ctx<'js>,
        path: StringRef<'js>,
        handler: Function<'js>,
    ) -> rquickjs::Result<()> {
        self.route(ctx, MethodFilter::GET, path, handler)
    }

    pub fn post(
        &mut self,
        ctx: Ctx<'js>,
        path: StringRef<'js>,
        handler: Function<'js>,
    ) -> rquickjs::Result<()> {
        self.route(ctx, MethodFilter::POST, path, handler)
    }

    pub fn patch(
        &mut self,
        ctx: Ctx<'js>,
        path: StringRef<'js>,
        handler: Function<'js>,
    ) -> rquickjs::Result<()> {
        self.route(ctx, MethodFilter::PATCH, path, handler)
    }

    pub fn put(
        &mut self,
        ctx: Ctx<'js>,
        path: StringRef<'js>,
        handler: Function<'js>,
    ) -> rquickjs::Result<()> {
        self.route(ctx, MethodFilter::PUT, path, handler)
    }

    pub fn delete(
        &mut self,
        ctx: Ctx<'js>,
        path: StringRef<'js>,
        handler: Function<'js>,
    ) -> rquickjs::Result<()> {
        self.route(ctx, MethodFilter::DELETE, path, handler)
    }
}
