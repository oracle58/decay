/// Visual state for standalone button rendering.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ButtonState {
    Idle,
    Focused,
    Pressed,
}

/// Standalone button display component.
pub struct Button {
    pub label: String,
    pub state: ButtonState,
    pub fg_idle: (u8, u8, u8),
    pub bg_idle: (u8, u8, u8),
    pub fg_focus: (u8, u8, u8),
    pub bg_focus: (u8, u8, u8),
    pub fg_press: (u8, u8, u8),
    pub bg_press: (u8, u8, u8),
    pub border_idle: (u8, u8, u8),
    pub border_focus: (u8, u8, u8),
    pub border_press: (u8, u8, u8),
    pub height: usize,
}

impl Button {
    pub fn new(label: &str) -> Self {
        Self {
            label: label.to_string(),
            state: ButtonState::Idle,
            fg_idle: (170, 170, 185),
            bg_idle: (45, 45, 58),
            fg_focus: (230, 235, 255),
            bg_focus: (50, 80, 160),
            fg_press: (240, 255, 240),
            bg_press: (40, 160, 80),
            border_idle: (70, 70, 90),
            border_focus: (90, 130, 220),
            border_press: (60, 200, 110),
            height: 3,
        }
    }

    pub fn with_state(mut self, state: ButtonState) -> Self {
        self.state = state;
        self
    }

    pub fn focused(self) -> Self {
        self.with_state(ButtonState::Focused)
    }

    pub fn pressed(self) -> Self {
        self.with_state(ButtonState::Pressed)
    }

    pub fn with_height(mut self, h: usize) -> Self {
        self.height = h;
        self
    }

    /// Render this button to a multi-line string for standalone use.
    pub fn render(&self, width: usize) -> String {
        let inner = width.saturating_sub(2);
        let (tl, tr, bl, br) = match self.state {
            ButtonState::Idle => ('┌', '┐', '└', '┘'),
            _ => ('╭', '╮', '╰', '╯'),
        };

        let prefix = match self.state {
            ButtonState::Focused => "▸ ",
            ButtonState::Pressed => "• ",
            ButtonState::Idle => "",
        };

        let display = format!("{prefix}{}", self.label);
        let display_width = display.chars().count();
        let pad_total = inner.saturating_sub(display_width);
        let pad_left = pad_total / 2;
        let pad_right = pad_total - pad_left;

        let label_row = format!(
            "│{}{display}{}│",
            " ".repeat(pad_left),
            " ".repeat(pad_right),
        );

        let top = format!("{tl}{}{tr}", "─".repeat(inner));
        let bottom = format!("{bl}{}{br}", "─".repeat(inner));
        let empty_row = format!("│{}│", " ".repeat(inner));

        let blank_above = (self.height.saturating_sub(1)) / 2;
        let blank_below = self.height.saturating_sub(1).saturating_sub(blank_above);

        let mut lines = Vec::with_capacity(self.height + 2);
        lines.push(top);
        for _ in 0..blank_above.saturating_sub(1) {
            lines.push(empty_row.clone());
        }
        lines.push(label_row);
        for _ in 0..blank_below.saturating_sub(1) {
            lines.push(empty_row.clone());
        }
        lines.push(bottom);
        lines.join("\n")
    }

    /// Print this button to stdout with ANSI colors.
    /// Works standalone without the full framework.
    pub fn print(&self, width: usize) {
        use std::io::Write;

        let (fg, bg, border) = match self.state {
            ButtonState::Idle => (self.fg_idle, self.bg_idle, self.border_idle),
            ButtonState::Focused => (self.fg_focus, self.bg_focus, self.border_focus),
            ButtonState::Pressed => (self.fg_press, self.bg_press, self.border_press),
        };

        let bold = match self.state {
            ButtonState::Idle => "",
            _ => "\x1b[1m",
        };

        let (fr, ffg, fb) = fg;
        let (br, bg_g, bb) = bg;
        let (dr, dg, db) = border;

        let fg_seq = format!("\x1b[38;2;{fr};{ffg};{fb}m");
        let bg_seq = format!("\x1b[48;2;{br};{bg_g};{bb}m");
        let border_seq = format!("\x1b[38;2;{dr};{dg};{db}m");
        let reset = "\x1b[0m";

        let rendered = self.render(width);
        for (i, line) in rendered.lines().enumerate() {
            if i == 0 || i == rendered.lines().count() - 1 {
                // Border lines
                print!("{border_seq}{bg_seq}{line}{reset}");
            } else {
                // Content lines: border chars in border color, content in fg color
                let inner = &line[3..line.len() - 3]; // skip │ on each side (3 bytes)
                print!(
                    "{border_seq}{bg_seq}│{bold}{fg_seq}{bg_seq}{inner}{border_seq}│{reset}",
                );
            }
            println!();
        }
        std::io::stdout().flush().ok();
    }
}
