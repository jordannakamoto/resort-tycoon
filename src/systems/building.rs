use crate::components::*;
use crate::systems::grid::*;
use crate::systems::Money;
use crate::ui::{BuildingType, OrderType, ToolbarState, UiInputBlocker};
use bevy::prelude::*;
use bevy::sprite::*;
use bevy::window::{PrimaryWindow, Window as BevyWindow};
use std::collections::HashSet;
use std::f32::consts::{FRAC_PI_2, PI};

const SINGLE_BED_SPRITE_PATH: &str = "generated/furniture/bed.png";
const DOUBLE_BED_SPRITE_PATH: &str = "generated/furniture/double_bed.png";
const DRESSER_FRONT_SPRITE_PATH: &str = "generated/furniture/dresser_front.png";
const DRESSER_BACK_SPRITE_PATH: &str = "generated/furniture/dresser_back.png";
const DRESSER_SIDE_SPRITE_PATH: &str = "generated/furniture/dresser_side.png";
const END_TABLE_SPRITE_PATH: &str = "generated/furniture/endtable.png";
const TUB_SPRITE_PATH: &str = "generated/bath/tub.png";
const SINK_SPRITE_PATH: &str = "generated/bath/sink.png";
const TOILET_SPRITE_PATH: &str = "generated/bath/toilet.png";
const COMPUTER_FRONT_SPRITE_PATH: &str = "generated/staff/computer.png";
const COMPUTER_BACK_SPRITE_PATH: &str = "generated/staff/computer_back.png";
const COMPUTER_SIDE_SPRITE_PATH: &str = "generated/staff/computer_side.png";

#[derive(Resource)]
pub struct BuildingMap {
    pub occupied: std::collections::HashSet<IVec2>, // Walls and windows (block movement)
    pub walls: std::collections::HashMap<IVec2, Entity>, // Wall entities by position
    pub doors: std::collections::HashMap<IVec2, Entity>, // Door tiles (can pass when open)
    pub floors: std::collections::HashSet<IVec2>,   // Floors (don't block building)
}

impl Default for BuildingMap {
    fn default() -> Self {
        Self {
            occupied: std::collections::HashSet::new(),
            walls: std::collections::HashMap::new(),
            doors: std::collections::HashMap::new(),
            floors: std::collections::HashSet::new(),
        }
    }
}

impl BuildingMap {
    pub fn is_occupied(&self, pos: IVec2) -> bool {
        self.occupied.contains(&pos) || self.walls.contains_key(&pos)
    }
}

#[derive(Resource, Default)]
pub struct DragState {
    pub is_dragging: bool,
    pub start_pos: Option<IVec2>,
    pub current_pos: Option<IVec2>,
}

#[derive(Resource)]
pub struct DoorPlacementState {
    pub orientation: DoorOrientation,
}

impl Default for DoorPlacementState {
    fn default() -> Self {
        Self {
            orientation: DoorOrientation::Horizontal,
        }
    }
}

#[derive(Resource)]
pub struct FurniturePlacementState {
    pub orientation: FurnitureOrientation,
}

impl Default for FurniturePlacementState {
    fn default() -> Self {
        Self {
            orientation: FurnitureOrientation::East,
        }
    }
}

impl DragState {
    pub fn start(&mut self, pos: IVec2) {
        self.is_dragging = true;
        self.start_pos = Some(pos);
        self.current_pos = Some(pos);
    }

    pub fn update(&mut self, pos: IVec2) {
        if self.is_dragging {
            self.current_pos = Some(pos);
        }
    }

    pub fn end(&mut self) -> Option<(IVec2, IVec2)> {
        if self.is_dragging {
            let result = self.start_pos.zip(self.current_pos);
            self.is_dragging = false;
            self.start_pos = None;
            self.current_pos = None;
            result
        } else {
            None
        }
    }

    pub fn get_drag_positions(&self) -> Vec<IVec2> {
        if let (Some(start), Some(end)) = (self.start_pos, self.current_pos) {
            let min_x = start.x.min(end.x);
            let max_x = start.x.max(end.x);
            let min_y = start.y.min(end.y);
            let max_y = start.y.max(end.y);

            let mut positions = Vec::new();
            for x in min_x..=max_x {
                for y in min_y..=max_y {
                    positions.push(IVec2::new(x, y));
                }
            }
            positions
        } else {
            Vec::new()
        }
    }
}

pub struct BuildingPlugin;

impl Plugin for BuildingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BuildingMap>()
            .init_resource::<DragState>()
            .init_resource::<DoorPlacementState>()
            .init_resource::<FurniturePlacementState>()
            .init_resource::<ContextMenuState>()
            .init_resource::<UiInputBlocker>()
            .add_systems(Startup, setup_context_menu)
            .add_systems(
                Update,
                (
                    handle_rotation_input,
                    handle_drag_input,
                    update_placement_preview,
                    handle_building_placement,
                    handle_deconstruction_placement,
                    handle_right_click_deconstruct,
                    update_context_menu,
                    handle_context_menu_clicks,
                    update_wall_projections,
                )
                    .chain(),
            );
    }
}

fn handle_rotation_input(
    mut door_state: ResMut<DoorPlacementState>,
    mut furniture_state: ResMut<FurniturePlacementState>,
    toolbar_state: Res<ToolbarState>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if !keyboard.just_pressed(KeyCode::KeyR) {
        return;
    }

    match toolbar_state.selected_building {
        Some(BuildingType::Door) => {
            door_state.orientation = match door_state.orientation {
                DoorOrientation::Horizontal => DoorOrientation::Vertical,
                DoorOrientation::Vertical => DoorOrientation::Horizontal,
            };
        }
        Some(BuildingType::Furniture(_)) => {
            furniture_state.orientation = furniture_state.orientation.next();
        }
        _ => {}
    }
}

