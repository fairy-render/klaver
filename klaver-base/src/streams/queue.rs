use std::collections::VecDeque;

use event_listener::Event;
use rquickjs::{Ctx, Function, Promise, Value, class::Trace};
use rquickjs_util::throw;

use crate::streams::queue_strategy::QueuingStrategy;

#[derive(Trace)]
pub struct Entry<'js> {
    pub chunk: Value<'js>,
    pub resolve: Function<'js>,
    pub reject: Function<'js>,
    pub size: u64,
}

pub struct Queue<'js> {
    chunks: VecDeque<Entry<'js>>,
    strategy: QueuingStrategy<'js>,
    current_size: u64,
    ready: Event,
}

impl<'js> Trace<'js> for Queue<'js> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        self.chunks.trace(tracer);
        self.strategy.trace(tracer);
    }
}

impl<'js> Queue<'js> {
    pub fn new(strategy: QueuingStrategy<'js>) -> Queue<'js> {
        Queue {
            chunks: Default::default(),
            strategy,
            current_size: 0,
            ready: Event::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.chunks.is_empty()
    }

    pub fn is_full(&self) -> bool {
        let max = self.strategy.high_water_mark();
        self.current_size >= max
    }

    pub fn clear(&mut self) {
        self.current_size = 0;
        self.chunks.clear();
    }

    pub fn push(
        &mut self,
        ctx: Ctx<'js>,
        chunk: Value<'js>,
    ) -> rquickjs::Result<(Promise<'js>, Function<'js>, Function<'js>)> {
        let size = self.strategy.size(ctx.clone(), &chunk)?;

        let (promise, resolve, reject) = Promise::new(&ctx)?;

        self.chunks.push_back(Entry {
            chunk,
            size,
            reject: reject.clone(),
            resolve: resolve.clone(),
        });
        self.current_size += size;

        Ok((promise, resolve, reject))
    }

    pub fn pop(&mut self) -> Option<Entry<'js>> {
        let entry = self.chunks.pop_back()?;
        if entry.size > self.current_size {
            self.current_size = 0;
        } else {
            self.current_size -= entry.size;
        }

        Some(entry)
    }
}
