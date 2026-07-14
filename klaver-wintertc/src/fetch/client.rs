use std::{cell::RefCell, rc::Rc};

use futures::future::LocalBoxFuture;
use klaver_runtime::{AsyncState, Resource, ResourceId};

use http::{Request, Response, Uri};
use klaver_core::{throw, throw_if};
use rquickjs::{Ctx, JsLifetime};

use crate::settings::WinterTcInstance;

use super::{
    RemoteBodyProducer,
    body::{JsBody, RemoteBody},
    body_static::Body,
};

pub trait LocalClient {
    fn send<'js, 'a>(
        &'a self,
        ctx: &'a Ctx<'js>,
        req: Request<JsBody<'js>>,
    ) -> LocalBoxFuture<'a, rquickjs::Result<Response<Body>>>;
}

pub trait SharedClient {
    fn send<'a>(
        &'a self,
        ctx: &'a Ctx<'_>,
        req: Request<RemoteBody>,
    ) -> LocalBoxFuture<'a, rquickjs::Result<Response<Body>>>;
}

pub struct LocalSharedClient<T>(T);

impl<T> LocalClient for LocalSharedClient<T>
where
    T: SharedClient,
{
    fn send<'js, 'a>(
        &'a self,
        ctx: &'a Ctx<'js>,
        req: Request<JsBody<'js>>,
    ) -> LocalBoxFuture<'a, rquickjs::Result<Response<Body>>> {
        Box::pin(async move {
            //

            let (parts, body) = req.into_parts();

            let (body, producer) = body.into_remote();

            AsyncState::push(ctx, producer)?;

            self.0.send(ctx, Request::from_parts(parts, body)).await
        })
    }
}

struct ClientState {
    local: Option<Box<dyn LocalClient>>,
    shared: Option<Box<dyn SharedClient>>,
    base_url: Uri,
}

#[derive(JsLifetime, Clone)]
pub struct Client {
    state: Rc<RefCell<ClientState>>,
}

impl Client {
    pub fn new() -> Self {
        Client {
            state: Rc::new(RefCell::new(ClientState {
                local: None,
                shared: None,
                base_url: Uri::from_static("http://localhost:3000/"),
            })),
        }
    }

    pub fn from_ctx<'a>(ctx: &'a Ctx<'_>) -> rquickjs::Result<Client> {
        let winter = WinterTcInstance::from_ctx(ctx)?;
        let client = winter.borrow().settings().get_http_client().clone();

        Ok(client)
    }

    pub fn base_url(&self) -> Uri {
        self.state.borrow().base_url.clone()
    }

    pub fn set_local_client<T>(&self, client: T)
    where
        T: LocalClient + 'static,
    {
        self.state.borrow_mut().local = Some(Box::new(client));
    }

    pub fn set_shared_client<T>(&self, client: T)
    where
        T: SharedClient + 'static,
    {
        self.state.borrow_mut().shared = Some(Box::new(client));
    }

    pub async fn send<'js>(
        &self,
        ctx: &Ctx<'js>,
        req: Request<JsBody<'js>>,
    ) -> rquickjs::Result<Response<Body>> {
        let this = self.state.borrow();

        if let Some(local) = &this.local {
            local.send(&ctx, req).await
        } else if let Some(shared) = &this.shared {
            let (parts, body) = req.into_parts();

            let (body, producer) = body.into_remote();
            let req = Request::from_parts(parts, body);

            AsyncState::push(&ctx, ClientResource { body: producer })?;

            shared.send(&ctx, req).await
        } else {
            throw!(ctx, "Could not find any Http client")
        }
    }
}

#[cfg(feature = "reqwest")]
impl SharedClient for reqwest::Client {
    fn send<'a>(
        &'a self,
        ctx: &'a Ctx<'_>,
        req: Request<RemoteBody>,
    ) -> LocalBoxFuture<'a, rquickjs::Result<Response<Body>>> {
        Box::pin(async {
            let (parts, body) = req.into_parts();

            let output = reqwest::Body::wrap(body);

            let ret = self
                .request(parts.method, parts.uri.to_string())
                .headers(parts.headers)
                .body(output)
                .send()
                .await;

            let resp: Response<_> = throw_if!(ctx, ret).into();

            let resp = resp.map(|b| Body::from_streaming(b));

            Ok(resp)
        })
    }
}

#[cfg(feature = "compio")]
impl LocalClient for cyper::Client {
    fn send<'js, 'a>(
        &'a self,
        ctx: &'a Ctx<'js>,
        req: Request<JsBody<'js>>,
    ) -> LocalBoxFuture<'a, rquickjs::Result<Response<Body>>> {
        use futures::TryStreamExt;
        Box::pin(async {
            use crate::fetch::body_static::to_bytes;

            let (parts, body) = req.into_parts();

            let bytes = throw_if!(ctx, to_bytes(body).await);

            let ret = self
                .request(parts.method, parts.uri.to_string())
                .unwrap()
                .headers(parts.headers)
                .body(bytes)
                .send()
                .await;

            let mut resp = throw_if!(ctx, ret);

            let headers = core::mem::take(resp.headers_mut());
            let status = resp.status();
            let version = resp.version();

            let mut resp = http::Response::new(Body::from_streaming(
                http_body_util::StreamBody::new(resp.map_ok(http_body::Frame::data)),
            ));

            *resp.headers_mut() = headers;
            *resp.status_mut() = status;
            *resp.version_mut() = version;

            Ok(resp)
        })
    }
}

struct ClientResourceId;

impl ResourceId for ClientResourceId {
    fn name() -> &'static str {
        "ClientRequest"
    }
}

struct ClientResource<'js> {
    body: RemoteBodyProducer<'js>,
}

impl<'js> Resource<'js> for ClientResource<'js> {
    type Id = ClientResourceId;

    const INTERNAL: bool = true;
    const SCOPED: bool = true;

    async fn run(self, ctx: klaver_runtime::Context<'js>) -> rquickjs::Result<()> {
        throw_if!(ctx.ctx(), self.body.await);
        Ok(())
    }
}
