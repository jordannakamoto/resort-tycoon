use std::fs;
use std::path::Path;

use bevy::prelude::*;
use bevy::sprite::*;
use serde::{Deserialize, Serialize};

use crate::components::*;
use crate::systems::grid::{grid_to_world, GridSettings};
use crate::systems::BuildingMap;

const DOOR_THICKNESS: f32 = 0.6;

#[derive(Resource)]
pub struct SaveLoadConfig {
    pub path: String,
}

impl Default for SaveLoadConfig {
    fn default() -> Self {
        Self {
            path: "assets/saves/test_room.json".to_string(),
        }
    }
}

#[derive(Resource)]
struct LoadRequestState {
    pending: bool,
}

impl Default for LoadRequestState {
    fn default() -> Self {
        Self { pending: true }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
struct GridPoint {
    x: i32,
    y: i32,
}

impl From<GridPoint> for IVec2 {
    fn from(value: GridPoint) -> Self {
        IVec2::new(value.x, value.y)
    }
}

impl From<&GridPosition> for GridPoint {
    fn from(value: &GridPosition) -> Self {
        GridPoint {
            x: value.x,
            y: value.y,
        }
    }
}

impl From<IVec2> for GridPoint {
    fn from(value: IVec2) -> Self {
        GridPoint {
            x: value.x,
            y: value.y,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DoorData {
    position: GridPoint,
    orientation: DoorOrientation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FloorData {
    position: GridPoint,
    floor_type: FloorType,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct SaveData {
    walls: Vec<GridPoint>,
    floors: Vec<FloorData>,
    doors: Vec<DoorData>,
}

pub struct SaveLoadPlugin;

impl Plugin for SaveLoadPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SaveLoadConfig>()
            .init_resource::<LoadRequestState>()
            .add_systems(Update, request_load_on_hotkey)
            .add_systems(Update, save_game_on_hotkey)
            .add_systems(
                Update,
                process_load_requests.after(request_load_on_hotkey),
            );
    }
}

fn request_load_on_hotkey(
    keys: Res<ButtonInput<KeyCode>>,
    mut load_state: ResMut<LoadRequestState>,
) {
    if keys.just_pressed(KeyCode::KeyL) {
        load_state.pending = true;
    }
}

fn save_game_on_hotkey(
    keys: Res<ButtonInput<KeyCode>>,
    config: Res<SaveLoadConfig>,
    wall_query: Query<&GridPosition, With<Wall>>,
    floor_query: Query<(&GridPosition, &Floor)>,
    door_query: Query<(&GridPosition, &Door)>,
) {
    if !keys.just_pressed(KeyCode::KeyP) {
        return;
    }

    let mut data = collect_save_data(&wall_query, &floor_query, &door_query);
    sort_save_data(&mut data);

    if let Err(err) = write_save_file(&config.path, &data) {
        error!("Failed to save map to {}: {}", config.path, err);
    } else {
        info!("Saved map to {}", config.path);
    }
}

fn process_load_requests(
    mut commands: Commands,
    mut load_state: ResMut<LoadRequestState>,
    config: Res<SaveLoadConfig>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    grid_settings: Res<GridSettings>,
    mut building_map: ResMut<BuildingMap>,
    wall_query: Query<Entity, With<Wall>>,
    floor_query: Query<Entity, With<Floor>>,
    door_query: Query<Entity, With<Door>>,
    blueprint_query: Query<Entity, With<Blueprint>>,
    construction_job_query: Query<Entity, With<ConstructionJob>>,
    deconstruction_job_query: Query<Entity, With<DeconstructionJob>>,
    marker_query: Query<Entity, With<DeconstructionMarker>>,
) {
    if !load_state.pending {
        return;
    }

    load_state.pending = false;

    let (data, source) = read_or_create_save_file(&config.path);
    clear_structures(
        &mut commands,
        &wall_query,
        &floor_query,
        &door_query,
        &blueprint_query,
        &construction_job_query,
        &deconstruction_job_query,
        &marker_query,
    );
    apply_save_data(
        &mut commands,
        &mut meshes,
        &mut materials,
        &grid_settings,
        &mut building_map,
        &data,
    );

    info!(
        "Loaded room from {} (walls: {}, floors: {}, doors: {})",
        source,
        data.walls.len(),
        data.floors.len(),
        data.doors.len()
    );
}

fn collect_save_data(
    wall_query: &Query<&GridPosition, With<Wall>>,
    floor_query: &Query<(&GridPosition, &Floor)>,
    door_query: &Query<(&GridPosition, &Door)>,
) -> SaveData {
    let mut data = SaveData::default();

    for pos in wall_query {
        data.walls.push(GridPoint::from(pos));
    }

    for (pos, floor) in floor_query {
        data.floors.push(FloorData {
            position: GridPoint::from(pos),
            floor_type: floor.floor_type,
        });
    }

    for (pos, door) in door_query {
        data.doors.push(DoorData {
            position: GridPoint::from(pos),
            orientation: door.orientation,
        });
    }

    data
}

fn sort_save_data(data: &mut SaveData) {
    data.walls.sort();
    data.floors.sort_by_key(|entry| (entry.position.x, entry.position.y));
    data.doors.sort_by_key(|entry| (entry.position.x, entry.position.y));
}

fn read_or_create_save_file(path: &str) -> (SaveData, String) {
    match fs::read_to_string(path) {
        Ok(contents) => match serde_json::from_str(&contents) {
            Ok(data) => (data, path.to_string()),
            Err(err) => {
                error!("Failed to parse {}: {}. Using default room.", path, err);
                let default = default_room_layout();
                let _ = write_save_file(path, &default);
                (default, "built-in default".to_string())
            }
        },
        Err(_) => {
            let default = default_room_layout();
            let _ = write_save_file(path, &default);
            (default, "built-in default".to_string())
        }
    }
}

fn write_save_file(path: &str, data: &SaveData) -> std::io::Result<()> {
    if let Some(parent) = Path::new(path).parent() {
        fs::create_dir_all(parent)?;
    }
    let serialized = serde_json::to_string_pretty(data).expect("save data serialization");
    fs::write(path, serialized)
}

fn clear_structures(
    commands: &mut Commands,
    wall_query: &Query<Entity, With<Wall>>,
    floor_query: &Query<Entity, With<Floor>>,
    door_query: &Query<Entity, With<Door>>,
    blueprint_query: &Query<Entity, With<Blueprint>>,
    construction_job_query: &Query<Entity, With<ConstructionJob>>,
    deconstruction_job_query: &Query<Entity, With<DeconstructionJob>>,
    marker_query: &Query<Entity, With<DeconstructionMarker>>,
) {
    for entity in wall_query {
        commands.entity(entity).despawn_recursive();
    }
    for entity in floor_query {
        commands.entity(entity).despawn_recursive();
    }
    for entity in door_query {
        commands.entity(entity).despawn_recursive();
    }
    for entity in blueprint_query {
        commands.entity(entity).despawn_recursive();
    }
    for entity in construction_job_query {
        commands.entity(entity).despawn_recursive();
    }
    for entity in deconstruction_job_query {
        commands.entity(entity).despawn_recursive();
    }
    for entity in marker_query {
        commands.entity(entity).despawn_recursive();
    }
}

fn apply_save_data(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    grid_settings: &GridSettings,
    building_map: &mut BuildingMap,
    data: &SaveData,
) {
    *building_map = BuildingMap::default();

    for floor in &data.floors {
        spawn_floor(commands, meshes, materials, grid_settings, building_map, floor);
    }

    for wall in &data.walls {
        spawn_wall(commands, meshes, materials, grid_settings, building_map, *wall);
    }

    for door in &data.doors {
        spawn_door(
            commands,
            meshes,
            materials,
            grid_settings,
            building_map,
            door,
        );
    }
}

fn spawn_floor(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    grid_settings: &GridSettings,
    building_map: &mut BuildingMap,
    floor: &FloorData,
) {
    let pos = IVec2::from(floor.position);
    let world_pos = grid_to_world(
        pos,
        grid_settings.tile_size,
        grid_settings.width,
        grid_settings.height,
    );

    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(
            grid_settings.tile_size,
            grid_settings.tile_size,
        ))),
        MeshMaterial2d(materials.add(floor.floor_type.color())),
        Transform::from_xyz(world_pos.x, world_pos.y, 0.5),
        Floor {
            floor_type: floor.floor_type,
        },
        GridPosition::new(pos.x, pos.y),
    ));

    building_map.floors.insert(pos);
}

fn spawn_wall(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    grid_settings: &GridSettings,
    building_map: &mut BuildingMap,
    wall_point: GridPoint,
) {
    let pos = IVec2::from(wall_point);
    let world_pos = grid_to_world(
        pos,
        grid_settings.tile_size,
        grid_settings.width,
        grid_settings.height,
    );

    let wall_entity = commands
        .spawn((
            Mesh2d(meshes.add(Rectangle::new(
                grid_settings.tile_size,
                grid_settings.tile_size,
            ))),
            MeshMaterial2d(materials.add(WallMaterial::Stone.color())),
            Transform::from_xyz(world_pos.x, world_pos.y, 2.0),
            Wall,
            Building,
            GridPosition::new(pos.x, pos.y),
        ))
        .id();

    building_map.occupied.insert(pos);
    building_map.walls.insert(pos, wall_entity);
}

fn spawn_door(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    grid_settings: &GridSettings,
    building_map: &mut BuildingMap,
    door_data: &DoorData,
) {
    let pos = IVec2::from(door_data.position);
    let door = Door::new(door_data.orientation);
    let tiles = door.tiles_occupied(pos);

    let (width, height, offset) = match door_data.orientation {
        DoorOrientation::Horizontal => (
            grid_settings.tile_size * 2.0,
            grid_settings.tile_size * DOOR_THICKNESS,
            Vec2::new(grid_settings.tile_size / 2.0, 0.0),
        ),
        DoorOrientation::Vertical => (
            grid_settings.tile_size * DOOR_THICKNESS,
            grid_settings.tile_size * 2.0,
            Vec2::new(0.0, grid_settings.tile_size / 2.0),
        ),
    };

    let base_world = grid_to_world(
        pos,
        grid_settings.tile_size,
        grid_settings.width,
        grid_settings.height,
    );
    let adjusted_pos = base_world + offset;

    let door_entity = commands
        .spawn((
            Mesh2d(meshes.add(Rectangle::new(width, height))),
            MeshMaterial2d(materials.add(Color::srgb(0.4, 0.3, 0.2))),
            Transform::from_xyz(adjusted_pos.x, adjusted_pos.y, 2.0),
            Door::new(door_data.orientation),
            Building,
            GridPosition::new(pos.x, pos.y),
        ))
        .id();

    for tile in tiles {
        building_map.doors.insert(tile, door_entity);
    }
}

fn default_room_layout() -> SaveData {
    let mut data = SaveData::default();

    let min = 48;
    let max = 52;
    let inner_min = min + 1;
    let inner_max = max - 1;

    for x in inner_min..=inner_max {
        for y in inner_min..=inner_max {
            data.floors.push(FloorData {
                position: GridPoint { x, y },
                floor_type: FloorType::Wood,
            });
        }
    }

    for x in min..=max {
        if x != 49 && x != 50 {
            data.walls.push(GridPoint { x, y: min });
        }
        data.walls.push(GridPoint { x, y: max });
    }

    for y in inner_min..=inner_max {
        data.walls.push(GridPoint { x: min, y });
        data.walls.push(GridPoint { x: max, y });
    }

    data.doors.push(DoorData {
        position: GridPoint { x: 49, y: min },
        orientation: DoorOrientation::Horizontal,
    });

    sort_save_data(&mut data);
    data
}
