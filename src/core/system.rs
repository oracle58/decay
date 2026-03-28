/// Lifecycle stages for the application loop.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Lifecycle {
    Setup,
    AfterSetup,
    BeforeUpdate,
    Update,
    BeforeRender,
    Render,
    AfterRender,
}

pub(crate) type StoredSystem = Box<dyn Fn(&mut super::context::Context) + Send + Sync>;
