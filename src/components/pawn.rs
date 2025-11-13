use bevy::prelude::*;
use super::GridPosition;

#[derive(Component)]
pub struct Pawn {
    pub name: String,
    pub move_speed: f32,
}

impl Default for Pawn {
    fn default() -> Self {
        Self {
            name: "Worker".to_string(),
            move_speed: 100.0, // pixels per second
        }
    }
}

#[derive(Component)]
pub struct MovementTarget {
    pub target: Vec2,
}

#[derive(Component, Default)]
pub struct CurrentJob {
    pub job_id: Option<Entity>,
}

// A pawn occupies 2x2 tiles
pub const PAWN_GRID_SIZE: i32 = 2;
