use std::collections::VecDeque;

use rquickjs::{Ctx, Value, class::Trace};
use rquickjs_util::throw;

use crate::streams::queue_strategy::QueuingStrategy;

#[derive(Trace)]
struct Entry<'js> {
    chunk: Value<'js>,
    size: u64,
}

#[derive(Trace)]
pub struct Queue<'js> {
    chunks: VecDeque<Entry<'js>>,
    strategy: QueuingStrategy<'js>,
    current_size: u64,
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

    pub fn push(&mut self, ctx: Ctx<'js>, chunk: Value<'js>) -> rquickjs::Result<()> {
        if self.is_full() {
            throw!(ctx, "Queue is full")
        }
        let size = self.strategy.size(ctx, &chunk)?;

        self.chunks.push_back(Entry { chunk, size });
        self.current_size += size;

        Ok(())
    }

    pub fn pop(&mut self) -> Option<Value<'js>> {
        let entry = self.chunks.pop_back()?;
        if entry.size > self.current_size {
            self.current_size = 0;
        } else {
            self.current_size -= entry.size;
        }

        Some(entry.chunk)
    }
}
