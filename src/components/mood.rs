use bevy::prelude::*;

/// Overall emotional state for a pawn/guest, driven by moodlets.
#[derive(Component, Debug, Clone)]
pub struct Mood {
    pub baseline: f32,
    pub current: f32,
    pub moodlets: Vec<Moodlet>,
}

impl Default for Mood {
    fn default() -> Self {
        Self {
            baseline: 0.0,
            current: 0.0,
            moodlets: Vec::new(),
        }
    }
}

impl Mood {
    pub fn recompute(&mut self) {
        let delta: f32 = self.moodlets.iter().map(|m| m.value).sum();
        self.current = (self.baseline + delta).clamp(-100.0, 100.0);
    }
}

#[derive(Debug, Clone)]
pub struct Moodlet {
    pub name: String,
    pub value: f32,
    pub remaining_seconds: f32,
}

impl Moodlet {
    pub fn new(name: impl Into<String>, value: f32, duration_seconds: f32) -> Self {
        Self {
            name: name.into(),
            value,
            remaining_seconds: duration_seconds,
        }
    }
}
