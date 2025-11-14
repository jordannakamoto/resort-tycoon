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
            path: "assets/saves/test-room.json".to_string(),
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

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FurnitureData {
    position: GridPoint,
    furniture_type: FurnitureType,
    orientation: FurnitureOrientation,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SaveData {
    pub walls: Vec<GridPoint>,
    pub floors: Vec<FloorData>,
    pub doors: Vec<DoorData>,
    #[serde(default)]
    pub furniture: Vec<FurnitureData>,
}

pub struct SaveLoadPlugin;

impl Plugin for SaveLoadPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SaveLoadConfig>()
            .init_resource::<LoadRequestState>()
            .add_systems(Update, request_load_on_hotkey)
            .add_systems(Update, save_game_on_hotkey)
            .add_systems(Update, process_load_requests.after(request_load_on_hotkey));
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
    furniture_query: Query<(
        &GridPosition,
        &Furniture,
        &FurnitureType,
        &FurnitureOrientation,
    )>,
) {
    if !keys.just_pressed(KeyCode::KeyP) {
        return;
    }

    let mut data = collect_save_data(&wall_query, &floor_query, &door_query, &furniture_query);
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
    asset_server: Res<AssetServer>,
    grid_settings: Res<GridSettings>,
    mut building_map: ResMut<BuildingMap>,
    wall_query: Query<Entity, With<Wall>>,
    floor_query: Query<Entity, With<Floor>>,
    door_query: Query<Entity, With<Door>>,
    furniture_query: Query<Entity, With<Furniture>>,
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
        &furniture_query,
        &blueprint_query,
        &construction_job_query,
        &deconstruction_job_query,
        &marker_query,
    );
    apply_save_data(
        &mut commands,
        &mut meshes,
        &mut materials,
        &asset_server,
        &grid_settings,
        &mut building_map,
        &data,
    );

    info!(
        "Loaded room from {} (walls: {}, floors: {}, doors: {}, furniture: {})",
        source,
        data.walls.len(),
        data.floors.len(),
        data.doors.len(),
        data.furniture.len()
    );
}

pub fn collect_save_data(
    wall_query: &Query<&GridPosition, With<Wall>>,
    floor_query: &Query<(&GridPosition, &Floor)>,
    door_query: &Query<(&GridPosition, &Door)>,
    furniture_query: &Query<(
        &GridPosition,
        &Furniture,
        &FurnitureType,
        &FurnitureOrientation,
    )>,
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

    for (pos, _furniture_marker, furniture_type, orientation) in furniture_query {
        data.furniture.push(FurnitureData {
            position: GridPoint::from(pos),
            furniture_type: *furniture_type,
            orientation: *orientation,
        });
    }

    data
}

pub fn sort_save_data(data: &mut SaveData) {
    data.walls.sort();
    data.floors
        .sort_by_key(|entry| (entry.position.x, entry.position.y));
    data.doors
        .sort_by_key(|entry| (entry.position.x, entry.position.y));
    data.furniture
        .sort_by_key(|entry| (entry.position.x, entry.position.y));
}

