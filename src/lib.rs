//! Decay: a zero-dependency TUI framework.
//!
//! Build terminal applications with a component tree architecture,
//! double-buffered rendering, and direct platform I/O.
//!
//! ```no_run
//! use decay::prelude::*;
//!
//! fn main() {
//!     App::new()
//!         .plugin(CorePlugins)
//!         .plugin(UiPlugin)
//!         .setup(|ctx: &mut Context| {
//!             ctx.add(Node::text("Hello, decay!")
//!                 .anchor(Anchor::new((0.0, 0.0), (0.0, 0.0), (4, 2, 34, 3))));
//!         })
//!         .quit_on(KeyCode::Char('q'))
//!         .run();
//! }
//! ```

pub mod anim;
pub mod core;
pub mod rand;
pub mod serde;
pub mod term;
pub mod time;

#[cfg(feature = "intro")]
pub mod intro;

#[cfg(feature = "ui")]
pub mod ui;

// Standalone components when ui feature is off but individual features are on
#[cfg(all(
    not(feature = "ui"),
    any(
        feature = "label", feature = "timer", feature = "panel",
        feature = "button", feature = "input", feature = "progress"
    )
))]
pub mod ui {
    #[cfg(feature = "label")]
    pub mod label;
    #[cfg(feature = "label")]
    pub use label::{Label, LabelAlign};

    #[cfg(feature = "timer")]
    pub mod timer;
    #[cfg(feature = "timer")]
    pub use timer::{TimerDisplay, TimerMode};

    #[cfg(feature = "panel")]
    pub mod panel;
    #[cfg(feature = "panel")]
    pub use panel::Panel;
    #[cfg(feature = "panel")]
    pub use crate::core::node::BorderStyle;

    #[cfg(feature = "button")]
    pub mod button;
    #[cfg(feature = "button")]
    pub use button::{Button, ButtonState};

    #[cfg(feature = "input")]
    pub mod input;
    #[cfg(feature = "input")]
    pub use input::{TextInput, InputState};

    #[cfg(feature = "progress")]
    pub mod progress;
    #[cfg(feature = "progress")]
    pub use progress::ProgressBar;
    #[cfg(feature = "progress")]
    pub use crate::core::node::ProgressStyle;
}

pub mod prelude {
    pub use crate::core::input::{Input, KeyCode, MouseEvent};
    pub use crate::core::{
        App, AppExit, DeltaTime, Context, Store, Lifecycle, NodeId, Node, Content,
        Style, Rect, TextAlign, Anchor, Interaction,
        BorderStyle, ProgressStyle, SpinnerStyle,
        Cmd, Events, IntroPlugin, State, Plugin, CorePlugins,
    };
    pub use crate::rand::Rng;
    #[cfg(feature = "ui")]
    pub use crate::ui::UiPlugin;
}
