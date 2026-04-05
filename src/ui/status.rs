use std::io::Write;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::core::node::SpinnerStyle;
use super::spinner::SpinnerFrames;

struct Inner {
    spinner: SpinnerFrames,
    message: String,
    finished: bool,
    fg: Option<(u8, u8, u8)>,
}

/// Standalone single-line status output with an animated spinner.
///
/// Displays a spinner character and a message on one terminal line,
/// updating in place without scrolling. Works without the full TUI
/// framework — no alt-screen, no double buffer, no `App::run()`.
///
/// # Example
///
/// ```no_run
/// use std::time::Duration;
/// use decay::ui::StatusLine;
/// use decay::core::node::SpinnerStyle;
///
/// let status = StatusLine::new(SpinnerStyle::Dots);
/// status.spinner_color(80, 130, 255);
/// status.enable_steady_tick(Duration::from_millis(100));
///
/// status.set_message("Connecting...");
/// // ...
/// status.finish_with_message("Done.");
/// ```
pub struct StatusLine {
    inner: Arc<Mutex<Inner>>,
    is_term: bool,
}

impl StatusLine {
    /// Create a new status line with a spinner style.
    pub fn new(style: SpinnerStyle) -> Self {
        Self {
            inner: Arc::new(Mutex::new(Inner {
                spinner: SpinnerFrames::new(style),
                message: String::new(),
                finished: false,
                fg: None,
            })),
            is_term: is_tty(),
        }
    }

    /// Set the spinner color (RGB).
    pub fn spinner_color(&self, r: u8, g: u8, b: u8) -> &Self {
        if let Ok(mut inner) = self.inner.lock() {
            inner.fg = Some((r, g, b));
        }
        self
    }

    /// Start auto-ticking the spinner on a background thread.
    ///
    /// The spinner character will cycle at the given interval,
    /// redrawing the current message each tick.
    pub fn enable_steady_tick(&self, interval: Duration) {
        let inner = Arc::clone(&self.inner);
        let is_term = self.is_term;
        std::thread::spawn(move || {
            loop {
                std::thread::sleep(interval);
                let mut guard = match inner.lock() {
                    Ok(g) => g,
                    Err(_) => break,
                };
                if guard.finished {
                    break;
                }
                if is_term {
                    draw(&mut guard);
                }
            }
        });
    }

    /// Update the message displayed next to the spinner.
    pub fn set_message(&self, msg: impl Into<String>) {
        if let Ok(mut inner) = self.inner.lock() {
            inner.message = msg.into();
            if !self.is_term && !inner.finished {
                println!("{}", inner.message);
            }
        }
    }

    /// Print a line above the status line without breaking it.
    ///
    /// The status line is cleared, the message is printed on its own
    /// line, and the status line is redrawn below.
    pub fn println(&self, msg: &str) {
        if let Ok(mut inner) = self.inner.lock() {
            if self.is_term {
                print!("\r\x1b[K{}\n", msg);
                if !inner.finished {
                    draw(&mut inner);
                }
            } else {
                println!("{}", msg);
            }
        }
    }

    /// Stop the spinner and leave the final message on screen.
    pub fn finish_with_message(&self, msg: impl Into<String>) {
        if let Ok(mut inner) = self.inner.lock() {
            inner.finished = true;
            inner.message = msg.into();
            if self.is_term {
                print!("\r\x1b[K{}\n", inner.message);
                std::io::stdout().flush().ok();
            } else {
                println!("{}", inner.message);
            }
        }
    }

    /// Stop the spinner and clear the line.
    pub fn finish_and_clear(&self) {
        if let Ok(mut inner) = self.inner.lock() {
            inner.finished = true;
            if self.is_term {
                print!("\r\x1b[K");
                std::io::stdout().flush().ok();
            }
        }
    }
}

fn draw(inner: &mut Inner) {
    let frame = inner.spinner.tick();
    if let Some((r, g, b)) = inner.fg {
        print!("\r\x1b[38;2;{r};{g};{b}m{frame}\x1b[0m {}\x1b[K", inner.message);
    } else {
        print!("\r{frame} {}\x1b[K", inner.message);
    }
    std::io::stdout().flush().ok();
}

/// Check if stdout is connected to a terminal.
fn is_tty() -> bool {
    #[cfg(windows)]
    {
        const STD_OUTPUT_HANDLE: u32 = 0xFFFF_FFF5;
        unsafe extern "system" {
            fn GetStdHandle(id: u32) -> isize;
            fn GetConsoleMode(handle: isize, mode: *mut u32) -> i32;
        }
        unsafe {
            let h = GetStdHandle(STD_OUTPUT_HANDLE);
            let mut mode = 0u32;
            GetConsoleMode(h, &mut mode) != 0
        }
    }
    #[cfg(unix)]
    {
        unsafe extern "C" {
            fn isatty(fd: core::ffi::c_int) -> core::ffi::c_int;
        }
        unsafe { isatty(1) != 0 }
    }
    #[cfg(not(any(windows, unix)))]
    {
        false
    }
}
