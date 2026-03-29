/// Visual state for standalone text input rendering.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum InputState {
    Idle,
    Focused,
}

/// Standalone text input field display.
pub struct TextInput {
    pub value: String,
    pub placeholder: String,
    pub cursor: usize,
    pub state: InputState,
    pub fg_idle: (u8, u8, u8),
    pub bg_idle: (u8, u8, u8),
    pub fg_focus: (u8, u8, u8),
    pub bg_focus: (u8, u8, u8),
    pub border_idle: (u8, u8, u8),
    pub border_focus: (u8, u8, u8),
}

impl TextInput {
    pub fn new(placeholder: &str) -> Self {
        Self {
            value: String::new(),
            placeholder: placeholder.to_string(),
            cursor: 0,
            state: InputState::Idle,
            fg_idle: (200, 200, 210),
            bg_idle: (45, 45, 58),
            fg_focus: (230, 235, 255),
            bg_focus: (50, 80, 160),
            border_idle: (70, 70, 90),
            border_focus: (90, 130, 220),
        }
    }

    pub fn with_value(mut self, text: &str) -> Self {
        self.value = text.to_string();
        self.cursor = self.value.len();
        self
    }

    pub fn with_cursor(mut self, pos: usize) -> Self {
        self.cursor = pos;
        self
    }

    pub fn with_state(mut self, state: InputState) -> Self {
        self.state = state;
        self
    }

    pub fn focused(self) -> Self {
        self.with_state(InputState::Focused)
    }

    /// Render this text input to a plain 3-line string (no ANSI colors).
    pub fn render(&self, width: usize) -> String {
        let inner = width.saturating_sub(2);
        let top = format!("\u{250c}{}\u{2510}", "\u{2500}".repeat(inner));
        let bottom = format!("\u{2514}{}\u{2518}", "\u{2500}".repeat(inner));

        let text = if self.value.is_empty() && self.state == InputState::Idle {
            &self.placeholder
        } else {
            &self.value
        };
        let truncated: String = text.chars().take(inner).collect();
        let pad = inner.saturating_sub(truncated.chars().count());
        let middle = format!("\u{2502}{}{}\u{2502}", truncated, " ".repeat(pad));

        format!("{}\n{}\n{}", top, middle, bottom)
    }

    /// Print this text input to stdout with ANSI colors based on state.
    pub fn print(&self, width: usize) {
        use std::io::Write;
        let inner = width.saturating_sub(2);

        let (fg, bg, border) = match self.state {
            InputState::Idle => (self.fg_idle, self.bg_idle, self.border_idle),
            InputState::Focused => (self.fg_focus, self.bg_focus, self.border_focus),
        };
        let (fr, fg_g, fb) = fg;
        let (br, bg_g, bb) = bg;
        let (dr, dg, db) = border;

        let border_style = format!("\x1b[38;2;{dr};{dg};{db}m\x1b[48;2;{br};{bg_g};{bb}m");
        let text_style = format!("\x1b[38;2;{fr};{fg_g};{fb}m\x1b[48;2;{br};{bg_g};{bb}m");

        // Top border
        print!("{border_style}\u{250c}{}\u{2510}\x1b[0m\n", "\u{2500}".repeat(inner));

        // Content row
        print!("{border_style}\u{2502}");
        if self.value.is_empty() {
            if self.state == InputState::Idle {
                // Dim placeholder
                print!("\x1b[2m{text_style}");
                let ph: String = self.placeholder.chars().take(inner).collect();
                let pad = inner.saturating_sub(ph.chars().count());
                print!("{ph}{}\x1b[22m", " ".repeat(pad));
            } else {
                // Focused, empty: show cursor block at position 0
                print!("{text_style}\u{2588}{}", " ".repeat(inner.saturating_sub(1)));
            }
        } else if self.state == InputState::Focused {
            // Show value with block cursor at cursor position
            let chars: Vec<char> = self.value.chars().collect();
            let visible: Vec<char> = chars.iter().take(inner).copied().collect();
            let cur = self.cursor.min(visible.len());
            let before: String = visible[..cur].iter().collect();
            let after: String = visible[cur..].iter().collect();
            print!("{text_style}{before}\u{2588}{after}");
            // +1 for cursor block
            let used = visible.len() + 1;
            if used < inner {
                print!("{}", " ".repeat(inner - used));
            }
        } else {
            // Idle with value
            let truncated: String = self.value.chars().take(inner).collect();
            let pad = inner.saturating_sub(truncated.chars().count());
            print!("{text_style}{truncated}{}", " ".repeat(pad));
        }
        print!("{border_style}\u{2502}\x1b[0m\n");

        // Bottom border
        print!("{border_style}\u{2514}{}\u{2518}\x1b[0m\n", "\u{2500}".repeat(inner));

        std::io::stdout().flush().ok();
    }
}
