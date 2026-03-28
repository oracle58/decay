//! Simple decay TUI application.
//!
//! Run with: cargo run --example simple

use decay::prelude::*;

fn main() {
    decay::term::spawn_window();

    App::new()
        .plugin(CorePlugins)
        .plugin(IntroPlugin)
        .plugin(UiPlugin)
        .setup(setup)
        .quit_on(KeyCode::Char('q'))
        .quit_on(KeyCode::Escape)
        .run();
}

fn setup(ctx: &mut Context) {
    let root = ctx.add(Node::container().anchor(Anchor::fill()));

    ctx.add_child(root, Node::text("Decay")
        .bold().fg(100, 180, 255).center()
        .anchor(Anchor { min: (0.0, 0.0), max: (1.0, 0.0), offset: (0, 2, 0, 3) }));

    ctx.add_child(root, Node::text("A TUI framework for Rust")
        .dim().center()
        .anchor(Anchor { min: (0.0, 0.0), max: (1.0, 0.0), offset: (0, 4, 0, 5) }));

    ctx.add_child(root, Node::separator()
        .anchor(Anchor { min: (0.1, 0.0), max: (0.9, 0.0), offset: (0, 6, 0, 7) }));

    ctx.add_child(root, Node::text("Direct platform I/O. No external dependencies.").center()
        .anchor(Anchor { min: (0.0, 0.0), max: (1.0, 0.0), offset: (0, 8, 0, 9) }));

    ctx.add_child(root, Node::text("Component architecture. Double-buffered rendering.").center()
        .anchor(Anchor { min: (0.0, 0.0), max: (1.0, 0.0), offset: (0, 9, 0, 10) }));

    ctx.add_child(root, Node::text("Modular. Use what you need.")
        .italic().fg(80, 200, 120).center()
        .anchor(Anchor { min: (0.0, 0.0), max: (1.0, 0.0), offset: (0, 11, 0, 12) }));

    ctx.add_child(root, Node::text("Press q to quit").dim().center()
        .anchor(Anchor { min: (0.0, 1.0), max: (1.0, 1.0), offset: (0, -2, 0, -1) }));
}
