/// Unique identifier for a node in the tree.
pub type NodeId = u32;

/// Axis-aligned rectangle.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Rect {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

impl Rect {
    pub fn new(x: u16, y: u16, width: u16, height: u16) -> Self {
        Self { x, y, width, height }
    }
    pub fn zero() -> Self {
        Self { x: 0, y: 0, width: 0, height: 0 }
    }
}

/// Positions and sizes a node relative to its parent (or the screen).
///
/// `min` and `max` are proportional (0.0..1.0) attachment points.
/// `offset` is a pixel fine-tune: (left, top, right, bottom).
///
/// When `min == max`, the node is fixed-size at that anchor point.
/// When `min != max`, the node stretches between those proportional points.
#[derive(Clone, Copy, Debug)]
pub struct Anchor {
    pub min: (f32, f32),
    pub max: (f32, f32),
    pub offset: (i16, i16, i16, i16),
}

impl Anchor {
    pub fn new(min: (f32, f32), max: (f32, f32), offset: (i16, i16, i16, i16)) -> Self {
        Self { min, max, offset }
    }
    /// Fill the entire parent area.
    pub fn fill() -> Self {
        Self { min: (0.0, 0.0), max: (1.0, 1.0), offset: (0, 0, 0, 0) }
    }
}

/// Text alignment within a node.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TextAlign {
    Left,
    Center,
    Right,
}

/// Interaction state for focusable nodes.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Interaction {
    None,
    Focused,
    Pressed,
}

/// Border drawing style for panels.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BorderStyle {
    Single,
    Rounded,
    Double,
    Heavy,
    Ascii,
}

/// Visual style for progress bars.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ProgressStyle {
    /// Smooth block characters
    Smooth,
    /// Classic ASCII: [####....]
    Classic,
    /// Dot style
    Dot,
}

/// Animation style for spinners.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SpinnerStyle {
    Dots,
    Line,
    Block,
    Circle,
}

/// Visual style applied to a node.
#[derive(Clone, Debug)]
pub struct Style {
    pub fg: Option<(u8, u8, u8)>,
    pub bg: Option<(u8, u8, u8)>,
    pub bold: bool,
    pub dim: bool,
    pub italic: bool,
    pub underline: bool,
    pub strikethrough: bool,
}

impl Style {
    pub fn new() -> Self {
        Self { fg: None, bg: None, bold: false, dim: false, italic: false, underline: false, strikethrough: false }
    }
    pub fn fg(r: u8, g: u8, b: u8) -> Self {
        Self { fg: Some((r, g, b)), ..Self::new() }
    }
    pub fn dim() -> Self {
        Self { dim: true, ..Self::new() }
    }
}

/// What a node displays. Determines rendering behavior.
#[derive(Clone, Debug)]
pub enum Content {
    /// Invisible layout container.
    Container,
    /// Static text.
    Text(String),
    /// Clickable button with a label.
    Button(String),
    /// Bordered panel.
    Panel { title: Option<String>, border: BorderStyle, shadow: bool },
    /// Horizontal separator line.
    Separator { label: Option<String> },
    /// Progress indicator.
    Progress {
        value: f32,
        style: ProgressStyle,
        fg_fill: (u8, u8, u8),
        fg_empty: (u8, u8, u8),
        gradient_end: Option<(u8, u8, u8)>,
        show_label: bool,
    },
    /// Animated spinner.
    Spinner { style: SpinnerStyle, elapsed: f32, speed: f32, label: Option<String> },
    /// Text input field.
    TextInput { value: String, cursor: usize, max_len: usize, placeholder: String },
    /// Typewriter-animated text.
    AnimatedText { text: String, speed: f32, elapsed: f32 },
}

/// A node in the UI tree. Owns all its data inline.
#[derive(Debug)]
pub struct Node {
    pub(crate) id: NodeId,
    pub rect: Rect,
    pub anchor: Option<Anchor>,
    pub style: Style,
    pub content: Content,
    pub align: TextAlign,
    pub parent: Option<NodeId>,
    pub(crate) children: Vec<NodeId>,
    pub focusable: bool,
    pub interaction: Interaction,
    pub z_index: i16,
    pub visible: bool,
}

// Constructors

impl Node {
    fn base(content: Content) -> Self {
        Self {
            id: 0,
            rect: Rect::zero(),
            anchor: None,
            style: Style::new(),
            content,
            align: TextAlign::Left,
            parent: None,
            children: Vec::new(),
            focusable: false,
            interaction: Interaction::None,
            z_index: 0,
            visible: true,
        }
    }

    pub fn container() -> Self {
        Self::base(Content::Container)
    }

    pub fn text(value: &str) -> Self {
        Self::base(Content::Text(value.into()))
    }

    pub fn button(label: &str) -> Self {
        let mut n = Self::base(Content::Button(label.into()));
        n.focusable = true;
        n.interaction = Interaction::None;
        n
    }

    pub fn panel(title: &str) -> Self {
        Self::base(Content::Panel { title: Some(title.into()), border: BorderStyle::Single, shadow: false })
    }

