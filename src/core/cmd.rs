use super::context::Context;
use super::node::{Node, NodeId};
use super::store::Store;

/// Deferred command buffer for tree mutations.
pub struct Cmd {
    queue: Vec<Box<dyn FnOnce(&mut Context)>>,
}

impl Store for Cmd {}

impl Cmd {
    pub fn new() -> Self {
        Self { queue: Vec::new() }
    }

    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    /// Deferred node creation.
    pub fn create(&mut self, node: Node) {
        self.queue.push(Box::new(move |ctx| { ctx.add(node); }));
    }

    /// Deferred child node creation.
    pub fn create_child(&mut self, parent: NodeId, node: Node) {
        self.queue.push(Box::new(move |ctx| { ctx.add_child(parent, node); }));
    }

    /// Deferred node removal.
    pub fn remove(&mut self, id: NodeId) {
        self.queue.push(Box::new(move |ctx| { ctx.remove(id); }));
    }

    /// Deferred recursive node removal.
    pub fn remove_recursive(&mut self, id: NodeId) {
        self.queue.push(Box::new(move |ctx| { ctx.remove_recursive(id); }));
    }

    /// Deferred store insertion.
    pub fn insert_store<T: Store>(&mut self, value: T) {
        self.queue.push(Box::new(move |ctx| { ctx.insert_store(value); }));
    }

    pub fn apply(&mut self, ctx: &mut Context) {
        for cmd in self.queue.drain(..) {
            cmd(ctx);
        }
    }
}