fn furniture_rotation_radians(orientation: FurnitureOrientation) -> f32 {
    match orientation {
        FurnitureOrientation::East => 0.0,
        FurnitureOrientation::South => FRAC_PI_2,
        FurnitureOrientation::West => PI,
        FurnitureOrientation::North => FRAC_PI_2 * 3.0,
    }
}

fn handle_drag_input(
    mut drag_state: ResMut<DragState>,
    toolbar_state: Res<ToolbarState>,
    grid_settings: Res<GridSettings>,
    window_query: Query<&BevyWindow, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    ui_blocker: Res<UiInputBlocker>,
) {
    // Allow dragging for walls and floors
    let allow_drag = matches!(
        toolbar_state.selected_building,
        Some(BuildingType::Wall) | Some(BuildingType::Floor(_))
    );

    if !allow_drag {
        if drag_state.is_dragging {
            drag_state.is_dragging = false;
            drag_state.start_pos = None;
            drag_state.current_pos = None;
        }
        return;
    }

    if ui_blocker.block_world_input {
        return;
    }

    let window = window_query.single();
    let (camera, camera_transform) = camera_query.single();

    if let Some(cursor_pos) = window.cursor_position() {
        // Ignore clicks in toolbar area (bottom 80 pixels)
        const TOOLBAR_HEIGHT: f32 = 80.0;
        if cursor_pos.y > window.height() - TOOLBAR_HEIGHT {
            return;
        }
        if let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) {
            if let Some(grid_pos) = world_to_grid(
                world_pos,
                grid_settings.tile_size,
                grid_settings.width,
                grid_settings.height,
            ) {
                // Start drag on mouse press
                if mouse_button.just_pressed(MouseButton::Left) {
                    drag_state.start(grid_pos);
                }
                // Update drag position while holding
                else if mouse_button.pressed(MouseButton::Left) && drag_state.is_dragging {
                    drag_state.update(grid_pos);
                }
            }
        }
    }

    // Note: Don't call drag_state.end() here - let handle_building_placement do it
}

