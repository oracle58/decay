pub mod theme;
pub mod label;
pub mod timer;
pub mod panel;
pub mod button;
pub mod input;
pub mod progress;

pub use theme::Theme;

use crate::core::app::{App, DeltaTime};
use crate::core::context::Context;
use crate::core::input::{Input, KeyCode, MouseButton, MouseKind};
use crate::core::node::*;
use crate::core::plugin::Plugin;
use crate::core::store::Store;
use crate::core::system::Lifecycle;
use crate::term::Term;

// Focus tracking store

struct UiState {
    focus_list: Vec<NodeId>,
    focus_index: usize,
}

impl Store for UiState {}

impl UiState {
    fn new() -> Self {
        Self { focus_list: Vec::new(), focus_index: 0 }
    }
}

/// Registers the UI interaction and rendering systems.
pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.ctx.insert_store(Theme::dark());
        app.ctx.insert_store(UiState::new());

        app.hook(Lifecycle::BeforeUpdate, resolve_anchors);
        app.hook(Lifecycle::BeforeUpdate, tick_animations);
        app.hook(Lifecycle::BeforeUpdate, update_interactions);
        app.hook(Lifecycle::BeforeRender, render_ui);
    }
}

// Anchor resolve system

fn resolve_anchors(ctx: &mut Context) {
    let (cols, rows) = ctx.store::<Term>()
        .map(|t| (t.cols(), t.rows()))
        .unwrap_or((80, 24));

    if let Some(root_id) = ctx.root() {
        resolve_node(ctx, root_id, Rect::new(0, 0, cols, rows));
    }
}

fn resolve_node(ctx: &mut Context, id: NodeId, parent_rect: Rect) {
    // Compute this node's rect from anchor
    if let Some(node) = ctx.node(id) {
        if let Some(anchor) = node.anchor {
            let rect = compute_anchor(anchor, parent_rect);
            if let Some(node) = ctx.node_mut(id) {
                node.rect = rect;
            }
        }
    }

    let children: Vec<NodeId> = ctx.children(id).to_vec();
    let my_rect = ctx.node(id).map(|n| n.rect).unwrap_or(parent_rect);
    for child_id in children {
        resolve_node(ctx, child_id, my_rect);
    }
}

fn compute_anchor(anchor: Anchor, parent: Rect) -> Rect {
    let px = parent.x as f32;
    let py = parent.y as f32;
    let pw = parent.width as f32;
    let ph = parent.height as f32;

    let x1 = px + pw * anchor.min.0 + anchor.offset.0 as f32;
    let y1 = py + ph * anchor.min.1 + anchor.offset.1 as f32;
    let x2 = px + pw * anchor.max.0 + anchor.offset.2 as f32;
    let y2 = py + ph * anchor.max.1 + anchor.offset.3 as f32;

    let x = x1.max(0.0) as u16;
    let y = y1.max(0.0) as u16;
    let w = (x2 - x1).max(0.0) as u16;
    let h = (y2 - y1).max(0.0) as u16;

    Rect::new(x, y, w, h)
}

// Animation tick

fn tick_animations(ctx: &mut Context) {
    let dt = ctx.store::<DeltaTime>().map(|d| d.0).unwrap_or(1.0 / 60.0);
    for node in ctx.nodes_mut() {
        match &mut node.content {
            Content::Spinner { elapsed, speed, .. } => {
                *elapsed += dt * *speed;
            }
            Content::AnimatedText { elapsed, speed, .. } => {
                *elapsed += dt * *speed;
            }
            _ => {}
        }
    }
}

// Interaction system

