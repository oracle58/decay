use super::store::Store;

/// Current application state of type `S`.
pub struct State<S: 'static + Clone + PartialEq> {
    current: S,
    previous: Option<S>,
    changed: bool,
}

impl<S: 'static + Clone + PartialEq> Store for State<S> {}

impl<S: 'static + Clone + PartialEq> State<S> {
    pub fn new(initial: S) -> Self {
        Self { current: initial, previous: None, changed: true }
    }

    pub fn get(&self) -> &S {
        &self.current
    }

    pub fn set(&mut self, state: S) {
        if self.current != state {
            self.previous = Some(self.current.clone());
            self.current = state;
            self.changed = true;
        }
    }

    pub fn changed(&self) -> bool {
        self.changed
    }

    pub fn previous(&self) -> Option<&S> {
        self.previous.as_ref()
    }

    pub fn clear_changed(&mut self) {
        self.changed = false;
    }
}