pub fn read_or_create_save_file(path: &str) -> (SaveData, String) {
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

pub fn write_save_file(path: &str, data: &SaveData) -> std::io::Result<()> {
    if let Some(parent) = Path::new(path).parent() {
        fs::create_dir_all(parent)?;
    }
    let serialized = serde_json::to_string_pretty(data).expect("save data serialization");
    fs::write(path, serialized)
}

pub fn clear_structures(
    commands: &mut Commands,
    wall_query: &Query<Entity, With<Wall>>,
    floor_query: &Query<Entity, With<Floor>>,
    door_query: &Query<Entity, With<Door>>,
    furniture_query: &Query<Entity, With<Furniture>>,
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
    for entity in furniture_query {
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

pub fn apply_save_data(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    asset_server: &AssetServer,
    grid_settings: &GridSettings,
    building_map: &mut BuildingMap,
    data: &SaveData,
) {
    *building_map = BuildingMap::default();

    for floor in &data.floors {
        spawn_floor(
            commands,
            meshes,
            materials,
            grid_settings,
            building_map,
            floor,
        );
    }

    for wall in &data.walls {
        spawn_wall(
            commands,
            meshes,
            materials,
            grid_settings,
            building_map,
            *wall,
        );
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

    for furniture in &data.furniture {
        spawn_furniture(
            commands,
            meshes,
            materials,
            asset_server,
            grid_settings,
            building_map,
            furniture,
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

fn spawn_furniture(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    asset_server: &AssetServer,
    grid_settings: &GridSettings,
    building_map: &mut BuildingMap,
    furniture_data: &FurnitureData,
) {
    let pos = IVec2::from(furniture_data.position);
    let furniture_type = furniture_data.furniture_type;
    let orientation = furniture_data.orientation;

    let furniture_tiles = furniture_type.tiles_occupied(pos, orientation);
    let (width_tiles, height_tiles) = furniture_type.oriented_dimensions(orientation);
    let width_mult = width_tiles as f32;
    let height_mult = height_tiles as f32;

    // Calculate center position for multi-tile furniture
    let offset = Vec2::new(
        (width_mult - 1.0) * grid_settings.tile_size / 2.0,
        (height_mult - 1.0) * grid_settings.tile_size / 2.0,
    );

    let base_world_pos = grid_to_world(
        pos,
        grid_settings.tile_size,
        grid_settings.width,
        grid_settings.height,
    );
    let furniture_pos = base_world_pos + offset;

    // Calculate rotation
    let rotation_radians = match orientation {
        FurnitureOrientation::East => 0.0,
        FurnitureOrientation::South => std::f32::consts::PI / 2.0,
        FurnitureOrientation::West => std::f32::consts::PI,
        FurnitureOrientation::North => -std::f32::consts::PI / 2.0,
    };

    // Sprite paths (matching building.rs constants)
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

    // Spawn furniture entity based on type
    let furniture_entity = match furniture_type {
        FurnitureType::Bed(bed_type) => {
            let (base_width_tiles, base_height_tiles) = furniture_type.base_dimensions();
            let sprite_size = Vec2::new(
                base_width_tiles as f32 * grid_settings.tile_size,
                base_height_tiles as f32 * grid_settings.tile_size,
            );

            let sprite_path = match bed_type {
                BedType::Single => SINGLE_BED_SPRITE_PATH,
                BedType::Double => DOUBLE_BED_SPRITE_PATH,
            };

            let mut transform = Transform::from_xyz(furniture_pos.x, furniture_pos.y, 3.0);
            transform.rotate_z(rotation_radians);

            commands
                .spawn((
                    Sprite {
                        image: asset_server.load(sprite_path),
                        custom_size: Some(sprite_size),
                        ..default()
                    },
                    transform,
                    GridPosition::new(pos.x, pos.y),
                    Furniture,
                    furniture_type,
                    orientation,
                ))
                .id()
        }
        FurnitureType::Dresser => {
            let (base_width_tiles, base_height_tiles) = furniture_type.base_dimensions();
            let sprite_size = Vec2::new(
                base_width_tiles as f32 * grid_settings.tile_size,
                base_height_tiles as f32 * grid_settings.tile_size,
            );
            let mut transform = Transform::from_xyz(furniture_pos.x, furniture_pos.y, 3.0);

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

            commands
                .spawn((
                    sprite,
                    transform,
                    GridPosition::new(pos.x, pos.y),
                    Furniture,
                    furniture_type,
                    orientation,
                ))
                .id()
        }
        FurnitureType::Tub => {
            let (base_width_tiles, base_height_tiles) = furniture_type.base_dimensions();
            let sprite_size = Vec2::new(
                base_width_tiles as f32 * grid_settings.tile_size,
                base_height_tiles as f32 * grid_settings.tile_size,
            );
            let mut transform = Transform::from_xyz(furniture_pos.x, furniture_pos.y, 3.0);
            transform.rotate_z(rotation_radians);

            commands
                .spawn((
                    Sprite {
                        image: asset_server.load(TUB_SPRITE_PATH),
                        custom_size: Some(sprite_size),
                        ..default()
                    },
                    transform,
                    GridPosition::new(pos.x, pos.y),
                    Furniture,
                    furniture_type,
                    orientation,
                ))
                .id()
        }
        FurnitureType::Toilet => {
            let (base_width_tiles, base_height_tiles) = furniture_type.base_dimensions();
            let sprite_size = Vec2::new(
                base_width_tiles as f32 * grid_settings.tile_size,
                base_height_tiles as f32 * grid_settings.tile_size,
            );
            let mut transform = Transform::from_xyz(furniture_pos.x, furniture_pos.y, 3.0);
            transform.rotate_z(rotation_radians);

            commands
                .spawn((
                    Sprite {
                        image: asset_server.load(TOILET_SPRITE_PATH),
                        custom_size: Some(sprite_size),
                        ..default()
                    },
                    transform,
                    GridPosition::new(pos.x, pos.y),
                    Furniture,
                    furniture_type,
                    orientation,
                ))
                .id()
        }
        FurnitureType::Sink => {
            let (base_width_tiles, base_height_tiles) = furniture_type.base_dimensions();
            let sprite_size = Vec2::new(
                base_width_tiles as f32 * grid_settings.tile_size,
                base_height_tiles as f32 * grid_settings.tile_size,
            );
            let mut transform = Transform::from_xyz(furniture_pos.x, furniture_pos.y, 3.0);
            transform.rotate_z(rotation_radians);

            commands
                .spawn((
                    Sprite {
                        image: asset_server.load(SINK_SPRITE_PATH),
                        custom_size: Some(sprite_size),
                        ..default()
                    },
                    transform,
                    GridPosition::new(pos.x, pos.y),
                    Furniture,
                    furniture_type,
                    orientation,
                ))
                .id()
        }
        FurnitureType::Nightstand => {
            let (base_width_tiles, base_height_tiles) = furniture_type.base_dimensions();
            let sprite_size = Vec2::new(
                base_width_tiles as f32 * grid_settings.tile_size,
                base_height_tiles as f32 * grid_settings.tile_size,
            );
            let mut transform = Transform::from_xyz(furniture_pos.x, furniture_pos.y, 3.0);
            transform.rotate_z(rotation_radians);

            commands
                .spawn((
                    Sprite {
                        image: asset_server.load(END_TABLE_SPRITE_PATH),
                        custom_size: Some(sprite_size),
                        ..default()
                    },
                    transform,
                    GridPosition::new(pos.x, pos.y),
                    Furniture,
                    furniture_type,
                    orientation,
                ))
                .id()
        }
        FurnitureType::ReceptionConsole => {
            let base_world_pos = grid_to_world(
                pos,
                grid_settings.tile_size,
                grid_settings.width,
                grid_settings.height,
            );

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

            commands
                .spawn((
                    sprite,
                    Transform::from_xyz(base_world_pos.x, base_world_pos.y, 3.5),
                    GridPosition::new(pos.x, pos.y),
                    Furniture,
                    furniture_type,
                    orientation,
                ))
                .id()
        }
        _ => {
            // Default fallback for other furniture types (desk, chair, etc.)
            let (base_width_tiles, base_height_tiles) = furniture_type.base_dimensions();
            let mut transform = Transform::from_xyz(furniture_pos.x, furniture_pos.y, 3.0);
            transform.rotate_z(rotation_radians);

            commands
                .spawn((
                    Mesh2d(meshes.add(Rectangle::new(
                        base_width_tiles as f32 * grid_settings.tile_size,
                        base_height_tiles as f32 * grid_settings.tile_size,
                    ))),
                    MeshMaterial2d(materials.add(furniture_type.color())),
                    transform,
                    GridPosition::new(pos.x, pos.y),
                    Furniture,
                    furniture_type,
                    orientation,
                ))
                .id()
        }
    };

    // Add specific furniture component markers
    match furniture_type {
        FurnitureType::Bed(bed_type) => {
            commands.entity(furniture_entity).insert(Bed::new(bed_type));
        }
        FurnitureType::Desk => {
            commands.entity(furniture_entity).insert(Desk);
        }
        FurnitureType::Chair => {
            commands.entity(furniture_entity).insert(Chair);
        }
        FurnitureType::Dresser => {
            commands.entity(furniture_entity).insert(Dresser);
        }
        FurnitureType::Nightstand => {
            commands.entity(furniture_entity).insert(Nightstand);
        }
        FurnitureType::Toilet => {
            commands.entity(furniture_entity).insert(Toilet);
        }
        FurnitureType::Sink => {
            commands.entity(furniture_entity).insert(Sink);
        }
        FurnitureType::Tub => {
            commands.entity(furniture_entity).insert(Tub);
        }
        FurnitureType::ReceptionConsole => {
            commands
                .entity(furniture_entity)
                .insert(ReceptionConsole::new());
        }
    }

    // Mark tiles as occupied
    for tile_pos in furniture_tiles {
        building_map.occupied.insert(tile_pos);
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