    pub fn panel_untitled() -> Self {
        Self::base(Content::Panel { title: None, border: BorderStyle::Single, shadow: false })
    }

    pub fn separator() -> Self {
        Self::base(Content::Separator { label: None })
    }

    pub fn separator_labeled(label: &str) -> Self {
        Self::base(Content::Separator { label: Some(label.into()) })
    }

    pub fn progress(value: f32) -> Self {
        Self::base(Content::Progress {
            value: value.clamp(0.0, 1.0),
            style: ProgressStyle::Smooth,
            fg_fill: (80, 200, 120),
            fg_empty: (50, 50, 60),
            gradient_end: None,
            show_label: false,
        })
    }

    pub fn progress_classic(value: f32) -> Self {
        Self::base(Content::Progress {
            value: value.clamp(0.0, 1.0),
            style: ProgressStyle::Classic,
            fg_fill: (80, 200, 120),
            fg_empty: (50, 50, 60),
            gradient_end: None,
            show_label: false,
        })
    }

    pub fn spinner(style: SpinnerStyle) -> Self {
        Self::base(Content::Spinner { style, elapsed: 0.0, speed: 1.0, label: None })
    }

    pub fn text_input(placeholder: &str, max_len: usize) -> Self {
        let mut n = Self::base(Content::TextInput {
            value: String::new(),
            cursor: 0,
            max_len,
            placeholder: placeholder.into(),
        });
        n.focusable = true;
        n
    }

    pub fn animated_text(text: &str, speed: f32) -> Self {
        Self::base(Content::AnimatedText { text: text.into(), speed, elapsed: 0.0 })
    }
}

// Builder methods (chainable)

impl Node {
    pub fn anchor(mut self, anchor: Anchor) -> Self {
        self.anchor = Some(anchor);
        self
    }

    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn fg(mut self, r: u8, g: u8, b: u8) -> Self {
        self.style.fg = Some((r, g, b));
        self
    }

    pub fn bg(mut self, r: u8, g: u8, b: u8) -> Self {
        self.style.bg = Some((r, g, b));
        self
    }

    pub fn bold(mut self) -> Self {
        self.style.bold = true;
        self
    }

    pub fn dim(mut self) -> Self {
        self.style.dim = true;
        self
    }

    pub fn italic(mut self) -> Self {
        self.style.italic = true;
        self
    }

    pub fn underline(mut self) -> Self {
        self.style.underline = true;
        self
    }

    pub fn center(mut self) -> Self {
        self.align = TextAlign::Center;
        self
    }

    pub fn right(mut self) -> Self {
        self.align = TextAlign::Right;
        self
    }

    pub fn z(mut self, z: i16) -> Self {
        self.z_index = z;
        self
    }

    pub fn hidden(mut self) -> Self {
        self.visible = false;
        self
    }

    // Panel-specific builders

    pub fn border(mut self, style: BorderStyle) -> Self {
        if let Content::Panel { ref mut border, .. } = self.content {
            *border = style;
        }
        self
    }

    pub fn shadow(mut self) -> Self {
        if let Content::Panel { ref mut shadow, .. } = self.content {
            *shadow = true;
        }
        self
    }

    // Progress-specific builders

    pub fn gradient(mut self, end: (u8, u8, u8)) -> Self {
        if let Content::Progress { ref mut gradient_end, .. } = self.content {
            *gradient_end = Some(end);
        }
        self
    }

    pub fn label(mut self) -> Self {
        if let Content::Progress { ref mut show_label, .. } = self.content {
            *show_label = true;
        }
        self
    }

    pub fn colors(mut self, fill: (u8, u8, u8), empty: (u8, u8, u8)) -> Self {
        if let Content::Progress { ref mut fg_fill, ref mut fg_empty, .. } = self.content {
            *fg_fill = fill;
            *fg_empty = empty;
        }
        self
    }

    // Spinner-specific builders

    pub fn with_label(mut self, text: &str) -> Self {
        if let Content::Spinner { ref mut label, .. } = self.content {
            *label = Some(text.into());
        }
        self
    }
}

// Accessor helpers

impl Node {
    pub fn id(&self) -> NodeId {
        self.id
    }

    pub fn children(&self) -> &[NodeId] {
        &self.children
    }

    pub fn is_container(&self) -> bool {
        matches!(self.content, Content::Container)
    }

    pub fn is_button(&self) -> bool {
        matches!(self.content, Content::Button(_))
    }

    pub fn button_label(&self) -> Option<&str> {
        match &self.content {
            Content::Button(label) => Some(label),
            _ => None,
        }
    }

    pub fn text_value(&self) -> Option<&str> {
        match &self.content {
            Content::Text(v) => Some(v),
            _ => None,
        }
    }

    pub fn set_text(&mut self, value: &str) {
        if let Content::Text(ref mut v) = self.content {
            v.clear();
            v.push_str(value);
        }
    }

    pub fn set_progress(&mut self, value: f32) {
        if let Content::Progress { value: ref mut v, .. } = self.content {
            *v = value.clamp(0.0, 1.0);
        }
    }
}
