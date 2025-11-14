use crate::components::*;
use crate::systems::building::BuildingMap;
use crate::systems::grid::*;
use bevy::prelude::*;
use bevy::sprite::*;


pub struct WorkPlugin;

impl Plugin for WorkPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                (
                    assign_jobs_to_pawns,
                    assign_deconstruction_jobs_to_pawns,
                    assign_reception_staff,
                )
                    .chain(),
                (work_on_blueprints, work_on_deconstruction).chain(),
                (update_blueprint_visuals, update_deconstruction_visuals),
                (complete_blueprints, complete_deconstruction).chain(),
                handle_door_interactions,
            ),
        );
    }
}

// Assign construction jobs to idle pawns
fn assign_jobs_to_pawns(
    mut commands: Commands,
    mut pawn_query: Query<(Entity, &Transform, &mut CurrentJob, &WorkAssignments), With<Pawn>>,
    mut job_query: Query<(Entity, &mut ConstructionJob)>,
    blueprint_query: Query<&GridPosition, With<Blueprint>>,
    grid_settings: Res<GridSettings>,
) {
    // Find idle pawns
    for (pawn_entity, pawn_transform, mut current_job, work_assignments) in &mut pawn_query {
        if current_job.job_id.is_some() {
            continue; // Pawn already has a job
        }

        // Check if pawn can do construction work
        if !work_assignments.can_do_work(WorkType::Construction) {
            continue;
        }

        // Find the nearest unassigned job
        let mut nearest_job: Option<(Entity, f32)> = None;
        let pawn_pos = pawn_transform.translation.truncate();

        for (job_entity, job) in &job_query {
            if job.assigned_pawn.is_some() {
                continue; // Job already assigned
            }

            if let Ok(blueprint_grid_pos) = blueprint_query.get(job.blueprint) {
                let blueprint_world_pos = grid_to_world(
                    blueprint_grid_pos.to_ivec2(),
                    grid_settings.tile_size,
                    grid_settings.width,
                    grid_settings.height,
                );
                let distance = pawn_pos.distance(blueprint_world_pos);

                if nearest_job.is_none() || distance < nearest_job.unwrap().1 {
                    nearest_job = Some((job_entity, distance));
                }
            }
        }

        // Assign the nearest job
        if let Some((job_entity, _)) = nearest_job {
            if let Ok((_, mut job)) = job_query.get_mut(job_entity) {
                job.assigned_pawn = Some(pawn_entity);
                current_job.job_id = Some(job_entity);

                // Add movement target to the blueprint location
                if let Ok(blueprint_grid_pos) = blueprint_query.get(job.blueprint) {
                    let target_pos = grid_to_world(
                        blueprint_grid_pos.to_ivec2(),
                        grid_settings.tile_size,
                        grid_settings.width,
                        grid_settings.height,
                    );
                    commands
                        .entity(pawn_entity)
                        .insert(MovementTarget { target: target_pos });
                }
            }
        }
    }
}

// Assign pawns to staff reception desks
fn assign_reception_staff(
    mut commands: Commands,
    mut pawn_query: Query<
        (Entity, &Transform, &CurrentJob, &WorkAssignments),
        (With<Pawn>, Without<StaffingReception>),
    >,
    console_query: Query<(Entity, &GridPosition), With<ReceptionConsole>>,
    staffed_query: Query<&StaffingReception>,
    grid_settings: Res<GridSettings>,
) {
    // Find unstaffed reception desks
    for (console_entity, console_pos) in &console_query {
        // Check if this desk is already staffed
        let is_staffed = staffed_query
            .iter()
            .any(|staffing| staffing.desk_entity == console_entity);

        if is_staffed {
            continue;
        }

        // Find idle pawn with reception work enabled
        for (pawn_entity, pawn_transform, current_job, work_assignments) in &pawn_query {
            // Pawn must be idle and able to do reception work
            if current_job.job_id.is_some() {
                continue;
            }

            if !work_assignments.can_do_work(WorkType::Reception) {
                continue;
            }

            // Assign this pawn to staff the desk
            let desk_world_pos = grid_to_world(
                console_pos.to_ivec2(),
                grid_settings.tile_size,
                grid_settings.width,
                grid_settings.height,
            );

            commands.entity(pawn_entity).insert((
                MovementTarget {
                    target: desk_world_pos,
                },
                StaffingReception {
                    desk_entity: console_entity,
                },
            ));

            // Only assign one pawn per desk
            break;
        }
    }
}

