use std::cell::RefCell;

use futures::future::LocalBoxFuture;
use klaver_task::{AsyncState, Resource, ResourceId};
use reggie::{Body, http_body_util::BodyExt};

use http::{Request, Response, Uri};
use klaver_util::{throw, throw_if};
use rquickjs::{Ctx, JsLifetime, runtime::UserDataGuard};

use crate::{
    RemoteBodyProducer,
    body::{JsBody, RemoteBody},
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
            todo!()
        })
    }
}

struct ClientState {
    local: Option<Box<dyn LocalClient>>,
    shared: Option<Box<dyn SharedClient>>,
    base_url: Uri,
}

#[derive(JsLifetime)]
pub struct Client {
    state: RefCell<ClientState>,
}

impl Client {
    pub fn from_ctx<'a>(ctx: &'a Ctx<'_>) -> rquickjs::Result<UserDataGuard<'a, Client>> {
        match ctx.userdata::<Client>() {
            Some(ret) => Ok(ret),
            None => {
                ctx.store_userdata(Client {
                    state: RefCell::new(ClientState {
                        local: None,
                        shared: None,
                        base_url: Uri::from_static("http://localhost:3000/"),
                    }),
                });

                Ok(ctx.userdata::<Client>().unwrap())
            }
        }
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

            // Workers::from_ctx(ctx)?.push(ctx.clone(), |ctx, mut shutdown| async move {
            //     if shutdown.is_killed() {
            //         return Ok(());
            //     }

            //     futures::select! {
            //       err = producer.fuse() => {
            //         return err.catch(&ctx).map_err(Into::into)
            //       }
            //       _ = shutdown => {}
            //     }

            //     Ok(())
            // });

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
            //

            let (parts, body) = req.into_parts();

            let output = reqwest::Body::wrap(body);

            let ret = self
                .request(parts.method, parts.uri.to_string())
                .headers(parts.headers)
                .body(output)
                .send()
                .await;

            let resp: Response<_> = throw_if!(ctx, ret).into();

            let resp = resp
                .map(|b| Body::from_streaming(b.map_err(|err| reggie::Error::Body(Box::new(err)))));

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

    async fn run(self, ctx: klaver_task::TaskCtx<'js>) -> rquickjs::Result<()> {
        self.body.await
    }
}
