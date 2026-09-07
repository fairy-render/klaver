use std::{collections::VecDeque, fmt::Debug};

use rquickjs::{Ctx, Function, Promise, Value, class::Trace};

use crate::streams::queue_strategy::QueuingStrategy;

#[derive(Trace, Debug)]
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
}

impl<'js> Debug for Queue<'js> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Queue")
            .field("chunks", &self.chunks)
            .field("current_size", &self.current_size)
            .finish()
    }
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

    /// Drains the queue, rejecting every pending write's promise with `reason` - per spec, an
    /// error/abort must reject *all* outstanding `writer.write()` promises, not just silently
    /// drop them.
    pub fn reject_all(&mut self, reason: Value<'js>) {
        self.current_size = 0;
        for entry in self.chunks.drain(..) {
            entry.reject.call::<_, ()>((reason.clone(),)).ok();
        }
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
        let entry = self.chunks.pop_front()?;
        if entry.size > self.current_size {
            self.current_size = 0;
        } else {
            self.current_size -= entry.size;
        }

        Some(entry)
    }

    /// The standard `desiredSize`: how much more (by strategy-defined size units) could be
    /// enqueued before the queue is considered full. Negative once over the high water mark.
    pub fn desired_size(&self) -> f64 {
        self.strategy.high_water_mark() as f64 - self.current_size as f64
    }
}
