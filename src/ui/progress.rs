use crate::core::node::ProgressStyle;
#[cfg(feature = "spinner")]
use crate::core::node::SpinnerStyle;
#[cfg(feature = "spinner")]
use super::spinner::SpinnerFrames;

/// Progress bar component with configurable style and colors.
pub struct ProgressBar {
    pub value: f32,
    pub style: ProgressStyle,
    pub fg_fill: (u8, u8, u8),
    pub fg_empty: (u8, u8, u8),
    pub gradient_end: Option<(u8, u8, u8)>,
    pub show_label: bool,
    #[cfg(feature = "spinner")]
    spinner: Option<SpinnerFrames>,
}


impl ProgressBar {
    pub fn new(value: f32) -> Self {
        Self {
            value: value.clamp(0.0, 1.0),
            style: ProgressStyle::Smooth,
            fg_fill: (80, 200, 120),
            fg_empty: (50, 50, 60),
            gradient_end: None,
            show_label: false,
            #[cfg(feature = "spinner")]
            spinner: None,
        }
    }

    pub fn classic(value: f32) -> Self {
        Self { style: ProgressStyle::Classic, ..Self::new(value) }
    }

    /// Prepend an animated spinner to the progress bar.
    ///
    /// Each call to `print()` advances the spinner one frame.
    #[cfg(feature = "spinner")]
    pub fn with_spinner(mut self, style: SpinnerStyle) -> Self {
        self.spinner = Some(SpinnerFrames::new(style));
        self
    }

    pub fn set(&mut self, value: f32) {
        self.value = value.clamp(0.0, 1.0);
    }

    pub fn with_gradient(mut self, end: (u8, u8, u8)) -> Self {
        self.gradient_end = Some(end);
        self
    }

    pub fn with_label(mut self) -> Self {
        self.show_label = true;
        self
    }

    pub fn with_colors(mut self, fill: (u8, u8, u8), empty: (u8, u8, u8)) -> Self {
        self.fg_fill = fill;
        self.fg_empty = empty;
        self
    }

    /// Render this progress bar to a string for standalone use.
    /// Returns a styled string representation of the progress bar.
    pub fn render(&self, width: usize) -> String {
        match self.style {
            ProgressStyle::Smooth => {
                let fill_width = (self.value * width as f32) as usize;
                let remainder = (self.value * width as f32 * 8.0) as usize % 8;
                let blocks = [
                    ' ', '\u{258f}', '\u{258e}', '\u{258d}', '\u{258c}', '\u{258b}',
                    '\u{258a}', '\u{2589}', '\u{2588}',
                ];
                let mut s = String::with_capacity(width + 20);
                for _ in 0..fill_width {
                    s.push('\u{2588}');
                }
                if fill_width < width {
                    s.push(blocks[remainder]);
                    for _ in (fill_width + 1)..width {
                        s.push('\u{2591}');
                    }
                }
                if self.show_label {
                    s.push_str(&format!(" {:>3}%", (self.value * 100.0) as u32));
                }
                s
            }
            ProgressStyle::Classic => {
                let fill = (self.value * width as f32) as usize;
                let empty = width.saturating_sub(fill);
                let mut s = String::with_capacity(width + 10);
                s.push('[');
                for _ in 0..fill {
                    s.push('#');
                }
                for _ in 0..empty {
                    s.push('-');
                }
                s.push(']');
                if self.show_label {
                    s.push_str(&format!(" {:>3}%", (self.value * 100.0) as u32));
                }
                s
            }
            ProgressStyle::Dot => {
                let fill = (self.value * width as f32) as usize;
                let empty = width.saturating_sub(fill);
                let mut s = String::with_capacity(width + 10);
                for _ in 0..fill {
                    s.push('\u{25cf}');
                }
                for _ in 0..empty {
                    s.push('\u{25cb}');
                }
                if self.show_label {
                    s.push_str(&format!(" {:>3}%", (self.value * 100.0) as u32));
                }
                s
            }
        }
    }

    /// Print this progress bar to stdout with ANSI colors.
    /// Works standalone without the full framework.
    pub fn print(&self, width: usize) {
        use std::io::Write;
        let (fr, fg, fb) = self.fg_fill;
        let (er, eg, eb) = self.fg_empty;
        let fill_count = (self.value * width as f32) as usize;
        // Move to start of line
        print!("\r");
        // Spinner prefix (if enabled)
        #[cfg(feature = "spinner")]
        if let Some(ref spinner) = self.spinner {
            let frame = spinner.tick();
            print!("{frame} ");
        }
        // Set fill color
        print!("\x1b[38;2;{fr};{fg};{fb}m");
        match self.style {
            ProgressStyle::Smooth => {
                let remainder = (self.value * width as f32 * 8.0) as usize % 8;
                let blocks = [
                    ' ', '\u{258f}', '\u{258e}', '\u{258d}', '\u{258c}', '\u{258b}',
                    '\u{258a}', '\u{2589}', '\u{2588}',
                ];
                for _ in 0..fill_count {
                    print!("\u{2588}");
                }
                if fill_count < width {
                    print!("{}", blocks[remainder]);
                    print!("\x1b[38;2;{er};{eg};{eb}m");
                    for _ in (fill_count + 1)..width {
                        print!("\u{2591}");
                    }
                }
            }
            ProgressStyle::Classic => {
                print!("[");
                for _ in 0..fill_count {
                    print!("#");
                }
                print!("\x1b[38;2;{er};{eg};{eb}m");
                for _ in 0..width.saturating_sub(fill_count) {
                    print!("-");
                }
                print!("]");
            }
            ProgressStyle::Dot => {
                for _ in 0..fill_count {
                    print!("\u{25cf}");
                }
                print!("\x1b[38;2;{er};{eg};{eb}m");
                for _ in 0..width.saturating_sub(fill_count) {
                    print!("\u{25cb}");
                }
            }
        }
        if self.show_label {
            print!("\x1b[0m {:>3}%", (self.value * 100.0) as u32);
        }
        print!("\x1b[0m");
        std::io::stdout().flush().ok();
    }
}
