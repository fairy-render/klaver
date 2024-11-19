use std::{collections::VecDeque, rc::Rc};

use rquickjs::{class::Trace, CaughtError, Class, Ctx, JsLifetime, Value};
use rquickjs_util::throw;
use tokio::sync::Notify;

use super::queue_strategy::QueuingStrategy;

pub enum State<'js> {
    Closed,
    Pulling,
    Canceled(Option<rquickjs::String<'js>>),
    Error(CaughtError<'js>),
}

impl<'js> State<'js> {
    fn is_pulling(&self) -> bool {
        matches!(self, Self::Pulling)
    }

    fn is_closed(&self) -> bool {
        matches!(self, Self::Closed)
    }

    fn is_canceled(&self) -> bool {
        matches!(self, Self::Canceled(_))
    }

    fn as_error(&self) -> Option<&CaughtError<'js>> {
        match self {
            Self::Error(err) => Some(err),
            _ => None,
        }
    }
}

impl<'js> Trace<'js> for State<'js> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        match self {
            Self::Canceled(reason) => reason.trace(tracer),
            Self::Error(err) => match err {
                CaughtError::Exception(err) => err.trace(tracer),
                CaughtError::Value(v) => v.trace(tracer),
                _ => {}
            },
            _ => {}
        }
    }
}

#[rquickjs::class]
pub struct ReadableStreamDefaultController<'js> {
    queue: VecDeque<Value<'js>>,
    queuing_strategy: QueuingStrategy<'js>,
    locked: bool,
    // Signal puller that something happended
    ready: Rc<Notify>,
    // Signal readers that something happended
    pub(super) wait: Rc<Notify>,
    pub state: State<'js>,
}

unsafe impl<'js> JsLifetime<'js> for ReadableStreamDefaultController<'js> {
    type Changed<'to> = ReadableStreamDefaultController<'to>;
}

impl<'js> ReadableStreamDefaultController<'js> {
    pub fn new(
        ready: Rc<Notify>,
        queuing_strategy: QueuingStrategy<'js>,
    ) -> ReadableStreamDefaultController<'js> {
        let notify = Rc::new(Notify::new());

        ReadableStreamDefaultController {
            queue: Default::default(),
            ready,
            wait: notify,
            queuing_strategy,
            locked: false,
            state: State::Pulling,
        }
    }
}

impl<'js> Trace<'js> for ReadableStreamDefaultController<'js> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        self.queue.trace(tracer);
        self.state.trace(tracer);
        self.queuing_strategy.trace(tracer);
    }
}

#[rquickjs::methods]
impl<'js> ReadableStreamDefaultController<'js> {
    pub fn enqueue(&mut self, chunk: Value<'js>) -> rquickjs::Result<()> {
        self.queue.push_back(chunk);
        self.wait.notify_waiters();

        Ok(())
    }

    pub fn close(&mut self) -> rquickjs::Result<()> {
        if !self.state.is_pulling() {
            return Ok(());
        }
        self.state = State::Closed;
        self.wait.notify_waiters();
        self.ready.notify_one();

        Ok(())
    }
}

impl<'js> ReadableStreamDefaultController<'js> {
    pub fn lock(&mut self, ctx: Ctx<'js>) -> rquickjs::Result<()> {
        if self.locked {
            throw!(ctx, "Readable stream already locked")
        }

        self.locked = true;

        Ok(())
    }

    fn release(&mut self) -> rquickjs::Result<()> {
        self.locked = false;

        Ok(())
    }

    pub fn set_error(&mut self, error: CaughtError<'js>) {
        self.state = State::Error(error);
        self.wait.notify_waiters();
    }

    // State
    #[inline]
    pub fn is_filled(&self) -> bool {
        self.queue.len() >= self.queuing_strategy.high_water_mark() as usize
    }

    pub fn is_done(&self) -> bool {
        self.queue.is_empty() && !self.state.is_pulling()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    #[inline]
    pub fn is_pulling(&self) -> bool {
        self.state.is_pulling()
    }

    #[inline]
    pub fn is_canceled(&self) -> bool {
        self.state.is_canceled()
    }

    #[inline]
    pub fn is_closed(&self) -> bool {
        self.state.is_closed()
    }

    #[inline]
    pub fn is_locked(&self) -> bool {
        self.locked
    }

    #[inline]
    pub fn has_error(&self) -> Option<&CaughtError<'js>> {
        self.state.as_error()
    }

    #[inline]
    pub fn cancel_reason(&self) -> Option<&rquickjs::String<'js>> {
        match &self.state {
            State::Canceled(err) => err.as_ref(),
            _ => None,
        }
    }

    pub fn cancel(
        &mut self,
        ctx: &Ctx<'js>,
        reason: Option<rquickjs::String<'js>>,
    ) -> rquickjs::Result<()> {
        if !self.state.is_pulling() {
            throw!(ctx, "Could not cancel. Stream is not running");
        }

        self.state = State::Canceled(reason);
        self.queue.clear();
        // Notify the runner
        self.ready.notify_one();
        // // Notify any readers
        self.wait.notify_waiters();

        Ok(())
    }

    #[inline]
    pub fn pop(&mut self) -> Option<Value<'js>> {
        let ret = self.queue.pop_front();
        if !self.is_filled() {
            self.ready.notify_one();
        }
        ret
    }
}

#[derive(Trace)]
pub struct ControllerWrap<'js> {
    pub(super) ctrl: Option<Class<'js, ReadableStreamDefaultController<'js>>>,
}

impl<'js> ControllerWrap<'js> {
    pub fn new(ctrl: Class<'js, ReadableStreamDefaultController<'js>>) -> ControllerWrap<'js> {
        ControllerWrap { ctrl: Some(ctrl) }
    }
}

impl<'js> ControllerWrap<'js> {
    pub fn release(&mut self) -> rquickjs::Result<()> {
        if let Some(ctrl) = self.ctrl.take() {
            ctrl.borrow_mut().release()?;
        }
        Ok(())
    }

    pub async fn wait(&self, ctx: Ctx<'js>) -> rquickjs::Result<()> {
        let notify = self.borrow(&ctx)?.wait.clone();

        notify.notified().await;

        Ok(())
    }

    pub fn borrow<'a>(
        &'a self,
        ctx: &Ctx<'js>,
    ) -> rquickjs::Result<rquickjs::class::Borrow<'a, 'js, ReadableStreamDefaultController<'js>>>
    {
        if let Some(ctrl) = self.ctrl.as_ref() {
            Ok(ctrl.borrow())
        } else {
            throw!(ctx, "Lock released")
        }
    }

    pub fn borrow_mut<'a>(
        &'a self,
        ctx: &Ctx<'js>,
    ) -> rquickjs::Result<rquickjs::class::BorrowMut<'a, 'js, ReadableStreamDefaultController<'js>>>
    {
        if let Some(ctrl) = self.ctrl.as_ref() {
            Ok(ctrl.borrow_mut())
        } else {
            throw!(ctx, "Lock released")
        }
    }
}