fn update_placement_preview(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    toolbar_state: Res<ToolbarState>,
    drag_state: Res<DragState>,
    door_state: Res<DoorPlacementState>,
    furniture_state: Res<FurniturePlacementState>,
    grid_settings: Res<GridSettings>,
    window_query: Query<&BevyWindow, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    preview_query: Query<Entity, With<PlacementPreview>>,
    building_map: Res<BuildingMap>,
    desk_query: Query<&GridPosition, With<Desk>>,
    ui_blocker: Res<UiInputBlocker>,
    asset_server: Res<AssetServer>,
) {
    let window = window_query.single();
    let (camera, camera_transform) = camera_query.single();

    // Remove old preview
    for entity in &preview_query {
        commands.entity(entity).despawn();
    }

    if ui_blocker.block_world_input {
        return;
    }

    // Only show preview if a building is selected
    if let Some(building_type) = toolbar_state.selected_building {
        // If dragging walls or floors, show all positions in the drag area
        let is_dragging_multi =
            matches!(building_type, BuildingType::Wall | BuildingType::Floor(_))
                && drag_state.is_dragging;

        if is_dragging_multi {
            let positions = drag_state.get_drag_positions();
            for grid_pos in positions {
                let world_pos = grid_to_world(
                    grid_pos,
                    grid_settings.tile_size,
                    grid_settings.width,
                    grid_settings.height,
                );

                // For floors, check if structure is blocking; for structures, check if occupied
                let is_blocked = match building_type {
                    BuildingType::Floor(_) => building_map.occupied.contains(&grid_pos),
                    _ => building_map.occupied.contains(&grid_pos),
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
        // Otherwise show single preview at cursor
        else if let Some(cursor_pos) = window.cursor_position() {
            // Don't show preview in toolbar area (bottom 80 pixels)
            const TOOLBAR_HEIGHT: f32 = 80.0;
            if cursor_pos.y > window.height() - TOOLBAR_HEIGHT {
                return;
            }

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

                    // Handle door preview (2x1)
                    if building_type == BuildingType::Door {
                        let door_tiles = match door_state.orientation {
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
                    } else if let BuildingType::Furniture(furniture_type) = building_type {
                        // Special preview for reception console - check for desk
                        if furniture_type == FurnitureType::ReceptionConsole {
                            // Check if there's a desk at this position (desk is 2x2, check all 4 tiles)
                            let has_desk = desk_query.iter().any(|desk_pos| {
                                let desk_tiles = vec![
                                    desk_pos.to_ivec2(),
                                    desk_pos.to_ivec2() + IVec2::new(1, 0),
                                    desk_pos.to_ivec2() + IVec2::new(0, 1),
                                    desk_pos.to_ivec2() + IVec2::new(1, 1),
                                ];
                                desk_tiles.contains(&grid_pos)
                            });

                            // Select sprite based on orientation
                            let orientation = furniture_state.orientation;
                            let (sprite_path, flip_x) = match orientation {
                                FurnitureOrientation::East => {
                                    (COMPUTER_SIDE_SPRITE_PATH, false)
                                }
                                FurnitureOrientation::West => (COMPUTER_SIDE_SPRITE_PATH, true),
                                FurnitureOrientation::South => {
                                    (COMPUTER_FRONT_SPRITE_PATH, false)
                                }
                                FurnitureOrientation::North => {
                                    (COMPUTER_BACK_SPRITE_PATH, false)
                                }
                            };

                            let preview_color = if !has_desk {
                                Color::srgba(1.0, 0.3, 0.3, 1.0)  // Red if no desk
                            } else {
                                Color::srgba(1.0, 1.0, 1.0, 0.7)  // White if desk present, preserves sprite alpha
                            };

                            let mut sprite = Sprite {
                                image: asset_server.load(sprite_path),
                                color: preview_color,
                                custom_size: Some(Vec2::new(
                                    grid_settings.tile_size,
                                    grid_settings.tile_size,
                                )),
                                ..default()
                            };
                            sprite.flip_x = flip_x;

                            // Use higher z-level so it appears above desk
                            commands.spawn((
                                sprite,
                                Transform::from_xyz(world_pos.x, world_pos.y, 4.0),
                                PlacementPreview,
                            ));
                        } else {
                            // Show actual furniture shape as preview
                            let orientation = furniture_state.orientation;
                            let furniture_tiles = furniture_type
                                .tiles_occupied(grid_pos, orientation);
                            let (width_tiles, height_tiles) =
                                furniture_type.oriented_dimensions(orientation);
                            let rotation_radians = furniture_rotation_radians(orientation);

                            // Check if furniture can be placed
                            let is_blocked = furniture_tiles.iter().any(|tile_pos| {
                                !building_map.floors.contains(tile_pos)
                                    || building_map.occupied.contains(tile_pos)
                                    || building_map.doors.contains_key(tile_pos)
                            });

                            // Calculate center position for preview
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

                            // Create preview based on furniture type
                            match furniture_type {
                                FurnitureType::Bed(bed_type) => {
                                    let (base_width_tiles, base_height_tiles) =
                                        furniture_type.base_dimensions();
                                    let sprite_size = Vec2::new(
                                        base_width_tiles as f32 * grid_settings.tile_size,
                                        base_height_tiles as f32 * grid_settings.tile_size,
                                    );

                                    let sprite_path = match bed_type {
                                        BedType::Single => SINGLE_BED_SPRITE_PATH,
                                        BedType::Double => DOUBLE_BED_SPRITE_PATH,
                                    };

                                    let mut transform =
                                        Transform::from_xyz(preview_pos.x, preview_pos.y, 4.0);
                                    transform.rotate_z(rotation_radians);

                                    // Use white tint with full alpha - sprite's own alpha channel will define the shape
                                    let preview_color = if is_blocked {
                                        Color::srgba(1.0, 0.3, 0.3, 1.0)  // Red for blocked
                                    } else {
                                        Color::srgba(1.0, 1.0, 1.0, 0.7)  // White for valid, preserves sprite alpha
                                    };

                                    commands.spawn((
                                        Sprite {
                                            image: asset_server.load(sprite_path),
                                            color: preview_color,
                                            custom_size: Some(sprite_size),
                                            ..default()
                                        },
                                        transform,
                                        PlacementPreview,
                                    ));
                                }
                                FurnitureType::Nightstand => {
                                    let (base_width_tiles, base_height_tiles) =
                                        furniture_type.base_dimensions();
                                    let sprite_size = Vec2::new(
                                        base_width_tiles as f32 * grid_settings.tile_size,
                                        base_height_tiles as f32 * grid_settings.tile_size,
                                    );

                                    let mut transform =
                                        Transform::from_xyz(preview_pos.x, preview_pos.y, 4.0);
                                    transform.rotate_z(rotation_radians);

                                    // Use white tint with full alpha - sprite's own alpha channel will define the shape
                                    let preview_color = if is_blocked {
                                        Color::srgba(1.0, 0.3, 0.3, 1.0)  // Red for blocked
                                    } else {
                                        Color::srgba(1.0, 1.0, 1.0, 0.7)  // White for valid, preserves sprite alpha
                                    };

                                    commands.spawn((
                                        Sprite {
                                            image: asset_server.load(END_TABLE_SPRITE_PATH),
                                            color: preview_color,
                                            custom_size: Some(sprite_size),
                                            ..default()
                                        },
                                        transform,
                                        PlacementPreview,
                                    ));
                                }
                                FurnitureType::Dresser => {
                                    let (base_width_tiles, base_height_tiles) =
                                        furniture_type.base_dimensions();
                                    let sprite_size = Vec2::new(
                                        base_width_tiles as f32 * grid_settings.tile_size,
                                        base_height_tiles as f32 * grid_settings.tile_size,
                                    );
                                    let transform =
                                        Transform::from_xyz(preview_pos.x, preview_pos.y, 4.0);

                                    // Keep dresser preview upright; swap textures based on facing.
                                    let (sprite_path, flip_x) = match orientation {
                                        FurnitureOrientation::South => {
                                            (DRESSER_FRONT_SPRITE_PATH, false)
                                        }
                                        FurnitureOrientation::North => {
                                            (DRESSER_BACK_SPRITE_PATH, false)
                                        }
                                        FurnitureOrientation::East => {
                                            (DRESSER_SIDE_SPRITE_PATH, false)
                                        }
                                        FurnitureOrientation::West => {
                                            (DRESSER_SIDE_SPRITE_PATH, true)
                                        }
                                    };

                                    // Use white tint with full alpha - sprite's own alpha channel will define the shape
                                    let preview_color = if is_blocked {
                                        Color::srgba(1.0, 0.3, 0.3, 1.0)  // Red for blocked
                                    } else {
                                        Color::srgba(1.0, 1.0, 1.0, 0.7)  // White for valid, preserves sprite alpha
                                    };

                                    let mut sprite = Sprite {
                                        image: asset_server.load(sprite_path),
                                        color: preview_color,
                                        custom_size: Some(sprite_size),
                                        ..default()
                                    };
                                    sprite.flip_x = flip_x;

                                    commands.spawn((
                                        sprite,
                                        transform,
                                        PlacementPreview,
                                    ));
                                }
                                _ => {
                                    // Other furniture: show as colored rectangle
                                    let (base_width_tiles, base_height_tiles) =
                                        furniture_type.base_dimensions();
                                    let mut transform =
                                        Transform::from_xyz(preview_pos.x, preview_pos.y, 4.0);
                                    transform.rotate_z(rotation_radians);

                                    let preview_color = if is_blocked {
                                        Color::srgba(1.0, 0.3, 0.3, 0.5)  // Red for blocked
                                    } else {
                                        Color::srgba(1.0, 1.0, 1.0, 0.5)  // White for valid
                                    };

                                    commands.spawn((
                                        Mesh2d(meshes.add(Rectangle::new(
                                            base_width_tiles as f32 * grid_settings.tile_size,
                                            base_height_tiles as f32 * grid_settings.tile_size,
                                        ))),
                                        MeshMaterial2d(materials.add(preview_color)),
                                        transform,
                                        PlacementPreview,
                                    ));
                                }
                            }
                        }
                    } else {
                        // Single tile preview for other buildings
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
    mut drag_state: ResMut<DragState>,
    door_state: Res<DoorPlacementState>,
    furniture_state: Res<FurniturePlacementState>,
    grid_settings: Res<GridSettings>,
    window_query: Query<&BevyWindow, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut building_map: ResMut<BuildingMap>,
    mut money: ResMut<Money>,
    asset_server: Res<AssetServer>,
    desk_query: Query<&GridPosition, With<Desk>>,
    ui_blocker: Res<UiInputBlocker>,
) {
    if ui_blocker.block_world_input {
        return;
    }

    if let Some(building_type) = toolbar_state.selected_building {
        // Handle drag building for walls and floors
        let is_drag_buildable =
            matches!(building_type, BuildingType::Wall | BuildingType::Floor(_));

        if is_drag_buildable && mouse_button.just_released(MouseButton::Left) {
            if let Some((start, end)) = drag_state.end() {
                // Place all buildings in the drag area
                let positions = {
                    let min_x = start.x.min(end.x);
                    let max_x = start.x.max(end.x);
                    let min_y = start.y.min(end.y);
                    let max_y = start.y.max(end.y);

                    let mut positions = Vec::new();
                    for x in min_x..=max_x {
                        for y in min_y..=max_y {
                            positions.push(IVec2::new(x, y));
                        }
                    }
                    positions
                };

                for grid_pos in positions {
                    // For structures, skip if occupied; for floors, skip if structure exists
                    let should_skip = match building_type {
                        BuildingType::Floor(_) => building_map.occupied.contains(&grid_pos),
                        _ => building_map.occupied.contains(&grid_pos),
                    };

                    if should_skip {
                        continue;
                    }

                    // Check if player can afford this tile
                    let cost = building_type.cost();
                    if !money.can_afford(cost) {
                        continue; // Skip this tile if can't afford
                    }

                    let world_pos = grid_to_world(
                        grid_pos,
                        grid_settings.tile_size,
                        grid_settings.width,
                        grid_settings.height,
                    );

                    let blueprint_type = match building_type {
                        BuildingType::Wall => BlueprintType::Wall,
                        BuildingType::Floor(floor_type) => BlueprintType::Floor(floor_type),
                        _ => continue,
                    };

                    // Deduct money
                    money.deduct(cost);

                    let blueprint_entity = spawn_blueprint(
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        blueprint_type,
                        grid_pos,
                        world_pos,
                        grid_settings.tile_size,
                    );

                    commands.spawn(ConstructionJob::new(blueprint_entity));

                    // Track placement
                    match building_type {
                        BuildingType::Floor(_) => {
                            building_map.floors.insert(grid_pos);
                        }
                        BuildingType::Wall => {
                            building_map.occupied.insert(grid_pos);
                            building_map.walls.insert(grid_pos, blueprint_entity);
                        }
                        _ => {
                            building_map.occupied.insert(grid_pos);
                        }
                    }
                }
                return;
            }
        }

        // Handle single building placement for non-walls or single clicks
        if mouse_button.just_pressed(MouseButton::Left) && !drag_state.is_dragging {
            let window = window_query.single();
            let (camera, camera_transform) = camera_query.single();

            if let Some(cursor_pos) = window.cursor_position() {
                // Ignore clicks in toolbar area (bottom 80 pixels)
                const TOOLBAR_HEIGHT: f32 = 80.0;
                if cursor_pos.y > window.height() - TOOLBAR_HEIGHT {
                    return;
                }
                if let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) {
                    if let Some(grid_pos) = world_to_grid(
                        world_pos,
                        grid_settings.tile_size,
                        grid_settings.width,
                        grid_settings.height,
                    ) {
                        // Handle door placement (2x1)
                        if building_type == BuildingType::Door {
                            let door_tiles = match door_state.orientation {
                                DoorOrientation::Horizontal => {
                                    vec![grid_pos, grid_pos + IVec2::new(1, 0)]
                                }
                                DoorOrientation::Vertical => {
                                    vec![grid_pos, grid_pos + IVec2::new(0, 1)]
                                }
                            };

                            // Check if all tiles are available (walls can be replaced, but not doors or windows)
                            let all_available = door_tiles.iter().all(|pos| {
                                let has_wall = building_map.walls.contains_key(pos);
                                let has_door = building_map.doors.contains_key(pos);
                                let has_other = building_map.occupied.contains(pos) && !has_wall;

                                // Allow if empty OR if it's a wall (we'll replace it)
                                !has_door && !has_other
                            });

                            if !all_available {
                                return;
                            }

                            // Check if player can afford the door
                            let cost = building_type.cost();
                            if !money.can_afford(cost) {
                                return;
                            }

                            // Deduct money
                            money.deduct(cost);

                            // Remove walls that are being replaced
                            for tile_pos in &door_tiles {
                                if let Some(wall_entity) = building_map.walls.remove(tile_pos) {
                                    commands.entity(wall_entity).despawn_recursive();
                                    building_map.occupied.remove(tile_pos);
                                }
                            }

                            // Calculate center position for door
                            let center_pos = match door_state.orientation {
                                DoorOrientation::Horizontal => Vec2::new(
                                    (door_tiles[0].x + door_tiles[1].x) as f32
                                        * grid_settings.tile_size
                                        / 2.0
                                        - (grid_settings.width as f32 * grid_settings.tile_size)
                                            / 2.0,
                                    door_tiles[0].y as f32 * grid_settings.tile_size
                                        - (grid_settings.height as f32 * grid_settings.tile_size)
                                            / 2.0
                                        + grid_settings.tile_size / 2.0,
                                ),
                                DoorOrientation::Vertical => Vec2::new(
                                    door_tiles[0].x as f32 * grid_settings.tile_size
                                        - (grid_settings.width as f32 * grid_settings.tile_size)
                                            / 2.0
                                        + grid_settings.tile_size / 2.0,
                                    (door_tiles[0].y + door_tiles[1].y) as f32
                                        * grid_settings.tile_size
                                        / 2.0
                                        - (grid_settings.height as f32 * grid_settings.tile_size)
                                            / 2.0,
                                ),
                            };

                            let blueprint_entity = spawn_door_blueprint(
                                &mut commands,
                                &mut meshes,
                                &mut materials,
                                grid_pos,
                                center_pos,
                                grid_settings.tile_size,
                                door_state.orientation,
                            );

                            commands.spawn(ConstructionJob::new(blueprint_entity));

                            // Track door placement - reserve tiles but don't block (pawns can pass when open)
                            for tile_pos in door_tiles {
                                building_map.doors.insert(tile_pos, blueprint_entity);
                            }
                        } else if let BuildingType::Furniture(furniture_type) = building_type {
                            // Special handling for reception console - must be placed on a desk
                            if furniture_type == FurnitureType::ReceptionConsole {
                                // Check if there's a desk at this position (desk is 2x2, check all 4 tiles)
                                let has_desk = desk_query.iter().any(|desk_pos| {
                                    let desk_tiles = vec![
                                        desk_pos.to_ivec2(),
                                        desk_pos.to_ivec2() + IVec2::new(1, 0),
                                        desk_pos.to_ivec2() + IVec2::new(0, 1),
                                        desk_pos.to_ivec2() + IVec2::new(1, 1),
                                    ];
                                    desk_tiles.contains(&grid_pos)
                                });

                                if !has_desk {
                                    // Can't place reception console without a desk
                                    return;
                                }

                                // Check if player can afford the reception console
                                let cost = building_type.cost();
                                if !money.can_afford(cost) {
                                    return;
                                }

                                // Deduct money
                                money.deduct(cost);

                                let base_world_pos = grid_to_world(
                                    grid_pos,
                                    grid_settings.tile_size,
                                    grid_settings.width,
                                    grid_settings.height,
                                );

                                let orientation = furniture_state.orientation;
                                let (sprite_path, flip_x) = match orientation {
                                    FurnitureOrientation::East => {
                                        (COMPUTER_SIDE_SPRITE_PATH, false)
                                    }
                                    FurnitureOrientation::West => (COMPUTER_SIDE_SPRITE_PATH, true),
                                    FurnitureOrientation::South => {
                                        (COMPUTER_FRONT_SPRITE_PATH, false)
                                    }
                                    FurnitureOrientation::North => {
                                        (COMPUTER_BACK_SPRITE_PATH, false)
                                    }
                                };

                                let mut sprite = Sprite {
                                    image: asset_server.load(sprite_path),
                                    custom_size: Some(Vec2::splat(grid_settings.tile_size * 0.9)),
                                    ..default()
                                };
                                sprite.flip_x = flip_x;

                                commands.spawn((
                                    sprite,
                                    Transform::from_xyz(base_world_pos.x, base_world_pos.y, 3.5),
                                    GridPosition::new(grid_pos.x, grid_pos.y),
                                    Furniture,
                                    furniture_type,
                                    orientation,
                                    ReceptionConsole::new(),
                                ));

                                // Don't mark tiles as occupied - desk already occupies them
                                return;
                            }

                            // Handle regular furniture placement
                            let orientation = furniture_state.orientation;
                            let furniture_tiles =
                                furniture_type.tiles_occupied(grid_pos, orientation);
                            let (width_tiles, height_tiles) =
                                furniture_type.oriented_dimensions(orientation);
                            let width_mult = width_tiles as f32;
                            let height_mult = height_tiles as f32;
                            let rotation_radians = furniture_rotation_radians(orientation);

                            // Check if all tiles have floors and are available
                            let can_place = furniture_tiles.iter().all(|pos| {
                                building_map.floors.contains(pos)
                                    && !building_map.occupied.contains(pos)
                                    && !building_map.doors.contains_key(pos)
                            });

                            if !can_place {
                                return;
                            }

                            // Check if player can afford the furniture
                            let cost = building_type.cost();
                            if !money.can_afford(cost) {
                                return;
                            }

                            // Deduct money
                            money.deduct(cost);

                            // Calculate center position for multi-tile furniture
                            let offset = Vec2::new(
                                (width_mult - 1.0) * grid_settings.tile_size / 2.0,
                                (height_mult - 1.0) * grid_settings.tile_size / 2.0,
                            );

                            let base_world_pos = grid_to_world(
                                grid_pos,
                                grid_settings.tile_size,
                                grid_settings.width,
                                grid_settings.height,
                            );
                            let furniture_pos = base_world_pos + offset;

                            // Spawn furniture directly (no construction needed)
                            let furniture_entity = match furniture_type {
                                FurnitureType::Bed(bed_type) => {
                                    let (base_width_tiles, base_height_tiles) =
                                        furniture_type.base_dimensions();
                                    let sprite_size = Vec2::new(
                                        base_width_tiles as f32 * grid_settings.tile_size,
                                        base_height_tiles as f32 * grid_settings.tile_size,
                                    );

                                    let sprite_path = match bed_type {
                                        BedType::Single => SINGLE_BED_SPRITE_PATH,
                                        BedType::Double => DOUBLE_BED_SPRITE_PATH,
                                    };

                                    let mut transform =
                                        Transform::from_xyz(furniture_pos.x, furniture_pos.y, 3.0);
                                    transform.rotate_z(rotation_radians);

                                    commands
                                        .spawn((
                                            Sprite {
                                                image: asset_server.load(sprite_path),
                                                custom_size: Some(sprite_size),
                                                ..default()
                                            },
                                            transform,
                                            GridPosition::new(grid_pos.x, grid_pos.y),
                                            Furniture,
                                            furniture_type,
                                            orientation,
                                        ))
                                        .id()
                                }
                                FurnitureType::Dresser => {
                                    let (base_width_tiles, base_height_tiles) =
                                        furniture_type.base_dimensions();
                                    let sprite_size = Vec2::new(
                                        base_width_tiles as f32 * grid_settings.tile_size,
                                        base_height_tiles as f32 * grid_settings.tile_size,
                                    );
                                    let mut transform =
                                        Transform::from_xyz(furniture_pos.x, furniture_pos.y, 3.0);

                                    // Keep dresser sprite upright; swap textures based on facing.
                                    let (sprite_path, flip_x) = match orientation {
                                        FurnitureOrientation::South => {
                                            (DRESSER_FRONT_SPRITE_PATH, false)
                                        }
                                        FurnitureOrientation::North => {
                                            (DRESSER_BACK_SPRITE_PATH, false)
                                        }
                                        FurnitureOrientation::East => {
                                            (DRESSER_SIDE_SPRITE_PATH, false)
                                        }
                                        FurnitureOrientation::West => {
                                            (DRESSER_SIDE_SPRITE_PATH, true)
                                        }
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
                                            GridPosition::new(grid_pos.x, grid_pos.y),
                                            Furniture,
                                            furniture_type,
                                            orientation,
                                        ))
                                        .id()
                                }
                                FurnitureType::Tub => {
                                    let (base_width_tiles, base_height_tiles) =
                                        furniture_type.base_dimensions();
                                    let sprite_size = Vec2::new(
                                        base_width_tiles as f32 * grid_settings.tile_size,
                                        base_height_tiles as f32 * grid_settings.tile_size,
                                    );
                                    let mut transform =
                                        Transform::from_xyz(furniture_pos.x, furniture_pos.y, 3.0);
                                    transform.rotate_z(rotation_radians);

                                    commands
                                        .spawn((
                                            Sprite {
                                                image: asset_server.load(TUB_SPRITE_PATH),
                                                custom_size: Some(sprite_size),
                                                ..default()
                                            },
                                            transform,
                                            GridPosition::new(grid_pos.x, grid_pos.y),
                                            Furniture,
                                            furniture_type,
                                            orientation,
                                        ))
                                        .id()
                                }
                                FurnitureType::Toilet => {
                                    let (base_width_tiles, base_height_tiles) =
                                        furniture_type.base_dimensions();
                                    let sprite_size = Vec2::new(
                                        base_width_tiles as f32 * grid_settings.tile_size,
                                        base_height_tiles as f32 * grid_settings.tile_size,
                                    );
                                    let mut transform =
                                        Transform::from_xyz(furniture_pos.x, furniture_pos.y, 3.0);
                                    transform.rotate_z(rotation_radians);

                                    commands
                                        .spawn((
                                            Sprite {
                                                image: asset_server.load(TOILET_SPRITE_PATH),
                                                custom_size: Some(sprite_size),
                                                ..default()
                                            },
                                            transform,
                                            GridPosition::new(grid_pos.x, grid_pos.y),
                                            Furniture,
                                            furniture_type,
                                            orientation,
                                        ))
                                        .id()
                                }
                                FurnitureType::Sink => {
                                    let (base_width_tiles, base_height_tiles) =
                                        furniture_type.base_dimensions();
                                    let sprite_size = Vec2::new(
                                        base_width_tiles as f32 * grid_settings.tile_size,
                                        base_height_tiles as f32 * grid_settings.tile_size,
                                    );
                                    let mut transform =
                                        Transform::from_xyz(furniture_pos.x, furniture_pos.y, 3.0);
                                    transform.rotate_z(rotation_radians);

                                    commands
                                        .spawn((
                                            Sprite {
                                                image: asset_server.load(SINK_SPRITE_PATH),
                                                custom_size: Some(sprite_size),
                                                ..default()
                                            },
                                            transform,
                                            GridPosition::new(grid_pos.x, grid_pos.y),
                                            Furniture,
                                            furniture_type,
                                            orientation,
                                        ))
                                        .id()
                                }
                                FurnitureType::Nightstand => {
                                    let (base_width_tiles, base_height_tiles) =
                                        furniture_type.base_dimensions();
                                    let sprite_size = Vec2::new(
                                        base_width_tiles as f32 * grid_settings.tile_size,
                                        base_height_tiles as f32 * grid_settings.tile_size,
                                    );
                                    let mut transform =
                                        Transform::from_xyz(furniture_pos.x, furniture_pos.y, 3.0);
                                    transform.rotate_z(rotation_radians);

                                    commands
                                        .spawn((
                                            Sprite {
                                                image: asset_server.load(END_TABLE_SPRITE_PATH),
                                                custom_size: Some(sprite_size),
                                                ..default()
                                            },
                                            transform,
                                            GridPosition::new(grid_pos.x, grid_pos.y),
                                            Furniture,
                                            furniture_type,
                                            orientation,
                                        ))
                                        .id()
                                }
                                _ => {
                                    let (base_width_tiles, base_height_tiles) =
                                        furniture_type.base_dimensions();
                                    let mut transform =
                                        Transform::from_xyz(furniture_pos.x, furniture_pos.y, 3.0);
                                    transform.rotate_z(rotation_radians);

                                    commands
                                        .spawn((
                                            Mesh2d(meshes.add(Rectangle::new(
                                                base_width_tiles as f32 * grid_settings.tile_size,
                                                base_height_tiles as f32 * grid_settings.tile_size,
                                            ))),
                                            MeshMaterial2d(materials.add(furniture_type.color())),
                                            transform,
                                            GridPosition::new(grid_pos.x, grid_pos.y),
                                            Furniture,
                                            furniture_type,
                                            orientation,
                                        ))
                                        .id()
                                }
                            };

                            // Add specific furniture component
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

                            // Mark tiles as occupied (furniture blocks placement but not movement)
                            for tile_pos in furniture_tiles {
                                building_map.occupied.insert(tile_pos);
                            }
                        } else {
                            // Regular building placement
                            let should_skip = match building_type {
                                BuildingType::Floor(_) => building_map.occupied.contains(&grid_pos),
                                BuildingType::Window => {
                                    // Windows can replace walls
                                    let has_wall = building_map.walls.contains_key(&grid_pos);
                                    let has_other =
                                        building_map.occupied.contains(&grid_pos) && !has_wall;
                                    has_other || building_map.doors.contains_key(&grid_pos)
                                }
                                _ => building_map.occupied.contains(&grid_pos),
                            };

                            if should_skip {
                                return;
                            }

                            // Check if player can afford this building
                            let cost = building_type.cost();
                            if !money.can_afford(cost) {
                                return;
                            }

                            // Deduct money
                            money.deduct(cost);

                            // Remove wall if placing window over it
                            if building_type == BuildingType::Window {
                                if let Some(wall_entity) = building_map.walls.remove(&grid_pos) {
                                    commands.entity(wall_entity).despawn_recursive();
                                    building_map.occupied.remove(&grid_pos);
                                }
                            }

                            let world_pos = grid_to_world(
                                grid_pos,
                                grid_settings.tile_size,
                                grid_settings.width,
                                grid_settings.height,
                            );

                            let blueprint_type = match building_type {
                                BuildingType::Wall => BlueprintType::Wall,
                                BuildingType::Window => BlueprintType::Window,
                                BuildingType::Floor(floor_type) => BlueprintType::Floor(floor_type),
                                _ => return,
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

                            commands.spawn(ConstructionJob::new(blueprint_entity));

                            // Track placement
                            match building_type {
                                BuildingType::Floor(_) => {
                                    building_map.floors.insert(grid_pos);
                                }
                                BuildingType::Wall => {
                                    building_map.occupied.insert(grid_pos);
                                    building_map.walls.insert(grid_pos, blueprint_entity);
                                }
                                _ => {
                                    building_map.occupied.insert(grid_pos);
                                }
                            }
                        }
                    }
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

fn spawn_door_blueprint(
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

// Handle left-click deconstruction placement with Deconstruct order selected
fn handle_deconstruction_placement(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    toolbar_state: Res<ToolbarState>,
    mut drag_state: ResMut<DragState>,
    grid_settings: Res<GridSettings>,
    window_query: Query<&BevyWindow, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    deconstructible_query: Query<
        (Entity, &GridPosition, &Transform),
        Or<(
            With<Wall>,
            With<Door>,
            With<crate::components::Window>,
            With<Furniture>,
        )>,
    >,
    marker_query: Query<&DeconstructionMarker>,
    ui_blocker: Res<UiInputBlocker>,
) {
    // Only handle when deconstruct order is selected
    if toolbar_state.selected_order != Some(OrderType::Deconstruct) {
        return;
    }

    if ui_blocker.block_world_input {
        return;
    }

    let window = window_query.single();
    let (camera, camera_transform) = camera_query.single();

    if let Some(cursor_pos) = window.cursor_position() {
        // Ignore clicks in toolbar area
        const TOOLBAR_HEIGHT: f32 = 80.0;
        if cursor_pos.y > window.height() - TOOLBAR_HEIGHT {
            return;
        }

        if let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) {
            if let Some(grid_pos) = world_to_grid(
                world_pos,
                grid_settings.tile_size,
                grid_settings.width,
                grid_settings.height,
            ) {
                // Start drag on mouse press
                if mouse_button.just_pressed(MouseButton::Left) {
                    drag_state.start(grid_pos);
                }
                // Update drag position while holding
                else if mouse_button.pressed(MouseButton::Left) && drag_state.is_dragging {
                    drag_state.update(grid_pos);
                }
            }
        }
    }

    // Handle drag end
    if mouse_button.just_released(MouseButton::Left) && drag_state.is_dragging {
        if let Some((start, end)) = drag_state.end() {
            let min_x = start.x.min(end.x);
            let max_x = start.x.max(end.x);
            let min_y = start.y.min(end.y);
            let max_y = start.y.max(end.y);

            for x in min_x..=max_x {
                for y in min_y..=max_y {
                    let grid_pos = IVec2::new(x, y);
                    // Find deconstructible entity at this position
                    for (entity, entity_grid_pos, entity_transform) in &deconstructible_query {
                        if entity_grid_pos.to_ivec2() == grid_pos {
                            // Check if already marked for deconstruction
                            let already_marked = marker_query
                                .iter()
                                .any(|marker| marker.target_entity == entity);
                            if already_marked {
                                continue;
                            }

                            // Create deconstruction marker
                            let marker_entity = commands
                                .spawn((
                                    Mesh2d(meshes.add(Rectangle::new(
                                        grid_settings.tile_size,
                                        grid_settings.tile_size,
                                    ))),
                                    MeshMaterial2d(materials.add(Color::srgba(1.0, 0.0, 0.0, 0.4))),
                                    Transform::from_xyz(
                                        entity_transform.translation.x,
                                        entity_transform.translation.y,
                                        10.0, // High z-level to render on top
                                    ),
                                    DeconstructionMarker::new(entity),
                                    GridPosition::new(grid_pos.x, grid_pos.y),
                                ))
                                .id();

                            // Create deconstruction job
                            commands.spawn(DeconstructionJob::new(marker_entity));
                            break;
                        }
                    }
                }
            }
        }
    }
}

#[derive(Resource, Default)]
pub struct ContextMenuState {
    pub visible: bool,
    pub target_entity: Option<Entity>,
    pub position: Vec2,
}

// Handle right-click to show context menu
fn handle_right_click_deconstruct(
    mut context_menu_state: ResMut<ContextMenuState>,
    grid_settings: Res<GridSettings>,
    window_query: Query<&BevyWindow, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    deconstructible_query: Query<
        (Entity, &GridPosition),
        Or<(
            With<Wall>,
            With<Door>,
            With<crate::components::Window>,
            With<Furniture>,
        )>,
    >,
    ui_blocker: Res<UiInputBlocker>,
) {
    if !mouse_button.just_pressed(MouseButton::Right) {
        return;
    }

    if ui_blocker.block_world_input {
        return;
    }

    let window = window_query.single();
    let (camera, camera_transform) = camera_query.single();

    if let Some(cursor_pos) = window.cursor_position() {
        // Ignore clicks in toolbar area
        const TOOLBAR_HEIGHT: f32 = 80.0;
        if cursor_pos.y > window.height() - TOOLBAR_HEIGHT {
            return;
        }

        if let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) {
            if let Some(grid_pos) = world_to_grid(
                world_pos,
                grid_settings.tile_size,
                grid_settings.width,
                grid_settings.height,
            ) {
                // Find deconstructible entity at this position
                for (entity, entity_grid_pos) in &deconstructible_query {
                    if entity_grid_pos.to_ivec2() == grid_pos {
                        // Show context menu
                        context_menu_state.visible = true;
                        context_menu_state.target_entity = Some(entity);
                        context_menu_state.position = cursor_pos;
                        return;
                    }
                }
            }
        }
    }

    // Close menu if clicking elsewhere
    context_menu_state.visible = false;
}

#[derive(Component)]
struct ContextMenu;

#[derive(Component)]
struct DeconstructButton;

fn setup_context_menu(mut commands: Commands) {
    // Create hidden context menu
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Px(120.0),
                height: Val::Px(40.0),
                flex_direction: FlexDirection::Column,
                display: Display::None, // Hidden by default
                ..default()
            },
            BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
            ContextMenu,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(40.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.3, 0.3, 0.3)),
                    DeconstructButton,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Deconstruct"),
                        TextFont {
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });
        });
}

fn update_context_menu(
    mut menu_query: Query<&mut Node, With<ContextMenu>>,
    context_menu_state: Res<ContextMenuState>,
    mut ui_blocker: ResMut<UiInputBlocker>,
) {
    for mut node in &mut menu_query {
        if context_menu_state.visible {
            node.display = Display::Flex;
            node.left = Val::Px(context_menu_state.position.x);
            node.top = Val::Px(context_menu_state.position.y);
        } else {
            node.display = Display::None;
        }
    }

    ui_blocker.context_menu_blocking = context_menu_state.visible;
    ui_blocker.recompute();
}

fn handle_context_menu_clicks(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut interaction_query: Query<(&Interaction, &DeconstructButton), Changed<Interaction>>,
    mut context_menu_state: ResMut<ContextMenuState>,
    deconstructible_query: Query<
        (&GridPosition, &Transform),
        Or<(
            With<Wall>,
            With<Door>,
            With<crate::components::Window>,
            With<Furniture>,
        )>,
    >,
    marker_query: Query<&DeconstructionMarker>,
    grid_settings: Res<GridSettings>,
) {
    for (interaction, _) in &mut interaction_query {
        if *interaction == Interaction::Pressed {
            if let Some(target_entity) = context_menu_state.target_entity {
                // Check if already marked
                let already_marked = marker_query
                    .iter()
                    .any(|marker| marker.target_entity == target_entity);

                if !already_marked {
                    if let Ok((grid_pos, transform)) = deconstructible_query.get(target_entity) {
                        // Create deconstruction marker
                        let marker_entity = commands
                            .spawn((
                                Mesh2d(meshes.add(Rectangle::new(
                                    grid_settings.tile_size,
                                    grid_settings.tile_size,
                                ))),
                                MeshMaterial2d(materials.add(Color::srgba(1.0, 0.0, 0.0, 0.4))),
                                Transform::from_xyz(
                                    transform.translation.x,
                                    transform.translation.y,
                                    10.0,
                                ),
                                DeconstructionMarker::new(target_entity),
                                GridPosition::new(grid_pos.x, grid_pos.y),
                            ))
                            .id();

                        commands.spawn(DeconstructionJob::new(marker_entity));
                    }
                }
            }

            // Close context menu
            context_menu_state.visible = false;
        }
    }
}

// Update wall projections based on adjacent walls
fn update_wall_projections(
    mut commands: Commands,
    structure_query: Query<
        (Entity, &GridPosition, Option<&WallProjection>),
        Or<(With<Wall>, With<crate::components::Window>)>,
    >,
    all_structures_query: Query<&GridPosition, Or<(With<Wall>, With<crate::components::Window>)>>,
) {
    let occupied_positions: HashSet<IVec2> = all_structures_query
        .iter()
        .map(|pos| pos.to_ivec2())
        .collect();

    for (entity, pos, existing_projection) in &structure_query {
        let current_pos = pos.to_ivec2();

        // Check adjacent positions
        let has_north = occupied_positions.contains(&(current_pos + IVec2::new(0, 1)));
        let has_east = occupied_positions.contains(&(current_pos + IVec2::new(1, 0)));
        let has_west = occupied_positions.contains(&(current_pos + IVec2::new(-1, 0)));

        let mut projection = WallProjection::new();

        // Show top shadow if no wall above - this creates the main depth effect
        if !has_north {
            projection = projection.with_north();
        }

        // Show east shadow if no wall to the east - creates right edge depth
        if !has_east {
            projection = projection.with_east();
        }

        // Show west shadow if no wall to the west - creates left edge depth
        if !has_west {
            projection = projection.with_west();
        }

        if existing_projection.copied() != Some(projection) {
            commands.entity(entity).insert(projection);
        }
    }
}
