use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component)]
pub struct Furniture;

#[derive(Component)]
pub struct Bed {
    pub bed_type: BedType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BedType {
    Single,
    Double,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum FurnitureOrientation {
    #[default]
    East,
    South,
    West,
    North,
}

impl FurnitureOrientation {
    pub fn next(self) -> Self {
        match self {
            FurnitureOrientation::East => FurnitureOrientation::South,
            FurnitureOrientation::South => FurnitureOrientation::West,
            FurnitureOrientation::West => FurnitureOrientation::North,
            FurnitureOrientation::North => FurnitureOrientation::East,
        }
    }

    pub fn is_horizontal(&self) -> bool {
        matches!(
            self,
            FurnitureOrientation::East | FurnitureOrientation::West
        )
    }
}

impl Bed {
    pub fn new(bed_type: BedType) -> Self {
        Self { bed_type }
    }

    pub fn tiles_occupied(&self, base_pos: IVec2) -> Vec<IVec2> {
        let (width, height) = match self.bed_type {
            BedType::Single => (2, 3),
            BedType::Double => (4, 4),
        };

        let mut tiles = Vec::new();
        for x in 0..width {
            for y in 0..height {
                tiles.push(base_pos + IVec2::new(x, y));
            }
        }
        tiles
    }
}

#[derive(Component)]
pub struct Desk;

#[derive(Component)]
pub struct Chair;

#[derive(Component)]
pub struct Dresser;

#[derive(Component)]
pub struct Nightstand;

#[derive(Component)]
pub struct Toilet;

#[derive(Component)]
pub struct Sink;

#[derive(Component)]
pub struct Tub;

#[derive(Component)]
pub struct ReceptionConsole {
    pub placed_on_desk: Option<Entity>, // Reference to the desk it's on
}

impl ReceptionConsole {
    pub fn new() -> Self {
        Self {
            placed_on_desk: None,
        }
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FurnitureType {
    Bed(BedType),
    Desk,
    Chair,
    Dresser,
    Nightstand,
    Toilet,
    Sink,
    Tub,
    ReceptionConsole,
}

impl FurnitureType {
    pub fn color(&self) -> Color {
        match self {
            FurnitureType::Bed(_) => Color::srgb(0.8, 0.7, 0.6), // Beige/tan
            FurnitureType::Desk => Color::srgb(0.5, 0.3, 0.1),   // Dark brown
            FurnitureType::Chair => Color::srgb(0.6, 0.4, 0.2),  // Medium brown
            FurnitureType::Dresser => Color::srgb(0.5, 0.3, 0.1), // Dark brown
            FurnitureType::Nightstand => Color::srgb(0.6, 0.4, 0.2), // Medium brown
            FurnitureType::Toilet => Color::srgb(0.9, 0.9, 0.95),
            FurnitureType::Sink => Color::srgb(0.9, 0.9, 0.95),
            FurnitureType::Tub => Color::srgb(0.9, 0.9, 0.95),
            FurnitureType::ReceptionConsole => Color::srgb(0.3, 0.5, 0.7), // Blue-gray
        }
    }

    pub fn tiles_occupied(&self, base_pos: IVec2, orientation: FurnitureOrientation) -> Vec<IVec2> {
        let (width, height) = self.oriented_dimensions(orientation);
        let mut tiles = Vec::new();
        for x in 0..width {
            for y in 0..height {
                tiles.push(base_pos + IVec2::new(x, y));
            }
        }
        tiles
    }

    pub fn size(&self) -> (f32, f32) {
        let (width, height) = self.base_dimensions();
        (width as f32, height as f32)
    }

    pub fn oriented_dimensions(&self, orientation: FurnitureOrientation) -> (i32, i32) {
        let (width, height) = self.base_dimensions();
        if orientation.is_horizontal() {
            (width, height)
        } else {
            (height, width)
        }
    }

    pub fn base_dimensions(&self) -> (i32, i32) {
        match self {
            FurnitureType::Bed(BedType::Single) => (2, 3),
            FurnitureType::Bed(BedType::Double) => (4, 4),
            FurnitureType::Desk => (2, 2),
            FurnitureType::Chair => (1, 1),
            FurnitureType::Dresser => (2, 2),
            FurnitureType::Nightstand => (1, 1),
            FurnitureType::Toilet => (2, 2),
            FurnitureType::Sink => (1, 1),
            FurnitureType::Tub => (2, 4),
            FurnitureType::ReceptionConsole => (1, 1),
        }
    }

    pub fn name(&self) -> &str {
        match self {
            FurnitureType::Bed(BedType::Single) => "Single Bed",
            FurnitureType::Bed(BedType::Double) => "Double Bed",
            FurnitureType::Desk => "Desk",
            FurnitureType::Chair => "Chair",
            FurnitureType::Dresser => "Dresser",
            FurnitureType::Nightstand => "Nightstand",
            FurnitureType::Toilet => "Toilet",
            FurnitureType::Sink => "Sink",
            FurnitureType::Tub => "Tub",
            FurnitureType::ReceptionConsole => "Reception Console",
        }
    }

    pub fn ascii_char(&self) -> char {
        match self {
            FurnitureType::Bed(_) => '▬',
            FurnitureType::Desk => '═',
            FurnitureType::Chair => 'π',
            FurnitureType::Dresser => '▓',
            FurnitureType::Nightstand => '□',
            FurnitureType::Toilet => '╥',
            FurnitureType::Sink => '○',
            FurnitureType::Tub => '≋',
            FurnitureType::ReceptionConsole => '▣',
        }
    }
}
