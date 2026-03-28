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

#[cfg(feature = "anim")]
pub mod anim;
#[cfg(feature = "core")]
pub mod core;
#[cfg(feature = "intro")]
pub mod intro;
#[cfg(feature = "rand")]
pub mod rand;
#[cfg(feature = "serde")]
pub mod serde;
#[cfg(feature = "term")]
pub mod term;
#[cfg(feature = "time")]
pub mod time;
#[cfg(feature = "ui")]
pub mod ui;

// Expose progress types standalone when progress feature is on but full ui is off
#[cfg(all(feature = "progress", not(feature = "ui")))]
pub mod ui {
    pub mod progress;
    pub use progress::ProgressBar;
    pub use crate::core::node::ProgressStyle;
}

pub mod prelude {
    #[cfg(feature = "core")]
    pub use crate::core::input::{Input, KeyCode, MouseEvent};
    #[cfg(feature = "core")]
    pub use crate::core::{
        App, AppExit, DeltaTime, Context, Store, Lifecycle, NodeId, Node, Content,
        Style, Rect, TextAlign, Anchor, Interaction,
        BorderStyle, ProgressStyle, SpinnerStyle,
        Cmd, Events, IntroPlugin, State, Plugin, CorePlugins,
    };
    #[cfg(feature = "rand")]
    pub use crate::rand::Rng;
    #[cfg(feature = "ui")]
    pub use crate::ui::UiPlugin;
}
