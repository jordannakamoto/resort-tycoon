use bevy::prelude::*;
use bevy::sprite::*;
use crate::components::*;
use crate::systems::grid::*;

pub struct WorkPlugin;

impl Plugin for WorkPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            assign_jobs_to_pawns,
            work_on_blueprints,
            update_blueprint_visuals,
            complete_blueprints,
        ).chain());
    }
}

// Assign construction jobs to idle pawns
fn assign_jobs_to_pawns(
    mut commands: Commands,
    mut pawn_query: Query<(Entity, &Transform, &mut CurrentJob), With<Pawn>>,
    mut job_query: Query<(Entity, &mut ConstructionJob)>,
    blueprint_query: Query<&GridPosition, With<Blueprint>>,
    grid_settings: Res<GridSettings>,
) {
    // Find idle pawns
    for (pawn_entity, pawn_transform, mut current_job) in &mut pawn_query {
        if current_job.job_id.is_some() {
            continue; // Pawn already has a job
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
                    commands.entity(pawn_entity).insert(MovementTarget { target: target_pos });
                }
            }
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
                if let Ok((blueprint_transform, mut blueprint)) = blueprint_query.get_mut(job.blueprint) {
                    let distance = pawn_transform
                        .translation
                        .truncate()
                        .distance(blueprint_transform.translation.truncate());

                    // Check if pawn is close enough to work (within 2 tiles)
                    if distance < TILE_SIZE * 3.0 {
                        // Remove movement target if present
                        commands.entity(pawn_entity).remove::<MovementTarget>();

                        // Do work
                        let work_speed = 10.0; // work units per second
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

            // Remove blueprint and spawn actual building
            commands.entity(blueprint_entity).despawn();

            match blueprint.building_type {
                BlueprintType::Wall => {
                    commands.spawn((
                        Mesh2d(meshes.add(Rectangle::new(grid_settings.tile_size, grid_settings.tile_size))),
                        MeshMaterial2d(materials.add(WallMaterial::Stone.color())),
                        Transform::from_xyz(
                            transform.translation.x,
                            transform.translation.y,
                            2.0,
                        ),
                        Wall,
                        Building,
                        GridPosition::new(grid_pos.x, grid_pos.y),
                    ));
                }
                BlueprintType::Door => {
                    commands.spawn((
                        Mesh2d(meshes.add(Rectangle::new(grid_settings.tile_size, grid_settings.tile_size))),
                        MeshMaterial2d(materials.add(Color::srgb(0.4, 0.3, 0.2))),
                        Transform::from_xyz(
                            transform.translation.x,
                            transform.translation.y,
                            2.0,
                        ),
                        Door,
                        Building,
                        GridPosition::new(grid_pos.x, grid_pos.y),
                    ));
                }
                BlueprintType::Window => {
                    commands.spawn((
                        Mesh2d(meshes.add(Rectangle::new(grid_settings.tile_size, grid_settings.tile_size))),
                        MeshMaterial2d(materials.add(Color::srgb(0.6, 0.8, 1.0))),
                        Transform::from_xyz(
                            transform.translation.x,
                            transform.translation.y,
                            2.0,
                        ),
                        Window,
                        Building,
                        GridPosition::new(grid_pos.x, grid_pos.y),
                    ));
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
                BlueprintType::Door => Color::srgb(0.4, 0.3, 0.2),
                BlueprintType::Window => Color::srgb(0.6, 0.8, 1.0),
            };

            material.color = base_color.with_alpha(alpha);
        }
    }
}
