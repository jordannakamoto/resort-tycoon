use bevy::prelude::*;
use crate::components::furniture::*;
use crate::components::*;
use crate::systems::grid::GridSettings;
use crate::systems::building::BuildingMap;
use super::super::factories::*;

/// Places a reception console on a desk
pub fn place_reception_console(
    commands: &mut Commands,
    grid_pos: IVec2,
    orientation: FurnitureOrientation,
    grid_settings: &GridSettings,
    asset_server: &AssetServer,
) -> Entity {
    let base_world_pos = crate::systems::grid::grid_to_world(
        grid_pos,
        grid_settings.tile_size,
        grid_settings.width,
        grid_settings.height,
    );

    // Create sprite using factory function
    let sprite_config = create_furniture_sprite(
        FurnitureType::ReceptionConsole,
        orientation,
        asset_server,
        grid_settings,
        false,
    );

    let sprite = match sprite_config {
        FurnitureSpriteConfig::Directional { sprite } => sprite,
        _ => panic!("Reception console should use directional sprite"),
    };

    let furniture_entity = commands.spawn((
        sprite,
        Transform::from_xyz(base_world_pos.x, base_world_pos.y, 3.5),
        GridPosition::new(grid_pos.x, grid_pos.y),
        Furniture,
    )).id();

    // Insert components using factory function
    insert_furniture_component(furniture_entity, FurnitureType::ReceptionConsole, orientation, commands);

    furniture_entity
}

/// Places regular furniture (beds, dressers, etc.)
pub fn place_regular_furniture(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    furniture_type: FurnitureType,
    grid_pos: IVec2,
    orientation: FurnitureOrientation,
    grid_settings: &GridSettings,
    asset_server: &AssetServer,
    building_map: &mut BuildingMap,
) -> Entity {
    let furniture_tiles = furniture_type.tiles_occupied(grid_pos, orientation);

    // Calculate center position for multi-tile furniture
    let (width_tiles, height_tiles) = furniture_type.oriented_dimensions(orientation);
    let offset = Vec2::new(
        (width_tiles as f32 - 1.0) * grid_settings.tile_size / 2.0,
        (height_tiles as f32 - 1.0) * grid_settings.tile_size / 2.0,
    );

    let base_world_pos = crate::systems::grid::grid_to_world(
        grid_pos,
        grid_settings.tile_size,
        grid_settings.width,
        grid_settings.height,
    );
    let furniture_pos = base_world_pos + offset;

    // Create sprite using factory function
    let sprite_config = create_furniture_sprite(
        furniture_type,
        orientation,
        asset_server,
        grid_settings,
        false,
    );

    // Spawn furniture entity based on sprite config
    let furniture_entity = match sprite_config {
        FurnitureSpriteConfig::Rotating { sprite, rotation_radians } => {
            let mut transform = Transform::from_xyz(furniture_pos.x, furniture_pos.y, 3.0);
            transform.rotate_z(rotation_radians);

            commands
                .spawn((
                    sprite,
                    transform,
                    GridPosition::new(grid_pos.x, grid_pos.y),
                    Furniture,
                ))
                .id()
        }
        FurnitureSpriteConfig::Directional { sprite } => {
            let transform = Transform::from_xyz(furniture_pos.x, furniture_pos.y, 3.0);

            commands
                .spawn((
                    sprite,
                    transform,
                    GridPosition::new(grid_pos.x, grid_pos.y),
                    Furniture,
                ))
                .id()
        }
        FurnitureSpriteConfig::Mesh { color } => {
            let (base_width_tiles, base_height_tiles) = furniture_type.base_dimensions();
            let rotation_radians = furniture_rotation_radians(orientation);
            let mut transform = Transform::from_xyz(furniture_pos.x, furniture_pos.y, 3.0);
            transform.rotate_z(rotation_radians);

            commands
                .spawn((
                    Mesh2d(meshes.add(Rectangle::new(
                        base_width_tiles as f32 * grid_settings.tile_size,
                        base_height_tiles as f32 * grid_settings.tile_size,
                    ))),
                    MeshMaterial2d(materials.add(color)),
                    transform,
                    GridPosition::new(grid_pos.x, grid_pos.y),
                    Furniture,
                ))
                .id()
        }
    };

    // Insert components using factory function
    insert_furniture_component(furniture_entity, furniture_type, orientation, commands);

    // Mark tiles as occupied (furniture blocks placement but not movement)
    for tile_pos in furniture_tiles {
        building_map.occupied.insert(tile_pos);
    }

    furniture_entity
}
