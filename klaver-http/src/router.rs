use std::{pin::Pin, rc::Rc};

use hyper::{Request, Response};
use router::{BoxHandler, BoxMiddleware, Handler as _, Middleware as _};
use routing::{
    Params,
    router::{MethodFilter, Router as RouteTree},
};
use rquickjs::{
    CatchResultExt, Class, Ctx, FromJs, Function, JsLifetime, Value, class::Trace, prelude::Func,
};
use rquickjs_util::RuntimeError;
use rquickjs_util::{StringRef, throw_if};

#[derive(Debug, Clone)]
pub struct JsRouteContext {}

#[derive(Clone)]
pub enum Handler<'js> {
    Script(Function<'js>),
    Handler(BoxHandler<reggie::Body, JsRouteContext>),
    ScriptMiddleware(Function<'js>, Box<Handler<'js>>),
}

impl<'js> Trace<'js> for Handler<'js> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        match self {
            Self::Script(script) => script.trace(tracer),
            Self::ScriptMiddleware(func, _) => func.trace(tracer),
            _ => {}
        }
    }
}

impl<'js> Handler<'js> {
    pub async fn call(
        &self,
        req: Request<reggie::Body>,
        context: JsRouteContext,
    ) -> Result<Response<reggie::Body>, RuntimeError> {
        match self {
            Self::Script(script) => {
                let req = klaver_wintercg::http::Request::from_request(script.ctx(), req)
                    .catch(script.ctx())?;
                let mut ret = script.call::<_, Value>((req,)).catch(script.ctx())?;
                if let Some(promise) = ret.as_promise() {
                    ret = promise
                        .clone()
                        .into_future::<Value>()
                        .await
                        .catch(script.ctx())?;
                }

                let ret = Class::<klaver_wintercg::http::Response>::from_js(script.ctx(), ret)
                    .catch(script.ctx())?;

                Ok(ret
                    .borrow_mut()
                    .to_reggie(script.ctx().clone())
                    .await
                    .catch(script.ctx())?)
            }
            Self::Handler(handler) => {
                let ret = handler
                    .call(&context, req)
                    .await
                    .map_err(|err| RuntimeError::Custom(Box::new(err)))?;
                Ok(ret)
            }
            Self::ScriptMiddleware(func, handler) => {
                let req = klaver_wintercg::http::Request::from_request(func.ctx(), req)
                    .catch(func.ctx())?;

                let mut ret = func
                    .call::<_, Value>((
                        req,
                        NextFunc {
                            handler: *handler.clone(),
                        },
                    ))
                    .catch(func.ctx())?;
                if let Some(promise) = ret.as_promise() {
                    ret = promise
                        .clone()
                        .into_future::<Value>()
                        .await
                        .catch(func.ctx())?;
                }

                let ret = Class::<klaver_wintercg::http::Response>::from_js(func.ctx(), ret)
                    .catch(func.ctx())?;

                Ok(ret
                    .borrow_mut()
                    .to_reggie(func.ctx().clone())
                    .await
                    .catch(func.ctx())?)
            }
        }
    }

    pub async fn call_js(
        &self,
        ctx: Ctx<'js>,
        req: Class<'js, klaver_wintercg::http::Request<'js>>,
        context: JsRouteContext,
    ) -> rquickjs::Result<Class<'js, klaver_wintercg::http::Response<'js>>> {
        match self {
            Self::Script(script) => {
                let mut ret = script.call::<_, Value>((req,))?;
                if let Some(promise) = ret.as_promise() {
                    ret = promise.clone().into_future::<Value>().await?;
                }

                let ret = Class::<klaver_wintercg::http::Response>::from_js(script.ctx(), ret)?;

                Ok(ret)
            }
            Self::Handler(handler) => {
                let (req, _) = req.borrow_mut().into_request(ctx.clone()).await?;

                let ret = throw_if!(ctx, handler.call(&context, req).await);
                Class::instance(
                    ctx.clone(),
                    klaver_wintercg::http::Response::from_response(ctx, "", ret)?,
                )
            }
            Self::ScriptMiddleware(func, handler) => {
                let mut ret = func.call::<_, Value>((
                    req,
                    NextFunc {
                        handler: *handler.clone(),
                    },
                ))?;
                if let Some(promise) = ret.as_promise() {
                    ret = promise.clone().into_future::<Value>().await?;
                }

                let ret = Class::<klaver_wintercg::http::Response>::from_js(&ctx, ret)?;

                Ok(ret)
            }
        }
    }
}

