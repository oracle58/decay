use crate::core::node::BorderStyle;

/// Returns (top-left, top-right, bottom-left, bottom-right, horizontal, vertical) chars.
fn border_chars(style: &BorderStyle) -> (char, char, char, char, char, char) {
    match style {
        BorderStyle::Single  => ('\u{250c}', '\u{2510}', '\u{2514}', '\u{2518}', '\u{2500}', '\u{2502}'),
        BorderStyle::Rounded => ('\u{256d}', '\u{256e}', '\u{2570}', '\u{256f}', '\u{2500}', '\u{2502}'),
        BorderStyle::Double  => ('\u{2554}', '\u{2557}', '\u{255a}', '\u{255d}', '\u{2550}', '\u{2551}'),
        BorderStyle::Heavy   => ('\u{250f}', '\u{2513}', '\u{2517}', '\u{251b}', '\u{2501}', '\u{2503}'),
        BorderStyle::Ascii   => ('+', '+', '+', '+', '-', '|'),
    }
}

/// Standalone bordered panel component with configurable style and colors.
pub struct Panel {
    pub title: Option<String>,
    pub border: BorderStyle,
    pub fg: (u8, u8, u8),
    pub bg: Option<(u8, u8, u8)>,
    pub shadow: bool,
    pub height: usize,
    pub content: Option<String>,
}

impl Panel {
    pub fn new(title: &str) -> Self {
        Self {
            title: Some(title.into()),
            border: BorderStyle::Single,
            fg: (70, 70, 90),
            bg: None,
            shadow: false,
            height: 5,
            content: None,
        }
    }

    pub fn untitled() -> Self {
        Self {
            title: None,
            border: BorderStyle::Single,
            fg: (70, 70, 90),
            bg: None,
            shadow: false,
            height: 5,
            content: None,
        }
    }

    pub fn with_border(mut self, style: BorderStyle) -> Self {
        self.border = style;
        self
    }

    pub fn with_colors(mut self, fg: (u8, u8, u8), bg: (u8, u8, u8)) -> Self {
        self.fg = fg;
        self.bg = Some(bg);
        self
    }

    pub fn with_shadow(mut self) -> Self {
        self.shadow = true;
        self
    }

    pub fn with_height(mut self, h: usize) -> Self {
        self.height = h;
        self
    }

    pub fn with_content(mut self, text: &str) -> Self {
        self.content = Some(text.into());
        self
    }

    /// Render this panel to a plain multi-line string. Shadow is not rendered.
    pub fn render(&self, width: usize) -> String {
        let (tl, tr, bl, br, h, v) = border_chars(&self.border);
        let inner = width.saturating_sub(2);
        let mut lines = Vec::with_capacity(self.height);

        // Top border
        let top = match &self.title {
            Some(t) => {
                let t_len = t.chars().count();
                let avail = inner.saturating_sub(2); // space padding around title
                let displayed: String = if t_len > avail {
                    t.chars().take(avail).collect()
                } else {
                    t.clone()
                };
                let d_len = displayed.chars().count();
                let right_fill = inner.saturating_sub(d_len + 2);
                format!("{tl}{h} {displayed} {}{tr}", str::repeat(&h.to_string(), right_fill))
            }
            None => {
                format!("{tl}{}{tr}", str::repeat(&h.to_string(), inner))
            }
        };
        lines.push(top);

        // Middle rows
        let middle_rows = self.height.saturating_sub(2);
        for i in 0..middle_rows {
            if i == 0 {
                if let Some(ref text) = self.content {
                    let t_len = text.chars().count();
                    let pad = inner.saturating_sub(t_len);
                    lines.push(format!("{v}{text}{}{v}", " ".repeat(pad)));
                    continue;
                }
            }
            lines.push(format!("{v}{}{v}", " ".repeat(inner)));
        }

        // Bottom border
        lines.push(format!("{bl}{}{br}", str::repeat(&h.to_string(), inner)));

        lines.join("\n")
    }

    /// Print this panel to stdout with ANSI colors. Flushes stdout.
    pub fn print(&self, width: usize) {
        use std::io::Write;
        let (tl, tr, bl, br, h, v) = border_chars(&self.border);
        let (fr, fg, fb) = self.fg;
        let inner = width.saturating_sub(2);

        let fg_esc = format!("\x1b[38;2;{fr};{fg};{fb}m");
        let bg_esc = self.bg.map(|(r, g, b)| format!("\x1b[48;2;{r};{g};{b}m")).unwrap_or_default();
        let reset = "\x1b[0m";

        // Top border with optional bold title
        match &self.title {
            Some(t) => {
                let t_len = t.chars().count();
                let avail = inner.saturating_sub(2);
                let displayed: String = if t_len > avail {
                    t.chars().take(avail).collect()
                } else {
                    t.clone()
                };
                let d_len = displayed.chars().count();
                let right_fill = inner.saturating_sub(d_len + 2);
                print!("{fg_esc}{tl}{h} \x1b[1m{displayed}{reset}{fg_esc} {}{tr}{reset}", str::repeat(&h.to_string(), right_fill));
            }
            None => {
                print!("{fg_esc}{tl}{}{tr}{reset}", str::repeat(&h.to_string(), inner));
            }
        }
        if self.shadow {
            print!("\x1b[38;2;8;8;12m\u{2592}{reset}");
        }
        println!();

        // Middle rows
        let middle_rows = self.height.saturating_sub(2);
        for i in 0..middle_rows {
            print!("{fg_esc}{v}{reset}");
            if i == 0 {
                if let Some(ref text) = self.content {
                    let t_len = text.chars().count();
                    let pad = inner.saturating_sub(t_len);
                    print!("{bg_esc}{text}{}{reset}", " ".repeat(pad));
                } else {
                    print!("{bg_esc}{}{reset}", " ".repeat(inner));
                }
            } else {
                print!("{bg_esc}{}{reset}", " ".repeat(inner));
            }
            print!("{fg_esc}{v}{reset}");
            if self.shadow {
                print!("\x1b[38;2;8;8;12m\u{2592}{reset}");
            }
            println!();
        }

        // Bottom border
        print!("{fg_esc}{bl}{}{br}{reset}", str::repeat(&h.to_string(), inner));
        if self.shadow {
            print!("\x1b[38;2;8;8;12m\u{2592}{reset}");
        }
        println!();

        // Shadow bottom row
        if self.shadow {
            print!(" \x1b[38;2;8;8;12m{}{reset}", str::repeat("\u{2592}", width));
            println!();
        }

        std::io::stdout().flush().ok();
    }
}
