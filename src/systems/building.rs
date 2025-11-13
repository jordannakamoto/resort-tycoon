use bevy::prelude::*;
use bevy::sprite::*;
use bevy::window::{PrimaryWindow, Window as BevyWindow};
use crate::components::*;
use crate::systems::grid::*;
use crate::systems::Money;
use crate::ui::{ToolbarState, BuildingType};

#[derive(Resource)]
pub struct BuildingMap {
    pub occupied: std::collections::HashSet<IVec2>,      // Walls and windows (block movement)
    pub walls: std::collections::HashMap<IVec2, Entity>, // Wall entities by position
    pub doors: std::collections::HashMap<IVec2, Entity>, // Door tiles (can pass when open)
    pub floors: std::collections::HashSet<IVec2>,        // Floors (don't block building)
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
            .add_systems(Update, (
                handle_door_rotation,
                handle_drag_input,
                update_placement_preview,
                handle_building_placement,
            ).chain());
    }
}

fn handle_door_rotation(
    mut door_state: ResMut<DoorPlacementState>,
    toolbar_state: Res<ToolbarState>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    // Only allow rotation when door is selected
    if toolbar_state.selected_building == Some(BuildingType::Door) {
        if keyboard.just_pressed(KeyCode::KeyR) {
            door_state.orientation = match door_state.orientation {
                DoorOrientation::Horizontal => DoorOrientation::Vertical,
                DoorOrientation::Vertical => DoorOrientation::Horizontal,
            };
        }
    }
}