impl<'js> router::Handler<reggie::Body, JsRouteContext> for Handler<'js> {
    type Response = Response<reggie::Body>;

    type Future<'a>
        = Pin<Box<dyn Future<Output = Result<Self::Response, router::Error>> + 'a>>
    where
        Self: 'a,
        JsRouteContext: 'a;

    fn call<'a>(
        &'a self,
        context: &'a JsRouteContext,
        req: Request<reggie::Body>,
    ) -> Self::Future<'a> {
        Box::pin(async move {
            self.call(req, context.clone())
                .await
                .map_err(router::Error::new)
        })
    }
}

//
#[derive(Clone)]
pub enum Middleware<'js> {
    Script(Function<'js>),
    Middleware(BoxMiddleware<reggie::Body, JsRouteContext, Handler<'js>>),
}

impl<'js> Trace<'js> for Middleware<'js> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        match self {
            Self::Script(script) => script.trace(tracer),
            _ => {}
        }
    }
}

impl<'js> Middleware<'js> {
    pub async fn call(
        &self,
        ctx: Ctx<'js>,
        req: Request<reggie::Body>,
        context: JsRouteContext,
        handler: Handler<'js>,
    ) -> rquickjs::Result<Response<reggie::Body>> {
        match self {
            Self::Script(script) => {
                let req = klaver_wintercg::http::Request::from_request(&ctx, req)?;

                let next = NextFunc { handler };
                let mut ret = script.call::<_, Value>((req, next))?;
                if let Some(promise) = ret.as_promise() {
                    ret = promise.clone().into_future::<Value>().await?;
                }

                let ret = Class::<klaver_wintercg::http::Response>::from_js(&ctx, ret)?;

                ret.borrow_mut().to_reggie(ctx).await
            }
            Self::Middleware(middleware) => {
                let wrapped = middleware.clone().wrap(handler);

                let ret = throw_if!(ctx, wrapped.call(&context, req).await);
                Ok(ret)
            }
        }
    }
}

impl<'js> router::Middleware<reggie::Body, JsRouteContext, Handler<'js>> for Middleware<'js> {
    type Handle = Handler<'js>;

    fn wrap(&self, handle: Handler<'js>) -> Self::Handle {
        match self {
            Middleware::Middleware(middleware) => Handler::Handler(middleware.wrap(handle)),
            Middleware::Script(script) => {
                Handler::ScriptMiddleware(script.clone(), Box::new(handle))
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
    middlewares: Vec<Middleware<'js>>,
}

unsafe impl<'js> JsLifetime<'js> for Router<'js> {
    type Changed<'to> = Router<'to>;
}

impl<'js> Trace<'js> for Router<'js> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        for (_, i) in self.tree.iter() {
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

    // pub fn from_router(router: router::Builder<JsRouteContext, reggie::Body>) -> Router<'js> {
    //     let (tree, middlewares) = router.into_parts();

    //     let tree = tree.map(Handler::Handler);
    //     let middlewares = middlewares
    //         .into_iter()
    //         .map(Middleware::Middleware)
    //         .collect();

    //     Router { tree, middlewares }
    // }

    pub fn match_route<P: Params>(
        &self,
        path: &str,
        method: MethodFilter,
        params: &mut P,
    ) -> Option<&Handler<'js>> {
        self.tree.match_route(path, method, params)
    }

    pub fn build(&self) -> ResolvedRouter<'js> {
        let tree = self
            .tree
            .clone()
            .map(|handler| compose(&self.middlewares, handler));
        ResolvedRouter { tree: tree.into() }
    }
}

