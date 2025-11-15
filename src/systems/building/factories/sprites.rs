use bevy::prelude::*;
use crate::components::furniture::*;
use crate::systems::grid::GridSettings;

// Sprite path constants
const SINGLE_BED_SPRITE_PATH: &str = "generated/furniture/bed.png";
const DOUBLE_BED_SPRITE_PATH: &str = "generated/furniture/double_bed.png";
const DRESSER_FRONT_SPRITE_PATH: &str = "generated/furniture/dresser.png";
const DRESSER_BACK_SPRITE_PATH: &str = "generated/furniture/dresser_back.png";
const DRESSER_SIDE_SPRITE_PATH: &str = "generated/furniture/dresser_side.png";
const TUB_SPRITE_PATH: &str = "generated/furniture/tub.png";
const TOILET_SPRITE_PATH: &str = "generated/furniture/toilet.png";
const SINK_SPRITE_PATH: &str = "generated/furniture/sink.png";
const END_TABLE_SPRITE_PATH: &str = "generated/furniture/end_table.png";
const COMPUTER_SIDE_SPRITE_PATH: &str = "generated/furniture/computer_side.png";
const COMPUTER_FRONT_SPRITE_PATH: &str = "generated/furniture/computer_front.png";
const COMPUTER_BACK_SPRITE_PATH: &str = "generated/furniture/computer_back.png";

pub enum FurnitureSpriteConfig {
    Rotating {
        sprite: Sprite,
        rotation_radians: f32,
    },
    Directional {
        sprite: Sprite,
    },
    Mesh {
        color: Color,
    },
}

/// Converts furniture orientation to rotation in radians
pub fn furniture_rotation_radians(orientation: FurnitureOrientation) -> f32 {
    match orientation {
        FurnitureOrientation::East => 0.0,
        FurnitureOrientation::South => std::f32::consts::PI / 2.0,
        FurnitureOrientation::West => std::f32::consts::PI,
        FurnitureOrientation::North => -std::f32::consts::PI / 2.0,
    }
}

/// Creates the sprite configuration for furniture placement/preview
pub fn create_furniture_sprite(
    furniture_type: FurnitureType,
    orientation: FurnitureOrientation,
    asset_server: &AssetServer,
    grid_settings: &GridSettings,
    _is_preview: bool,
) -> FurnitureSpriteConfig {
    let (base_width_tiles, base_height_tiles) = furniture_type.base_dimensions();
    let sprite_size = Vec2::new(
        base_width_tiles as f32 * grid_settings.tile_size,
        base_height_tiles as f32 * grid_settings.tile_size,
    );

    match furniture_type {
        FurnitureType::Bed(bed_type) => {
            let sprite_path = match bed_type {
                BedType::Single => SINGLE_BED_SPRITE_PATH,
                BedType::Double => DOUBLE_BED_SPRITE_PATH,
            };

            FurnitureSpriteConfig::Rotating {
                sprite: Sprite {
                    image: asset_server.load(sprite_path),
                    custom_size: Some(sprite_size),
                    ..default()
                },
                rotation_radians: furniture_rotation_radians(orientation),
            }
        }
        FurnitureType::Dresser => {
            let (sprite_path, flip_x) = match orientation {
                FurnitureOrientation::South => (DRESSER_FRONT_SPRITE_PATH, false),
                FurnitureOrientation::North => (DRESSER_BACK_SPRITE_PATH, false),
                FurnitureOrientation::East => (DRESSER_SIDE_SPRITE_PATH, false),
                FurnitureOrientation::West => (DRESSER_SIDE_SPRITE_PATH, true),
            };

            let mut sprite = Sprite {
                image: asset_server.load(sprite_path),
                custom_size: Some(sprite_size),
                ..default()
            };
            sprite.flip_x = flip_x;

            FurnitureSpriteConfig::Directional { sprite }
        }
        FurnitureType::Tub => {
            FurnitureSpriteConfig::Rotating {
                sprite: Sprite {
                    image: asset_server.load(TUB_SPRITE_PATH),
                    custom_size: Some(sprite_size),
                    ..default()
                },
                rotation_radians: furniture_rotation_radians(orientation),
            }
        }
        FurnitureType::Toilet => {
            FurnitureSpriteConfig::Rotating {
                sprite: Sprite {
                    image: asset_server.load(TOILET_SPRITE_PATH),
                    custom_size: Some(sprite_size),
                    ..default()
                },
                rotation_radians: furniture_rotation_radians(orientation),
            }
        }
        FurnitureType::Sink => {
            FurnitureSpriteConfig::Rotating {
                sprite: Sprite {
                    image: asset_server.load(SINK_SPRITE_PATH),
                    custom_size: Some(sprite_size),
                    ..default()
                },
                rotation_radians: furniture_rotation_radians(orientation),
            }
        }
        FurnitureType::Nightstand => {
            FurnitureSpriteConfig::Rotating {
                sprite: Sprite {
                    image: asset_server.load(END_TABLE_SPRITE_PATH),
                    custom_size: Some(sprite_size),
                    ..default()
                },
                rotation_radians: furniture_rotation_radians(orientation),
            }
        }
        FurnitureType::ReceptionConsole => {
            let (sprite_path, flip_x) = match orientation {
                FurnitureOrientation::East => (COMPUTER_SIDE_SPRITE_PATH, false),
                FurnitureOrientation::West => (COMPUTER_SIDE_SPRITE_PATH, true),
                FurnitureOrientation::South => (COMPUTER_FRONT_SPRITE_PATH, false),
                FurnitureOrientation::North => (COMPUTER_BACK_SPRITE_PATH, false),
            };

            let mut sprite = Sprite {
                image: asset_server.load(sprite_path),
                custom_size: Some(Vec2::splat(grid_settings.tile_size * 0.9)),
                ..default()
            };
            sprite.flip_x = flip_x;

            FurnitureSpriteConfig::Directional { sprite }
        }
        // Default fallback for furniture types without specific sprites
        _ => FurnitureSpriteConfig::Mesh {
            color: furniture_type.color(),
        },
    }
}
