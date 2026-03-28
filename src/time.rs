use crate::core::Store;

pub use crate::core::app::DeltaTime;

/// A countdown timer that can be repeating.
pub struct Timer {
    pub duration: f32,
    pub elapsed: f32,
    pub repeating: bool,
    pub finished: bool,
}

impl Store for Timer {}

impl Timer {
    pub fn once(duration: f32) -> Self {
        Self { duration, elapsed: 0.0, repeating: false, finished: false }
    }

    pub fn repeating(duration: f32) -> Self {
        Self { duration, elapsed: 0.0, repeating: true, finished: false }
    }

    pub fn tick(&mut self, dt: f32) {
        if self.finished && !self.repeating {
            return;
        }
        self.elapsed += dt;
        if self.elapsed >= self.duration {
            self.finished = true;
            if self.repeating {
                self.elapsed -= self.duration;
            }
        } else {
            self.finished = false;
        }
    }

    pub fn fraction(&self) -> f32 {
        (self.elapsed / self.duration).min(1.0)
    }

    pub fn reset(&mut self) {
        self.elapsed = 0.0;
        self.finished = false;
    }
}
