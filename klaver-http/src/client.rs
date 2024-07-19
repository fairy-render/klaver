use rquickjs::{class::Trace, Class, Ctx, Exception, Value};

use crate::{get_client, request::Request, response::Response};

#[rquickjs::class]
pub struct Client {
    inner: reggie::Client,
}

impl<'js> Trace<'js> for Client {
    fn trace<'a>(&self, _tracer: rquickjs::class::Tracer<'a, 'js>) {}
}

#[rquickjs::methods]
impl Client {
    #[qjs(constructor)]
    pub fn new(ctx: Ctx<'_>) -> rquickjs::Result<Client> {
        Ok(Client {
            inner: get_client(&ctx)?,
        })
    }

    // pub async fn get<'js>(
    //     &self,
    //     ctx: Ctx<'js>,
    //     url: String,
    // ) -> rquickjs::Result<Class<'js, Response<'js>>> {
    //     let ret = self.inner.get(url).send().await.unwrap();
    //     Class::instance(ctx.clone(), Response::from_reqwest(ctx.clone(), ret)?)
    // }

    pub async fn send<'js>(
        &self,
        ctx: Ctx<'js>,
        request: Class<'js, Request<'js>>,
    ) -> rquickjs::Result<Class<'js, Response<'js>>> {
        let req = request.borrow();
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
