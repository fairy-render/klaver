use rquickjs::{class::Trace, Class, Ctx, Exception, Value};

use super::{facotry::get_http_client, Request, Response};

#[rquickjs::class]
pub struct Client {
    pub(crate) inner: reggie::Client,
}

impl<'js> Trace<'js> for Client {
    fn trace<'a>(&self, _tracer: rquickjs::class::Tracer<'a, 'js>) {}
}

impl Client {
    pub fn reggie_client(&self) -> &reggie::Client {
        &self.inner
    }
}

#[rquickjs::methods]
impl Client {
    // #[qjs(constructor)]
    // pub fn new(ctx: Ctx<'_>) -> rquickjs::Result<Client> {
    //     Ok(Client {
    //         inner: get_http_client(&ctx)?,
    //     })
    // }

    pub async fn send<'js>(
        &self,
        ctx: Ctx<'js>,
        request: Class<'js, Request<'js>>,
    ) -> rquickjs::Result<Class<'js, Response<'js>>> {
        let mut req = request.borrow_mut();
        let (req, cancel) = req.into_request(ctx.clone()).await?;

        let url = req.uri().clone();

        let run = || async {
            let resp = match self.inner.request(req).await {
                Ok(ret) => ret,
                Err(err) => {
                    return Err(ctx.throw(Value::from_exception(Exception::from_message(
                        ctx.clone(),
                        &err.to_string(),
                    )?)));
                }
            };

            Ok(resp)
        };

        let resp = if let Some(mut cancel) = cancel {
            tokio::select! {
                _ = &mut cancel => {
                     return Err(ctx.throw(Value::from_exception(Exception::from_message(
                        ctx.clone(),
                        "CANCEL",
                    )?)))
                }
                resp = run() => {
                     resp?
                }
            }
        } else {
            run().await?
        };

        Class::instance(
            ctx.clone(),
            Response::from_response(ctx.clone(), &url.to_string(), resp)?,
        )
    }
}
