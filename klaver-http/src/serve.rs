use reggie::RequestExt;
use rquickjs::{CatchResultExt, Class, Ctx, FromJs, Function, Value};
use std::collections::HashMap;
use std::net::SocketAddr;

use http_body_util::BodyExt;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;

use crate::router::Router;

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

#[rquickjs::function]
pub async fn serve_router<'js>(
    ctx: Ctx<'js>,
    router: Class<'js, Router<'js>>,
) -> rquickjs::Result<()> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    // We create a TcpListener and bind it to 127.0.0.1:3000
    let listener = TcpListener::bind(addr).await?;

    loop {
        let (stream, _) = listener.accept().await?;

        // Use an adapter to access something implementing `tokio::io` traits as if they implement
        // `hyper::rt` IO traits.
        let io = TokioIo::new(stream);

        let cloned_ctx = ctx.clone();
        let cloned_router = router.clone();
        // Spawn a tokio task to serve multiple connections concurrently
        ctx.spawn(async move {
            // Finally, we bind the incoming connection to our `hello` service

            if let Err(err) = http1::Builder::new()
                // `service_fn` converts our function in a `Service`
                .serve_connection(
                    io,
                    service_fn(move |req: Request<hyper::body::Incoming>| {
                        let ctx = cloned_ctx.clone();
                        let router = cloned_router.clone();
                        async move {
                            let mut params = HashMap::default();
                            let Some(route) = router
                                .borrow()
                                .match_route(
                                    req.uri().path(),
                                    req.method().clone().into(),
                                    &mut params,
                                )
                                .cloned()
                            else {
                                return Ok(Response::new(reggie::Body::empty()));
                            };

                            let resp = route
                                .call(
                                    ctx.clone(),
                                    req.map_body(|body| {
                                        reggie::Body::from_streaming(
                                            body.map_err(reggie::Error::conn),
                                        )
                                    }),
                                )
                                .await
                                .catch(&ctx)?;

                            Result::<_, rquickjs_util::RuntimeError>::Ok(resp)
                        }
                    }),
                )
                .await
            {
                // eprintln!("Error serving connection: {:?}", err);
            }
        });
    }

    Ok(())
}
