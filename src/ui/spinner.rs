use std::cell::Cell;

use crate::core::node::SpinnerStyle;

/// Standalone spinner frame iterator.
///
/// Cycles through animation frames for a given spinner style.
/// Works without the full TUI framework.
pub struct SpinnerFrames {
    frames: &'static [char],
    index: Cell<usize>,
}

impl SpinnerFrames {
    /// Create a new spinner with the given style.
    pub fn new(style: SpinnerStyle) -> Self {
        Self {
            frames: frames_for(style),
            index: Cell::new(0),
        }
    }

    /// Return the current frame character and advance to the next.
    pub fn tick(&self) -> char {
        let i = self.index.get();
        let ch = self.frames[i];
        self.index.set((i + 1) % self.frames.len());
        ch
    }

    /// Return the current frame character without advancing.
    pub fn current(&self) -> char {
        self.frames[self.index.get()]
    }
}

/// Get the frame characters for a spinner style.
pub(crate) fn frames_for(style: SpinnerStyle) -> &'static [char] {
    match style {
        SpinnerStyle::Dots   => &['\u{280B}', '\u{2819}', '\u{2839}', '\u{2838}', '\u{283C}', '\u{2834}', '\u{2826}', '\u{2827}', '\u{2807}', '\u{280F}'],
        SpinnerStyle::Line   => &['\u{2500}', '\\', '\u{2502}', '/'],
        SpinnerStyle::Block  => &['\u{2596}', '\u{2598}', '\u{259D}', '\u{2597}'],
        SpinnerStyle::Circle => &['\u{25D0}', '\u{25D3}', '\u{25D1}', '\u{25D2}'],
    }
}
