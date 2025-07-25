use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    usize,
};

use event_listener::{Event, listener};
use klaver_util::{CaugthException, throw_if};
use rquickjs::{Ctx, JsLifetime, runtime::UserDataGuard};

use crate::shutdown::Shutdown;

struct Inner {
    shutdown: Event,
    is_shutdown: Rc<Cell<bool>>,
    events: Event,
    error: RefCell<Option<CaugthException>>,
    worker_count: Cell<usize>,
}

#[derive(Clone)]
pub struct Workers(Rc<Inner>);

impl Workers {
    pub fn from_ctx<'a>(ctx: &'a Ctx<'_>) -> rquickjs::Result<UserDataGuard<'a, Self>> {
        match ctx.userdata::<Self>() {
            Some(ret) => Ok(ret),
            None => {
                let workers = Workers::new();
                throw_if!(ctx, ctx.store_userdata(workers));
                Ok(ctx.userdata().expect("userdata"))
            }
        }
    }

    pub fn new() -> Workers {
        Workers(Rc::new(Inner {
            shutdown: Event::new(),
            events: Event::new(),
            error: RefCell::new(None),
            worker_count: Cell::new(0),
            is_shutdown: Rc::new(Cell::new(false)),
        }))
    }

    pub fn error(&self) -> Option<CaugthException> {
        self.0.error.borrow().clone()
    }

    pub fn push<'js, T, U>(&self, ctx: Ctx<'js>, func: T)
    where
        T: FnOnce(Ctx<'js>, Shutdown) -> U + 'js,
        U: Future<Output = Result<(), CaugthException>> + 'js,
    {
        let inner = self.0.clone();
        self.0.worker_count.update(|m| m + 1);
        self.0.events.notify(usize::MAX);

        ctx.clone().spawn(async move {
            let ret = func(
                ctx,
                Shutdown::new(inner.shutdown.listen(), inner.is_shutdown.clone()),
            )
            .await;

            if let Err(err) = ret {
                inner.error.replace(Some(err));
            }
            inner.worker_count.update(|m| m - 1);
            inner.events.notify(usize::MAX);
        });
    }
}

impl Workers {
    pub(crate) fn shutdown(&self) {
        if self.0.is_shutdown.get() {
            return;
        }
        self.0.shutdown.notify(usize::MAX);
        self.0.events.notify(usize::MAX);
        self.0.is_shutdown.set(true);
    }

    pub(crate) fn reset(&self) {
        self.0.is_shutdown.set(false);
        self.0.error.replace(None);
        self.0.worker_count.set(0);
    }

    pub(crate) async fn wait(&self) -> Result<(), CaugthException> {
        loop {
            if let Some(error) = self.0.error.borrow_mut().take() {
                return Err(error);
            }

            if self.0.worker_count.get() == 0 && self.0.is_shutdown.get() {
                return Ok(());
            }

            listener!(self.0.events => events);

            events.await;

            if let Some(error) = self.0.error.borrow_mut().take() {
                return Err(error);
            }

            if self.0.worker_count.get() == 0 {
                return Ok(());
            }
        }
    }
}

unsafe impl<'js> JsLifetime<'js> for Workers {
    type Changed<'to> = Workers;
}
