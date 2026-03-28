/// A keyboard key identifier.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum KeyCode {
    Char(char),
    Enter,
    Escape,
    Tab,
    BackTab,
    Up,
    Down,
    Left,
    Right,
    Backspace,
}

/// Mouse button type.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

/// Kind of mouse event.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MouseKind {
    Press(MouseButton),
    Release(MouseButton),
    Move,
    ScrollUp,
    ScrollDown,
}

/// A mouse event with position and kind.
#[derive(Clone, Copy, Debug)]
pub struct MouseEvent {
    pub kind: MouseKind,
    pub col: u16,
    pub row: u16,
}

/// Tracks keys and mouse events during the current frame.
pub struct Input {
    pressed: Vec<KeyCode>,
    mouse: Vec<MouseEvent>,
}

impl super::store::Store for Input {}

impl Input {
    pub fn new() -> Self {
        Self { pressed: Vec::with_capacity(8), mouse: Vec::with_capacity(4) }
    }

    pub fn just_pressed(&self, key: KeyCode) -> bool {
        self.pressed.contains(&key)
    }

    pub fn pressed_keys(&self) -> &[KeyCode] {
        &self.pressed
    }

    pub fn mouse_events(&self) -> &[MouseEvent] {
        &self.mouse
    }

    pub(crate) fn clear(&mut self) {
        self.pressed.clear();
        self.mouse.clear();
    }

    pub(crate) fn press(&mut self, key: KeyCode) {
        self.pressed.push(key);
    }

    pub(crate) fn push_mouse(&mut self, event: MouseEvent) {
        self.mouse.push(event);
    }
}
