use event_listener::Event;
use rquickjs::{Ctx, JsLifetime, String, Value, class::Trace, prelude::Opt};
use std::{cell::RefCell, collections::VecDeque, rc::Rc};

use crate::streams::{queue_strategy::QueuingStrategy, writable::state::StreamData};

// pub struct StreamReadyState {
//     pub locked: bool,
//     pub wait: Rc<Event>,
// }

// impl<'js> Trace<'js> for StreamReadyState {
//     fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {}
// }

// // #[derive(Trace)]
// // pub enum WritableStreamController<'js> {
// //     Default(WritableStreamDefaultController<'js>),
// // }

// #[derive(Trace)]
// enum ControllerState<'js> {
//     Aborted(Option<String<'js>>),
//     Failed(Value<'js>),
//     Closed,
//     Running,
// }

#[rquickjs::class]
#[derive(Trace)]
pub struct WritableStreamDefaultController<'js> {
    // pub queue: RefCell<VecDeque<Value<'js>>>,
    // pub wait: StreamReadyState,
    // pub queing_strategy: QueuingStrategy<'js>,
    // pub state: ControllerState<'js>,
    pub data: Rc<StreamData<'js>>,
}

// impl<'js> Trace<'js> for WritableStreamDefaultController<'js> {
//     fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
//         self.queing_strategy.trace(tracer);
//         self.queue.borrow().trace(tracer);
//         self.wait.trace(tracer);
//         self.state.trace(tracer);
//     }
// }

unsafe impl<'js> JsLifetime<'js> for WritableStreamDefaultController<'js> {
    type Changed<'to> = WritableStreamDefaultController<'to>;
}

#[rquickjs::methods]
impl<'js> WritableStreamDefaultController<'js> {
    fn error(&self, error: Value<'js>) -> rquickjs::Result<()> {
        Ok(())
    }
}

// impl<'js> WritableStreamDefaultController<'js> {
//     pub fn is_ready(&self) -> bool {
//         self.data.is_ready()
//     }

//     pub fn close(&self, ctx: Ctx<'js>) -> rquickjs::Result<()> {
//         self.data.close(ctx)
//     }

//     pub fn abort(&self, ctx: Ctx<'js>, reason: Option<String<'js>>) -> rquickjs::Result<()> {
//         self.data.abort(ctx, reason)
//     }

//     pub fn is_closed(&self) -> bool {
//         self.data.is_closed()
//     }

//     pub fn is_running(&self) -> bool {
//         self.data.is_running()
//     }

//     pub fn is_aborted(&self) -> bool {
//         self.data.is_aborted()
//     }

//     pub fn abort_reason(&self) -> Option<String<'js>> {
//         self.data.abort_reason()
//     }

//     pub fn unlock(&self) {
//         self.data.unlock();
//     }

//     pub fn lock(&self) {
//         self.data.lock();
//     }

//     pub fn is_locked(&self) -> bool {
//         self.data.is_locked()
//     }

//     pub async fn ready(&self) -> rquickjs::Result<()> {
//         self.data.ready().await
//     }

//     pub async fn write(&self, chunk: Value<'js>) -> rquickjs::Result<()> {
//         self.data.push(chunk).await
//     }
// }
