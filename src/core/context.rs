use std::any::{Any, TypeId};
use std::collections::HashMap;

use super::node::{Node, NodeId};

/// Central application context holding the node tree and global stores.
pub struct Context {
    nodes: Vec<Option<Node>>,
    free_list: Vec<u32>,
    next_id: u32,
    root: Option<NodeId>,
    pub(crate) stores: HashMap<TypeId, Box<dyn Any>>,
    tick: u32,
}

impl Context {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            free_list: Vec::new(),
            next_id: 0,
            root: None,
            stores: HashMap::new(),
            tick: 0,
        }
    }

    // Node operations

    /// Add a node to the tree. Returns its ID.
    /// The first node added automatically becomes the root.
    pub fn add(&mut self, mut node: Node) -> NodeId {
        let id = if let Some(recycled) = self.free_list.pop() {
            self.nodes[recycled as usize] = None; // clear slot
            recycled
        } else {
            let id = self.next_id;
            self.next_id += 1;
            self.nodes.push(None);
            id
        };
        node.id = id;
        self.nodes[id as usize] = Some(node);
        if self.root.is_none() {
            self.root = Some(id);
        }
        id
    }

    /// Add a node as a child of `parent`. Returns the child's ID.
    pub fn add_child(&mut self, parent: NodeId, node: Node) -> NodeId {
        let id = self.add(node);
        if let Some(child) = self.nodes[id as usize].as_mut() {
            child.parent = Some(parent);
        }
        if let Some(parent_node) = self.nodes[parent as usize].as_mut() {
            parent_node.children.push(id);
        }
        id
    }

    /// Remove a single node (does not remove children).
    pub fn remove(&mut self, id: NodeId) {
        if let Some(node) = self.nodes.get_mut(id as usize).and_then(|slot| slot.take()) {
            if let Some(parent_id) = node.parent {
                if let Some(parent) = self.nodes.get_mut(parent_id as usize).and_then(|s| s.as_mut()) {
                    parent.children.retain(|&c| c != id);
                }
            }
            if self.root == Some(id) {
                self.root = None;
            }
            self.free_list.push(id);
        }
    }

    /// Remove a node and all its descendants.
    pub fn remove_recursive(&mut self, id: NodeId) {
        let children: Vec<NodeId> = self.nodes.get(id as usize)
            .and_then(|s| s.as_ref())
            .map(|n| n.children.clone())
            .unwrap_or_default();

        for child_id in children {
            self.remove_recursive(child_id);
        }
        self.remove(id);
    }

    /// Get an immutable reference to a node.
    pub fn node(&self, id: NodeId) -> Option<&Node> {
        self.nodes.get(id as usize).and_then(|s| s.as_ref())
    }

    /// Get a mutable reference to a node.
    pub fn node_mut(&mut self, id: NodeId) -> Option<&mut Node> {
        self.nodes.get_mut(id as usize).and_then(|s| s.as_mut())
    }

    /// The root node ID, if any.
    pub fn root(&self) -> Option<NodeId> {
        self.root
    }

    /// Set the root node.
    pub fn set_root(&mut self, id: NodeId) {
        self.root = Some(id);
    }

    /// Direct children of the given node.
    pub fn children(&self, id: NodeId) -> &[NodeId] {
        self.nodes.get(id as usize)
            .and_then(|s| s.as_ref())
            .map(|n| n.children.as_slice())
            .unwrap_or(&[])
    }

    /// Iterate over all live nodes.
    pub fn nodes(&self) -> impl Iterator<Item = &Node> {
        self.nodes.iter().filter_map(|s| s.as_ref())
    }

    /// Iterate mutably over all live nodes.
    pub fn nodes_mut(&mut self) -> impl Iterator<Item = &mut Node> {
        self.nodes.iter_mut().filter_map(|s| s.as_mut())
    }

    /// Find the first node matching a predicate.
    pub fn find<F: Fn(&Node) -> bool>(&self, pred: F) -> Option<&Node> {
        self.nodes.iter().filter_map(|s| s.as_ref()).find(|n| pred(n))
    }

    /// Find the first node matching a predicate (mutable).
    pub fn find_mut<F: FnMut(&Node) -> bool>(&mut self, mut pred: F) -> Option<&mut Node> {
        self.nodes.iter_mut().filter_map(|s| s.as_mut()).find(|n| pred(n))
    }

    /// Total number of live nodes.
    pub fn node_count(&self) -> usize {
        self.nodes.iter().filter(|s| s.is_some()).count()
    }

    // Store operations (global singleton state)

    pub fn insert_store<T: 'static>(&mut self, value: T) {
        self.stores.insert(TypeId::of::<T>(), Box::new(value));
    }

    pub fn store<T: 'static>(&self) -> Option<&T> {
        self.stores.get(&TypeId::of::<T>())?.downcast_ref()
    }

    pub fn store_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.stores.get_mut(&TypeId::of::<T>())?.downcast_mut()
    }

    // Tick

    pub fn tick(&self) -> u32 {
        self.tick
    }

    pub fn increment_tick(&mut self) {
        self.tick = self.tick.wrapping_add(1);
    }
}
