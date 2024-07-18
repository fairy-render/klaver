use std::{cell::RefCell, rc::Rc};

use futures::{stream::BoxStream, Stream, StreamExt};
use rquickjs::{
    atom::PredefinedAtom,
    class::{Mutability, Trace},
    function::Func,
    ArrayBuffer, Class, Ctx, Exception, Object, Symbol, Value,
};

pub fn async_iterator<'js, S>(ctx: Ctx<'js>, stream: S) -> rquickjs::Result<Object<'js>>
where
    S: Stream<Item = Result<Vec<u8>, AsyncByteIterError>> + Send + 'static,
{
    let obj = Object::new(ctx.clone())?;

    let stream = Rc::new(RefCell::new(Some(stream)));

    obj.set(
        Symbol::async_iterator(ctx.clone()),
        Func::from({
            let stream = stream.clone();
            move |ctx: Ctx<'js>| {
                let Some(stream) = stream.borrow_mut().take() else {
                    return Err(ctx.throw(Value::from_exception(Exception::from_message(
                        ctx.clone(),
                        "iterator is exhausted",
                    )?)));
                };

                Class::instance(
                    ctx,
                    AsyncByteIter {
                        inner: Box::pin(stream),
                    },
                )
            }
        }),
    )?;

    Ok(obj)
}

// pub struct AsyncByteIterable<T> {
//     inner: T,
// }

// static CLASS_NAME: ClassId = ClassId::new();

// impl<'js, T> JsClass<'js> for AsyncByteIterable<T> {
//     const NAME: &'static str = "AsyncByteIterable";

//     type Mutable = Writable;

//     fn class_id() -> &'static rquickjs::class::ClassId {
//         &CLASS_NAME
//     }

//     fn prototype(ctx: &Ctx<'js>) -> rquickjs::Result<Option<Object<'js>>> {
//         todo!()
//     }

//     fn constructor(
//         ctx: &Ctx<'js>,
//     ) -> rquickjs::Result<Option<rquickjs::function::Constructor<'js>>> {
//         Ok(None)
//     }
// }

// impl<'js, T> Trace<'js> for AsyncByteIterable<T> {
//     fn trace<'a>(&self, _tracer: rquickjs::class::Tracer<'a, 'js>) {}
// }

pub struct AsyncByteIterError;

#[rquickjs::class]
pub struct AsyncByteIter {
    inner: BoxStream<'static, Result<Vec<u8>, AsyncByteIterError>>,
}

impl<'js> Trace<'js> for AsyncByteIter {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {}
}

#[rquickjs::methods]
impl AsyncByteIter {
    #[qjs(rename = PredefinedAtom::Next)]
    pub async fn next<'js>(&mut self, ctx: Ctx<'js>) -> rquickjs::Result<Object<'js>> {
        let output = Object::new(ctx.clone())?;

        let Some(next) = self.inner.next().await else {
            output.set("done", true)?;
            return Ok(output);
        };

        let ret = match next {
            Ok(ret) => ret,
            Err(err) => {
                return Err(ctx.throw(Value::from_exception(Exception::from_message(
                    ctx.clone(),
                    "could not stuff",
                )?)))
            }
        };

        output.set("done", false)?;
        output.set("value", ArrayBuffer::new(ctx.clone(), ret))?;

        Ok(output)
    }
}
