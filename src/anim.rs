use crate::core::NodeId;

/// Easing function for animation interpolation.
#[derive(Clone, Copy)]
pub enum Easing {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
}

impl Easing {
    pub fn apply(self, t: f32) -> f32 {
        match self {
            Easing::Linear => t,
            Easing::EaseIn => t * t,
            Easing::EaseOut => 1.0 - (1.0 - t) * (1.0 - t),
            Easing::EaseInOut => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
                }
            }
        }
    }
}

/// Animatable property target.
#[derive(Clone, Copy)]
pub enum AnimProp {
    X,
    Y,
    Width,
    Height,
}

/// A tween that interpolates a property over time.
pub struct Tween {
    pub target: NodeId,
    pub prop: AnimProp,
    pub from: f32,
    pub to: f32,
    pub duration: f32,
    pub elapsed: f32,
    pub easing: Easing,
    pub finished: bool,
}


impl Tween {
    pub fn new(target: NodeId, prop: AnimProp, from: f32, to: f32, duration: f32) -> Self {
        Self {
            target,
            prop,
            from,
            to,
            duration,
            elapsed: 0.0,
            easing: Easing::Linear,
            finished: false,
        }
    }

    pub fn with_easing(mut self, easing: Easing) -> Self {
        self.easing = easing;
        self
    }

    pub fn advance(&mut self, dt: f32) -> f32 {
        if self.duration <= 0.0 {
            self.finished = true;
            return self.to;
        }
        self.elapsed += dt;
        let t = (self.elapsed / self.duration).min(1.0);
        if t >= 1.0 {
            self.finished = true;
        }
        let eased = self.easing.apply(t);
        self.from + (self.to - self.from) * eased
    }
}

/// A sequence of keyframes for complex animations.
pub struct KeyframeTrack {
    pub target: NodeId,
    pub prop: AnimProp,
    pub keyframes: Vec<(f32, f32)>,
    pub easing: Easing,
    pub elapsed: f32,
    pub finished: bool,
}


impl KeyframeTrack {
    pub fn new(target: NodeId, prop: AnimProp, keyframes: Vec<(f32, f32)>) -> Self {
        Self { target, prop, keyframes, easing: Easing::Linear, elapsed: 0.0, finished: false }
    }

    pub fn advance(&mut self, dt: f32) -> f32 {
        self.elapsed += dt;
        if self.keyframes.len() < 2 {
            self.finished = true;
            return self.keyframes.first().map(|k| k.1).unwrap_or(0.0);
        }
        let total = self.keyframes.last().unwrap().0;
        if self.elapsed >= total {
            self.finished = true;
            return self.keyframes.last().unwrap().1;
        }
        let mut i = 0;
        while i + 1 < self.keyframes.len() && self.keyframes[i + 1].0 <= self.elapsed {
            i += 1;
        }
        if i + 1 >= self.keyframes.len() {
            return self.keyframes.last().unwrap().1;
        }
        let (t0, v0) = self.keyframes[i];
        let (t1, v1) = self.keyframes[i + 1];
        let span = t1 - t0;
        let local_t = if span > 0.0 { (self.elapsed - t0) / span } else { 1.0 };
        let eased = self.easing.apply(local_t);
        v0 + (v1 - v0) * eased
    }
}
