use bevy::prelude::*;

#[derive(Component, Debug, Clone)]
pub struct Guest {
    pub name: String,
    pub tier: GuestTier,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuestTier {
    Budget,
    Standard,
    Luxury,
}

/// Simple need weights for extensibility.
#[derive(Component, Debug, Clone)]
pub struct GuestNeeds {
    pub rest: f32,
    pub bladder: f32,
}

impl Default for GuestNeeds {
    fn default() -> Self {
        Self {
            rest: 0.0,
            bladder: 0.0,
        }
    }
}