#[rquickjs::methods]
impl<'js> Router<'js> {
    #[qjs(constructor)]
    fn new() -> Router<'js> {
        Router {
            tree: RouteTree::new(),
            middlewares: Vec::default(),
        }
    }

    fn get(
        &mut self,
        ctx: Ctx<'js>,
        path: StringRef<'js>,
        handler: Function<'js>,
    ) -> rquickjs::Result<()> {
        self.route(ctx, MethodFilter::GET, path, handler)
    }

    fn post(
        &mut self,
        ctx: Ctx<'js>,
        path: StringRef<'js>,
        handler: Function<'js>,
    ) -> rquickjs::Result<()> {
        self.route(ctx, MethodFilter::POST, path, handler)
    }

    fn patch(
        &mut self,
        ctx: Ctx<'js>,
        path: StringRef<'js>,
        handler: Function<'js>,
    ) -> rquickjs::Result<()> {
        self.route(ctx, MethodFilter::PATCH, path, handler)
    }

    fn put(
        &mut self,
        ctx: Ctx<'js>,
        path: StringRef<'js>,
        handler: Function<'js>,
    ) -> rquickjs::Result<()> {
        self.route(ctx, MethodFilter::PUT, path, handler)
    }

    fn delete(
        &mut self,
        ctx: Ctx<'js>,
        path: StringRef<'js>,
        handler: Function<'js>,
    ) -> rquickjs::Result<()> {
        self.route(ctx, MethodFilter::DELETE, path, handler)
    }

    fn any(
        &mut self,
        ctx: Ctx<'js>,
        path: StringRef<'js>,
        handler: Function<'js>,
    ) -> rquickjs::Result<()> {
        self.route(ctx, MethodFilter::all(), path, handler)
    }

    #[qjs(rename = "use")]
    fn middleware(&mut self, handler: Function<'js>) -> rquickjs::Result<()> {
        self.middlewares.push(Middleware::Script(handler));
        Ok(())
    }
}

#[derive(Trace)]
#[rquickjs::class]
pub struct NextFunc<'js> {
    handler: Handler<'js>,
}

unsafe impl<'js> JsLifetime<'js> for NextFunc<'js> {
    type Changed<'to> = NextFunc<'to>;
}

#[rquickjs::methods]
impl<'js> NextFunc<'js> {
    async fn call(
        &self,
        ctx: Ctx<'js>,
        req: Class<'js, klaver_wintercg::http::Request<'js>>,
    ) -> rquickjs::Result<Class<'js, klaver_wintercg::http::Response<'js>>> {
        self.handler.call_js(ctx, req, JsRouteContext {}).await
    }
}

#[derive(Clone)]
pub struct ResolvedRouter<'js> {
    tree: Rc<RouteTree<Handler<'js>>>,
}

impl<'js> Trace<'js> for ResolvedRouter<'js> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        for (_, i) in self.tree.iter() {
            for entry in &i.entries {
                entry.handler.trace(tracer);
            }
        }
    }
}

impl<'js> ResolvedRouter<'js> {
    pub fn match_route<P: Params>(
        &self,
        path: &str,
        method: MethodFilter,
        params: &mut P,
    ) -> Option<&Handler<'js>> {
        self.tree.match_route(path, method, params)
    }
}

pub fn compose<'js>(middlewares: &[Middleware<'js>], task: Handler<'js>) -> Handler<'js> {
    let mut iter = middlewares.iter();
    let Some(middleware) = iter.next() else {
        return task;
    };

    let mut handler = middleware.wrap(task);
    while let Some(middleware) = iter.next() {
        handler = middleware.wrap(handler);
    }

    handler
}
