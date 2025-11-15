use bevy::prelude::*;
use crate::components::*;

const WINDOW_THICKNESS: f32 = 0.2;
const DOOR_THICKNESS: f32 = 0.2;

/// Spawns a blueprint for structures (walls, windows, floors)
pub fn spawn_blueprint(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    blueprint_type: BlueprintType,
    grid_pos: IVec2,
    world_pos: Vec2,
    tile_size: f32,
) -> Entity {
    // Blueprints are translucent white (floors lighter, structures more visible)
    let (color, z_level, mesh_size) = match blueprint_type {
        BlueprintType::Wall => (
            Color::srgba(1.0, 1.0, 1.0, 0.6),  // More opaque for walls
            1.5,
            (tile_size, tile_size)  // Full square
        ),
        BlueprintType::Door(_) => (
            Color::srgba(1.0, 1.0, 1.0, 0.6),
            1.5,
            (tile_size, tile_size)  // Full square
        ),
        BlueprintType::Window => (
            Color::srgba(1.0, 1.0, 1.0, 0.6),
            1.5,
            (tile_size, tile_size * WINDOW_THICKNESS)  // Thin for windows
        ),
        BlueprintType::Floor(_) => (
            Color::srgba(1.0, 1.0, 1.0, 0.3),  // More translucent for floors
            0.5,
            (tile_size, tile_size)  // Full square
        ),
        BlueprintType::Furniture(_) => (
            Color::srgba(1.0, 1.0, 1.0, 0.6),
            2.5,
            (tile_size, tile_size)  // Full square
        ),
    };

    commands
        .spawn((
            Mesh2d(meshes.add(Rectangle::new(mesh_size.0, mesh_size.1))),
            MeshMaterial2d(materials.add(color)),
            Transform::from_xyz(world_pos.x, world_pos.y, z_level),
            Blueprint::new(blueprint_type),
            GridPosition::new(grid_pos.x, grid_pos.y),
        ))
        .id()
}

/// Spawns a blueprint specifically for doors (2x1 size)
pub fn spawn_door_blueprint(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    grid_pos: IVec2,
    center_pos: Vec2,
    tile_size: f32,
    orientation: DoorOrientation,
) -> Entity {
    let (width, height, offset) = match orientation {
        DoorOrientation::Horizontal => {
            // 2 tiles wide: shift right by half a tile to center between both tiles
            (
                tile_size * 2.0,
                tile_size * DOOR_THICKNESS,
                Vec2::new(tile_size / 2.0, 0.0),
            )
        }
        DoorOrientation::Vertical => {
            // 2 tiles tall: shift up by half a tile to center between both tiles
            (
                tile_size * DOOR_THICKNESS,
                tile_size * 2.0,
                Vec2::new(0.0, tile_size / 2.0),
            )
        }
    };

    let adjusted_pos = center_pos + offset;

    commands
        .spawn((
            Mesh2d(meshes.add(Rectangle::new(width, height))),
            MeshMaterial2d(materials.add(Color::srgba(0.4, 0.3, 0.2, 0.5))),
            Transform::from_xyz(adjusted_pos.x, adjusted_pos.y, 1.5),
            Blueprint::new(BlueprintType::Door(orientation)),
            GridPosition::new(grid_pos.x, grid_pos.y),
        ))
        .id()
}
