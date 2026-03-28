use std::collections::HashMap;
use std::time::{Duration, Instant};

use super::context::Context;
use super::event::EventRegistry;
use super::input::{Input, KeyCode};
use super::store::Store;
use super::system::{Lifecycle, StoredSystem};
use crate::term::Term;

/// Set to true to exit the application loop.
pub struct AppExit(pub bool);
impl Store for AppExit {}

pub struct App {
    pub(crate) ctx: Context,
    pub(crate) systems: HashMap<Lifecycle, Vec<StoredSystem>>,
}

impl App {
    pub fn new() -> Self {
        Self { ctx: Context::new(), systems: HashMap::new() }
    }

    pub fn plugin<P: super::plugin::Plugin>(&mut self, plugin: P) -> &mut Self {
        plugin.build(self);
        self
    }

    /// Register a handler on a specific lifecycle stage (advanced).
    pub fn hook<F>(&mut self, stage: Lifecycle, f: F) -> &mut Self
    where
        F: Fn(&mut Context) + Send + Sync + 'static,
    {
        self.systems.entry(stage).or_default().push(Box::new(f));
        self
    }

    /// Register a setup handler (runs once at startup).
    pub fn setup<F>(&mut self, f: F) -> &mut Self
    where F: Fn(&mut Context) + Send + Sync + 'static,
    {
        self.hook(Lifecycle::Setup, f)
    }

    /// Register an update handler (runs every frame).
    pub fn update<F>(&mut self, f: F) -> &mut Self
    where F: Fn(&mut Context) + Send + Sync + 'static,
    {
        self.hook(Lifecycle::Update, f)
    }

    /// Register a handler that fires when a key is pressed.
    pub fn on_key<F>(&mut self, key: KeyCode, f: F) -> &mut Self
    where F: Fn(&mut Context) + Send + Sync + 'static,
    {
        self.hook(Lifecycle::Update, move |ctx: &mut Context| {
            if ctx.store::<Input>().is_some_and(|i| i.just_pressed(key)) {
                f(ctx);
            }
        })
    }

    /// Exit the application when the given key is pressed.
    pub fn quit_on(&mut self, key: KeyCode) -> &mut Self {
        self.on_key(key, |ctx| {
            ctx.store_mut::<AppExit>().unwrap().0 = true;
        })
    }

    /// Register an event type for double-buffered event passing.
    pub fn add_event<T: 'static>(&mut self) -> &mut Self {
        use super::event::Events;
        self.ctx.insert_store(Events::<T>::new());
        if self.ctx.store::<EventRegistry>().is_none() {
            self.ctx.insert_store(EventRegistry::new());
        }
        self.ctx.store_mut::<EventRegistry>().unwrap().register::<T>();
        self
    }

    pub fn run(&mut self) {
        self.ctx.insert_store(AppExit(false));

        Self::run_stage(&mut self.ctx, self.systems.get(&Lifecycle::Setup));
        Self::run_stage(&mut self.ctx, self.systems.get(&Lifecycle::AfterSetup));

        let has_loop = [
            Lifecycle::BeforeUpdate, Lifecycle::Update,
            Lifecycle::BeforeRender, Lifecycle::Render, Lifecycle::AfterRender,
        ].iter().any(|s| self.systems.get(s).is_some_and(|v| !v.is_empty()));

        if !has_loop { return; }

        let target_frame = Duration::from_millis(16);

        loop {
            let frame_start = Instant::now();
            self.ctx.increment_tick();

            // Swap event buffers
            Self::swap_events(&mut self.ctx);

            // Poll input
            {
                if let Some(input) = self.ctx.store_mut::<Input>() {
                    input.clear();
                }
                let (keys, mouse_events) = crate::term::poll_input();
                if let Some(input) = self.ctx.store_mut::<Input>() {
                    for key in keys { input.press(key); }
                    for me in mouse_events { input.push_mouse(me); }
                }
            }

            Self::run_stage(&mut self.ctx, self.systems.get(&Lifecycle::BeforeUpdate));
            if self.ctx.store::<AppExit>().is_some_and(|e| e.0) { break; }

            Self::run_stage(&mut self.ctx, self.systems.get(&Lifecycle::Update));
            if self.ctx.store::<AppExit>().is_some_and(|e| e.0) { break; }

            Self::run_stage(&mut self.ctx, self.systems.get(&Lifecycle::BeforeRender));
            Self::run_stage(&mut self.ctx, self.systems.get(&Lifecycle::Render));
            Self::run_stage(&mut self.ctx, self.systems.get(&Lifecycle::AfterRender));

            let elapsed = frame_start.elapsed();
            if elapsed < target_frame { std::thread::sleep(target_frame - elapsed); }
        }

        if let Some(term) = self.ctx.store_mut::<Term>() {
            term.show_cursor().leave_alt_screen().flush().ok();
        }
        crate::term::cleanup();
    }

    fn run_stage(ctx: &mut Context, systems: Option<&Vec<StoredSystem>>) {
        if let Some(systems) = systems {
            for system in systems {
                system(ctx);
            }
        }
    }

    fn swap_events(ctx: &mut Context) {
        if let Some(registry) = ctx.store::<EventRegistry>() {
            let registry = registry as *const EventRegistry;
            unsafe { &*registry }.swap_all(ctx);
        }
    }
}
