use super::app::App;
use super::input::Input;
use super::system::Lifecycle;
use super::context::Context;
use crate::term::Term;

/// Extension point for registering handlers and stores on an App.
pub trait Plugin {
    fn build(&self, app: &mut App);
}

/// Registers terminal I/O, input polling, and frame rendering.
pub struct CorePlugins;

impl Plugin for CorePlugins {
    fn build(&self, app: &mut App) {
        app.hook(Lifecycle::Setup, |ctx| {
            crate::term::enable_raw_mode().expect("failed to enable raw mode");
            let mut term = Term::new().expect("failed to init terminal");
            term.enter_alt_screen().hide_cursor().flush().ok();
            ctx.insert_store(term);
            ctx.insert_store(Input::new());
        });

        app.hook(Lifecycle::BeforeRender, |ctx| {
            if let Some(term) = ctx.store_mut::<Term>() {
                term.clear();
            }
        });

        app.hook(Lifecycle::AfterRender, |ctx| {
            if let Some(term) = ctx.store_mut::<Term>() {
                term.flush().ok();
            }
        });
    }
}

/// Plays the decay tree intro animation on startup.
pub struct IntroPlugin;

impl Plugin for IntroPlugin {
    fn build(&self, app: &mut App) {
        app.hook(Lifecycle::AfterSetup, play_intro);
    }
}

fn play_intro(_ctx: &mut Context) {
    #[cfg(feature = "intro")]
    if let Some(term) = _ctx.store_mut::<Term>() {
        crate::intro::play(term);
    }
}
