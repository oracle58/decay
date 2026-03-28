//! Dashboard example showcasing decay's TUI widgets.
//!
//! Run with: cargo run --example dashboard

use decay::prelude::*;

fn main() {
    decay::term::spawn_window();

    App::new()
        .plugin(CorePlugins)
        .plugin(IntroPlugin)
        .plugin(UiPlugin)
        .setup(setup)
        .update(animate)
        .update(handle_input)
        .run();
}

struct Bars { cpu: NodeId, mem: NodeId, disk: NodeId, net: NodeId }
impl Store for Bars {}

struct StatusLabel(NodeId);
impl Store for StatusLabel {}

struct FrameCount(u32);
impl Store for FrameCount {}

fn setup(ctx: &mut Context) {
    ctx.insert_store(FrameCount(0));
    let root = ctx.add(Node::container().anchor(Anchor::fill()));

    // Header panel
    let header = ctx.add_child(root, Node::panel("Decay Dashboard")
        .border(BorderStyle::Double).shadow().fg(100, 160, 255)
        .anchor(Anchor { min: (0.0, 0.0), max: (1.0, 0.0), offset: (2, 1, -2, 4) }));

    ctx.add_child(header, Node::text("TUI framework")
        .italic().fg(130, 130, 145).center()
        .anchor(Anchor { min: (0.0, 0.0), max: (1.0, 1.0), offset: (2, 1, -2, -1) }));

    // System monitor section
    ctx.add_child(root, Node::separator_labeled("System Monitor")
        .anchor(Anchor { min: (0.0, 0.0), max: (1.0, 0.0), offset: (2, 4, -2, 5) }));

    // Progress bar labels
    let labels = ["CPU", "Memory", "Disk", "Network"];
    for (i, lbl) in labels.iter().enumerate() {
        ctx.add_child(root, Node::text(lbl).dim()
            .anchor(Anchor { min: (0.0, 0.0), max: (0.0, 0.0), offset: (3, 5 + i as i16, 12, 6 + i as i16) }));
    }

    // Progress bars
    let cpu = ctx.add_child(root, Node::progress(0.64).gradient((255, 100, 80)).label()
        .anchor(Anchor { min: (0.15, 0.0), max: (0.55, 0.0), offset: (0, 5, 0, 6) }));
    let mem = ctx.add_child(root, Node::progress(0.78).gradient((100, 180, 255)).label().colors((60, 160, 240), (40, 40, 50))
        .anchor(Anchor { min: (0.15, 0.0), max: (0.55, 0.0), offset: (0, 6, 0, 7) }));
    let disk = ctx.add_child(root, Node::progress(0.21).label()
        .anchor(Anchor { min: (0.15, 0.0), max: (0.55, 0.0), offset: (0, 7, 0, 8) }));
    let net = ctx.add_child(root, Node::progress_classic(0.45).colors((200, 180, 60), (50, 50, 60)).label()
        .anchor(Anchor { min: (0.15, 0.0), max: (0.55, 0.0), offset: (0, 8, 0, 9) }));
    ctx.insert_store(Bars { cpu, mem, disk, net });

    // Info panel (right side)
    let info = ctx.add_child(root, Node::panel("Info").border(BorderStyle::Rounded).fg(80, 200, 120)
        .anchor(Anchor { min: (0.58, 0.0), max: (1.0, 0.0), offset: (0, 4, -2, 10) }));
    ctx.add_child(info, Node::text("Platform: native I/O")
        .anchor(Anchor { min: (0.0, 0.0), max: (1.0, 0.0), offset: (2, 1, -2, 2) }));
    ctx.add_child(info, Node::text("Deps: 0")
        .anchor(Anchor { min: (0.0, 0.0), max: (1.0, 0.0), offset: (2, 2, -2, 3) }));
    ctx.add_child(info, Node::text("Renderer: double-buffered")
        .anchor(Anchor { min: (0.0, 0.0), max: (1.0, 0.0), offset: (2, 3, -2, 4) }));

    // Controls
    ctx.add_child(root, Node::separator_labeled("Controls")
        .anchor(Anchor { min: (0.0, 0.0), max: (1.0, 0.0), offset: (2, 10, -2, 11) }));

    ctx.add_child(root, Node::button("Overview")
        .anchor(Anchor { min: (0.0, 0.0), max: (0.48, 0.0), offset: (3, 11, 0, 14) }));
    ctx.add_child(root, Node::button("Details")
        .anchor(Anchor { min: (0.5, 0.0), max: (1.0, 0.0), offset: (0, 11, -2, 14) }));
    ctx.add_child(root, Node::button("Refresh")
        .anchor(Anchor { min: (0.0, 0.0), max: (0.48, 0.0), offset: (3, 14, 0, 17) }));
    ctx.add_child(root, Node::button("Exit")
        .anchor(Anchor { min: (0.5, 0.0), max: (1.0, 0.0), offset: (0, 14, -2, 17) }));

    // Activity
    ctx.add_child(root, Node::separator_labeled("Activity")
        .anchor(Anchor { min: (0.0, 0.0), max: (1.0, 0.0), offset: (2, 17, -2, 18) }));

    ctx.add_child(root, Node::spinner(SpinnerStyle::Dots).with_label("Processing data...").fg(100, 200, 255)
        .anchor(Anchor { min: (0.0, 0.0), max: (0.5, 0.0), offset: (3, 18, 0, 19) }));

    ctx.add_child(root, Node::animated_text("Decay: build terminal apps with zero dependencies", 15.0).fg(180, 180, 195)
        .anchor(Anchor { min: (0.0, 0.0), max: (0.8, 0.0), offset: (3, 19, 0, 20) }));

    let status = ctx.add_child(root, Node::text("Select a button and press Enter...").dim()
        .anchor(Anchor { min: (0.0, 0.0), max: (0.8, 0.0), offset: (3, 21, 0, 22) }));
    ctx.insert_store(StatusLabel(status));

    ctx.add_child(root, Node::text("Up/Down/Tab Navigate   Enter/Space Click   q Quit").dim()
        .anchor(Anchor { min: (0.0, 0.0), max: (0.8, 0.0), offset: (3, 22, 0, 23) }));
}

