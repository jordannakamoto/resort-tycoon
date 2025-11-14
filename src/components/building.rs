use bevy::prelude::*;
use serde::{Deserialize, Serialize};

pub const DOOR_THICKNESS: f32 = 0.6;
pub const WINDOW_THICKNESS: f32 = 0.75;

#[derive(Component, Debug, Clone, Copy)]
pub struct GridPosition {
    pub x: i32,
    pub y: i32,
}

impl GridPosition {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub fn to_ivec2(&self) -> IVec2 {
        IVec2::new(self.x, self.y)
    }
}

#[derive(Component)]
pub struct Wall;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct WallProjection {
    pub north: bool, // Has projection on top
    pub east: bool,  // Has projection on right
    pub west: bool,  // Has projection on left
}

impl WallProjection {
    pub fn new() -> Self {
        Self {
            north: false,
            east: false,
            west: false,
        }
    }

    pub fn with_north(mut self) -> Self {
        self.north = true;
        self
    }

    pub fn with_east(mut self) -> Self {
        self.east = true;
        self
    }

    pub fn with_west(mut self) -> Self {
        self.west = true;
        self
    }
}

#[derive(Component)]
pub struct Door {
    pub orientation: DoorOrientation,
    pub state: DoorState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DoorOrientation {
    Horizontal, // 2 tiles wide (left-right)
    Vertical,   // 2 tiles tall (up-down)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DoorState {
    Closed,
    Open,
}

impl Door {
    pub fn new(orientation: DoorOrientation) -> Self {
        Self {
            orientation,
            state: DoorState::Closed,
        }
    }

    pub fn tiles_occupied(&self, base_pos: IVec2) -> Vec<IVec2> {
        match self.orientation {
            DoorOrientation::Horizontal => vec![base_pos, base_pos + IVec2::new(1, 0)],
            DoorOrientation::Vertical => vec![base_pos, base_pos + IVec2::new(0, 1)],
        }
    }
}

#[derive(Component)]
pub struct Window;

#[derive(Component)]
pub struct Building;

#[derive(Component)]
pub struct Floor {
    pub floor_type: FloorType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FloorType {
    Wood,
    Stone,
    Carpet,
    Tile,
}

impl FloorType {
    pub fn color(&self) -> Color {
        match self {
            FloorType::Wood => Color::srgb(0.6, 0.4, 0.2),
            FloorType::Stone => Color::srgb(0.4, 0.4, 0.4),
            FloorType::Carpet => Color::srgb(0.7, 0.3, 0.3),
            FloorType::Tile => Color::srgb(0.9, 0.9, 0.9),
        }
    }
}

#[derive(Component)]
pub struct PlacementPreview;

// Material types for buildings
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WallMaterial {
    Wood,
    Stone,
    Concrete,
}

impl WallMaterial {
    pub fn color(&self) -> Color {
        match self {
            WallMaterial::Wood => Color::srgb(0.6, 0.4, 0.2),
            WallMaterial::Stone => Color::srgb(0.5, 0.5, 0.5),
            WallMaterial::Concrete => Color::srgb(0.7, 0.7, 0.7),
        }
    }
}