// Pawns work on blueprints when nearby
fn work_on_blueprints(
    mut commands: Commands,
    mut pawn_query: Query<(Entity, &Transform, &CurrentJob), With<Pawn>>,
    mut job_query: Query<&ConstructionJob>,
    mut blueprint_query: Query<(&Transform, &mut Blueprint)>,
    time: Res<Time>,
) {
    for (pawn_entity, pawn_transform, current_job) in &mut pawn_query {
        if let Some(job_id) = current_job.job_id {
            if let Ok(job) = job_query.get_mut(job_id) {
                if let Ok((blueprint_transform, mut blueprint)) =
                    blueprint_query.get_mut(job.blueprint)
                {
                    let distance = pawn_transform
                        .translation
                        .truncate()
                        .distance(blueprint_transform.translation.truncate());

                    // Check if pawn is close enough to work (within 2 tiles)
                    if distance < TILE_SIZE * 3.0 {
                        // Remove movement target if present
                        commands.entity(pawn_entity).remove::<MovementTarget>();

                        // Do work
                        let work_speed = 50.0; // work units per second (faster building)
                        blueprint.work_done += work_speed * time.delta_secs();
                        blueprint.work_done = blueprint.work_done.min(blueprint.work_required);
                    }
                }
            }
        }
    }
}