fn handle_drag_input(
    mut drag_state: ResMut<DragState>,
    toolbar_state: Res<ToolbarState>,
    grid_settings: Res<GridSettings>,
    window_query: Query<&BevyWindow, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mouse_button: Res<ButtonInput<MouseButton>>,
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
    grid_settings: Res<GridSettings>,
    window_query: Query<&BevyWindow, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    preview_query: Query<Entity, With<PlacementPreview>>,
    building_map: Res<BuildingMap>,
    desk_query: Query<&GridPosition, With<Desk>>,
) {
    let window = window_query.single();
    let (camera, camera_transform) = camera_query.single();

    // Remove old preview
    for entity in &preview_query {
        commands.entity(entity).despawn();
    }

    // Only show preview if a building is selected
    if let Some(building_type) = toolbar_state.selected_building {
        // If dragging walls or floors, show all positions in the drag area
        let is_dragging_multi = matches!(building_type, BuildingType::Wall | BuildingType::Floor(_)) && drag_state.is_dragging;

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
                    Color::srgba(1.0, 0.3, 0.3, 0.5)
                } else {
                    Color::srgba(0.3, 1.0, 0.3, 0.5)
                };

                commands.spawn((
                    Mesh2d(meshes.add(Rectangle::new(grid_settings.tile_size, grid_settings.tile_size))),
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
                            DoorOrientation::Horizontal => vec![grid_pos, grid_pos + IVec2::new(1, 0)],
                            DoorOrientation::Vertical => vec![grid_pos, grid_pos + IVec2::new(0, 1)],
                        };

                        for tile_pos in door_tiles {
                            let tile_world_pos = grid_to_world(
                                tile_pos,
                                grid_settings.tile_size,
                                grid_settings.width,
                                grid_settings.height,
                            );

                            let is_blocked = building_map.occupied.contains(&tile_pos) || building_map.doors.contains_key(&tile_pos);
                            let color = if is_blocked {
                                Color::srgba(1.0, 0.3, 0.3, 0.5)
                            } else {
                                Color::srgba(0.3, 1.0, 0.3, 0.5)
                            };

                            commands.spawn((
                                Mesh2d(meshes.add(Rectangle::new(grid_settings.tile_size, grid_settings.tile_size))),
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

                            let color = if !has_desk {
                                Color::srgba(1.0, 0.3, 0.3, 0.5) // Red if no desk
                            } else {
                                Color::srgba(0.3, 1.0, 0.3, 0.5) // Green if desk present
                            };

                            commands.spawn((
                                Mesh2d(meshes.add(Rectangle::new(grid_settings.tile_size, grid_settings.tile_size))),
                                MeshMaterial2d(materials.add(color)),
                                Transform::from_xyz(world_pos.x, world_pos.y, 1.0),
                                PlacementPreview,
                            ));
                        } else {
                            // Multi-tile furniture preview
                            let furniture_tiles = furniture_type.tiles_occupied(grid_pos);

                            for tile_pos in furniture_tiles {
                                let tile_world_pos = grid_to_world(
                                    tile_pos,
                                    grid_settings.tile_size,
                                    grid_settings.width,
                                    grid_settings.height,
                                );

                                // Furniture needs floor and available space
                                let has_floor = building_map.floors.contains(&tile_pos);
                                let is_occupied = building_map.occupied.contains(&tile_pos) || building_map.doors.contains_key(&tile_pos);
                                let is_blocked = !has_floor || is_occupied;

                                let color = if is_blocked {
                                    Color::srgba(1.0, 0.3, 0.3, 0.5)
                                } else {
                                    Color::srgba(0.3, 1.0, 0.3, 0.5)
                                };

                                commands.spawn((
                                    Mesh2d(meshes.add(Rectangle::new(grid_settings.tile_size, grid_settings.tile_size))),
                                    MeshMaterial2d(materials.add(color)),
                                    Transform::from_xyz(tile_world_pos.x, tile_world_pos.y, 1.0),
                                    PlacementPreview,
                                ));
                            }
                        }
                    } else {
                        // Single tile preview for other buildings
                        let is_occupied = building_map.occupied.contains(&grid_pos);
                        let color = if is_occupied {
                            Color::srgba(1.0, 0.3, 0.3, 0.5)
                        } else {
                            Color::srgba(0.3, 1.0, 0.3, 0.5)
                        };

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

fn handle_building_placement(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    toolbar_state: Res<ToolbarState>,
    mut drag_state: ResMut<DragState>,
    door_state: Res<DoorPlacementState>,
    grid_settings: Res<GridSettings>,
    window_query: Query<&BevyWindow, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut building_map: ResMut<BuildingMap>,
    mut money: ResMut<Money>,
    desk_query: Query<&GridPosition, With<Desk>>,
) {
    if let Some(building_type) = toolbar_state.selected_building {
        // Handle drag building for walls and floors
        let is_drag_buildable = matches!(building_type, BuildingType::Wall | BuildingType::Floor(_));

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
                                DoorOrientation::Horizontal => vec![grid_pos, grid_pos + IVec2::new(1, 0)],
                                DoorOrientation::Vertical => vec![grid_pos, grid_pos + IVec2::new(0, 1)],
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
                                    (door_tiles[0].x + door_tiles[1].x) as f32 * grid_settings.tile_size / 2.0 - (grid_settings.width as f32 * grid_settings.tile_size) / 2.0,
                                    door_tiles[0].y as f32 * grid_settings.tile_size - (grid_settings.height as f32 * grid_settings.tile_size) / 2.0 + grid_settings.tile_size / 2.0,
                                ),
                                DoorOrientation::Vertical => Vec2::new(
                                    door_tiles[0].x as f32 * grid_settings.tile_size - (grid_settings.width as f32 * grid_settings.tile_size) / 2.0 + grid_settings.tile_size / 2.0,
                                    (door_tiles[0].y + door_tiles[1].y) as f32 * grid_settings.tile_size / 2.0 - (grid_settings.height as f32 * grid_settings.tile_size) / 2.0,
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

                                // Spawn reception console on top of the desk
                                let console_entity = commands.spawn((
                                    Mesh2d(meshes.add(Rectangle::new(
                                        grid_settings.tile_size,
                                        grid_settings.tile_size,
                                    ))),
                                    MeshMaterial2d(materials.add(furniture_type.color())),
                                    Transform::from_xyz(base_world_pos.x, base_world_pos.y, 3.5), // Higher than desk (3.0)
                                    GridPosition::new(grid_pos.x, grid_pos.y),
                                    Furniture,
                                    ReceptionConsole::new(),
                                )).id();

                                // Don't mark tiles as occupied - desk already occupies them
                                return;
                            }

                            // Handle regular furniture placement
                            let furniture_tiles = furniture_type.tiles_occupied(grid_pos);
                            let (width_mult, height_mult) = furniture_type.size();

                            // Check if all tiles have floors and are available
                            let can_place = furniture_tiles.iter().all(|pos| {
                                building_map.floors.contains(pos) &&
                                !building_map.occupied.contains(pos) &&
                                !building_map.doors.contains_key(pos)
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
                            let furniture_entity = commands.spawn((
                                Mesh2d(meshes.add(Rectangle::new(
                                    width_mult * grid_settings.tile_size,
                                    height_mult * grid_settings.tile_size,
                                ))),
                                MeshMaterial2d(materials.add(furniture_type.color())),
                                Transform::from_xyz(furniture_pos.x, furniture_pos.y, 3.0), // Above floors and structures
                                GridPosition::new(grid_pos.x, grid_pos.y),
                                Furniture,
                            )).id();

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
                                FurnitureType::ReceptionConsole => {
                                    commands.entity(furniture_entity).insert(ReceptionConsole::new());
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
                                    let has_other = building_map.occupied.contains(&grid_pos) && !has_wall;
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
    // Blueprints are semi-transparent versions of the final building
    let (color, z_level) = match blueprint_type {
        BlueprintType::Wall => (Color::srgba(0.5, 0.5, 0.5, 0.5), 1.5),
        BlueprintType::Door(_) => (Color::srgba(0.4, 0.3, 0.2, 0.5), 1.5),
        BlueprintType::Window => (Color::srgba(0.6, 0.8, 1.0, 0.5), 1.5),
        BlueprintType::Floor(floor_type) => {
            let base_color = floor_type.color();
            (base_color.with_alpha(0.5), 0.5) // Floors at lower z-level
        }
        BlueprintType::Furniture(furniture_type) => {
            let base_color = furniture_type.color();
            (base_color.with_alpha(0.5), 2.5) // Furniture at higher z-level
        }
    };

    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(tile_size, tile_size))),
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
            (tile_size * 2.0, tile_size, Vec2::new(tile_size / 2.0, 0.0))
        }
        DoorOrientation::Vertical => {
            // 2 tiles tall: shift up by half a tile to center between both tiles
            (tile_size, tile_size * 2.0, Vec2::new(0.0, tile_size / 2.0))
        }
    };

    let adjusted_pos = center_pos + offset;

    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(width, height))),
        MeshMaterial2d(materials.add(Color::srgba(0.4, 0.3, 0.2, 0.5))),
        Transform::from_xyz(adjusted_pos.x, adjusted_pos.y, 1.5),
        Blueprint::new(BlueprintType::Door(orientation)),
        GridPosition::new(grid_pos.x, grid_pos.y),
    ))
    .id()
}
