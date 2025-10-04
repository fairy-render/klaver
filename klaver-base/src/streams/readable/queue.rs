use std::collections::VecDeque;

use klaver_util::sync::{Listener, Notify};
use rquickjs::{Ctx, Value, class::Trace};

use crate::streams::queue_strategy::QueuingStrategy;

#[derive(Trace)]
struct Entry<'js> {
    value: Value<'js>,
    size: u64,
}

#[derive(Trace)]
pub struct Queue<'js> {
    strategy: QueuingStrategy<'js>,
    items: VecDeque<Entry<'js>>,
    size: u64,
    notify: Notify,
}

impl<'js> Queue<'js> {
    pub fn new(strategy: QueuingStrategy<'js>) -> Queue<'js> {
        Queue {
            items: Default::default(),
            strategy,
            size: 0,
            notify: Notify::default(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn is_full(&self) -> bool {
        let max = self.strategy.high_water_mark();
        self.size >= max
    }

    pub fn clear(&mut self) {
        self.size = 0;
        self.items.clear();
        self.notify.notify();
    }

    pub fn push(&mut self, ctx: &Ctx<'js>, chunk: Value<'js>) -> rquickjs::Result<()> {
        let size = self.strategy.size(ctx.clone(), &chunk)?;

        self.items.push_back(Entry { value: chunk, size });
        self.size += size;

        self.notify.notify();

        Ok(())
    }

    pub fn subscribe(&self) -> Listener {
        self.notify.listen()
    }

    pub fn pop(&mut self) -> Option<Value<'js>> {
        let entry = self.items.pop_back()?;
        if entry.size > self.size {
            self.size = 0;
        } else {
            self.size -= entry.size;
        }

        self.notify.notify();

        Some(entry.value)
    }
}
