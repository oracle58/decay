use std::marker::PhantomData;

use super::store::Store;
use super::context::Context;

/// Double-buffered event storage for a specific event type.
pub struct Events<T: 'static> {
    buffers: [Vec<T>; 2],
    current: usize,
}

impl<T: 'static> Store for Events<T> {}

impl<T: 'static> Events<T> {
    pub fn new() -> Self {
        Self { buffers: [Vec::new(), Vec::new()], current: 0 }
    }

    pub fn send(&mut self, event: T) {
        self.buffers[self.current].push(event);
    }

    pub fn read(&self) -> &[T] {
        let prev = 1 - self.current;
        &self.buffers[prev]
    }

    pub fn swap(&mut self) {
        self.current = 1 - self.current;
        self.buffers[self.current].clear();
    }
}

// EventSwap registry

pub(crate) trait EventSwapper {
    fn swap(&self, ctx: &mut Context);
}

struct TypedSwapper<T: 'static>(PhantomData<T>);

impl<T: 'static> EventSwapper for TypedSwapper<T> {
    fn swap(&self, ctx: &mut Context) {
        if let Some(events) = ctx.store_mut::<Events<T>>() {
            events.swap();
        }
    }
}

/// Registry of all event types, used to swap buffers at frame start.
pub(crate) struct EventRegistry {
    swappers: Vec<Box<dyn EventSwapper>>,
}

impl Store for EventRegistry {}

impl EventRegistry {
    pub fn new() -> Self {
        Self { swappers: Vec::new() }
    }

    pub fn register<T: 'static>(&mut self) {
        self.swappers.push(Box::new(TypedSwapper::<T>(PhantomData)));
    }

    pub fn swap_all(&self, ctx: &mut Context) {
        for swapper in &self.swappers {
            swapper.swap(ctx);
        }
    }
}
