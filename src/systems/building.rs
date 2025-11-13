use bevy::prelude::*;
use bevy::sprite::*;
use bevy::window::{PrimaryWindow, Window as BevyWindow};
use crate::components::*;
use crate::systems::grid::*;
use crate::ui::{ToolbarState, BuildingType};

#[derive(Resource)]
pub struct BuildingMap {
    pub occupied: std::collections::HashSet<IVec2>,
}

impl Default for BuildingMap {
    fn default() -> Self {
        Self {
            occupied: std::collections::HashSet::new(),
        }
    }
}

pub struct BuildingPlugin;

impl Plugin for BuildingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BuildingMap>()
            .add_systems(Update, (
                update_placement_preview,
                handle_building_placement,
            ).chain());
    }
}

fn update_placement_preview(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    toolbar_state: Res<ToolbarState>,
    grid_settings: Res<GridSettings>,
    window_query: Query<&BevyWindow, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    preview_query: Query<Entity, With<PlacementPreview>>,
    building_map: Res<BuildingMap>,
) {
    let window = window_query.single();
    let (camera, camera_transform) = camera_query.single();

    // Remove old preview
    for entity in &preview_query {
        commands.entity(entity).despawn();
    }

    // Only show preview if a building is selected
    if let Some(building_type) = toolbar_state.selected_building {
        if let Some(cursor_pos) = window.cursor_position() {
            if let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) {
                if let Some(grid_pos) = world_to_grid(
                    world_pos,
                    grid_settings.tile_size,
                    grid_settings.width,
                    grid_settings.height,
                ) {
                    let world_pos = grid_to_world(
                        grid_pos,
                        grid_settings.tile_size,
                        grid_settings.width,
                        grid_settings.height,
                    );

                    // Check if position is occupied
                    let is_occupied = building_map.occupied.contains(&grid_pos);
                    let color = if is_occupied {
                        Color::srgba(1.0, 0.3, 0.3, 0.5) // Red if occupied
                    } else {
                        Color::srgba(0.3, 1.0, 0.3, 0.5) // Green if free
                    };

                    // Spawn preview based on building type
                    match building_type {
                        BuildingType::Wall => {
                            commands.spawn((
                                Mesh2d(meshes.add(Rectangle::new(grid_settings.tile_size, grid_settings.tile_size))),
                                MeshMaterial2d(materials.add(color)),
                                Transform::from_xyz(world_pos.x, world_pos.y, 1.0),
                                PlacementPreview,
                            ));
                        }
                        BuildingType::Door | BuildingType::Window => {
                            // Similar preview for doors and windows
                            commands.spawn((
                                Mesh2d(meshes.add(Rectangle::new(grid_settings.tile_size, grid_settings.tile_size))),
                                MeshMaterial2d(materials.add(color)),
                                Transform::from_xyz(world_pos.x, world_pos.y, 1.0),
                                PlacementPreview,
                            ));
                        }
                    }
                }
            }
        }
    }
}

fn handle_building_placement(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    toolbar_state: Res<ToolbarState>,
    grid_settings: Res<GridSettings>,
    window_query: Query<&BevyWindow, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut building_map: ResMut<BuildingMap>,
) {
    if !mouse_button.just_pressed(MouseButton::Left) {
        return;
    }

    if let Some(building_type) = toolbar_state.selected_building {
        let window = window_query.single();
        let (camera, camera_transform) = camera_query.single();

        if let Some(cursor_pos) = window.cursor_position() {
            if let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) {
                if let Some(grid_pos) = world_to_grid(
                    world_pos,
                    grid_settings.tile_size,
                    grid_settings.width,
                    grid_settings.height,
                ) {
                    // Check if position is already occupied
                    if building_map.occupied.contains(&grid_pos) {
                        return;
                    }

                    let world_pos = grid_to_world(
                        grid_pos,
                        grid_settings.tile_size,
                        grid_settings.width,
                        grid_settings.height,
                    );

                    // Place blueprint and create construction job
                    let blueprint_type = match building_type {
                        BuildingType::Wall => BlueprintType::Wall,
                        BuildingType::Door => BlueprintType::Door,
                        BuildingType::Window => BlueprintType::Window,
                    };

                    let blueprint_entity = spawn_blueprint(
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        blueprint_type,
                        grid_pos,
                        world_pos,
                        grid_settings.tile_size,
                    );

                    // Create construction job for this blueprint
                    commands.spawn(ConstructionJob::new(blueprint_entity));

                    // Mark position as occupied
                    building_map.occupied.insert(grid_pos);
                }
            }
        }
    }
}

fn spawn_blueprint(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    blueprint_type: BlueprintType,
    grid_pos: IVec2,
    world_pos: Vec2,
    tile_size: f32,
) -> Entity {
    // Blueprints are semi-transparent versions of the final building
    let color = match blueprint_type {
        BlueprintType::Wall => Color::srgba(0.5, 0.5, 0.5, 0.5),
        BlueprintType::Door => Color::srgba(0.4, 0.3, 0.2, 0.5),
        BlueprintType::Window => Color::srgba(0.6, 0.8, 1.0, 0.5),
    };

    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(tile_size, tile_size))),
        MeshMaterial2d(materials.add(color)),
        Transform::from_xyz(world_pos.x, world_pos.y, 1.5),
        Blueprint::new(blueprint_type),
        GridPosition::new(grid_pos.x, grid_pos.y),
    ))
    .id()
}