// Complete blueprints and turn them into actual buildings
fn complete_blueprints(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    blueprint_query: Query<(Entity, &Blueprint, &GridPosition, &Transform)>,
    job_query: Query<(Entity, &ConstructionJob)>,
    mut pawn_query: Query<&mut CurrentJob, With<Pawn>>,
    grid_settings: Res<GridSettings>,
    mut building_map: ResMut<BuildingMap>,
) {
    for (blueprint_entity, blueprint, grid_pos, transform) in &blueprint_query {
        if blueprint.is_complete() {
            // Find and remove the associated job
            for (job_entity, job) in &job_query {
                if job.blueprint == blueprint_entity {
                    // Clear pawn's current job
                    if let Some(pawn_entity) = job.assigned_pawn {
                        if let Ok(mut current_job) = pawn_query.get_mut(pawn_entity) {
                            current_job.job_id = None;
                        }
                    }
                    commands.entity(job_entity).despawn();
                }
            }

            // Remove blueprint and spawn actual building (including any child visuals)
            commands.entity(blueprint_entity).despawn_recursive();

            match blueprint.building_type {
                BlueprintType::Wall => {
                    let wall_entity = commands
                        .spawn((
                            Mesh2d(meshes.add(Rectangle::new(
                                grid_settings.tile_size,
                                grid_settings.tile_size,
                            ))),
                            MeshMaterial2d(materials.add(WallMaterial::Stone.color())),
                            Transform::from_xyz(
                                transform.translation.x,
                                transform.translation.y,
                                2.0,
                            ),
                            Wall,
                            Building,
                            GridPosition::new(grid_pos.x, grid_pos.y),
                        ))
                        .id();

                    // Update building map to track the completed wall entity
                    building_map.walls.insert(grid_pos.to_ivec2(), wall_entity);
                }
                BlueprintType::Door(orientation) => {
                    let (width, height, offset) = match orientation {
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

                    let world_pos = grid_to_world(
                        grid_pos.to_ivec2(),
                        grid_settings.tile_size,
                        grid_settings.width,
                        grid_settings.height,
                    ) + offset;

                    commands.spawn((
                        Mesh2d(meshes.add(Rectangle::new(width, height))),
                        MeshMaterial2d(materials.add(Color::srgb(0.4, 0.3, 0.2))),
                        Transform::from_xyz(world_pos.x, world_pos.y, 2.0),
                        Door::new(orientation),
                        Building,
                        GridPosition::new(grid_pos.x, grid_pos.y),
                    ));
                }
                BlueprintType::Window => {
                    commands.spawn((
                        Mesh2d(meshes.add(Rectangle::new(
                            grid_settings.tile_size,
                            grid_settings.tile_size * WINDOW_THICKNESS,
                        ))),
                        MeshMaterial2d(materials.add(Color::srgb(0.6, 0.8, 1.0))),
                        Transform::from_xyz(transform.translation.x, transform.translation.y, 2.0),
                        Window,
                        Building,
                        GridPosition::new(grid_pos.x, grid_pos.y),
                    ));
                }
                BlueprintType::Floor(floor_type) => {
                    commands.spawn((
                        Mesh2d(meshes.add(Rectangle::new(
                            grid_settings.tile_size,
                            grid_settings.tile_size,
                        ))),
                        MeshMaterial2d(materials.add(floor_type.color())),
                        Transform::from_xyz(
                            transform.translation.x,
                            transform.translation.y,
                            0.5, // Floors render below everything else
                        ),
                        Floor { floor_type },
                        GridPosition::new(grid_pos.x, grid_pos.y),
                    ));
                }
                BlueprintType::Furniture(_furniture_type) => {
                    // Furniture is spawned directly without blueprints, so this case shouldn't occur
                    // But we need it for pattern matching completeness
                    warn!("Furniture blueprint completed unexpectedly - furniture should spawn directly");
                }
            }
        }
    }
}

// Update blueprint visuals to show construction progress
fn update_blueprint_visuals(
    query: Query<(&Blueprint, &MeshMaterial2d<ColorMaterial>), Changed<Blueprint>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for (blueprint, material_handle) in &query {
        if let Some(material) = materials.get_mut(&material_handle.0) {
            let progress = blueprint.progress();
            // Increase opacity as construction progresses
            let alpha = 0.3 + (progress * 0.5);

            let base_color = match blueprint.building_type {
                BlueprintType::Wall => Color::srgb(0.5, 0.5, 0.5),
                BlueprintType::Door(_) => Color::srgb(0.4, 0.3, 0.2),
                BlueprintType::Window => Color::srgb(0.6, 0.8, 1.0),
                BlueprintType::Floor(floor_type) => floor_type.color(),
                BlueprintType::Furniture(furniture_type) => furniture_type.color(),
            };

            material.color = base_color.with_alpha(alpha);
        }
    }
}

// Assign deconstruction jobs to idle pawns
fn assign_deconstruction_jobs_to_pawns(
    mut commands: Commands,
    mut pawn_query: Query<(Entity, &Transform, &mut CurrentJob, &WorkAssignments), With<Pawn>>,
    mut job_query: Query<(Entity, &mut DeconstructionJob)>,
    marker_query: Query<&GridPosition, With<DeconstructionMarker>>,
    grid_settings: Res<GridSettings>,
) {
    // Find idle pawns
    for (pawn_entity, pawn_transform, mut current_job, work_assignments) in &mut pawn_query {
        if current_job.job_id.is_some() {
            continue; // Pawn already has a job
        }

        // Check if pawn can do construction work (deconstruction uses the same skill)
        if !work_assignments.can_do_work(WorkType::Construction) {
            continue;
        }

        // Find the nearest unassigned deconstruction job
        let mut nearest_job: Option<(Entity, f32)> = None;
        let pawn_pos = pawn_transform.translation.truncate();

        for (job_entity, job) in &job_query {
            if job.assigned_pawn.is_some() {
                continue; // Job already assigned
            }

            if let Ok(marker_grid_pos) = marker_query.get(job.marker) {
                let marker_world_pos = grid_to_world(
                    marker_grid_pos.to_ivec2(),
                    grid_settings.tile_size,
                    grid_settings.width,
                    grid_settings.height,
                );
                let distance = pawn_pos.distance(marker_world_pos);

                if nearest_job.is_none() || distance < nearest_job.unwrap().1 {
                    nearest_job = Some((job_entity, distance));
                }
            }
        }

        // Assign the nearest job
        if let Some((job_entity, _)) = nearest_job {
            if let Ok((_, mut job)) = job_query.get_mut(job_entity) {
                job.assigned_pawn = Some(pawn_entity);
                current_job.job_id = Some(job_entity);

                // Add movement target to the marker location
                if let Ok(marker_grid_pos) = marker_query.get(job.marker) {
                    let target_pos = grid_to_world(
                        marker_grid_pos.to_ivec2(),
                        grid_settings.tile_size,
                        grid_settings.width,
                        grid_settings.height,
                    );
                    commands
                        .entity(pawn_entity)
                        .insert(MovementTarget { target: target_pos });
                }
            }
        }
    }
}

// Pawns work on deconstruction when nearby
fn work_on_deconstruction(
    mut commands: Commands,
    mut pawn_query: Query<(Entity, &Transform, &CurrentJob), With<Pawn>>,
    mut job_query: Query<&DeconstructionJob>,
    mut marker_query: Query<(&Transform, &mut DeconstructionMarker)>,
    time: Res<Time>,
) {
    for (pawn_entity, pawn_transform, current_job) in &mut pawn_query {
        if let Some(job_id) = current_job.job_id {
            if let Ok(job) = job_query.get_mut(job_id) {
                if let Ok((marker_transform, mut marker)) = marker_query.get_mut(job.marker) {
                    let distance = pawn_transform
                        .translation
                        .truncate()
                        .distance(marker_transform.translation.truncate());

                    // Check if pawn is close enough to work (within 3 tiles)
                    if distance < TILE_SIZE * 3.0 {
                        // Remove movement target if present
                        commands.entity(pawn_entity).remove::<MovementTarget>();

                        // Do work
                        let work_speed = 40.0; // Deconstruction is faster than construction
                        marker.work_done += work_speed * time.delta_secs();
                        marker.work_done = marker.work_done.min(marker.work_required);
                    }
                }
            }
        }
    }
}

// Update deconstruction marker visuals to show progress
fn update_deconstruction_visuals(
    query: Query<
        (&DeconstructionMarker, &MeshMaterial2d<ColorMaterial>),
        Changed<DeconstructionMarker>,
    >,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for (marker, material_handle) in &query {
        if let Some(material) = materials.get_mut(&material_handle.0) {
            let progress = marker.progress();
            // Increase opacity as deconstruction progresses
            let alpha = 0.4 + (progress * 0.4);
            material.color = Color::srgba(1.0, 0.0, 0.0, alpha);
        }
    }
}

// Complete deconstruction and remove target entities
fn complete_deconstruction(
    mut commands: Commands,
    marker_query: Query<(Entity, &DeconstructionMarker, &GridPosition)>,
    job_query: Query<(Entity, &DeconstructionJob)>,
    mut pawn_query: Query<&mut CurrentJob, With<Pawn>>,
    mut building_map: ResMut<BuildingMap>,
    wall_query: Query<&GridPosition, With<Wall>>,
    door_query: Query<&Door>,
    furniture_query: Query<(), With<Furniture>>,
) {
    for (marker_entity, marker, grid_pos) in &marker_query {
        if marker.is_complete() {
            // Find and remove the associated job
            for (job_entity, job) in &job_query {
                if job.marker == marker_entity {
                    // Clear pawn's current job
                    if let Some(pawn_entity) = job.assigned_pawn {
                        if let Ok(mut current_job) = pawn_query.get_mut(pawn_entity) {
                            current_job.job_id = None;
                        }
                    }
                    commands.entity(job_entity).despawn();
                }
            }

            // Remove the target entity and update building map
            let target_entity = marker.target_entity;
            let grid_ivec = grid_pos.to_ivec2();

            // Update building map based on what was deconstructed
            if wall_query.get(target_entity).is_ok() {
                building_map.walls.remove(&grid_ivec);
                building_map.occupied.remove(&grid_ivec);
            } else if let Ok(door) = door_query.get(target_entity) {
                // Remove all door tiles
                let door_tiles = match door.orientation {
                    DoorOrientation::Horizontal => vec![grid_ivec, grid_ivec + IVec2::new(1, 0)],
                    DoorOrientation::Vertical => vec![grid_ivec, grid_ivec + IVec2::new(0, 1)],
                };
                for tile in door_tiles {
                    building_map.doors.remove(&tile);
                }
            } else if furniture_query.get(target_entity).is_ok() {
                // Furniture - remove all potentially occupied tiles around this position
                // Since we don't store orientation, check a 2x2 area
                for x in 0..=1 {
                    for y in 0..=1 {
                        building_map
                            .occupied
                            .remove(&(grid_ivec + IVec2::new(x, y)));
                    }
                }
            } else {
                // Window or other single-tile structure
                building_map.occupied.remove(&grid_ivec);
            }

            // Despawn both the marker and the target entity
            commands.entity(marker_entity).despawn_recursive(); // Use recursive to remove ASCII text child
            commands.entity(target_entity).despawn_recursive();
        }
    }
}

// Handle door opening and closing based on pawn proximity
fn handle_door_interactions(
    mut door_query: Query<(&mut Transform, &mut Door, &MeshMaterial2d<ColorMaterial>)>,
    pawn_query: Query<&Transform, (With<Pawn>, Without<Door>)>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    time: Res<Time>,
) {
    const DOOR_OPEN_DISTANCE: f32 = TILE_SIZE * 4.0; // Doors open when pawns are within 4 tiles
    const DOOR_ANIMATION_SPEED: f32 = 3.0; // Radians per second

    for (mut door_transform, mut door, material_handle) in &mut door_query {
        let door_pos = door_transform.translation.truncate();

        // Check if any pawn is near this door
        let mut should_be_open = false;
        for pawn_transform in &pawn_query {
            let pawn_pos = pawn_transform.translation.truncate();
            let distance = door_pos.distance(pawn_pos);

            if distance < DOOR_OPEN_DISTANCE {
                should_be_open = true;
                break;
            }
        }

        // Update door state if it changed
        let previous_state = door.state;
        door.state = if should_be_open {
            DoorState::Open
        } else {
            DoorState::Closed
        };

        // Animate door rotation
        let target_rotation = match door.state {
            DoorState::Open => std::f32::consts::PI / 4.0, // 45 degrees open
            DoorState::Closed => 0.0,
        };

        let current_rotation = door_transform.rotation.to_euler(EulerRot::XYZ).2;
        let rotation_diff = target_rotation - current_rotation;

        if rotation_diff.abs() > 0.01 {
            let rotation_step = rotation_diff.signum() * DOOR_ANIMATION_SPEED * time.delta_secs();
            let new_rotation = if rotation_diff.abs() < rotation_step.abs() {
                target_rotation
            } else {
                current_rotation + rotation_step
            };
            door_transform.rotation = Quat::from_rotation_z(new_rotation);
        }

        // Update visual appearance when state changes
        if previous_state != door.state {
            if let Some(material) = materials.get_mut(&material_handle.0) {
                match door.state {
                    DoorState::Open => {
                        // Make door more transparent when open
                        material.color = Color::srgb(0.4, 0.3, 0.2).with_alpha(0.3);
                    }
                    DoorState::Closed => {
                        // Solid color when closed
                        material.color = Color::srgb(0.4, 0.3, 0.2);
                    }
                }
            }
        }
    }
}
