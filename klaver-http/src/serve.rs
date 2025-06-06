use reggie::RequestExt;
use rquickjs::prelude::Opt;
use rquickjs::{CatchResultExt, Class, Ctx, FromJs, Function, Object, Value};
use std::collections::HashMap;
use std::net::SocketAddr;

use http_body_util::BodyExt;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;

use crate::router::{JsRouteContext, Router};

#[rquickjs::function]
pub async fn serve<'js>(ctx: Ctx<'js>, callback: Function<'js>) -> rquickjs::Result<()> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    // We create a TcpListener and bind it to 127.0.0.1:3000
    let listener = TcpListener::bind(addr).await?;

    loop {
        let (stream, _) = listener.accept().await?;

        // Use an adapter to access something implementing `tokio::io` traits as if they implement
        // `hyper::rt` IO traits.
        let io = TokioIo::new(stream);

        let callback = callback.clone();
        let cloned_ctx = ctx.clone();
        // Spawn a tokio task to serve multiple connections concurrently
        ctx.spawn(async move {
            // Finally, we bind the incoming connection to our `hello` service

            if let Err(err) = http1::Builder::new()
                // `service_fn` converts our function in a `Service`
                .serve_connection(
                    io,
                    service_fn(move |req: Request<hyper::body::Incoming>| {
                        let ctx = cloned_ctx.clone();
                        let callback = callback.clone();
                        async move {
                            let req = klaver_wintercg::http::Request::from_request(&ctx, req)
                                .catch(&ctx)?;

                            let mut value: Value = callback.call((req,)).catch(&ctx)?;
                            if let Some(promise) = value.as_promise() {
                                value = promise.clone().into_future::<Value>().await.catch(&ctx)?;
                            }

                            let resp =
                                Class::<klaver_wintercg::http::Response>::from_js(&ctx, value)?;

                            let resp =
                                resp.borrow_mut().to_reggie(ctx.clone()).await.catch(&ctx)?;

                            Result::<_, rquickjs_util::RuntimeError>::Ok(resp)
                        }
                    }),
                )
                .await
            {
                eprintln!("Error serving connection: {:?}", err);
            }
        });
    }

    Ok(())
}

#[derive(Debug, Clone)]
pub struct ServerOptions {
    port: u16,
    debug: bool,
}

impl Default for ServerOptions {
    fn default() -> Self {
        ServerOptions {
            port: 3000,
            debug: false,
        }
    }
}

impl<'js> FromJs<'js> for ServerOptions {
    fn from_js(ctx: &Ctx<'js>, value: Value<'js>) -> rquickjs::Result<Self> {
        let obj = Object::from_js(ctx, value)?;
        Ok(ServerOptions {
            port: obj.get("port").unwrap_or(3000),
            debug: obj.get("debug").unwrap_or(false),
        })
    }
}

#[rquickjs::function]
pub async fn serve_router<'js>(
    ctx: Ctx<'js>,
    router: Class<'js, Router<'js>>,
    options: Opt<ServerOptions>,
) -> rquickjs::Result<()> {
    let opts = options.0.unwrap_or_default();

    let addr = SocketAddr::from(([127, 0, 0, 1], opts.port));

    // We create a TcpListener and bind it to 127.0.0.1:3000
    let listener = TcpListener::bind(addr).await?;

    let router = router.borrow().build();

    loop {
        let (stream, _) = listener.accept().await?;

        // Use an adapter to access something implementing `tokio::io` traits as if they implement
        // `hyper::rt` IO traits.
        let io = TokioIo::new(stream);

        let cloned_router = router.clone();
        let opts = opts.clone();
        let cloned_ctx = ctx.clone();
        // Spawn a tokio task to serve multiple connections concurrently
        ctx.spawn(async move {
            // Finally, we bind the incoming connection to our `hello` service

            if let Err(err) = http1::Builder::new()
                // `service_fn` converts our function in a `Service`
                .serve_connection(
                    io,
                    service_fn(move |req: Request<hyper::body::Incoming>| {
                        let router = cloned_router.clone();
                        let ctx = cloned_ctx.clone();
                        async move {
                            let resp = router
                                .handle(
                                    ctx,
                                    req.map_body(|body| {
                                        reggie::Body::from_streaming(
                                            body.map_err(reggie::Error::conn),
                                        )
                                    }),
                                    JsRouteContext {},
                                )
                                .await;

                            match resp {
                                Ok(ret) => Result::<_, rquickjs_util::RuntimeError>::Ok(ret),
                                Err(err) => {
                                    let body = if opts.debug {
                                        reggie::Body::from(err.to_string())
                                    } else {
                                        reggie::Body::empty()
                                    };

                                    let mut resp = Response::new(body);
                                    *resp.status_mut() =
                                        reggie::http::StatusCode::INTERNAL_SERVER_ERROR;
                                    Ok(resp)
                                }
                            }
                        }
                    }),
                )
                .await
            {
                // eprintln!("Error serving connection: {:?}", err);
            }
        });
    }
}

pub struct ServeService<'js> {
    router: Class<'js, Router<'js>>,
    options: ServerOptions,
}