fn animate(ctx: &mut Context) {
    ctx.store_mut::<FrameCount>().unwrap().0 += 1;
    let t = ctx.store::<FrameCount>().unwrap().0 as f32 * 0.02;

    let bars = ctx.store::<Bars>().unwrap();
    let (cpu, mem, disk, net) = (bars.cpu, bars.mem, bars.disk, bars.net);

    ctx.node_mut(cpu).unwrap().set_progress(0.55 + 0.35 * (t * 0.7).sin());
    ctx.node_mut(mem).unwrap().set_progress(0.70 + 0.20 * (t * 0.5 + 1.0).sin());
    ctx.node_mut(disk).unwrap().set_progress(0.20 + 0.15 * (t * 0.3 + 2.0).sin());
    ctx.node_mut(net).unwrap().set_progress(0.40 + 0.30 * (t * 0.9 + 3.0).sin());
}

fn handle_input(ctx: &mut Context) {
    if ctx.store::<Input>().is_some_and(|i| {
        i.just_pressed(KeyCode::Char('q')) || i.just_pressed(KeyCode::Escape)
    }) {
        ctx.store_mut::<AppExit>().unwrap().0 = true;
        return;
    }

    // Find pressed button
    let pressed_label: Option<String> = ctx.find(|n| n.is_button() && n.interaction == Interaction::Pressed)
        .and_then(|n| n.button_label().map(|s| s.to_string()));

    if let Some(label) = pressed_label {
        if label == "Exit" {
            ctx.store_mut::<AppExit>().unwrap().0 = true;
            return;
        }

        let status_id = ctx.store::<StatusLabel>().unwrap().0;
        let msg = match label.as_str() {
            "Overview" => "Overview panel activated",
            "Details" => "Details panel activated",
            "Refresh" => "Data refreshed",
            _ => "Unknown action",
        };
        ctx.node_mut(status_id).unwrap().set_text(msg);
    }
}
