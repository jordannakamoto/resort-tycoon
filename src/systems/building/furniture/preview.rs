use bevy::prelude::*;
use crate::components::furniture::*;
use crate::components::*;
use crate::systems::grid::{GridSettings, grid_to_world};
use super::super::BuildingMap;
use super::super::factories::*;

/// Shows preview for reception console (special case - must be on desk)
pub fn show_reception_console_preview(
    commands: &mut Commands,
    grid_pos: IVec2,
    orientation: FurnitureOrientation,
    grid_settings: &GridSettings,
    building_map: &BuildingMap,
    asset_server: &AssetServer,
    desk_query: &Query<&GridPosition, With<Desk>>,
) {
    // Validate placement
    let is_valid = validate_furniture_placement(
        FurnitureType::ReceptionConsole,
        grid_pos,
        orientation,
        building_map,
        Some(desk_query),
    );

    let preview_color = if !is_valid {
        Color::srgba(1.0, 0.3, 0.3, 1.0)  // Red if no desk
    } else {
        Color::srgba(1.0, 1.0, 1.0, 0.7)  // White if desk present, preserves sprite alpha
    };

    let world_pos = grid_to_world(
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
        true,
    );

    let mut sprite = match sprite_config {
        FurnitureSpriteConfig::Directional { sprite } => sprite,
        _ => panic!("Reception console should use directional sprite"),
    };
    sprite.color = preview_color;

    // Use higher z-level so it appears above desk
    commands.spawn((
        sprite,
        Transform::from_xyz(world_pos.x, world_pos.y, 4.0),
        PlacementPreview,
    ));
}

/// Shows preview for regular furniture
pub fn show_regular_furniture_preview(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    furniture_type: FurnitureType,
    grid_pos: IVec2,
    orientation: FurnitureOrientation,
    grid_settings: &GridSettings,
    building_map: &BuildingMap,
    asset_server: &AssetServer,
) {
    // Validate placement
    let is_blocked = !validate_furniture_placement(
        furniture_type,
        grid_pos,
        orientation,
        building_map,
        None,
    );

    // Calculate center position for preview
    let (width_tiles, height_tiles) = furniture_type.oriented_dimensions(orientation);
    let offset = Vec2::new(
        (width_tiles as f32 - 1.0) * grid_settings.tile_size / 2.0,
        (height_tiles as f32 - 1.0) * grid_settings.tile_size / 2.0,
    );

    let base_world_pos = grid_to_world(
        grid_pos,
        grid_settings.tile_size,
        grid_settings.width,
        grid_settings.height,
    );
    let preview_pos = base_world_pos + offset;

    // Create sprite using factory function
    let sprite_config = create_furniture_sprite(
        furniture_type,
        orientation,
        asset_server,
        grid_settings,
        true,
    );

    // Apply color tint based on placement validity
    let preview_color = if is_blocked {
        Color::srgba(1.0, 0.3, 0.3, 1.0)  // Red for blocked
    } else {
        Color::srgba(1.0, 1.0, 1.0, 0.7)  // White for valid, preserves sprite alpha
    };

    // Spawn preview based on sprite config
    match sprite_config {
        FurnitureSpriteConfig::Rotating { mut sprite, rotation_radians } => {
            sprite.color = preview_color;
            let mut transform = Transform::from_xyz(preview_pos.x, preview_pos.y, 4.0);
            transform.rotate_z(rotation_radians);

            commands.spawn((
                sprite,
                transform,
                PlacementPreview,
            ));
        }
        FurnitureSpriteConfig::Directional { mut sprite } => {
            sprite.color = preview_color;
            let transform = Transform::from_xyz(preview_pos.x, preview_pos.y, 4.0);

            commands.spawn((
                sprite,
                transform,
                PlacementPreview,
            ));
        }
        FurnitureSpriteConfig::Mesh { color: _ } => {
            // For mesh-based furniture, use semi-transparent color
            let mesh_color = if is_blocked {
                Color::srgba(1.0, 0.3, 0.3, 0.5)  // Red for blocked
            } else {
                Color::srgba(1.0, 1.0, 1.0, 0.5)  // White for valid
            };

            let (base_width_tiles, base_height_tiles) = furniture_type.base_dimensions();
            let rotation_radians = furniture_rotation_radians(orientation);
            let mut transform = Transform::from_xyz(preview_pos.x, preview_pos.y, 4.0);
            transform.rotate_z(rotation_radians);

            commands.spawn((
                Mesh2d(meshes.add(Rectangle::new(
                    base_width_tiles as f32 * grid_settings.tile_size,
                    base_height_tiles as f32 * grid_settings.tile_size,
                ))),
                MeshMaterial2d(materials.add(mesh_color)),
                transform,
                PlacementPreview,
            ));
        }
    }
}
