/// Text alignment for standalone label.
pub enum LabelAlign {
    Left,
    Center,
    Right,
}

/// Standalone styled text label.
pub struct Label {
    pub text: String,
    pub fg: (u8, u8, u8),
    pub bg: Option<(u8, u8, u8)>,
    pub bold: bool,
    pub dim: bool,
    pub italic: bool,
    pub underline: bool,
    pub strikethrough: bool,
    pub align: LabelAlign,
}

impl Label {
    pub fn new(text: &str) -> Self {
        Self {
            text: text.to_string(),
            fg: (200, 200, 210),
            bg: None,
            bold: false,
            dim: false,
            italic: false,
            underline: false,
            strikethrough: false,
            align: LabelAlign::Left,
        }
    }

    pub fn with_fg(mut self, r: u8, g: u8, b: u8) -> Self {
        self.fg = (r, g, b);
        self
    }

    pub fn with_bg(mut self, r: u8, g: u8, b: u8) -> Self {
        self.bg = Some((r, g, b));
        self
    }

    pub fn bold(mut self) -> Self {
        self.bold = true;
        self
    }

    pub fn dim(mut self) -> Self {
        self.dim = true;
        self
    }

    pub fn italic(mut self) -> Self {
        self.italic = true;
        self
    }

    pub fn underline(mut self) -> Self {
        self.underline = true;
        self
    }

    pub fn strikethrough(mut self) -> Self {
        self.strikethrough = true;
        self
    }

    pub fn center(mut self) -> Self {
        self.align = LabelAlign::Center;
        self
    }

    pub fn right(mut self) -> Self {
        self.align = LabelAlign::Right;
        self
    }

    /// Render this label to a plain string padded/truncated to `width`.
    /// Returns aligned text without ANSI escape codes.
    pub fn render(&self, width: usize) -> String {
        let text = if self.text.len() > width {
            &self.text[..width]
        } else {
            &self.text
        };
        match self.align {
            LabelAlign::Left => format!("{:<width$}", text, width = width),
            LabelAlign::Center => format!("{:^width$}", text, width = width),
            LabelAlign::Right => format!("{:>width$}", text, width = width),
        }
    }

    /// Print this label to stdout with ANSI colors and styles.
    /// Works standalone without the full framework.
    pub fn print(&self, width: usize) {
        use std::io::Write;
        let content = self.render(width);
        let (r, g, b) = self.fg;
        // Foreground color
        print!("\x1b[38;2;{r};{g};{b}m");
        // Background color
        if let Some((br, bg, bb)) = self.bg {
            print!("\x1b[48;2;{br};{bg};{bb}m");
        }
        // Style attributes
        if self.bold { print!("\x1b[1m"); }
        if self.dim { print!("\x1b[2m"); }
        if self.italic { print!("\x1b[3m"); }
        if self.underline { print!("\x1b[4m"); }
        if self.strikethrough { print!("\x1b[9m"); }
        // Content and reset
        print!("{content}\x1b[0m");
        std::io::stdout().flush().ok();
    }
}
