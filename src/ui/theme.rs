use crate::core::store::Store;

/// Application-wide color theme.
#[derive(Clone, Copy)]
pub struct Theme {
    pub bg: (u8, u8, u8),
    pub fg: (u8, u8, u8),
    pub btn_bg_idle: (u8, u8, u8),
    pub btn_fg_idle: (u8, u8, u8),
    pub btn_bg_focus: (u8, u8, u8),
    pub btn_fg_focus: (u8, u8, u8),
    pub btn_bg_press: (u8, u8, u8),
    pub btn_fg_press: (u8, u8, u8),
    pub border_idle: (u8, u8, u8),
    pub border_focus: (u8, u8, u8),
    pub border_press: (u8, u8, u8),
    pub shadow: (u8, u8, u8),
}

impl Store for Theme {}

impl Theme {
    pub const fn dark() -> Self {
        Self {
            bg: (24, 24, 32),
            fg: (200, 200, 210),
            btn_bg_idle: (45, 45, 58),
            btn_fg_idle: (170, 170, 185),
            btn_bg_focus: (50, 80, 160),
            btn_fg_focus: (230, 235, 255),
            btn_bg_press: (40, 160, 80),
            btn_fg_press: (240, 255, 240),
            border_idle: (70, 70, 90),
            border_focus: (90, 130, 220),
            border_press: (60, 200, 110),
            shadow: (8, 8, 12),
        }
    }
}
