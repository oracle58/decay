/// Timer display mode.
pub enum TimerMode {
    Elapsed,
    Countdown,
}

/// Standalone timer display component.
pub struct TimerDisplay {
    pub elapsed_secs: f32,
    pub total_secs: f32,
    pub mode: TimerMode,
    pub fg: (u8, u8, u8),
    pub dim_fg: (u8, u8, u8),
    pub show_millis: bool,
}

impl TimerDisplay {
    pub fn elapsed(secs: f32) -> Self {
        Self {
            elapsed_secs: secs,
            total_secs: 0.0,
            mode: TimerMode::Elapsed,
            fg: (200, 200, 210),
            dim_fg: (100, 100, 110),
            show_millis: false,
        }
    }

    pub fn countdown(total: f32, remaining: f32) -> Self {
        Self {
            elapsed_secs: (total - remaining).max(0.0),
            total_secs: total,
            mode: TimerMode::Countdown,
            fg: (200, 200, 210),
            dim_fg: (100, 100, 110),
            show_millis: false,
        }
    }

    pub fn with_colors(mut self, fg: (u8, u8, u8), dim_fg: (u8, u8, u8)) -> Self {
        self.fg = fg;
        self.dim_fg = dim_fg;
        self
    }

    pub fn with_millis(mut self) -> Self {
        self.show_millis = true;
        self
    }

    pub fn set(&mut self, secs: f32) {
        self.elapsed_secs = secs;
    }

    fn display_secs(&self) -> f32 {
        match self.mode {
            TimerMode::Elapsed => self.elapsed_secs.max(0.0),
            TimerMode::Countdown => (self.total_secs - self.elapsed_secs).max(0.0),
        }
    }

    fn format_time(&self) -> String {
        let secs = self.display_secs();
        let total = secs as u32;
        let h = total / 3600;
        let m = (total % 3600) / 60;
        let s = total % 60;
        let millis = ((secs - secs.floor().max(0.0)) * 1000.0) as u32;
        if h > 0 {
            if self.show_millis {
                format!("{h:02}:{m:02}:{s:02}.{millis:03}")
            } else {
                format!("{h:02}:{m:02}:{s:02}")
            }
        } else if self.show_millis {
            format!("{m:02}:{s:02}.{millis:03}")
        } else {
            format!("{m:02}:{s:02}")
        }
    }

    /// Render the timer to a plain string, padded or truncated to width.
    /// No ANSI codes.
    pub fn render(&self, width: usize) -> String {
        let time = self.format_time();
        if time.len() >= width {
            time[..width].to_string()
        } else {
            format!("{time:>width$}")
        }
    }

    /// Print the timer to stdout with ANSI colors.
    /// Digits use `fg`, colons and dots use `dim_fg`.
    pub fn print(&self, width: usize) {
        use std::io::Write;
        let rendered = self.render(width);
        let (fr, fg, fb) = self.fg;
        let (dr, dg, db) = self.dim_fg;
        for ch in rendered.chars() {
            match ch {
                ':' | '.' => print!("\x1b[38;2;{dr};{dg};{db}m{ch}"),
                _ => print!("\x1b[38;2;{fr};{fg};{fb}m{ch}"),
            }
        }
        print!("\x1b[0m");
        std::io::stdout().flush().ok();
    }
}
