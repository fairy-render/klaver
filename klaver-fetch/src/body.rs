use std::cell::RefCell;

use futures::TryStreamExt;
use klaver_base::streams::ReadableStream;
use reggie::Body;
use rquickjs::{Class, Ctx, Value, class::Trace};
use rquickjs_util::{Bytes, Static, throw, throw_if};

pub enum BodyState<'js> {
    Empty,
    HttpBody(Option<Body>),
    ReadableStream(Class<'js, ReadableStream<'js>>),
}

impl<'js> Trace<'js> for BodyState<'js> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        match self {
            BodyState::ReadableStream(stream) => stream.trace(tracer),
            _ => {}
        }
    }
}

pub struct BodyMixin<'js> {
    state: RefCell<BodyState<'js>>,
}

impl<'js> BodyMixin<'js> {
    pub fn body_read(&self) -> bool {
        match &*self.state.borrow() {
            BodyState::Empty => true,
            BodyState::HttpBody(state) => state.is_none(),
            BodyState::ReadableStream(stream) => stream.borrow().disturbed(),
        }
    }

    pub fn body(
        &self,
        ctx: &Ctx<'js>,
    ) -> rquickjs::Result<Option<Class<'js, ReadableStream<'js>>>> {
        match &mut *self.state.borrow_mut() {
            BodyState::Empty => Ok(None),
            BodyState::HttpBody(body) => {
                let Some(body) = body.take() else {
                    throw!(ctx, "Body is None")
                };

                let stream = ReadableStream::from_stream(
                    ctx.clone(),
                    Static(reggie::body::to_stream(body).map_ok(|m| Bytes(m.to_vec()))),
                )?;

                let stream = Class::instance(ctx.clone(), stream)?;

                self.state
                    .replace(BodyState::ReadableStream(stream.clone()));

                Ok(Some(stream))
            }
            BodyState::ReadableStream(stream) => Ok(Some(stream.clone())),
        }
    }

    pub async fn to_bytes(&self, ctx: &Ctx<'js>) -> rquickjs::Result<Vec<u8>> {
        let Some(body) = self.body(ctx)? else {
            throw!(ctx, "Body already ready")
        };

        body.borrow().to_bytes(ctx.clone()).await
    }

    pub async fn to_text(&self, ctx: &Ctx<'js>) -> rquickjs::Result<String> {
        let bytes = self.to_bytes(ctx).await?;
        let string = throw_if!(ctx, std::string::String::from_utf8(bytes));
        Ok(string)
    }
}

impl<'js> Trace<'js> for BodyMixin<'js> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        self.state.borrow().trace(tracer);
    }
}
