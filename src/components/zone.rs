use bevy::prelude::*;
use std::collections::HashSet;

/// Represents a zone/district in the resort
#[derive(Component)]
pub struct Zone {
    pub zone_type: ZoneType,
    pub tiles: HashSet<IVec2>,
    pub quality: ZoneQuality,
    pub name: String,
}

impl Zone {
    pub fn new(zone_type: ZoneType, name: String) -> Self {
        Self {
            zone_type,
            tiles: HashSet::new(),
            quality: ZoneQuality::None,
            name,
        }
    }

    pub fn contains_tile(&self, pos: IVec2) -> bool {
        self.tiles.contains(&pos)
    }

    pub fn add_tile(&mut self, pos: IVec2) {
        self.tiles.insert(pos);
    }

    pub fn remove_tile(&mut self, pos: IVec2) {
        self.tiles.remove(&pos);
    }

    pub fn tile_count(&self) -> usize {
        self.tiles.len()
    }
}

/// Types of zones in the resort
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ZoneType {
    Lobby,
    GuestBedroom,
    Relaxation,
    Luxury,
    FamilyFun,
    Adventure,
    Culinary,
}

impl ZoneType {
    pub fn name(&self) -> &str {
        match self {
            ZoneType::Lobby => "Lobby",
            ZoneType::GuestBedroom => "Guest Bedroom",
            ZoneType::Relaxation => "Relaxation Zone",
            ZoneType::Luxury => "Luxury Zone",
            ZoneType::FamilyFun => "Family/Fun Zone",
            ZoneType::Adventure => "Adventure Zone",
            ZoneType::Culinary => "Culinary Zone",
        }
    }

    pub fn color(&self) -> Color {
        match self {
            ZoneType::Lobby => Color::srgba(0.7, 0.7, 0.9, 0.3),          // Light purple
            ZoneType::GuestBedroom => Color::srgba(0.5, 0.7, 1.0, 0.3),   // Light blue
            ZoneType::Relaxation => Color::srgba(0.5, 1.0, 0.7, 0.3),     // Light green
            ZoneType::Luxury => Color::srgba(1.0, 0.8, 0.3, 0.3),         // Gold
            ZoneType::FamilyFun => Color::srgba(1.0, 0.5, 0.7, 0.3),      // Pink
            ZoneType::Adventure => Color::srgba(1.0, 0.5, 0.2, 0.3),      // Orange
            ZoneType::Culinary => Color::srgba(0.9, 0.3, 0.3, 0.3),       // Red
        }
    }

    /// Returns the minimum requirements for this zone type
    pub fn requirements(&self) -> ZoneRequirements {
        match self {
            ZoneType::Lobby => ZoneRequirements {
                min_tiles: 15,  // Modest lobby area
                required_furniture: vec![RequiredFurniture::ReceptionConsole],
            },
            ZoneType::GuestBedroom => ZoneRequirements {
                min_tiles: 12,  // At least a small room (roughly 3x4 tiles)
                required_furniture: vec![RequiredFurniture::Bed],
            },
            ZoneType::Relaxation => ZoneRequirements {
                min_tiles: 20,
                required_furniture: vec![],
            },
            ZoneType::Luxury => ZoneRequirements {
                min_tiles: 30,
                required_furniture: vec![],
            },
            ZoneType::FamilyFun => ZoneRequirements {
                min_tiles: 25,
                required_furniture: vec![],
            },
            ZoneType::Adventure => ZoneRequirements {
                min_tiles: 25,
                required_furniture: vec![],
            },
            ZoneType::Culinary => ZoneRequirements {
                min_tiles: 20,
                required_furniture: vec![],
            },
        }
    }
}

/// Quality rating for a zone
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ZoneQuality {
    None,       // Not valid/missing requirements
    Basic,      // Meets minimum requirements
    Good,       // Has some optional features
    Excellent,  // Has most/all optional features
    Luxury,     // Exceeds all expectations
}

impl ZoneQuality {
    pub fn stars(&self) -> u8 {
        match self {
            ZoneQuality::None => 0,
            ZoneQuality::Basic => 1,
            ZoneQuality::Good => 2,
            ZoneQuality::Excellent => 3,
            ZoneQuality::Luxury => 4,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            ZoneQuality::None => "Invalid",
            ZoneQuality::Basic => "Basic",
            ZoneQuality::Good => "Good",
            ZoneQuality::Excellent => "Excellent",
            ZoneQuality::Luxury => "Luxury",
        }
    }
}

/// Requirements for a zone to be valid
pub struct ZoneRequirements {
    pub min_tiles: usize,
    pub required_furniture: Vec<RequiredFurniture>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RequiredFurniture {
    Bed,
    Desk,
    Chair,
    Dresser,
    Nightstand,
    ReceptionConsole,
}

/// Represents a room (enclosed area) in the resort
#[derive(Component)]
pub struct Room {
    pub tiles: HashSet<IVec2>,
}

impl Room {
    pub fn new(tiles: HashSet<IVec2>) -> Self {
        Self {
            tiles,
        }
    }

    pub fn contains_tile(&self, pos: IVec2) -> bool {
        self.tiles.contains(&pos)
    }

    pub fn tile_count(&self) -> usize {
        self.tiles.len()
    }
}
