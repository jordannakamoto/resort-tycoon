use bevy::prelude::*;
use crate::components::*;
use crate::systems::grid::{GridSettings, grid_to_world};
use super::super::BuildingMap;

/// Shows preview for door placement (2x1 tiles)
pub fn show_door_preview(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    grid_pos: IVec2,
    orientation: DoorOrientation,
    grid_settings: &GridSettings,
    building_map: &BuildingMap,
) {
    let door_tiles = match orientation {
        DoorOrientation::Horizontal => {
            vec![grid_pos, grid_pos + IVec2::new(1, 0)]
        }
        DoorOrientation::Vertical => {
            vec![grid_pos, grid_pos + IVec2::new(0, 1)]
        }
    };

    for tile_pos in door_tiles {
        let tile_world_pos = grid_to_world(
            tile_pos,
            grid_settings.tile_size,
            grid_settings.width,
            grid_settings.height,
        );

        let is_blocked = building_map.occupied.contains(&tile_pos)
            || building_map.doors.contains_key(&tile_pos);
        let color = if is_blocked {
            Color::srgba(1.0, 0.3, 0.3, 0.5)  // Red for blocked
        } else {
            Color::srgba(1.0, 1.0, 1.0, 0.5)  // White for valid
        };

        commands.spawn((
            Mesh2d(meshes.add(Rectangle::new(
                grid_settings.tile_size,
                grid_settings.tile_size,
            ))),
            MeshMaterial2d(materials.add(color)),
            Transform::from_xyz(tile_world_pos.x, tile_world_pos.y, 1.0),
            PlacementPreview,
        ));
    }
}

/// Shows preview for single-tile structures (walls, windows)
pub fn show_single_tile_preview(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    grid_pos: IVec2,
    grid_settings: &GridSettings,
    building_map: &BuildingMap,
) {
    let world_pos = grid_to_world(
        grid_pos,
        grid_settings.tile_size,
        grid_settings.width,
        grid_settings.height,
    );

    let is_occupied = building_map.occupied.contains(&grid_pos);
    let color = if is_occupied {
        Color::srgba(1.0, 0.3, 0.3, 0.5)  // Red for blocked
    } else {
        Color::srgba(1.0, 1.0, 1.0, 0.5)  // White for valid
    };

    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(
            grid_settings.tile_size,
            grid_settings.tile_size,
        ))),
        MeshMaterial2d(materials.add(color)),
        Transform::from_xyz(world_pos.x, world_pos.y, 1.0),
        PlacementPreview,
    ));
}

/// Shows preview for drag area (walls or floors)
pub fn show_drag_area_preview(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    positions: Vec<IVec2>,
    grid_settings: &GridSettings,
    building_map: &BuildingMap,
    is_floor: bool,
) {
    for grid_pos in positions {
        let world_pos = grid_to_world(
            grid_pos,
            grid_settings.tile_size,
            grid_settings.width,
            grid_settings.height,
        );

        // For floors, check if structure is blocking; for structures, check if occupied
        let is_blocked = if is_floor {
            building_map.occupied.contains(&grid_pos)
        } else {
            building_map.occupied.contains(&grid_pos)
        };

        let color = if is_blocked {
            Color::srgba(1.0, 0.3, 0.3, 0.5)  // Red for blocked
        } else {
            Color::srgba(1.0, 1.0, 1.0, 0.5)  // White for valid
        };

        commands.spawn((
            Mesh2d(meshes.add(Rectangle::new(
                grid_settings.tile_size,
                grid_settings.tile_size,
            ))),
            MeshMaterial2d(materials.add(color)),
            Transform::from_xyz(world_pos.x, world_pos.y, 1.0),
            PlacementPreview,
        ));
    }
}
