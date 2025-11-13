use bevy::prelude::*;

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

#[derive(Component)]
pub struct Door;

#[derive(Component)]
pub struct Window;

#[derive(Component)]
pub struct Building;

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