fn update_interactions(ctx: &mut Context) {
    // Build focus list from focusable visible nodes, sorted by position
    let mut focusable: Vec<(NodeId, u16, u16)> = ctx.nodes()
        .filter(|n| n.focusable && n.visible)
        .map(|n| (n.id, n.rect.y, n.rect.x))
        .collect();

    focusable.sort_by_key(|&(_, y, x)| (y, x));

    let entities: Vec<NodeId> = focusable.iter().map(|&(id, _, _)| id).collect();

    if entities.is_empty() {
        return;
    }

    let (tab_next, tab_prev, up, down, left, right, activate) = {
        let input = match ctx.store::<Input>() {
            Some(i) => i,
            None => return,
        };
        (
            input.just_pressed(KeyCode::Tab),
            input.just_pressed(KeyCode::BackTab),
            input.just_pressed(KeyCode::Up),
            input.just_pressed(KeyCode::Down),
            input.just_pressed(KeyCode::Left),
            input.just_pressed(KeyCode::Right),
            input.just_pressed(KeyCode::Enter) || input.just_pressed(KeyCode::Char(' ')),
        )
    };

    // Check mouse clicks
    let mouse_events: Vec<(MouseKind, u16, u16)> = ctx.store::<Input>()
        .map(|i| i.mouse_events().iter().map(|me| (me.kind, me.col, me.row)).collect())
        .unwrap_or_default();

    let mut mouse_clicked_id: Option<NodeId> = None;
    for &(kind, col, row) in &mouse_events {
        if matches!(kind, MouseKind::Press(MouseButton::Left)) {
            for &(fid, _, _) in &focusable {
                if let Some(node) = ctx.node(fid) {
                    let r = node.rect;
                    if col >= r.x && col < r.x + r.width
                        && row >= r.y && row < r.y + r.height
                    {
                        mouse_clicked_id = Some(fid);
                        break;
                    }
                }
            }
        }
    }

    let focused_entity = {
        let ui = ctx.store_mut::<UiState>().unwrap();
        ui.focus_list = entities.clone();

        if ui.focus_index >= ui.focus_list.len() {
            ui.focus_index = 0;
        }

        // Handle mouse click focus
        if let Some(clicked) = mouse_clicked_id {
            if let Some(idx) = ui.focus_list.iter().position(|&id| id == clicked) {
                ui.focus_index = idx;
            }
        }

        let len = ui.focus_list.len();
        if tab_next {
            ui.focus_index = (ui.focus_index + 1) % len;
        }
        if tab_prev {
            ui.focus_index = (ui.focus_index + len - 1) % len;
        }
        if up || down || left || right {
            let current_id = ui.focus_list[ui.focus_index];
            let ci = focusable.iter().position(|&(id, _, _)| id == current_id).unwrap_or(0);
            let (_, cy, cx) = focusable[ci];
            let mut best: Option<(usize, u32)> = None;
            for (i, &(_, y, x)) in focusable.iter().enumerate() {
                if i == ci {
                    continue;
                }
                let ok = if up {
                    y < cy
                } else if down {
                    y > cy
                } else if left {
                    x < cx
                } else {
                    x > cx
                };
                if !ok {
                    continue;
                }
                let dx = (x as i32 - cx as i32).unsigned_abs();
                let dy = (y as i32 - cy as i32).unsigned_abs();
                let cost = if up || down { dy * 10000 + dx } else { dx * 10000 + dy };
                if best.is_none() || cost < best.unwrap().1 {
                    best = Some((i, cost));
                }
            }
            if let Some((idx, _)) = best {
                // Map focusable index back to focus_list index
                let target_id = focusable[idx].0;
                if let Some(fi) = ui.focus_list.iter().position(|&id| id == target_id) {
                    ui.focus_index = fi;
                }
            }
        }
        ui.focus_list[ui.focus_index]
    };

    for &entity in &entities {
        if let Some(node) = ctx.node_mut(entity) {
            if entity == focused_entity {
                node.interaction = if activate || mouse_clicked_id == Some(entity) {
                    Interaction::Pressed
                } else {
                    Interaction::Focused
                };
            } else {
                node.interaction = Interaction::None;
            }
        }
    }

    // Drive TextInput keyboard handling for the focused node
    let is_text_input = ctx.node(focused_entity)
        .is_some_and(|n| matches!(n.content, Content::TextInput { .. }));

    if is_text_input {
        let keys: Vec<KeyCode> = ctx.store::<Input>()
            .map(|i| i.pressed_keys().to_vec())
            .unwrap_or_default();

        if let Some(node) = ctx.node_mut(focused_entity) {
            if let Content::TextInput { ref mut value, ref mut cursor, max_len, .. } = node.content {
                for key in &keys {
                    match key {
                        KeyCode::Char(c) => {
                            if value.chars().count() < max_len {
                                let byte_pos = value.char_indices()
                                    .nth(*cursor)
                                    .map(|(i, _)| i)
                                    .unwrap_or(value.len());
                                value.insert(byte_pos, *c);
                                *cursor += 1;
                            }
                        }
                        KeyCode::Backspace => {
                            if *cursor > 0 {
                                *cursor -= 1;
                                let byte_pos = value.char_indices()
                                    .nth(*cursor)
                                    .map(|(i, _)| i)
                                    .unwrap_or(value.len());
                                value.remove(byte_pos);
                            }
                        }
                        KeyCode::Left => {
                            if *cursor > 0 { *cursor -= 1; }
                        }
                        KeyCode::Right => {
                            if *cursor < value.chars().count() { *cursor += 1; }
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}

// Render helpers


const SMOOTH_BLOCKS: [char; 8] = [
    '\u{258F}', '\u{258E}', '\u{258D}', '\u{258C}',
    '\u{258B}', '\u{258A}', '\u{2589}', '\u{2588}',
];

fn lerp_color(a: (u8, u8, u8), b: (u8, u8, u8), t: f32) -> (u8, u8, u8) {
    (
        (a.0 as f32 + (b.0 as f32 - a.0 as f32) * t) as u8,
        (a.1 as f32 + (b.1 as f32 - a.1 as f32) * t) as u8,
        (a.2 as f32 + (b.2 as f32 - a.2 as f32) * t) as u8,
    )
}

fn apply_attrs(term: &mut Term, style: &Style) {
    if style.bold { term.bold(); }
    if style.dim { term.dim(); }
    if style.italic { term.italic(); }
    if style.underline { term.underline(); }
    if style.strikethrough { term.strikethrough(); }
}

fn border_chars(border: BorderStyle) -> (char, char, char, char, char, char) {
    match border {
        BorderStyle::Single  => ('\u{250C}', '\u{2510}', '\u{2514}', '\u{2518}', '\u{2500}', '\u{2502}'),
        BorderStyle::Rounded => ('\u{256D}', '\u{256E}', '\u{2570}', '\u{256F}', '\u{2500}', '\u{2502}'),
        BorderStyle::Double  => ('\u{2554}', '\u{2557}', '\u{255A}', '\u{255D}', '\u{2550}', '\u{2551}'),
        BorderStyle::Heavy   => ('\u{250F}', '\u{2513}', '\u{2517}', '\u{251B}', '\u{2501}', '\u{2503}'),
        BorderStyle::Ascii   => ('+', '+', '+', '+', '-', '|'),
    }
}

fn spinner_frames(style: SpinnerStyle) -> &'static [char] {
    match style {
        SpinnerStyle::Dots   => &['\u{280B}', '\u{2819}', '\u{2839}', '\u{2838}', '\u{283C}', '\u{2834}', '\u{2826}', '\u{2827}', '\u{2807}', '\u{280F}'],
        SpinnerStyle::Line   => &['\u{2500}', '\\', '\u{2502}', '/'],
        SpinnerStyle::Block  => &['\u{2596}', '\u{2598}', '\u{259D}', '\u{2597}'],
        SpinnerStyle::Circle => &['\u{25D0}', '\u{25D3}', '\u{25D1}', '\u{25D2}'],
    }
}

// Clipping

fn compute_clip(ctx: &Context, id: NodeId) -> Option<(u16, u16, u16, u16)> {
    let mut clip: Option<(u16, u16, u16, u16)> = None;
    let mut current = id;

    while let Some(node) = ctx.node(current) {
        let parent_id = match node.parent {
            Some(p) => p,
            None => break,
        };
        if let Some(parent) = ctx.node(parent_id) {
            let is_container = parent.is_container();
            let is_panel = matches!(parent.content, Content::Panel { .. });

            if is_container || is_panel {
                let r = parent.rect;
                let rect = if is_panel {
                    (r.x + 1, r.y + 1, r.width.saturating_sub(2), r.height.saturating_sub(2))
                } else {
                    (r.x, r.y, r.width, r.height)
                };
                clip = Some(match clip {
                    Some(c) => intersect_rects(c, rect),
                    None => rect,
                });
            }
        }
        current = parent_id;
    }

    clip
}

fn intersect_rects(
    a: (u16, u16, u16, u16),
    b: (u16, u16, u16, u16),
) -> (u16, u16, u16, u16) {
    let x = a.0.max(b.0);
    let y = a.1.max(b.1);
    let x1 = (a.0 + a.2).min(b.0 + b.2);
    let y1 = (a.1 + a.3).min(b.1 + b.3);
    (x, y, x1.saturating_sub(x), y1.saturating_sub(y))
}

fn apply_clip(term: &mut Term, clip: Option<(u16, u16, u16, u16)>) {
    if let Some((x, y, w, h)) = clip {
        term.set_clip(x, y, w, h);
    }
}

// Main render system

fn render_ui(ctx: &mut Context) {
    let theme = ctx.store::<Theme>().copied().unwrap_or(Theme::dark());

    // Collect visible nodes sorted by z_index
    let mut render_list: Vec<(NodeId, i16)> = ctx.nodes()
        .filter(|n| n.visible)
        .map(|n| (n.id, n.z_index))
        .collect();
    render_list.sort_by_key(|&(_, z)| z);

    // Collect clip info while we still have shared access
    let clip_map: Vec<(NodeId, Option<(u16, u16, u16, u16)>)> = render_list.iter()
        .map(|&(id, _)| (id, compute_clip(ctx, id)))
        .collect();

    // Snapshot all node data we need for rendering (avoids borrow conflicts with Term)
    struct NodeSnapshot {
        rect: Rect,
        style: Style,
        content: Content,
        align: TextAlign,
        interaction: Interaction,
        clip: Option<(u16, u16, u16, u16)>,
    }

    let snapshots: Vec<Option<NodeSnapshot>> = render_list.iter()
        .enumerate()
        .map(|(i, &(id, _))| {
            ctx.node(id).map(|node| NodeSnapshot {
                rect: node.rect,
                style: node.style.clone(),
                content: node.content.clone(),
                align: node.align,
                interaction: node.interaction,
                clip: clip_map[i].1,
            })
        })
        .collect();

    let term = match ctx.store_mut::<Term>() {
        Some(t) => t,
        None => return,
    };

    let cols = term.cols();
    let rows = term.rows();

    // Fill background
    let (br, bg, bb) = theme.bg;
    term.bg_rgb(br, bg, bb);
    for row in 0..rows {
        term.move_to(0, row).print_n(' ', cols as usize);
    }

    // Render each node
    for snap_opt in &snapshots {
        let snap = match snap_opt {
            Some(s) => s,
            None => continue,
        };

        let r = snap.rect;
        if r.width == 0 || r.height == 0 {
            continue;
        }

        apply_clip(term, snap.clip);

        match &snap.content {
            Content::Container => {}
            Content::Text(text) => {
                render_text(term, r, text, snap.align, &snap.style, &theme);
            }
            Content::Button(label) => {
                render_button(term, r, label, snap.interaction, &theme);
            }
            Content::Panel { title, border, shadow } => {
                render_panel(term, r, title.as_deref(), *border, *shadow, &snap.style, &theme);
            }
            Content::Separator { label } => {
                render_separator(term, r, label.as_deref(), &theme);
            }
            Content::Progress { value, style, fg_fill, fg_empty, gradient_end, show_label } => {
                render_progress(term, r, *value, *style, *fg_fill, *fg_empty, *gradient_end, *show_label, &snap.style, &theme);
            }
            Content::Spinner { style, elapsed, label, .. } => {
                render_spinner(term, r, *style, *elapsed, label.as_deref(), &snap.style, &theme);
            }
            Content::TextInput { value, cursor, placeholder, .. } => {
                render_text_input(term, r, value, *cursor, placeholder, snap.interaction, &theme);
            }
            Content::AnimatedText { text, elapsed, .. } => {
                render_animated_text(term, r, text, *elapsed, &snap.style, &theme);
            }
        }

        term.clear_clip();
    }

    term.reset();
}

// Individual render functions

fn render_text(term: &mut Term, r: Rect, text: &str, align: TextAlign, style: &Style, theme: &Theme) {
    let (fr, fg, fb) = style.fg.unwrap_or(theme.fg);
    let (br, bg, bb) = style.bg.unwrap_or(theme.bg);

    term.move_to(r.x, r.y).fg_rgb(fr, fg, fb).bg_rgb(br, bg, bb);
    apply_attrs(term, style);

    let w = r.width as usize;
    let char_count = text.chars().count();
    let text_len = char_count.min(w);
    let truncated: String = text.chars().take(w).collect();

    match align {
        TextAlign::Left => {
            term.print(&truncated);
            if text_len < w {
                term.print_n(' ', w - text_len);
            }
        }
        TextAlign::Center => {
            let left_pad = (w - text_len) / 2;
            let right_pad = w - text_len - left_pad;
            term.print_n(' ', left_pad);
            term.print(&truncated);
            term.print_n(' ', right_pad);
        }
        TextAlign::Right => {
            term.print_n(' ', w - text_len);
            term.print(&truncated);
        }
    }

    term.reset();
}

fn render_button(term: &mut Term, r: Rect, label: &str, interaction: Interaction, theme: &Theme) {
    let (fg, bg, border_color) = match interaction {
        Interaction::Pressed => (theme.btn_fg_press, theme.btn_bg_press, theme.border_press),
        Interaction::Focused => (theme.btn_fg_focus, theme.btn_bg_focus, theme.border_focus),
        Interaction::None => (theme.btn_fg_idle, theme.btn_bg_idle, theme.border_idle),
    };

    let (tl, tr, bl, br_ch, h, v) = match interaction {
        Interaction::Focused | Interaction::Pressed => {
            ('\u{256D}', '\u{256E}', '\u{2570}', '\u{256F}', '\u{2500}', '\u{2502}')
        }
        Interaction::None => {
            ('\u{250C}', '\u{2510}', '\u{2514}', '\u{2518}', '\u{2500}', '\u{2502}')
        }
    };

    let inner = (r.width as usize).saturating_sub(2);

    // Top border
    term.move_to(r.x, r.y)
        .fg_rgb(border_color.0, border_color.1, border_color.2)
        .bg_rgb(theme.bg.0, theme.bg.1, theme.bg.2);
    term.print_char(tl).print_n(h, inner).print_char(tr);

    // Middle rows
    let text_row = r.height / 2;
    for row in 1..r.height.saturating_sub(1) {
        term.move_to(r.x, r.y + row)
            .fg_rgb(border_color.0, border_color.1, border_color.2)
            .bg_rgb(theme.bg.0, theme.bg.1, theme.bg.2)
            .print_char(v);

        term.fg_rgb(fg.0, fg.1, fg.2).bg_rgb(bg.0, bg.1, bg.2);

        if row == text_row {
            let formatted = match interaction {
                Interaction::Focused => format!(" \u{25B8} {} ", label),
                Interaction::Pressed => format!(" \u{2022} {} ", label),
                Interaction::None => label.to_string(),
            };
            let text_len = formatted.chars().count().min(inner);
            let left_pad = inner.saturating_sub(text_len) / 2;
            let right_pad = inner.saturating_sub(text_len).saturating_sub(left_pad);

            if interaction != Interaction::None {
                term.bold();
            }

            let truncated: String = formatted.chars().take(text_len).collect();
            term.print_n(' ', left_pad);
            term.print(&truncated);
            term.print_n(' ', right_pad);

            if interaction != Interaction::None {
                term.reset();
            }
        } else {
            term.print_n(' ', inner);
        }

        term.fg_rgb(border_color.0, border_color.1, border_color.2)
            .bg_rgb(theme.bg.0, theme.bg.1, theme.bg.2)
            .print_char(v);
    }

    // Bottom border
    term.move_to(r.x, r.y + r.height - 1)
        .fg_rgb(border_color.0, border_color.1, border_color.2)
        .bg_rgb(theme.bg.0, theme.bg.1, theme.bg.2);
    term.print_char(bl).print_n(h, inner).print_char(br_ch);
}

fn render_panel(
    term: &mut Term,
    r: Rect,
    title: Option<&str>,
    border: BorderStyle,
    shadow: bool,
    style: &Style,
    theme: &Theme,
) {
    let (tl, tr, bl, br, h, v) = border_chars(border);
    let border_fg = style.fg.unwrap_or(theme.border_idle);
    let panel_bg = style.bg.unwrap_or(theme.bg);
    let inner = (r.width as usize).saturating_sub(2);

    // Shadow
    if shadow {
        let sh = theme.shadow;
        term.bg_rgb(sh.0, sh.1, sh.2).fg_rgb(sh.0, sh.1, sh.2);
        for row in (r.y + 1)..(r.y + r.height + 1) {
            term.move_to(r.x + r.width, row).print_char(' ');
        }
        term.move_to(r.x + 1, r.y + r.height).print_n(' ', r.width as usize);
    }

    // Top border
    term.fg_rgb(border_fg.0, border_fg.1, border_fg.2)
        .bg_rgb(panel_bg.0, panel_bg.1, panel_bg.2);
    term.move_to(r.x, r.y).print_char(tl);

    if let Some(title_text) = title {
        let max_title = inner.saturating_sub(4);
        let title_str: String = title_text.chars().take(max_title).collect();
        let title_len = title_str.chars().count();
        term.print_char(h).print_char(' ');
        term.fg_rgb(theme.fg.0, theme.fg.1, theme.fg.2).bold();
        term.print(&title_str);
        term.reset();
        term.fg_rgb(border_fg.0, border_fg.1, border_fg.2)
            .bg_rgb(panel_bg.0, panel_bg.1, panel_bg.2);
        term.print_char(' ');
        let remaining = inner.saturating_sub(title_len + 3);
        term.print_n(h, remaining);
    } else {
        term.print_n(h, inner);
    }
    term.print_char(tr);

    // Middle rows
    for row in 1..r.height.saturating_sub(1) {
        term.fg_rgb(border_fg.0, border_fg.1, border_fg.2)
            .bg_rgb(panel_bg.0, panel_bg.1, panel_bg.2);
        term.move_to(r.x, r.y + row).print_char(v);
        term.print_n(' ', inner);
        term.print_char(v);
    }

    // Bottom border
    term.fg_rgb(border_fg.0, border_fg.1, border_fg.2)
        .bg_rgb(panel_bg.0, panel_bg.1, panel_bg.2);
    term.move_to(r.x, r.y + r.height - 1);
    term.print_char(bl).print_n(h, inner).print_char(br);
}

fn render_separator(term: &mut Term, r: Rect, label: Option<&str>, theme: &Theme) {
    let fg = theme.border_idle;
    term.fg_rgb(fg.0, fg.1, fg.2)
        .bg_rgb(theme.bg.0, theme.bg.1, theme.bg.2);
    term.move_to(r.x, r.y);

    if let Some(label_text) = label {
        let max_label = (r.width as usize).saturating_sub(6);
        let truncated: String = label_text.chars().take(max_label).collect();
        let label_w = truncated.chars().count();
        term.print_n('\u{2500}', 3);
        term.print_char(' ');
        term.fg_rgb(theme.fg.0, theme.fg.1, theme.fg.2).bold();
        term.print(&truncated);
        term.reset();
        term.fg_rgb(fg.0, fg.1, fg.2)
            .bg_rgb(theme.bg.0, theme.bg.1, theme.bg.2);
        term.print_char(' ');
        let remaining = (r.width as usize).saturating_sub(5 + label_w);
        term.print_n('\u{2500}', remaining);
    } else {
        term.print_n('\u{2500}', r.width as usize);
    }
}

fn render_progress(
    term: &mut Term,
    r: Rect,
    value: f32,
    style: ProgressStyle,
    fg_fill: (u8, u8, u8),
    fg_empty: (u8, u8, u8),
    gradient_end: Option<(u8, u8, u8)>,
    show_label: bool,
    node_style: &Style,
    theme: &Theme,
) {
    let bg = node_style.bg.unwrap_or(theme.bg);
    term.move_to(r.x, r.y).bg_rgb(bg.0, bg.1, bg.2);

    // Reserve space for label inside the rect so it doesn't overflow
    let label_width = if show_label { 5 } else { 0 }; // " XX%"
    let bar_width = (r.width as usize).saturating_sub(label_width);

    match style {
        ProgressStyle::Smooth => {
            let w = bar_width;
            let filled_f = value * w as f32;
            let filled = filled_f as usize;
            let frac = filled_f - filled as f32;
            let end = gradient_end.unwrap_or(fg_fill);

            for i in 0..filled.min(w) {
                let t = if w > 1 { i as f32 / (w - 1) as f32 } else { 0.0 };
                let c = lerp_color(fg_fill, end, t);
                term.fg_rgb(c.0, c.1, c.2).print_char('\u{2588}');
            }

            let mut empty_start = filled;
            if frac > 0.05 && filled < w {
                let idx = ((frac * 8.0) as usize).min(7);
                let t = if w > 1 { filled as f32 / (w - 1) as f32 } else { 0.0 };
                let c = lerp_color(fg_fill, end, t);
                term.fg_rgb(c.0, c.1, c.2).print_char(SMOOTH_BLOCKS[idx]);
                empty_start = filled + 1;
            }

            let remaining = w.saturating_sub(empty_start);
            term.fg_rgb(fg_empty.0, fg_empty.1, fg_empty.2);
            term.print_n('\u{2591}', remaining);
        }
        ProgressStyle::Classic => {
            let inner = bar_width.saturating_sub(2);
            let filled = (value * inner as f32) as usize;
            let empty = inner.saturating_sub(filled);
            term.fg_rgb(fg_fill.0, fg_fill.1, fg_fill.2).print_char('[');
            term.print_n('#', filled);
            term.fg_rgb(fg_empty.0, fg_empty.1, fg_empty.2);
            term.print_n('-', empty);
            term.fg_rgb(fg_fill.0, fg_fill.1, fg_fill.2).print_char(']');
        }
        ProgressStyle::Dot => {
            let w = bar_width;
            let filled = (value * w as f32) as usize;
            let empty = w.saturating_sub(filled);
            term.fg_rgb(fg_fill.0, fg_fill.1, fg_fill.2);
            term.print_n('\u{25CF}', filled);
            term.fg_rgb(fg_empty.0, fg_empty.1, fg_empty.2);
            term.print_n('\u{25CB}', empty);
        }
    }

    if show_label {
        let pct = (value * 100.0) as u32;
        let label = format!(" {:>3}%", pct);
        term.fg_rgb(fg_fill.0, fg_fill.1, fg_fill.2)
            .bg_rgb(bg.0, bg.1, bg.2);
        term.print(&label);
    }
}

fn render_spinner(
    term: &mut Term,
    r: Rect,
    style: SpinnerStyle,
    elapsed: f32,
    label: Option<&str>,
    node_style: &Style,
    theme: &Theme,
) {
    let frames = spinner_frames(style);
    let idx = (elapsed * 10.0) as usize % frames.len();
    let ch = frames[idx];

    let fg = node_style.fg.unwrap_or(theme.fg);
    term.move_to(r.x, r.y)
        .fg_rgb(fg.0, fg.1, fg.2)
        .bg_rgb(theme.bg.0, theme.bg.1, theme.bg.2);
    term.print_char(ch);
    if let Some(label_text) = label {
        term.print_char(' ').print(label_text);
    }
}

fn render_text_input(
    term: &mut Term,
    r: Rect,
    value: &str,
    cursor: usize,
    placeholder: &str,
    interaction: Interaction,
    theme: &Theme,
) {
    let w = r.width as usize;
    let focused = interaction == Interaction::Focused || interaction == Interaction::Pressed;

    let border_color = if focused { theme.border_focus } else { theme.border_idle };
    let fg = if focused { theme.btn_fg_focus } else { theme.fg };
    let bg = if focused { theme.btn_bg_focus } else { theme.btn_bg_idle };

    // Border top
    term.move_to(r.x, r.y)
        .fg_rgb(border_color.0, border_color.1, border_color.2)
        .bg_rgb(theme.bg.0, theme.bg.1, theme.bg.2);
    let inner = w.saturating_sub(2);
    term.print_char('\u{250C}').print_n('\u{2500}', inner).print_char('\u{2510}');

    // Content row
    if r.height >= 2 {
        term.move_to(r.x, r.y + 1)
            .fg_rgb(border_color.0, border_color.1, border_color.2)
            .bg_rgb(theme.bg.0, theme.bg.1, theme.bg.2)
            .print_char('\u{2502}');

        term.fg_rgb(fg.0, fg.1, fg.2).bg_rgb(bg.0, bg.1, bg.2);

        if value.is_empty() && !focused {
            // Show placeholder
            let ph: String = placeholder.chars().take(inner).collect();
            let ph_len = ph.chars().count();
            term.dim();
            term.print(&ph);
            term.print_n(' ', inner.saturating_sub(ph_len));
            term.reset();
        } else {
            // Show value with cursor
            let display: String = value.chars().take(inner).collect();
            let disp_len = display.chars().count();
            term.print(&display);
            if focused && cursor <= disp_len && cursor < inner {
                // Blinking cursor is handled by the cursor position
            }
            term.print_n(' ', inner.saturating_sub(disp_len));
        }

        term.fg_rgb(border_color.0, border_color.1, border_color.2)
            .bg_rgb(theme.bg.0, theme.bg.1, theme.bg.2)
            .print_char('\u{2502}');
    }

    // Border bottom
    if r.height >= 3 {
        term.move_to(r.x, r.y + r.height - 1)
            .fg_rgb(border_color.0, border_color.1, border_color.2)
            .bg_rgb(theme.bg.0, theme.bg.1, theme.bg.2);
        term.print_char('\u{2514}').print_n('\u{2500}', inner).print_char('\u{2518}');
    }
}

fn render_animated_text(
    term: &mut Term,
    r: Rect,
    full_text: &str,
    elapsed: f32,
    style: &Style,
    theme: &Theme,
) {
    let fg = style.fg.unwrap_or(theme.fg);
    let bg = style.bg.unwrap_or(theme.bg);
    term.move_to(r.x, r.y)
        .fg_rgb(fg.0, fg.1, fg.2)
        .bg_rgb(bg.0, bg.1, bg.2);
    apply_attrs(term, style);

    let total_chars = full_text.chars().count();
    // Speed is stored in Content::AnimatedText, but we receive elapsed which already
    // includes speed factor from tick_animations. The visible count is based on elapsed
    // as chars-per-second equivalent: elapsed itself is scaled by speed already.
    // We use the raw elapsed as a character index (matching the old AnimatedText logic
    // where speed was in chars/sec).
    let visible = (elapsed as usize).min(total_chars);
    let visible_text: String = full_text.chars().take(visible).collect();
    let finished = visible >= total_chars;

    let text_len = visible_text.chars().count();
    let w = r.width as usize;

    if text_len < w {
        term.print(&visible_text);
        if !finished {
            let show_cursor = ((elapsed * 2.0) as u32) % 2 == 0;
            term.print_char(if show_cursor { '\u{2588}' } else { ' ' });
            term.print_n(' ', w.saturating_sub(text_len + 1));
        } else {
            term.print_n(' ', w - text_len);
        }
    } else {
        let truncated: String = visible_text.chars().take(w).collect();
        term.print(&truncated);
    }

    term.reset();
}
