use decay::core::*;

#[test]
fn app_builds_and_runs() {
    App::new().hook(Lifecycle::Setup, |_ctx: &mut Context| {}).run();
}

#[test]
fn store_roundtrip() {
    let mut ctx = Context::new();
    ctx.insert_store(42u32);
    assert_eq!(*ctx.store::<u32>().unwrap(), 42);
}

#[test]
fn store_mut() {
    let mut ctx = Context::new();
    ctx.insert_store(0u32);
    *ctx.store_mut::<u32>().unwrap() = 7;
    assert_eq!(*ctx.store::<u32>().unwrap(), 7);
}

#[test]
fn add_node() {
    let mut ctx = Context::new();
    let id = ctx.add(Node::text("hello"));
    assert_eq!(ctx.node(id).unwrap().text_value(), Some("hello"));
}

#[test]
fn add_child_node() {
    let mut ctx = Context::new();
    let root = ctx.add(Node::container());
    let child = ctx.add_child(root, Node::text("child"));
    assert_eq!(ctx.children(root), &[child]);
    assert_eq!(ctx.node(child).unwrap().parent, Some(root));
}

#[test]
fn remove_node() {
    let mut ctx = Context::new();
    let id = ctx.add(Node::text("gone"));
    ctx.remove(id);
    assert!(ctx.node(id).is_none());
}

#[test]
fn remove_recursive() {
    let mut ctx = Context::new();
    let root = ctx.add(Node::container());
    let child = ctx.add_child(root, Node::text("a"));
    let grandchild = ctx.add_child(child, Node::text("b"));
    ctx.remove_recursive(root);
    assert!(ctx.node(root).is_none());
    assert!(ctx.node(child).is_none());
    assert!(ctx.node(grandchild).is_none());
}

#[test]
fn node_id_recycling() {
    let mut ctx = Context::new();
    let id1 = ctx.add(Node::text("a"));
    ctx.remove(id1);
    let id2 = ctx.add(Node::text("b"));
    assert_eq!(id1, id2);
    assert_eq!(ctx.node(id2).unwrap().text_value(), Some("b"));
}

#[test]
fn node_mut_update() {
    let mut ctx = Context::new();
    let id = ctx.add(Node::text("old"));
    ctx.node_mut(id).unwrap().set_text("new");
    assert_eq!(ctx.node(id).unwrap().text_value(), Some("new"));
}

#[test]
fn find_node() {
    let mut ctx = Context::new();
    ctx.add(Node::text("a"));
    ctx.add(Node::button("click"));
    let found = ctx.find(|n| n.is_button());
    assert!(found.is_some());
    assert_eq!(found.unwrap().button_label(), Some("click"));
}

#[test]
fn progress_node() {
    let mut ctx = Context::new();
    let id = ctx.add(Node::progress(0.5));
    ctx.node_mut(id).unwrap().set_progress(0.75);
    if let Content::Progress { value, .. } = &ctx.node(id).unwrap().content {
        assert!((value - 0.75).abs() < 0.01);
    } else {
        panic!("expected progress content");
    }
}

#[test]
fn node_builders() {
    let n = Node::text("hello").bold().fg(255, 0, 0).center();
    assert!(n.style.bold);
    assert_eq!(n.style.fg, Some((255, 0, 0)));
    assert_eq!(n.align, TextAlign::Center);
}

#[test]
fn events_send_and_read() {
    let mut events = Events::<u32>::new();
    events.send(1);
    events.send(2);
    assert_eq!(events.read(), &[]);
    events.swap();
    assert_eq!(events.read(), &[1, 2]);
}

#[test]
fn cmd_deferred_operations() {
    let mut ctx = Context::new();
    let mut cmd = Cmd::new();
    cmd.create(Node::text("deferred"));
    assert_eq!(ctx.node_count(), 0);
    cmd.apply(&mut ctx);
    assert_eq!(ctx.node_count(), 1);
}

#[test]
fn multiple_children() {
    let mut ctx = Context::new();
    let root = ctx.add(Node::container());
    let a = ctx.add_child(root, Node::text("a"));
    let b = ctx.add_child(root, Node::text("b"));
    let c = ctx.add_child(root, Node::text("c"));
    assert_eq!(ctx.children(root), &[a, b, c]);
}

#[test]
fn root_auto_set() {
    let mut ctx = Context::new();
    let first = ctx.add(Node::container());
    assert_eq!(ctx.root(), Some(first));
}
