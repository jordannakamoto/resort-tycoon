use bevy::prelude::*;

#[derive(Component)]
pub struct Furniture;

#[derive(Component)]
pub struct Bed {
    pub bed_type: BedType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BedType {
    Single,
    Double,
}

impl Bed {
    pub fn new(bed_type: BedType) -> Self {
        Self { bed_type }
    }

    pub fn tiles_occupied(&self, base_pos: IVec2) -> Vec<IVec2> {
        match self.bed_type {
            BedType::Single => vec![
                base_pos,
                base_pos + IVec2::new(1, 0),
                base_pos + IVec2::new(0, 1),
                base_pos + IVec2::new(1, 1),
                base_pos + IVec2::new(0, 2),
                base_pos + IVec2::new(1, 2),
            ], // 2x3 tiles
            BedType::Double => vec![
                base_pos,
                base_pos + IVec2::new(1, 0),
                base_pos + IVec2::new(2, 0),
                base_pos + IVec2::new(0, 1),
                base_pos + IVec2::new(1, 1),
                base_pos + IVec2::new(2, 1),
                base_pos + IVec2::new(0, 2),
                base_pos + IVec2::new(1, 2),
                base_pos + IVec2::new(2, 2),
            ], // 3x3 tiles
        }
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
pub struct ReceptionConsole {
    pub placed_on_desk: Option<Entity>,  // Reference to the desk it's on
}

impl ReceptionConsole {
    pub fn new() -> Self {
        Self {
            placed_on_desk: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FurnitureType {
    Bed(BedType),
    Desk,
    Chair,
    Dresser,
    Nightstand,
    ReceptionConsole,
}

impl FurnitureType {
    pub fn color(&self) -> Color {
        match self {
            FurnitureType::Bed(_) => Color::srgb(0.8, 0.7, 0.6),      // Beige/tan
            FurnitureType::Desk => Color::srgb(0.5, 0.3, 0.1),        // Dark brown
            FurnitureType::Chair => Color::srgb(0.6, 0.4, 0.2),       // Medium brown
            FurnitureType::Dresser => Color::srgb(0.5, 0.3, 0.1),     // Dark brown
            FurnitureType::Nightstand => Color::srgb(0.6, 0.4, 0.2),  // Medium brown
            FurnitureType::ReceptionConsole => Color::srgb(0.3, 0.5, 0.7),  // Blue-gray
        }
    }

    pub fn tiles_occupied(&self, base_pos: IVec2) -> Vec<IVec2> {
        match self {
            FurnitureType::Bed(bed_type) => {
                match bed_type {
                    BedType::Single => vec![
                        base_pos,
                        base_pos + IVec2::new(1, 0),
                        base_pos + IVec2::new(0, 1),
                        base_pos + IVec2::new(1, 1),
                        base_pos + IVec2::new(0, 2),
                        base_pos + IVec2::new(1, 2),
                    ], // 2x3
                    BedType::Double => vec![
                        base_pos,
                        base_pos + IVec2::new(1, 0),
                        base_pos + IVec2::new(2, 0),
                        base_pos + IVec2::new(0, 1),
                        base_pos + IVec2::new(1, 1),
                        base_pos + IVec2::new(2, 1),
                        base_pos + IVec2::new(0, 2),
                        base_pos + IVec2::new(1, 2),
                        base_pos + IVec2::new(2, 2),
                    ], // 3x3
                }
            }
            FurnitureType::Desk => vec![
                base_pos,
                base_pos + IVec2::new(1, 0),
                base_pos + IVec2::new(0, 1),
                base_pos + IVec2::new(1, 1),
            ], // 2x2
            FurnitureType::Chair => vec![base_pos], // 1x1
            FurnitureType::Dresser => vec![
                base_pos,
                base_pos + IVec2::new(1, 0),
                base_pos + IVec2::new(0, 1),
                base_pos + IVec2::new(1, 1),
            ], // 2x2
            FurnitureType::Nightstand => vec![base_pos], // 1x1
            FurnitureType::ReceptionConsole => vec![base_pos], // 1x1 (placed on desk)
        }
    }

    pub fn size(&self) -> (f32, f32) {
        match self {
            FurnitureType::Bed(BedType::Single) => (2.0, 3.0),
            FurnitureType::Bed(BedType::Double) => (3.0, 3.0),
            FurnitureType::Desk => (2.0, 2.0),
            FurnitureType::Chair => (1.0, 1.0),
            FurnitureType::Dresser => (2.0, 2.0),
            FurnitureType::Nightstand => (1.0, 1.0),
            FurnitureType::ReceptionConsole => (1.0, 1.0),
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
            FurnitureType::ReceptionConsole => '▣',
        }
    }
}
